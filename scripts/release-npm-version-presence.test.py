"""Offline version-presence execution contracts; no network or registry writes."""
# cspell:words importlib readback ENEEDAUTH finalizer postpublish ECONNRESET
import base64
import copy
import hashlib
import importlib.util
import json
import io
import os
import tempfile
import urllib.error
from types import SimpleNamespace
from pathlib import Path
import unittest
from unittest import mock


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    value = importlib.util.module_from_spec(spec); spec.loader.exec_module(value); return value


old = load('presence_fixtures', 'release-npm-best-effort.test.py')
module = old.module


def observed(name, body, fields=None):
    return {'status': 'published', 'name': name, 'version': '2.0.0', 'archiveSha256': hashlib.sha256(body).hexdigest(),
            'archiveBytes': len(body), 'evidenceKind': 'registry-archive', 'integrity': 'sha512-' + base64.b64encode(hashlib.sha512(body).digest()).decode(),
            'tarballURL': 'https://registry.npmjs.org/' + name + '/archive.tgz',
            'metadata': {key: (fields or {}).get(key, {}) for key in module.selector.METADATA_FIELDS}}


def metadata_only(value):
    return {**{key: item for key, item in value.items() if key not in ['archiveSha256', 'archiveBytes']}, 'evidenceKind': 'registry-metadata'}


class Registry(old.Registry):
    def __init__(self, bodies, fields):
        super().__init__(); self.bodies = bodies; self.fields = fields; self.public = {}; self.reads = []
    def observe(self, row, *, verify_archive=False):
        name = row['name']; self.reads.append(name)
        if name in self.read_fail: raise OSError('private bearer sentinel')
        value = copy.deepcopy(self.public.get(name, {'status': 'missing', 'name': name, 'version': row['version']}))
        return value if verify_archive or value['status'] != 'published' else metadata_only(value)
    def publish(self, row, body):
        self.writes.append((row['name'], body)); name = row['name']
        if name in self.fail:
            raise module.publisher.npm_failure('publish', row, code=1, stderr='npm error code ENEEDAUTH\nprivate bearer sentinel')
        assert name not in self.public
        self.public[name] = observed(name, body, self.fields[name])
        if name in self.lost: raise TimeoutError('private bearer sentinel')


def fixture(packages=None, retained=(), unknown=(), failed_build=()):
    packages = packages or [('a', {}), ('b', {'dependencies': {'a': '2.0.0'}}), ('independent', {})]
    manifest, bodies, _ = old.Contracts().fixture(packages)
    fields = dict(packages); registry = Registry(bodies, fields)
    manifest['family'] = [{'kind': 'npm', 'name': name, 'version': '2.0.0'} for name, _ in packages]
    snapshot = {'schemaVersion': 2, 'kind': 'exact-registry-version-observation', 'sourceSHA': manifest['sourceSHA'],
                'sourceTree': manifest['sourceTree'], 'registryKind': 'npm', 'packages': {}}
    for name, value in packages:
        row = {'status': 'missing', 'name': name, 'version': '2.0.0'}
        if name in retained:
            # Published bytes intentionally differ from newly generated source.
            row = observed(name, old.archive(name, description='prior source', **value), value)
            registry.public[name] = copy.deepcopy(row)
        if name in unknown: row.update(status='unknown', reason='registry-http-403')
        snapshot['packages']['npm:' + name] = metadata_only(row) if row['status'] == 'published' else row
    missing = {key for key, value in snapshot['packages'].items() if value['status'] == 'missing'}
    manifest.update(registrySnapshot=snapshot, selectionPolicy='registry-exact-version-v2', releaseValidation='archive-integrity-v1',
                    publish=sorted(missing), rebuild=sorted(missing),
                    buildFailures={'npm:' + name: 'native-build-failed' for name in failed_build})
    manifest['packages'] = [row for row in manifest['packages'] if 'npm:' + row['name'] in missing and row['name'] not in failed_build]
    return manifest, bodies, registry


def execute(manifest, bodies, registry, **options):
    clock = options.pop('clock', old.Clock()); report = module.new_report(manifest)
    module.publish_packages(manifest, lambda row: bodies[row['name']], registry, report, lambda value: None,
                            clock=clock, sleep=clock.sleep, **options)
    return report, {row['name']: row for row in report['packages']}


class Contracts(unittest.TestCase):
    def test_retained_different_source_bytes_not_uploaded_but_latest_confirmed(self):
        manifest, bodies, registry = fixture(retained=['a'])
        report, rows = execute(manifest, bodies, registry)
        self.assertTrue(report['success']); self.assertEqual(rows['a']['originKind'], 'registry')
        self.assertNotIn('archiveSha256', rows['a']); self.assertFalse(rows['a']['registryBytesVerified'])
        self.assertEqual(rows['a']['evidenceKind'], 'registry-metadata')
        self.assertEqual([name for name, _ in registry.writes], ['b', 'independent'])
        self.assertIn('a', registry.tags); self.assertEqual(report['counts']['retained'], 1)
    def test_all_retained_empty_candidate_tag_only(self):
        manifest, bodies, registry = fixture(retained=['a', 'b', 'independent'])
        self.assertEqual(manifest['packages'], [])
        report, rows = execute(manifest, {}, registry)
        self.assertTrue(report['success']); self.assertEqual(registry.writes, [])
        self.assertEqual(set(registry.tags), set(rows)); self.assertEqual(report['counts']['retained'], 3)
    def test_unknown_and_build_failure_do_not_stop_independent(self):
        manifest, bodies, registry = fixture(unknown=['a'], failed_build=['b'])
        report, rows = execute(manifest, bodies, registry)
        self.assertFalse(report['success']); self.assertTrue(report['complete'])
        self.assertEqual(rows['independent']['status'], 'success')
        self.assertNotIn('a', registry.reads); self.assertEqual([name for name, _ in registry.writes], ['independent'])
    def test_297_complete_partition(self):
        packages = [('p' + str(i), {}) for i in range(297)]
        manifest, bodies, registry = fixture(packages, retained=['p' + str(i) for i in range(245)], unknown=['p245'])
        registry.fail.add('p246')
        report, rows = execute(manifest, bodies, registry)
        self.assertEqual(report['counts'], {'total': 297, 'success': 295, 'failed': 2, 'published': 50, 'retained': 245})
        self.assertEqual(len(registry.writes), 51); self.assertEqual(len(rows), 297)
    def test_race_uses_registry_metadata_and_no_source_bytes_comparison(self):
        manifest, bodies, registry = fixture([('a', {'dependencies': {'bad': '2.0.0'}}), ('bad', {}), ('independent', {})], unknown=['bad'])
        registry.public['a'] = observed('a', old.archive('a', description='different'), {})
        report, rows = execute(manifest, bodies, registry)
        self.assertEqual(rows['a']['status'], 'success'); self.assertEqual(rows['a']['originKind'], 'registry-race')
        self.assertEqual([name for name, _ in registry.writes], ['independent']); self.assertIn('a', registry.tags)
    def test_race_actual_required_unknown_blocks_tag(self):
        manifest, bodies, registry = fixture([('a', {}), ('bad', {}), ('independent', {})], unknown=['bad'])
        registry.public['a'] = observed('a', old.archive('a'), {'dependencies': {'bad': '2.0.0'}})
        report, rows = execute(manifest, bodies, registry)
        self.assertEqual(rows['a']['status'], 'failed'); self.assertNotIn('a', registry.tags)
        self.assertEqual(rows['independent']['status'], 'success')
    def test_upload_lost_response_remains_failure_but_verified_dependency_usable(self):
        manifest, bodies, registry = fixture(); registry.lost.add('a')
        report, rows = execute(manifest, bodies, registry)
        self.assertFalse(report['success']); self.assertTrue(rows['a']['registryBytesVerified'])
        self.assertEqual(rows['b']['status'], 'success'); self.assertEqual(len([n for n, _ in registry.writes if n == 'a']), 1)
        self.assertNotIn('private bearer', json.dumps(report)); self.assertFalse(rows['a']['publishOutcome']['accepted'])
    def test_after_attempt_foreign_bytes_conflict_not_retained(self):
        manifest, bodies, registry = fixture()
        original = registry.publish
        def publish(row, body):
            original(row, body)
            if row['name'] == 'a': registry.public['a'] = observed('a', old.archive('a', different=True))
        registry.publish = publish
        report, rows = execute(manifest, bodies, registry)
        self.assertEqual(rows['a']['status'], 'failed'); self.assertEqual(rows['a']['originKind'], 'source')
        self.assertEqual(rows['b']['status'], 'failed'); self.assertEqual(rows['independent']['status'], 'success')
    def test_peer_cycle_and_transitive_missing_peer(self):
        packages = [('a', {'dependencies': {'b': '2.0.0'}}), ('b', {'peerDependencies': {'a': '2.0.0', 'c': '2.0.0'}}), ('c', {}), ('independent', {})]
        manifest, bodies, registry = fixture(packages); registry.fail.add('c')
        report, rows = execute(manifest, bodies, registry)
        self.assertEqual(registry.tags, ['independent']); self.assertEqual(rows['a']['status'], 'failed')
        manifest, bodies, registry = fixture(packages)
        report, rows = execute(manifest, bodies, registry); self.assertTrue(report['success'])
    def test_tag_error_does_not_block_required_consumer(self):
        manifest, bodies, registry = fixture(retained=['a']); registry.tag_fail.add('a')
        report, rows = execute(manifest, bodies, registry, visibility_seconds=3)
        self.assertFalse(report['success']); self.assertEqual(rows['b']['status'], 'success')
        self.assertEqual(registry.writes[0][0], 'b')
    def test_bad_archive_isolated_and_missing_partition_rejected(self):
        manifest, bodies, registry = fixture(); bodies['a'] = b'corrupt'
        report, rows = execute(manifest, bodies, registry)
        self.assertEqual(rows['independent']['status'], 'success'); self.assertEqual(rows['a']['status'], 'failed')
        manifest['packages'].pop()
        with self.assertRaisesRegex(ValueError, 'partition'): module.new_report(manifest)
    def test_auth_rejection_structured_no_uncertain_upload(self):
        manifest, bodies, registry = fixture(); registry.fail.add('a')
        report, rows = execute(manifest, bodies, registry)
        self.assertEqual(rows['a']['publishOutcome'], {'operation': 'publish', 'returncode': 1, 'timedOut': False,
                         'npmCodes': ['ENEEDAUTH'], 'definiteRejection': True, 'accepted': False})
        self.assertFalse(rows['a']['outcomeUncertain'])

    def test_visibility_after_fifteen_minutes_releases_required_consumer(self):
        manifest, bodies, registry = fixture(); clock = old.Clock(); original = registry.observe
        def observe(row, **options):
            if row['name'] == 'a' and clock() < 950:
                return {'status': 'missing', 'name': 'a', 'version': '2.0.0'}
            return original(row, **options)
        registry.observe = observe
        report, rows = execute(manifest, bodies, registry, clock=clock)
        self.assertTrue(report['success']); self.assertGreaterEqual(clock(), 950)
        self.assertEqual([name for name, _ in registry.writes].count('a'), 1)
        self.assertEqual([name for name, _ in registry.writes], ['a', 'independent', 'b'])
    def test_independent_latest_not_starved_by_unrelated_visibility_at_global_deadline(self):
        manifest, bodies, registry = fixture([('slow', {}), ('ready', {})]); clock = old.Clock()
        original = registry.observe
        registry.observe = lambda row, **options: ({'status': 'missing', 'name': row['name'], 'version': row['version']}
                                       if row['name'] == 'slow' else original(row, **options))
        deadline = module.Deadline(900, clock); report = module.new_report(manifest)
        with self.assertRaises(module.PublicationInterrupted):
            module.publish_packages(manifest, lambda row: bodies[row['name']], registry, report, lambda value: None,
                                    clock=clock, sleep=clock.sleep, remaining=deadline.remaining)
        rows = {row['name']: row for row in report['packages']}
        self.assertEqual(rows['ready']['status'], 'success'); self.assertEqual(registry.tags, ['ready'])
        self.assertTrue(rows['slow']['outcomeUncertain']); self.assertFalse(report['success'])
    def test_lost_tag_response_never_repeats_write_or_masks_error(self):
        manifest, bodies, registry = fixture(retained=['a']); original = registry.promote_latest
        def promote(row):
            original(row)
            if row['name'] == 'a': raise TimeoutError('secret')
        registry.promote_latest = promote
        report, rows = execute(manifest, bodies, registry)
        self.assertFalse(report['success']); self.assertTrue(rows['a']['latestVerified'])
        self.assertEqual(registry.tags.count('a'), 1); self.assertEqual(rows['b']['status'], 'success')
    def test_partial_reads_charge_real_shared_counter_but_cannot_exhaust_others(self):
        manifest, bodies, registry = fixture([('slow', {}), ('ready', {})]); clock = old.Clock()
        registry.reader = module.selector.ExactRegistryReadOnly({'GITHUB_ACTIONS': 'true'}, clock=clock)
        original = registry.observe
        def observe(row, **options):
            if row['name'] in registry.public and row['name'] == 'slow':
                registry.reader.total_bytes += 95 * 1024 ** 2
                if registry.reader.total_bytes > registry.reader.total_byte_limit:
                    raise ValueError('Registry archive byte budget exceeded')
                raise TimeoutError('partial stream')
            result = original(row, **options)
            if result['status'] == 'published': registry.reader.total_bytes += result.get('archiveBytes', 0)
            return result
        registry.observe = observe
        report, rows = execute(manifest, bodies, registry, clock=clock)
        self.assertFalse(report['success']); self.assertEqual(rows['ready']['status'], 'success')
        self.assertGreaterEqual(rows['slow']['registryReadBytes'], 100 * 1024 ** 2)
        self.assertLess(registry.reader.total_bytes, 3 * 1024 ** 3)
        self.assertEqual(registry.tags, ['ready'])
    def test_real_reader_stream_cannot_cross_visibility_window_and_claim_success(self):
        manifest, bodies, registry = fixture([('a', {})]); clock = old.Clock()
        body = bodies['a']
        class Response(io.BytesIO):
            def getcode(self): return 200
            def read(self, size=-1):
                clock.now += 6
                return super().read(size)
        class Opener:
            def open(self, request, timeout):
                if request.full_url.endswith('archive.tgz'): return Response(body)
                value = {'name': 'a', 'version': '2.0.0', 'dist': {
                    'tarball': 'https://registry.npmjs.org/a/archive.tgz',
                    'integrity': observed('a', body)['integrity']}}
                return Response(json.dumps(value).encode())
        registry.reader = module.selector.ExactRegistryReadOnly({'GITHUB_ACTIONS': 'true'}, Opener(), clock)
        actual = module.selector.Registry(registry.reader)
        registry.observe = lambda row, **options: actual.lookup(row, **options) if registry.writes else {'status': 'missing', 'name': 'a', 'version': '2.0.0'}
        report, rows = execute(manifest, bodies, registry, clock=clock, visibility_seconds=5)
        self.assertEqual(rows['a']['status'], 'failed'); self.assertFalse(rows['a']['registryBytesVerified'])
        self.assertEqual(len(registry.writes), 1); self.assertEqual(registry.tags, [])
    def test_retained_latest_current_or_higher_uses_real_writer_no_mutation(self):
        manifest, bodies, registry = fixture([('a', {})], retained=['a'])
        for order in [0, 1]:
            writer = module.publisher.RegistryWriter.__new__(module.publisher.RegistryWriter)
            writer.latest_order = mock.Mock(return_value=order); writer.npm = mock.Mock(side_effect=AssertionError('must not initialize npm'))
            registry.promote_latest = writer.promote_latest; registry.latest_visible = writer.latest_visible
            report, rows = execute(manifest, bodies, registry)
            self.assertTrue(report['success']); writer.npm.assert_not_called(); self.assertEqual(registry.writes, [])
    def test_before_upload_race_after_initial_missing_uses_new_registry_origin(self):
        manifest, bodies, registry = fixture([('a', {})]); original = registry.observe; counts = [0]
        def observe(row, **options):
            counts[0] += 1
            if counts[0] == 2: registry.public['a'] = observed('a', old.archive('a', description='another source'))
            return original(row, **options)
        registry.observe = observe
        report, rows = execute(manifest, bodies, registry)
        self.assertTrue(report['success']); self.assertEqual(registry.writes, [])
        self.assertEqual(rows['a']['originKind'], 'registry-race'); self.assertEqual(registry.tags, ['a'])

    def test_race_existing_never_opens_corrupt_or_missing_local_candidate(self):
        manifest, bodies, registry = fixture([('a', {}), ('independent', {})])
        registry.public['a'] = observed('a', old.archive('a', description='prior existing'))
        bodies['a'] = b'corrupt'
        report, rows = execute(manifest, bodies, registry)
        self.assertTrue(report['success']); self.assertEqual(rows['a']['originKind'], 'registry-race')
        self.assertIn('a', registry.tags); self.assertFalse(rows['a']['registryBytesVerified'])
        self.assertEqual([name for name, _ in registry.writes], ['independent'])
    def test_initial_slow_metadata_cannot_starve_already_ready_retained(self):
        manifest, bodies, registry = fixture([('ready', {}), ('slow', {})], retained=['ready', 'slow'])
        clock = old.Clock(); deadline = module.Deadline(900, clock); original = registry.observe
        def observe(row, **options):
            if row['name'] == 'slow':
                clock.now = 900; deadline.remaining()
            return original(row, **options)
        registry.observe = observe; report = module.new_report(manifest); history = []
        with self.assertRaises(module.PublicationInterrupted):
            module.publish_packages(manifest, lambda row: bodies[row['name']], registry, report,
                                    lambda value: history.append(copy.deepcopy(value)),
                                    clock=clock, sleep=clock.sleep, remaining=deadline.remaining)
        rows = {row['name']: row for row in report['packages']}
        self.assertEqual(rows['ready']['status'], 'success'); self.assertEqual(registry.tags, ['ready'])
        self.assertTrue(any(value['packages'][0]['status'] == 'success' for value in history))

    def main_deadline_fixture(self, retained):
        manifest, bodies, registry = fixture([('ready', {}), ('slow', {})], retained=['ready', 'slow'] if retained else [])
        clock = old.Clock(); deadline = module.Deadline(900, clock); original = registry.observe
        def observe(row, **options):
            if row['name'] == 'slow': clock.now = 900; deadline.remaining()
            return original(row, **options)
        reader = module.selector.ExactRegistryReadOnly({'GITHUB_ACTIONS': 'true'}, clock=clock)
        context = mock.MagicMock(); context.__enter__.return_value = registry
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'manifest.json').write_text(json.dumps(manifest)); output = root / 'result.json'
            (root / 'archives').mkdir()
            for row in manifest['packages']: (root / 'archives' / row['archive']).write_bytes(bodies[row['name']])
            output.write_text('{"success": true}')
            with mock.patch.object(module.sys, 'argv', ['publisher', '--candidate', str(root), '--output', str(output)]), \
                 mock.patch.dict(os.environ, {'GITHUB_RUN_ID': '10', 'GITHUB_RUN_ATTEMPT': '1'}, clear=True), \
                 mock.patch.object(module, 'authorize'), mock.patch.object(module, 'Deadline', return_value=deadline), \
                 mock.patch.object(module.publisher, 'RegistryWriter', return_value=context), \
                 mock.patch.object(module.selector, 'ExactRegistryReadOnly', return_value=reader), \
                 mock.patch.object(module.selector, 'Registry', return_value=SimpleNamespace(lookup=observe)):
                self.assertEqual(module.main(), 1)
            result = json.loads(output.read_text()); rows = {row['name']: row for row in result['packages']}
            self.assertFalse(result['success']); self.assertFalse(result['complete'])
            self.assertEqual(rows['ready']['status'], 'success'); self.assertEqual(rows['slow']['status'], 'failed')
            self.assertEqual(registry.tags, ['ready']); self.assertEqual(len(result['packages']), 2)
    def test_main_real_deadline_persists_retained_ready_before_slow_initial_metadata(self):
        self.main_deadline_fixture(True)
    def test_main_real_deadline_persists_new_ready_before_slow_initial_metadata(self):
        self.main_deadline_fixture(False)
    def test_v2_finalizer_dispatch_uses_presence_proof_not_legacy_byte_recovery(self):
        manifest, _, _ = fixture(retained=['a', 'b', 'independent'])
        verifier = module.SourceProofVerifier(mock.Mock(), {'runId': 10, 'attempt': 1})
        with mock.patch.object(module.entry.ReleaseVerifier, 'verify', return_value={'verified': True}) as current, \
             mock.patch.object(module.publisher.RecoveryVerifier, 'verify', side_effect=AssertionError('legacy')):
            self.assertEqual(verifier.verify(manifest, Path('.'), phase='postpublish'), {'verified': True})
        self.assertEqual(current.call_args.kwargs['phase'], 'postpublish')
        self.assertEqual(current.call_args.kwargs['current_run'], {'runId': 10, 'attempt': 1})

    def test_mixed_auth_and_transport_error_never_claims_definite_rejection(self):
        manifest, bodies, registry = fixture([('a', {})])
        error = module.publisher.npm_failure('publish', manifest['packages'][0], code=1,
                    stderr='npm error code ENEEDAUTH\nnpm error code ECONNRESET')
        self.assertFalse(module.definite_rejection(error))
        registry.publish = mock.Mock(side_effect=error)
        report, rows = execute(manifest, bodies, registry, visibility_seconds=2)
        self.assertFalse(rows['a']['publishOutcome']['definiteRejection'])
        self.assertTrue(rows['a']['outcomeUncertain']); self.assertEqual(registry.publish.call_count, 1)

    def test_real_main_rerun_blocks_accepted_but_not_public_submission(self):
        manifest, bodies, registry = fixture([('a', {})])
        registry.observe = lambda row, **options: {'status': 'missing', 'name': 'a', 'version': '2.0.0'}
        first, rows = execute(manifest, bodies, registry, visibility_seconds=1)
        self.assertTrue(rows['a']['publishOutcome']['accepted']); self.assertTrue(rows['a']['outcomeUncertain'])
        self.assertEqual(len(registry.writes), 1)
        for attempt in ['2', '3']:
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory); (root / 'manifest.json').write_text(json.dumps(manifest)); output = root / 'result.json'
                with mock.patch.object(module.sys, 'argv', ['publisher', '--candidate', str(root), '--output', str(output)]), \
                     mock.patch.dict(os.environ, {'GITHUB_RUN_ID': '10', 'GITHUB_RUN_ATTEMPT': attempt}, clear=True), \
                     mock.patch.object(module, 'authorize') as auth, \
                     mock.patch.object(module.publisher, 'RegistryWriter') as writer, \
                     mock.patch.object(module.selector, 'Registry') as reader:
                    self.assertEqual(module.main(), 1)
                auth.assert_not_called(); writer.assert_not_called(); reader.assert_not_called()
                report = json.loads(output.read_text()); self.assertFalse(report['success']); self.assertFalse(report['complete'])
                self.assertEqual(report['fatal']['code'], 'rerun-publication-disabled')
                self.assertTrue(report['previousAttemptUnreconciled'])
                self.assertFalse(report['packages'][0]['outcomeUncertain']); self.assertFalse(report['packages'][0]['uploadAttempted'])
                history = load('rerun_history_integration', 'release-version-history.py')
                self.assertFalse(history.possible_submission(report['packages'][0]))
                self.assertTrue(history.possible_submission(first['packages'][0]))
        self.assertEqual(len(registry.writes), 1)
    def test_authorize_rerun_blocks_before_source_proof_or_registry(self):
        manifest, _, _ = fixture()
        env = {'FARM_EXPLICIT_PUBLICATION': 'true', 'GITHUB_JOB': 'publisher', 'GITHUB_RUN_ATTEMPT': '2'}
        with mock.patch.object(module.entry, 'actions_transport') as transport:
            with self.assertRaisesRegex(ValueError, 'Repeated publication'):
                module.authorize(manifest, Path('.'), env)
        transport.assert_not_called()
    def test_legacy_full_source_main_rerun_also_refuses_registry_writes(self):
        manifest, _, _ = old.Contracts().fixture()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'manifest.json').write_text(json.dumps(manifest)); output = root / 'result.json'
            with mock.patch.object(module.sys, 'argv', ['publisher', '--candidate', str(root), '--output', str(output)]), \
                 mock.patch.dict(os.environ, {'GITHUB_RUN_ID': '10', 'GITHUB_RUN_ATTEMPT': '2'}, clear=True), \
                 mock.patch.object(module, 'authorize') as auth, mock.patch.object(module.publisher, 'RegistryWriter') as writer:
                self.assertEqual(module.main(), 1)
            auth.assert_not_called(); writer.assert_not_called()
            self.assertEqual(len(json.loads(output.read_text())['packages']), 3)


if __name__ == '__main__': unittest.main(verbosity=2)
