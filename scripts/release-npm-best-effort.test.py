"""Offline full-family publication contracts. No real registry writes."""
# cspell:words importlib readback USTAR ENEEDAUTH
import copy
import gzip
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('best_effort_tests', Path(__file__).with_name('release-npm-best-effort.py'))
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)


def archive(name, **fields):
    package = {'name': name, 'version': '2.0.0', 'repository': 'https://github.com/farm-fe/farm', **fields}
    raw = json.dumps(package).encode(); stream = io.BytesIO()
    with tarfile.open(fileobj=stream, mode='w', format=tarfile.USTAR_FORMAT) as tar:
        member = tarfile.TarInfo('package/package.json'); member.size = len(raw)
        tar.addfile(member, io.BytesIO(raw))
    return gzip.compress(stream.getvalue(), mtime=0)


class Registry:
    def __init__(self):
        self.bytes, self.writes, self.tags = {}, [], []
        self.fail, self.lost, self.read_fail, self.tag_fail = set(), set(), set(), set()
    def lookup(self, row):
        if row['name'] in self.read_fail: raise OSError('private bearer sentinel')
        return self.bytes.get(row['name'])
    def publish(self, row, body):
        self.writes.append((row['name'], body))
        if row['name'] in self.fail:
            raise module.publisher.npm_failure('publish', row, code=1, stderr='npm error code ENEEDAUTH\nprivate bearer sentinel')
        assert row['name'] not in self.bytes
        self.bytes[row['name']] = hashlib.sha256(body).hexdigest()
        if row['name'] in self.lost: raise TimeoutError('private bearer sentinel')
    def confirm(self, row):
        if row['name'] in self.tag_fail: raise OSError('private bearer sentinel')
        self.tags.append(row['name'])


class Contracts(unittest.TestCase):
    def fixture(self, packages=None):
        packages = packages or [('dependency', {}), ('consumer', {'dependencies': {'dependency': '2.0.0'}}), ('independent', {})]
        bodies, rows = {}, []
        for index, (name, metadata) in enumerate(packages):
            body = archive(name, **metadata); bodies[name] = body
            rows.append({'kind': 'npm', 'name': name, 'version': '2.0.0', 'archive': str(index) + '.tgz',
                         'archiveBytes': len(body), 'archiveSha256': hashlib.sha256(body).hexdigest()})
        names = ['npm:' + row['name'] for row in rows]
        manifest = {'chain': 'release', 'profile': 'release', 'namespace': 'farm-release-v1', 'publicationScope': 'npm',
                    'sourceSHA': 'a' * 40, 'sourceTree': 'b' * 40, 'planHash': 'c' * 64, 'baseline': None,
                    'producerRunId': 10, 'producerAttempt': 1, 'publish': names, 'rebuild': names,
                    'family': [{'kind': row['kind'], 'name': row['name']} for row in rows], 'packages': rows}
        return manifest, bodies, Registry()
    def run_fixture(self, manifest, bodies, registry):
        report = module.new_report(manifest); history = []
        module.publish_packages(manifest, lambda row: bodies[row['name']], registry, report,
                                lambda value: history.append(copy.deepcopy(value)))
        self.assertFalse(any(item['success'] for item in history[:-1]))
        self.assertTrue(all(item['baselineAdvanced'] is False and item['releaseCreatedByThisStage'] is False for item in history))
        self.assertEqual(len(report['packages']), len(manifest['packages']))
        return report, {row['name']: row for row in report['packages']}
    def test_partial_auth_failure_continues_independent_and_blocks_required_dependency(self):
        manifest, bodies, registry = self.fixture(); registry.fail.add('dependency')
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertFalse(report['success']); self.assertTrue(report['complete'])
        self.assertEqual(rows['independent']['status'], 'success')
        self.assertEqual(rows['consumer']['status'], 'failed'); self.assertFalse(rows['consumer']['uploadAttempted'])
        self.assertEqual([name for name, _ in registry.writes], ['dependency', 'independent'])
        self.assertIn('ENEEDAUTH', json.dumps(report)); self.assertNotIn('private bearer sentinel', json.dumps(report))
    def test_297_targets_include_48_failures_and_249_successes(self):
        manifest, bodies, registry = self.fixture([('package-' + str(i), {}) for i in range(297)])
        registry.fail = {'package-' + str(i) for i in range(48)}
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual(report['counts'], {'total': 297, 'success': 249, 'failed': 48})
        self.assertEqual(len(registry.writes), 297); self.assertFalse(report['success'])
    def test_immutable_conflict_is_never_overwritten_and_does_not_stop_unrelated(self):
        manifest, bodies, registry = self.fixture(); registry.bytes['dependency'] = 'f' * 64
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual(rows['dependency']['errors'][0]['code'], 'immutable-version-conflict')
        self.assertEqual([name for name, _ in registry.writes], ['independent'])
        self.assertEqual(registry.bytes['dependency'], 'f' * 64); self.assertFalse(report['success'])
    def test_lost_response_is_read_back_and_never_retried_but_still_fails_job(self):
        manifest, bodies, registry = self.fixture(); registry.lost.add('dependency')
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertTrue(rows['dependency']['registryBytesVerified']); self.assertEqual(rows['dependency']['status'], 'failed')
        self.assertEqual(rows['consumer']['status'], 'success'); self.assertFalse(report['success'])
        self.assertEqual([name for name, _ in registry.writes].count('dependency'), 1)
    def test_tag_failure_does_not_block_dependency_with_verified_immutable_bytes(self):
        manifest, bodies, registry = self.fixture(); registry.tag_fail.add('dependency')
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual(rows['dependency']['status'], 'failed'); self.assertEqual(rows['consumer']['status'], 'success')
        self.assertFalse(report['success'])
    def test_registry_read_failure_is_not_absence(self):
        manifest, bodies, registry = self.fixture(); registry.read_fail.add('dependency')
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual([name for name, _ in registry.writes], ['independent'])
        self.assertFalse(report['success']); self.assertNotIn('private bearer sentinel', json.dumps(report))
    def test_exact_bytes_repeated_runs_read_registry_not_stale_success(self):
        manifest, bodies, registry = self.fixture()
        report, _ = self.run_fixture(manifest, bodies, registry); self.assertTrue(report['success'])
        self.assertEqual(dict(registry.writes), bodies)
        second, rows = self.run_fixture(manifest, bodies, registry)
        self.assertTrue(second['success']); self.assertEqual(len(registry.writes), 3)
        self.assertEqual({row['outcome'] for row in rows.values()}, {'already-published'})
        registry.bytes['dependency'] = 'f' * 64
        third, rows = self.run_fixture(manifest, bodies, registry)
        self.assertFalse(third['success']); self.assertEqual(rows['dependency']['status'], 'failed')
    def test_changed_upload_body_cannot_reach_registry(self):
        manifest, bodies, registry = self.fixture([('one', {}), ('two', {})]); reads = {}
        def reader(row):
            name = row['name']; reads[name] = reads.get(name, 0) + 1
            return b'changed' if name == 'one' and reads[name] > 1 else bodies[name]
        report = module.new_report(manifest)
        module.publish_packages(manifest, reader, registry, report, lambda _: None)
        self.assertEqual([name for name, _ in registry.writes], ['two']); self.assertFalse(report['success'])
    def test_required_peer_is_verified_optional_payload_is_not_required(self):
        manifest, bodies, registry = self.fixture([('consumer', {'peerDependencies': {'dependency': '^2.0.0'},
                         'optionalDependencies': {'optional': '2.0.0'}}), ('dependency', {}), ('optional', {})])
        registry.fail.add('optional'); report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual(rows['consumer']['status'], 'success'); self.assertFalse(report['success'])
        self.assertIn('consumer', registry.tags)
    def test_real_docs_theme_shape_peer_cycle_uploads_and_verifies_before_tags(self):
        manifest, bodies, registry = self.fixture([('a', {'peerDependencies': {'b': '2.0.0'}}),
                  ('b', {'dependencies': {'a': '2.0.0'}}), ('independent', {})])
        original = registry.confirm
        def confirm(row):
            self.assertTrue({'a', 'b'}.issubset(registry.bytes))
            original(row)
        registry.confirm = confirm
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual([name for name, _ in registry.writes], ['a', 'b', 'independent'])
        self.assertTrue(report['success'])
    def test_failed_required_peer_prevents_latest_without_stopping_unrelated(self):
        manifest, bodies, registry = self.fixture([('a', {'peerDependencies': {'b': '2.0.0'}}), ('b', {}), ('independent', {})])
        registry.fail.add('b'); report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual(rows['a']['status'], 'failed'); self.assertTrue(rows['a']['registryBytesVerified'])
        self.assertNotIn('a', registry.tags); self.assertEqual(rows['independent']['status'], 'success')
    def test_transitive_failed_peer_prevents_all_affected_latest_tags(self):
        manifest, bodies, registry = self.fixture([('a', {'dependencies': {'b': '2.0.0'}}),
                  ('b', {'peerDependencies': {'c': '2.0.0'}}), ('c', {}), ('independent', {})])
        registry.fail.add('c'); report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertFalse(report['success']); self.assertEqual(registry.tags, ['independent'])
        self.assertEqual({name: row['status'] for name, row in rows.items()},
                         {'a': 'failed', 'b': 'failed', 'c': 'failed', 'independent': 'success'})
        self.assertTrue(rows['a']['registryBytesVerified']); self.assertTrue(rows['b']['registryBytesVerified'])
    def test_hard_dependency_cycle_does_not_upload_its_members(self):
        manifest, bodies, registry = self.fixture([('a', {'dependencies': {'b': '2.0.0'}}),
                  ('b', {'dependencies': {'a': '2.0.0'}}), ('independent', {})])
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual([name for name, _ in registry.writes], ['independent'])
        self.assertFalse(report['success']); self.assertEqual(rows['a']['errors'][0]['stage'], 'required-dependency-cycle')
    def test_duplicate_incomplete_or_historical_inventory_fails_before_writes(self):
        manifest, _, _ = self.fixture()
        for mutate in [lambda m: m['packages'].append(m['packages'][0]), lambda m: m['packages'].pop(),
                       lambda m: m.update(baseline='d' * 64), lambda m: m.update(registrySnapshot={})]:
            value = copy.deepcopy(manifest); mutate(value)
            with self.assertRaises(ValueError): module.new_report(value)
    def test_bad_package_archive_does_not_stop_independent_packages(self):
        manifest, bodies, registry = self.fixture(); bodies['independent'] = b'corrupted'
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual([name for name, _ in registry.writes], ['dependency', 'consumer'])
        self.assertFalse(report['success']); self.assertEqual(rows['independent']['status'], 'failed')
    def test_invalid_required_range_does_not_stop_independent_packages(self):
        manifest, bodies, registry = self.fixture([('consumer', {'dependencies': {'dependency': 'invalid'}}),
                                                  ('dependency', {}), ('independent', {})])
        report, rows = self.run_fixture(manifest, bodies, registry)
        self.assertEqual([name for name, _ in registry.writes], ['dependency', 'independent'])
        self.assertFalse(report['success']); self.assertEqual(rows['consumer']['status'], 'failed')
    def test_main_partial_failure_exits_one_and_keeps_complete_report(self):
        manifest, bodies, registry = self.fixture(); registry.fail.add('dependency')
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'archives').mkdir()
            (root / 'manifest.json').write_text(json.dumps(manifest))
            for row in manifest['packages']: (root / 'archives' / row['archive']).write_bytes(bodies[row['name']])
            registry.reader = module.entry.evidence.registry_module.RegistryReadOnly(environment={'GITHUB_ACTIONS': 'true'})
            context = mock.MagicMock(); context.__enter__.return_value = registry
            output = root / 'result.json'; output.write_text('{"success": true}')
            with mock.patch.object(module.sys, 'argv', ['publisher', '--candidate', str(root), '--output', str(output)]), \
                 mock.patch.dict(os.environ, {'GITHUB_RUN_ID': '10', 'GITHUB_RUN_ATTEMPT': '2'}, clear=True), \
                 mock.patch.object(module, 'authorize') as authorization, \
                 mock.patch.object(module.publisher, 'RegistryWriter', return_value=context):
                self.assertEqual(module.main(), 1)
            authorization.assert_called_once(); result = json.loads(output.read_text())
            self.assertTrue(result['complete']); self.assertFalse(result['success'])
            self.assertEqual(result['counts']['total'], 3)
    def test_bad_proof_fails_all_unattempted_without_opening_writer(self):
        manifest, _, _ = self.fixture()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'manifest.json').write_text(json.dumps(manifest))
            output = root / 'result.json'
            with mock.patch.object(module.sys, 'argv', ['publisher', '--candidate', str(root), '--output', str(output)]), \
                 mock.patch.dict(os.environ, {'GITHUB_RUN_ID': '10', 'GITHUB_RUN_ATTEMPT': '1'}, clear=True), \
                 mock.patch.object(module, 'authorize', side_effect=ValueError('private bearer sentinel')), \
                 mock.patch.object(module.publisher, 'RegistryWriter') as writer:
                self.assertEqual(module.main(), 1)
            writer.assert_not_called(); report = json.loads(output.read_text())
            self.assertFalse(report['complete']); self.assertFalse(report['success'])
            self.assertTrue(all(row['uploadAttempted'] is False for row in report['packages']))
            self.assertNotIn('private bearer sentinel', output.read_text())
    def test_workflow_uses_full_source_without_durable_state_and_always_uploads(self):
        workflow = (Path(__file__).parents[1] / '.github/workflows/incremental-release.yaml').read_text()
        self.assertIn('plan --full-source --source', workflow)
        self.assertIn('group: farm-independent-release\n  cancel-in-progress: false', workflow)
        finalizer = workflow.split('\n  finalize:\n')[1].split('\n  sign-frozen-release:')[0]
        self.assertIn('release-npm-best-effort.py --finalize', finalizer)
        self.assertIn("needs.publisher.result == 'success'", finalizer)
        self.assertIn("needs.sign-public-acceptance.result == 'success'", finalizer)
        self.assertNotIn('incremental-release-publish.py finalize', finalizer)
        self.assertNotIn('incremental-release-publish.py upload', workflow)
        block = workflow.split('\n  publisher:\n')[1].split('\n  public-proof:')[0]
        self.assertIn('contents: read\n      actions: read\n      id-token: write', block)
        self.assertIn('Upload all package outcomes even when publication fails\n        if: always()', block)
        self.assertIn('Preserve preparation failure when publication could not start\n        if: always()', block)
        self.assertNotIn('continue-on-error:', block)
        self.assertIn('time.monotonic() + 75 * 60', block)
        self.assertIn('timeout-minutes: 90', block)
        self.assertIn('Reject manual publication outside the trusted main push signer', workflow)
        policy = module.entry.load('best_effort_template_policy', 'ci-attested-bindings.py')
        root = Path(__file__).parents[1]; path = '.github/workflows/incremental-release.yaml'
        body = (root / path).read_bytes()
        for scope in ['frozen', 'registry', 'native', 'public']:
            mirror = root / 'scripts/release-provenance-templates' / scope / path
            policy.verify_caller_templates({path: body}, {path: mirror.read_bytes()}, 'a' * 40)
    def test_real_registry_budget_accepts_deadline_conversion_and_bounds_calls(self):
        now = [10.0]; deadline = module.Deadline(seconds=1.5, clock=lambda: now[0])
        reader = module.entry.evidence.registry_module.RegistryReadOnly(environment={'GITHUB_ACTIONS': 'true'}, clock=lambda: now[0])
        with reader.budget(deadline.registry_seconds()):
            self.assertEqual(reader.remaining(), 2)
            with mock.patch.object(module.subprocess, 'run') as command:
                deadline.command(['fixture'], timeout=120)
                self.assertEqual(command.call_args.kwargs['timeout'], 1.5)
            now[0] = 12
            with self.assertRaises(module.PublicationInterrupted): deadline.remaining()
            now[0] = 10
        with self.assertRaisesRegex(ValueError, 'budget'):
            with reader.budget(1.5): pass
    def test_total_byte_budget_covers_authenticated_family_rechecks(self):
        manifest, bodies, registry = self.fixture([('package-' + str(i), {}) for i in range(10)])
        # Size-only simulator avoids allocating a multi-gigabyte test fixture;
        # the real reader's byte-limit branch below uses real small bytes.
        class SizedBody(bytes):
            def __len__(self): return 90 * 1024 ** 2
        for row in manifest['packages']: row['archiveBytes'] = 90 * 1024 ** 2
        registry.reader = module.entry.evidence.registry_module.RegistryReadOnly(environment={'GITHUB_ACTIONS': 'true'})
        for row in manifest['packages']: registry.bytes[row['name']] = row['archiveSha256']
        original = registry.lookup
        def lookup(row):
            registry.reader.total_bytes += row['archiveBytes']
            self.assertLessEqual(registry.reader.total_bytes, registry.reader.total_byte_limit)
            return original(row)
        registry.lookup = lookup
        with mock.patch.object(module.publisher.npm_tarball, 'validate', side_effect=lambda row, body: {'name': row['name']}):
            report = module.new_report(manifest)
            module.publish_packages(manifest, lambda row: SizedBody(bodies[row['name']]), registry, report, lambda _: None)
        self.assertTrue(report['success']); self.assertGreater(registry.reader.total_bytes, 3 * 1024 ** 3)
        reader = module.entry.evidence.registry_module.RegistryReadOnly(environment={'GITHUB_ACTIONS': 'true'}, total_byte_limit=4)
        with mock.patch.object(reader, 'open', return_value=io.BytesIO(b'12345')):
            with self.assertRaisesRegex(ValueError, 'byte budget'): reader.archive('https://registry.npmjs.org/test')
    def test_deadline_after_started_upload_is_recorded_and_not_retried(self):
        manifest, bodies, registry = self.fixture([('a', {}), ('b', {})])
        def interrupt(row, body):
            registry.writes.append((row['name'], body))
            raise module.PublicationInterrupted('deadline')
        registry.publish = interrupt
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'archives').mkdir()
            (root / 'manifest.json').write_text(json.dumps(manifest))
            for row in manifest['packages']: (root / 'archives' / row['archive']).write_bytes(bodies[row['name']])
            registry.reader = module.entry.evidence.registry_module.RegistryReadOnly(environment={'GITHUB_ACTIONS': 'true'})
            context = mock.MagicMock(); context.__enter__.return_value = registry; output = root / 'result.json'
            with mock.patch.object(module.sys, 'argv', ['publisher', '--candidate', str(root), '--output', str(output)]), \
                 mock.patch.dict(os.environ, {'GITHUB_RUN_ID': '10', 'GITHUB_RUN_ATTEMPT': '2'}, clear=True), \
                 mock.patch.object(module, 'authorize'), mock.patch.object(module.publisher, 'RegistryWriter', return_value=context):
                self.assertEqual(module.main(), 1)
            report = json.loads(output.read_text()); self.assertFalse(report['complete']); self.assertFalse(report['success'])
            self.assertEqual([row['uploadAttempted'] for row in report['packages']], [True, False])
            self.assertEqual(len(registry.writes), 1)
    def test_finalizer_requires_complete_proof_and_never_opens_state_store(self):
        manifest, _, _ = self.fixture(); proof = {'verified': True}
        with mock.patch.object(module, 'authorize', return_value=proof) as authorize, \
             mock.patch.object(module.publisher, 'GitHubWriter') as writer, \
             mock.patch.object(module.publisher, 'confirm_release', return_value={'verified': True}) as confirm, \
             mock.patch.object(module.publisher.state, 'GitRefStore', side_effect=AssertionError('state forbidden')):
            self.assertTrue(module.finalize_release(manifest, Path('.'), {})['verified'])
        self.assertTrue(authorize.call_args.kwargs['finalize'])
        confirm.assert_called_once_with(writer.return_value, manifest, proof)
        with mock.patch.object(module, 'authorize', side_effect=ValueError('failed public proof')), \
             mock.patch.object(module.publisher, 'confirm_release') as confirm:
            with self.assertRaisesRegex(ValueError, 'failed public proof'): module.finalize_release(manifest, Path('.'), {})
        confirm.assert_not_called()


if __name__ == '__main__':
    unittest.main(verbosity=2)
