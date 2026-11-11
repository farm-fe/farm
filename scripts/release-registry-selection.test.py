"""Version selection contracts with real source graphs and explicit registry fixtures."""
# cspell:words importlib urllib SIGALRM SIGTERM ITIMER setitimer getitimer
import base64
import copy
from contextlib import contextmanager
import importlib.util
import json
import signal
import time
from pathlib import Path
import unittest
from unittest import mock
import urllib.error


def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

module=load('registry_selection_tests','release-registry-selection.py')
entry=load('registry_entry_tests','incremental-release-entry.py')
fixtures=load('registry_selection_fixtures','incremental-chain-plan.test.py')
INTEGRITY = 'sha512-' + base64.b64encode(b'a' * 64).decode()


def source_fixture():
    files,family,source=fixtures.fixture()
    files['packages/core/package.json']=b'{"name":"@farmfe/core","version":"1.0.0","private":true}'
    files['crates/core/src/lib.rs']+=b'\npub const VERSION: &str = "0.5.4";\n'
    files['docs/releases/v2-candidate.json']=json.dumps(family).encode()
    return {**source,'files':files,'modes':{name:'100644' for name in files}},family


def observed(row,published=False):
    value={'name':row['name'],'version':row['version'],'status':'published' if published else 'missing'}
    if published:value.update(archiveSha256='c'*64,archiveBytes=123,integrity=INTEGRITY,evidenceKind='registry-archive',metadata={},tarballURL='https://registry.npmjs.org/'+row['name']+'/-/fixture.tgz')
    return value


class Contracts(unittest.TestCase):
    def test_candidate_package_inputs_do_not_change_registry_selection(self):
        source, family = source_fixture()
        before = {}
        for kind in ['npm', 'cargo']:
            rows = module.records(source, kind)
            first = sorted(rows)[0]
            before[kind] = module.collect(source, kind, lambda row: observed(row, row['name'] == rows[first]['name']))
        expected = entry.source_plan(source, registry_snapshot=before['npm'])
        updated = copy.deepcopy(family)
        for row in updated['cargo']:
            row['packageInputsSha256'] = 'f' * 64
        source['files']['docs/releases/v2-candidate.json'] = json.dumps(updated).encode()
        for kind in ['npm', 'cargo']:
            rows = module.records(source, kind)
            first = sorted(rows)[0]
            actual = module.collect(source, kind, lambda row: observed(row, row['name'] == rows[first]['name']))
            self.assertEqual(actual, before[kind])
        actual = entry.source_plan(source, registry_snapshot=before['npm'])
        for key in ['publish', 'build', 'versionRequired', 'registryRetained']:
            self.assertEqual(actual[key], expected[key], key)

    def test_no_baseline_missing_versions_build_full_family(self):
        source,family=source_fixture();snapshot=module.collect(source,'npm',observed)
        result=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(result['publish'],['npm:a','npm:b']);self.assertEqual(result['versionRequired'],[])
        self.assertFalse(result['publicationBlocked']);self.assertIsNone(result['baselineSHA'])

    def test_only_missing_version_uploads_but_published_dependency_builds(self):
        source,family=source_fixture();snapshot=module.collect(source,'npm',lambda row:observed(row,row['name']=='a'))
        result=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(result['publish'],['npm:b']);self.assertEqual(result['build'],['npm:b'])
        self.assertIn('npm:packages/a',result['buildNodes']);self.assertIn('npm:a',result['buildDependencyClosure'])
        self.assertIn('npm:a',result['registryRetained'])

    def test_control_and_product_diff_do_not_republish_unchanged_versions(self):
        source,family=source_fixture();source['files']['scripts/new-control.py']=b'changed';source['files']['packages/a/src/index.ts']=b'changed'
        source['modes']={name:'100644' for name in source['files']}
        snapshot=module.collect(source,'npm',lambda row:observed(row,True));result=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(result['publish'],[]);self.assertEqual(result['buildNodes'],[]);self.assertEqual(result['versionRequired'],[])

    def test_unknown_error_missing_family_wrong_source_and_digest_reject(self):
        for defect in ['error','missing','source','checksum']:
            source,_=source_fixture();snapshot=module.collect(source,'npm',lambda row:observed(row,True))
            if defect=='error':snapshot['packages']['npm:a']['status']='error'
            elif defect=='missing':del snapshot['packages']['npm:a']
            elif defect=='source':snapshot['sourceSHA']='f'*40
            else:snapshot['packages']['npm:a']['archiveSha256']='invalid'
            with self.assertRaises(ValueError):entry.source_plan(source,registry_snapshot=snapshot)

    def test_registry_404_only_is_absence(self):
        row={'kind':'npm','name':'a','version':'1.0.0'}
        for code in [404,401,403,429,500,502]:
            reader=mock.Mock();reader.json.side_effect=urllib.error.HTTPError('https://registry.npmjs.org/a/1.0.0',code,'fixture',{},None)
            if code==404:self.assertEqual(module.Registry(reader).lookup(row)['status'],'missing')
            else:
                with self.assertRaises(urllib.error.HTTPError):module.Registry(reader).lookup(row)
        for error in [TimeoutError(),ValueError('invalid JSON')]:
            reader=mock.Mock();reader.json.side_effect=error
            with self.assertRaises(type(error)):module.Registry(reader).lookup(row)

    def test_published_dependency_declarations_and_checksum_rechecked(self):
        source,_=source_fixture();snapshot=module.collect(source,'npm',lambda row:observed(row,True),schema_version=1)
        snapshot['packages']['npm:b']['metadata']={'dependencies':{'a':'2.0.0'}}
        with self.assertRaisesRegex(ValueError,'dependency incompatible'):entry.source_plan(source,registry_snapshot=snapshot)
        snapshot=module.collect(source,'npm',lambda row:observed(row,True))
        with self.assertRaisesRegex(ValueError,'Retained published version'):module.verify_retained(source,snapshot,observed)

    def test_stable_range_bounds_and_unknown_tags_fail_closed(self):
        for version,requirement,result in [('2.0.0','2.0.0',True),('2.0.1','2.0.0',False),('2.3.0','^2.0.0',True),('3.0.0','^2.0.0',False),('0.2.3','^0.2.0',True),('0.3.0','^0.2.0',False),('0.0.2','^0.0.1',False),('2.1.0','~2.0.0',False)]:
            self.assertEqual(module.stable_satisfies(version,requirement),result)
        with self.assertRaises(ValueError):module.stable_satisfies('2.0.0','latest')

    def test_frozen_missing_set_survives_partial_publication(self):
        source,_=source_fixture();snapshot=module.collect(source,'npm',observed)
        before=entry.source_plan(source,registry_snapshot=snapshot)
        module.verify_retained(source,snapshot,lambda row:observed(row,True))
        self.assertEqual(entry.source_plan(source,registry_snapshot=snapshot),before)
        # Fresh plans see registry changes; an existing transaction keeps originals.
        after=entry.source_plan(source,registry_snapshot=module.collect(source,'npm',lambda row:observed(row,True)))
        self.assertEqual(after['publish'],[]);self.assertNotEqual(before['planHash'],after['planHash'])

    def test_state_baseline_fence_changes_hash_without_changing_upload_scope(self):
        source,_=source_fixture();snapshot=module.collect(source,'npm',observed,schema_version=1)
        before=entry.source_plan(source,registry_snapshot=snapshot);snapshot['stateBaseline']='e'*64
        after=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(before['publish'],after['publish']);self.assertNotEqual(before['planHash'],after['planHash'])
        self.assertEqual(after['baselineBinding'],'e'*64)

    def test_locked_registry_revalidation_preserves_snapshot_and_rejects_drift(self):
        publisher=load('registry_locked_publisher','incremental-release-publish.py')
        source,_=source_fixture();snapshot=module.collect(source,'npm',lambda row:observed(row,row['name']=='a'),schema_version=1)
        plan=entry.source_plan(source,registry_snapshot=snapshot)
        manifest={key:plan[key] for key in ['sourceSHA','sourceTree','planHash','versionRequired','publish']}
        manifest.update(registrySnapshot=snapshot,baseline=None,rebuild=plan['build'])
        transaction=mock.Mock();transaction.manifest=manifest
        transaction.owned.return_value=(None,{'active':{'phase':'uploading','packages':{}}})
        registry=mock.Mock();registry.lookup.side_effect=lambda row, **options:observed(row,True)
        baseline_reader=mock.Mock(side_effect=AssertionError('No historical baseline lookup permitted'))
        with mock.patch.object(publisher.entry,'load',return_value=module),mock.patch.object(module,'Registry',return_value=registry):
            result=publisher.verify_locked_plan(transaction,source,mock.Mock(),baseline_reader=baseline_reader,planner=entry.source_plan)
            self.assertEqual(result,plan);baseline_reader.assert_not_called()
            self.assertTrue(all(call.kwargs.get('verify_archive') is True for call in registry.lookup.call_args_list))
            registry.lookup.side_effect=TimeoutError('registry unavailable')
            with self.assertRaises(TimeoutError):publisher.verify_locked_plan(transaction,source,mock.Mock(),baseline_reader=baseline_reader,planner=entry.source_plan)
            transaction.reject_unwritten.assert_called_once()
            transaction.reject_unwritten.reset_mock()
            # After any possible upload, a failure preserves the original transaction.
            transaction.owned.return_value=(None,{'active':{'phase':'uploading','packages':{'npm:b':{'status':'started'}}}})
            with self.assertRaises(TimeoutError):publisher.verify_locked_plan(transaction,source,mock.Mock(),baseline_reader=baseline_reader,planner=entry.source_plan)
            transaction.reject_unwritten.assert_not_called()

    def test_collect_isolates_unknown_and_never_copies_raw_errors(self):
        source, _ = source_fixture()
        for error in [urllib.error.HTTPError('https://user:secret@example.test', 403, 'secret', {}, None),
                      TimeoutError('token=secret'), ValueError('invalid JSON secret'), RuntimeError('secret')]:
            def lookup(row):
                if row['name'] == 'a':
                    raise error
                return observed(row)
            snapshot = module.collect(source, 'npm', lookup)
            result = entry.source_plan(source, registry_snapshot=snapshot)
            self.assertEqual(result['build'], ['npm:b'])
            self.assertEqual(result['publish'], ['npm:b'])
            self.assertEqual(set(result['registryUnknown']), {'npm:a'})
            self.assertEqual(result['registryRetained'], {})
            self.assertNotIn('secret', json.dumps(snapshot))
            self.assertLessEqual(len(snapshot['packages']['npm:a']['reason']), 80)
            self.assertIsNone(result['baselineBinding'])
            self.assertFalse(result['publicationBlocked'])

    def test_invalid_registry_identity_is_unknown_not_missing(self):
        source, _ = source_fixture()
        snapshot = module.collect(source, 'npm', lambda row: {'status': 'published', 'name': 'other', 'version': row['version']})
        self.assertEqual({row['status'] for row in snapshot['packages'].values()}, {'unknown'})
        result = entry.source_plan(source, registry_snapshot=snapshot)
        self.assertEqual(result['publish'], [])
        self.assertEqual(set(result['registryUnknown']), {'npm:a', 'npm:b'})
        self.assertEqual(result['registryRetained'], {})

    def test_stateless_snapshot_rejects_historical_state(self):
        source, _ = source_fixture()
        snapshot = module.collect(source, 'npm', observed)
        snapshot['stateBaseline'] = None
        with self.assertRaisesRegex(ValueError, 'Stateless'):
            entry.source_plan(source, registry_snapshot=snapshot)

    def test_only_metadata_404_is_missing_and_archive_404_is_unknown(self):
        source, _ = source_fixture()
        reader = mock.Mock()
        def metadata(url):
            name = url.split('/')[-2]
            return {'name': name, 'version': '1.0.0', 'dist': {'tarball': 'https://registry.npmjs.org/' + name + '/-/fixture.tgz', 'integrity': INTEGRITY}}
        reader.json.side_effect = metadata
        reader.archive.side_effect = urllib.error.HTTPError('https://registry.npmjs.org/a/-/fixture.tgz', 404, 'missing archive', {}, None)
        snapshot = module.collect(source, 'npm', lambda row: module.Registry(reader).lookup(row, verify_archive=True))
        self.assertEqual({row['status'] for row in snapshot['packages'].values()}, {'unknown'})
        reader.json.side_effect = urllib.error.HTTPError('https://registry.npmjs.org/a/1.0.0', 404, 'missing version', {}, None)
        snapshot = module.collect(source, 'npm', lambda row: module.Registry(reader).lookup(row, verify_archive=True))
        self.assertEqual({row['status'] for row in snapshot['packages'].values()}, {'missing'})

    def test_registry_metadata_requires_200_and_tarball_url_is_safe(self):
        reader = module.ExactRegistryReadOnly(environment={'GITHUB_ACTIONS': 'true'})
        response = mock.MagicMock()
        response.__enter__.return_value = response
        response.read.return_value = b'{}'
        for status in [201, 204, 206, 304]:
            response.getcode.return_value = status
            with mock.patch.object(reader, 'open', return_value=response):
                with self.assertRaisesRegex(ValueError, 'HTTP 200'):
                    reader.json('https://registry.npmjs.org/a/1.0.0')
        for url in ['http://registry.npmjs.org/a.tgz', 'https://registry.npmjs.org@evil.test/a.tgz',
                    'https://registry.npmjs.org/a.tgz?token=secret', 'https://user:secret@registry.npmjs.org/a.tgz',
                    'https://registry.npmjs.org:444/a.tgz', 'https://registry.npmjs.org/a.tgz#fragment']:
            with self.assertRaises(ValueError):
                module.safe_tarball_url(url)

    def test_registry_observation_preserves_optional_peer_metadata(self):
        row = {'kind': 'npm', 'name': 'a', 'version': '1.0.0'}
        reader = mock.Mock()
        reader.json.return_value = {'name': 'a', 'version': '1.0.0',
            'dist': {'tarball': 'https://registry.npmjs.org/a/-/a-1.0.0.tgz', 'integrity': INTEGRITY},
            'peerDependencies': {'b': '^1.0.0'}, 'peerDependenciesMeta': {'b': {'optional': True}}}
        reader.archive.return_value = {'sha256': 'c' * 64, 'sha512': INTEGRITY[7:], 'sha1': 'd' * 40, 'bytes': 123}
        result = module.Registry(reader).lookup(row)
        self.assertEqual(result['metadata']['peerDependenciesMeta'], {'b': {'optional': True}})
        self.assertEqual(result['tarballURL'], reader.json.return_value['dist']['tarball'])
        self.assertEqual(result['evidenceKind'], 'registry-metadata')
        self.assertNotIn('archiveSha256', result)
        self.assertNotIn('archiveBytes', result)
        reader.archive.assert_not_called()
        executed = module.Registry(reader).lookup(row, verify_archive=True)
        self.assertEqual(executed['archiveSha256'], 'c' * 64)
        self.assertEqual(executed['evidenceKind'], 'registry-archive')

    def test_exact_platform_cells_and_registry_host_do_not_expand_owner(self):
        sha = entry.chain.impact.git('rev-parse', 'HEAD').decode().strip()
        source = entry.evidence.source_reader(sha)
        missing = {'@farmfe/core-darwin-arm64', '@farmfe/js-plugin-less'}
        snapshot = module.collect(source, 'npm', lambda row: observed(row, row['name'] not in missing))
        result = entry.source_plan(source, registry_snapshot=snapshot)
        self.assertEqual(result['nativeBuildCells'], [{'owner': '@farmfe/core', 'platform': 'darwin-arm64', 'package': 'npm:@farmfe/core-darwin-arm64'}])
        self.assertEqual(result['preparationNativeCells'], [])
        self.assertEqual(set(result['registryBuildInputs']), {'npm:@farmfe/core-linux-x64-gnu'})
        self.assertEqual(result['registryBuildInputs']['npm:@farmfe/core-linux-x64-gnu']['origin'], 'registry-present')
        self.assertEqual(result['buildBlocked'], {})
        self.assertNotIn('npm:packages/core/npm/linux-x64-gnu', result['buildNodes'])
        self.assertNotIn('npm:@farmfe/core-linux-x64-gnu', result['buildDependencyClosure'])
        self.assertNotIn('npm:packages/core/npm/darwin-x64', result['buildNodes'])
        self.assertIn('npm:packages/core', result['buildNodes'])
        self.assertNotIn('npm:@farmfe/core', result['build'])
        self.assertEqual(set(result['publish']), {'npm:' + name for name in missing})

    def test_missing_host_builds_only_gnu_unknown_host_only_blocks_js(self):
        sha = entry.chain.impact.git('rev-parse', 'HEAD').decode().strip()
        source = entry.evidence.source_reader(sha)
        missing = {'@farmfe/core-darwin-arm64', '@farmfe/js-plugin-less', '@farmfe/core-linux-x64-gnu'}
        snapshot = module.collect(source, 'npm', lambda row: observed(row, row['name'] not in missing))
        result = entry.source_plan(source, registry_snapshot=snapshot)
        host = 'npm:@farmfe/core-linux-x64-gnu'
        self.assertEqual(result['preparationNativeCells'], [{'owner': '@farmfe/core', 'platform': 'linux-x64-gnu', 'package': host}])
        self.assertEqual(result['registryBuildInputs'], {})
        snapshot['packages'][host] = module.unknown(module.records(source, 'npm')[host], TimeoutError())
        result = entry.source_plan(source, registry_snapshot=snapshot)
        self.assertEqual(result['preparationNativeCells'], [])
        self.assertEqual(set(result['buildBlocked']), {'npm:@farmfe/js-plugin-less'})
        self.assertEqual(result['buildBlocked']['npm:@farmfe/js-plugin-less']['dependencies'], [host])
        self.assertEqual(result['nativeBuildCells'], [{'owner': '@farmfe/core', 'platform': 'darwin-arm64', 'package': 'npm:@farmfe/core-darwin-arm64'}])
        self.assertEqual(set(result['registryUnknown']), {host})

    def test_metadata_only_evidence_rejects_measured_byte_claims_and_bad_integrity(self):
        source, _ = source_fixture()
        row = module.records(source, 'npm')['npm:a']
        metadata = observed(row, True)
        metadata['evidenceKind'] = 'registry-metadata'
        del metadata['archiveSha256']
        del metadata['archiveBytes']
        module.validate_observed(row, metadata, 2)
        for field, value in [('archiveSha256', 'c' * 64), ('archiveBytes', 123), ('registryBytesVerified', True), ('integrity', 'sha512-YQ==')]:
            bad = {**metadata, field: value}
            with self.assertRaises(ValueError):
                module.validate_observed(row, bad, 2)

    def test_collection_deadline_preserves_verified_rows_and_marks_remaining_unknown(self):
        source, _ = source_fixture()
        now = [0]
        def lookup(row):
            now[0] = 241
            return observed(row, True)
        snapshot = module.collect(source, 'npm', lookup, clock=lambda: now[0])
        self.assertEqual(snapshot['packages']['npm:a']['status'], 'published')
        self.assertEqual(snapshot['packages']['npm:b']['status'], 'unknown')
        self.assertNotIn('deadline exceeded', json.dumps(snapshot))

    def test_real_lookup_deadline_interrupts_blocking_body_and_continues_independent_missing(self):
        source, _ = source_fixture()
        previous_handler = signal.getsignal(signal.SIGALRM)
        previous_term = signal.getsignal(signal.SIGTERM)
        calls = []
        def lookup(row):
            calls.append(row['name'])
            if row['name'] == 'a':
                time.sleep(2)
            return observed(row)
        started = time.monotonic()
        snapshot = module.collect(source, 'npm', lookup, seconds=0.5, per_package_seconds=0.05)
        self.assertLess(time.monotonic() - started, 0.5)
        self.assertEqual(calls, ['a', 'b'])
        self.assertEqual(snapshot['packages']['npm:a']['status'], 'unknown')
        self.assertEqual(snapshot['packages']['npm:b']['status'], 'missing')
        self.assertEqual(signal.getsignal(signal.SIGALRM), previous_handler)
        self.assertEqual(signal.getsignal(signal.SIGTERM), previous_term)
        self.assertEqual(signal.getitimer(signal.ITIMER_REAL)[0], 0)

    def test_real_alarm_interrupts_metadata_response_read_not_just_socket_timeout(self):
        source, _ = source_fixture()
        reader = module.ExactRegistryReadOnly(environment={'GITHUB_ACTIONS': 'true'})
        class SlowResponse:
            def __enter__(self): return self
            def __exit__(self, *args): return False
            def getcode(self): return 200
            def read(self, _limit):
                time.sleep(2)
                return b'{}'
        def open_response(url):
            if '/a/' in url:
                return SlowResponse()
            raise urllib.error.HTTPError(url, 404, 'No exact version', {}, None)
        started = time.monotonic()
        with mock.patch.object(reader, 'open', side_effect=open_response):
            snapshot = module.collect(source, 'npm', module.Registry(reader).lookup, seconds=0.5, per_package_seconds=0.05)
        self.assertLess(time.monotonic() - started, 0.5)
        self.assertEqual(snapshot['packages']['npm:a']['status'], 'unknown')
        self.assertEqual(snapshot['packages']['npm:b']['status'], 'missing')
        self.assertIsNone(reader.deadline)
        self.assertEqual(reader.total_byte_limit, 3 * 1024 ** 3)

    def test_real_collection_deadline_preserves_completed_rows_and_marks_all_remaining(self):
        source, _ = source_fixture()
        family = copy.deepcopy(module.records(source, 'npm'))
        family['npm:c'] = {**family['npm:b'], 'name': 'c'}
        calls = []
        def lookup(row):
            calls.append(row['name'])
            if row['name'] == 'b':
                time.sleep(2)
            return observed(row, row['name'] == 'a')
        started = time.monotonic()
        with mock.patch.object(module, 'records', return_value=family):
            snapshot = module.collect(source, 'npm', lookup, seconds=0.05, per_package_seconds=1)
        self.assertLess(time.monotonic() - started, 0.5)
        self.assertEqual(calls, ['a', 'b'])
        self.assertEqual(snapshot['packages']['npm:a']['status'], 'published')
        self.assertEqual(snapshot['packages']['npm:b']['status'], 'unknown')
        self.assertEqual(snapshot['packages']['npm:c']['status'], 'unknown')
        self.assertEqual(signal.getitimer(signal.ITIMER_REAL)[0], 0)

    def test_shorter_external_alarm_propagates_and_control_interrupts_are_not_unknown(self):
        source, _ = source_fixture()
        class OuterDeadline(Exception):
            pass
        original = signal.getsignal(signal.SIGALRM)
        def interrupt(_number, _frame):
            raise OuterDeadline('Outer execution deadline')
        signal.signal(signal.SIGALRM, interrupt)
        signal.setitimer(signal.ITIMER_REAL, 0.05)
        try:
            with self.assertRaises(OuterDeadline):
                module.collect(source, 'npm', lambda row: time.sleep(2), seconds=1, per_package_seconds=1)
            self.assertEqual(signal.getsignal(signal.SIGALRM), interrupt)
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)
            signal.signal(signal.SIGALRM, original)
        for error in [KeyboardInterrupt(), SystemExit(3)]:
            with self.assertRaises(type(error)):
                module.collect(source, 'npm', mock.Mock(side_effect=error))
        self.assertEqual(signal.getsignal(signal.SIGALRM), original)
        self.assertEqual(signal.getitimer(signal.ITIMER_REAL)[0], 0)

    def test_longer_external_timer_is_restored_with_elapsed_time_deducted(self):
        source, _ = source_fixture()
        original = signal.getsignal(signal.SIGALRM)
        def interrupt(_number, _frame):
            raise AssertionError('Long outer timer should not fire')
        signal.signal(signal.SIGALRM, interrupt)
        signal.setitimer(signal.ITIMER_REAL, 1, 0.25)
        try:
            snapshot = module.collect(source, 'npm', observed, seconds=0.2, per_package_seconds=0.1)
            self.assertEqual({row['status'] for row in snapshot['packages'].values()}, {'missing'})
            delay, interval = signal.getitimer(signal.ITIMER_REAL)
            self.assertTrue(0.5 < delay < 1)
            self.assertEqual(interval, 0.25)
            self.assertEqual(signal.getsignal(signal.SIGALRM), interrupt)
        finally:
            signal.setitimer(signal.ITIMER_REAL, 0)
            signal.signal(signal.SIGALRM, original)

    def test_collection_stream_budgets_are_per_package_and_restore_reader(self):
        source, _ = source_fixture()
        class Reader:
            total_bytes = 0
            total_byte_limit = 3 * 1024 ** 3
            budgets = []
            @contextmanager
            def budget(self, seconds):
                self.budgets.append(seconds)
                yield
        class Lookup:
            reader = Reader()
            allowances = []
            def lookup(self, row):
                self.allowances.append(self.reader.total_byte_limit - self.reader.total_bytes)
                self.reader.total_bytes += 100 * 1024 ** 2 + 1
                if row['name'] == 'a':
                    raise ValueError('oversized archive secret')
                return observed(row)
        lookup = Lookup()
        snapshot = module.collect(source, 'npm', lookup.lookup)
        self.assertEqual(snapshot['packages']['npm:a']['status'], 'unknown')
        self.assertEqual(snapshot['packages']['npm:b']['status'], 'missing')
        self.assertEqual(lookup.reader.budgets, [30, 30])
        self.assertEqual(lookup.allowances, [100 * 1024 ** 2, 100 * 1024 ** 2])
        self.assertEqual(lookup.reader.total_byte_limit, 3 * 1024 ** 3)

    def test_legacy_snapshot_without_new_optional_observation_fields_still_verifies(self):
        source, _ = source_fixture()
        def legacy(row):
            return {'status': 'published', 'name': row['name'], 'version': row['version'],
                    'archiveSha256': 'c' * 64, 'integrity': INTEGRITY,
                    'metadata': {'dependencies': {}, 'optionalDependencies': {}, 'peerDependencies': {}}}
        snapshot = module.collect(source, 'npm', legacy, schema_version=1)
        def current(row):
            return {**legacy(row), 'archiveBytes': 123, 'evidenceKind': 'registry-archive',
                    'tarballURL': 'https://registry.npmjs.org/' + row['name'] + '/archive.tgz',
                    'metadata': {**legacy(row)['metadata'], 'peerDependenciesMeta': {}}}
        self.assertEqual(module.verify_retained(source, snapshot, current), {})
        with self.assertRaisesRegex(ValueError, 'Retained published version'):
            module.verify_retained(source, snapshot, lambda row: {**current(row), 'archiveSha256': 'f' * 64})

    def test_cargo_default_observation_remains_legacy_fail_closed(self):
        source, _ = source_fixture()
        snapshot = module.collect(source, 'cargo', observed)
        self.assertEqual(snapshot['schemaVersion'], 1)
        with self.assertRaises(TimeoutError):
            module.collect(source, 'cargo', mock.Mock(side_effect=TimeoutError()))

    def test_cargo_versions_come_from_fixed_sdk_manifests_not_npm_candidate(self):
        sha=entry.chain.impact.git('rev-parse','HEAD').decode().strip();source=entry.evidence.source_reader(sha)
        records=module.records(source,'cargo')
        row=records['cargo:farmfe_core'];file=row['manifestPath']
        parts=list(map(int,row['version'].split('.')));parts[2]+=1;next_version='.'.join(map(str,parts))
        before=source['files'][file];after=before.replace(('version = "'+row['version']+'"').encode(),('version = "'+next_version+'"').encode(),1)
        self.assertNotEqual(before,after)
        source['files'][file]=after
        snapshot=module.collect(source,'cargo',observed)
        self.assertEqual(snapshot['packages']['cargo:farmfe_core']['version'],next_version)
        self.assertEqual(len(module.validate(source,'cargo',snapshot)),28)
        old=next(row for row in json.loads(source['files']['docs/releases/v2-candidate.json'])['cargo'] if row['name']=='farmfe_core')
        self.assertEqual(old['version'],row['version'])

    def test_real_fixed_main_family_and_shared_graph(self):
        sha=entry.chain.impact.git('rev-parse','HEAD').decode().strip();source=entry.evidence.source_reader(sha)
        snapshot=module.collect(source,'npm',observed);plan=entry.source_plan(source,registry_snapshot=snapshot)
        self.assertEqual(len(plan['publish']),297);self.assertEqual(len(module.records(source,'cargo')),28)
        self.assertEqual(plan['versionRequired'],[]);self.assertTrue(plan['preparationNativeOwners'])
        self.assertIn('cargo:crates/core',plan['buildNodes'])

if __name__=='__main__':unittest.main()
