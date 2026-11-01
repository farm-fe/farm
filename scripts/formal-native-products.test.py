"""Offline resolver integration tests: real source graph, synthetic API/bytes."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location('formal', Path(__file__).with_name('formal-native-products.py'))
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
ABI = 'darwin-arm64'


class FormalProducts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = formal.evidence.source_reader(formal.chain.impact.git('rev-parse', 'HEAD').decode().strip())
        cls.products = formal.product_inputs(cls.source, ABI)

    def context(self):
        return {'toolchain': self.products['@farmfe/core']['recipe']['toolchain'], 'rustc': 'pinned rustc',
                'cargo': 'pinned cargo', 'linker': {'version': 'Apple clang'},
                'platform': {'sdk': '27.2', 'host': {'node': 'v22.20.0', 'os': 'darwin', 'arch': 'arm64', 'release': '27.2'}},
                'formalFlagsHash': 'f' * 64}

    def fixture(self, transform=None):
        context = self.context()
        records, bodies = [], {}
        for index, product in enumerate(self.products.values()):
            member = str(index) + '.node'
            body = b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01' + ('synthetic byte fixture for ' + product['owner']).encode()
            bodies[member] = body
            records.append({**copy.deepcopy(product), 'member': member, 'bytes': len(body), 'sha256': hashlib.sha256(body).hexdigest()})
        manifest = {'schemaVersion': 2, 'kind': 'main-formal-native-products', 'sourceSHA': self.source['sha'],
                    'sourceTree': self.source['tree'], 'runId': 10, 'attempt': 2, 'abi': ABI, 'context': context, 'records': records}
        if transform: transform(manifest)
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, 'w') as archive:
            archive.writestr('catalog.json', json.dumps(manifest))
            for name, body in bodies.items(): archive.writestr(name, body)
        body = buffer.getvalue()
        run = {'id': 10, 'path': formal.WORKFLOW, 'head_branch': 'main', 'event': 'push', 'status': 'completed',
               'conclusion': 'success', 'repository': {'id': 1}, 'head_repository': {'id': 1}, 'run_attempt': 2, 'head_sha': self.source['sha']}
        artifact = {'id': 20, 'workflow_run': {'id': 10}, 'expired': False, 'name': 'formal-native-' + ABI + '-2',
                    'size_in_bytes': len(body), 'digest': 'sha256:' + hashlib.sha256(body).hexdigest(), 'created_at': '2026-11-01T10:00:01Z'}
        job = {'run_id': 10, 'run_attempt': 2, 'name': 'Build and Upload User Artifacts - ' + ABI, 'status': 'completed', 'conclusion': 'success',
               'steps': [{'name': formal.UPLOAD, 'status': 'completed', 'conclusion': 'success',
                          'started_at': '2026-11-01T10:00:00Z', 'completed_at': '2026-11-01T10:00:02Z'}]}
        desired = {**context, 'repositoryId': 1}
        return body, artifact, run, job, desired

    def inspect(self, values, current=None):
        body, artifact, run, job, desired = values
        return formal.inspect_catalog(body, artifact, run, job, self.source, current or self.source, ABI, desired)

    def test_matching_main_products_reused_without_test_acceptance(self):
        accepted = self.inspect(self.fixture())
        self.assertEqual(set(accepted), set(self.products))
        self.assertTrue(all(value['originSHA'] == self.source['sha'] for value in accepted.values()))

    def test_plugin_source_change_preserves_core_key(self):
        current = copy.deepcopy(self.source)
        source_file = next(file for file in current['files'] if file.startswith('rust-plugins/replace-dirname/src/') and file.endswith('.rs'))
        current['files'][source_file] += b'\n// changed input\n'
        changed = formal.product_inputs(current, ABI)
        self.assertEqual(changed['@farmfe/core']['contentKey'], self.products['@farmfe/core']['contentKey'])
        owner = next(owner for owner, row in changed.items() if row['contentKey'] != self.products[owner]['contentKey'])
        accepted = self.inspect(self.fixture(), current)
        self.assertNotIn(owner, accepted)
        self.assertIn('@farmfe/core', accepted)

    def test_symlink_ancestor_never_creates_outside_directory(self):
        with tempfile.TemporaryDirectory() as root, tempfile.TemporaryDirectory() as outside:
            Path(root, 'redirect').symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, 'redirects'):
                formal.workspace_path(root, 'redirect/new/output.node', create_parents=True)
            self.assertFalse(Path(outside, 'new').exists())

    def test_binary_architecture_rejection(self):
        self.assertTrue(formal.native_matches(b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01', ABI))
        self.assertFalse(formal.native_matches(b'\xcf\xfa\xed\xfe\x07\x00\x00\x01', ABI))
        self.assertFalse(formal.native_matches(b'not a binary', ABI))

    def test_create_binding_and_embedded_templates_invalidate_create(self):
        for prefix in ['packages/create-farm/src/', 'crates/create-farm-rs/templates/']:
            current = copy.deepcopy(self.source)
            file = next(file for file in current['files'] if file.startswith(prefix))
            current['files'][file] += b' changed'
            changed = formal.product_inputs(current, ABI)
            self.assertNotEqual(changed['create-farm']['contentKey'], self.products['create-farm']['contentKey'])

    def test_main_sha_is_provenance_not_content_key(self):
        current = {**self.source, 'sha': '1' * 40, 'tree': '2' * 40}
        self.assertEqual(formal.product_inputs(current, ABI), self.products)

    def test_wrong_profile_fingerprint_rejected(self):
        values = self.fixture(lambda manifest: manifest['records'][0]['recipe'].update(profile='ci'))
        with self.assertRaisesRegex(ValueError, 'fingerprint'): self.inspect(values)

    def test_wrong_manifest_abi_rejected(self):
        with self.assertRaisesRegex(ValueError, 'ABI'):
            self.inspect(self.fixture(lambda manifest: manifest.update(abi='darwin-x64')))

    def test_wrong_job_attempt_rejected(self):
        values = self.fixture(); values[3]['run_attempt'] = 1
        with self.assertRaisesRegex(ValueError, 'attempt'): self.inspect(values)

    def test_expired_damaged_fork_or_failed_producer_rejected(self):
        for field, value in [('expired', True), ('digest', 'sha256:' + '0' * 64)]:
            with self.subTest(field=field):
                values = self.fixture(); values[1][field] = value
                with self.assertRaises(ValueError): self.inspect(values)
        for update in [{'conclusion': 'failure'}, {'head_repository': {'id': 9}}, {'head_branch': 'feature'}]:
            values = self.fixture(); values[2].update(update)
            with self.assertRaises(ValueError): self.inspect(values)

    def test_measured_compiler_mismatch_or_missing_rebuilds(self):
        for field in ['toolchain', 'rustc', 'cargo', 'linker']:
            for missing in [False, True]:
                with self.subTest(field=field, missing=missing):
                    values = self.fixture()
                    if missing: del values[4][field]
                    else: values[4][field] = 'unmatched'
                    with self.assertRaisesRegex(ValueError, 'compiler or linker'): self.inspect(values)

    def test_environment_mismatch_rebuilds(self):
        values = self.fixture(); values[4]['formalFlagsHash'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'flags'): self.inspect(values)

    def test_latest_failed_attempt_does_not_read_old_attempt(self):
        requests = []; source = self.source
        class Transport:
            def get(self, route):
                requests.append(route)
                if route == '/commits/main': return {'sha': source['sha']}
                if '/workflows/' in route: return {'workflow_runs': [{'id': 10, 'run_attempt': 2}]}
                if route == '/actions/runs/10': return {'status': 'completed', 'conclusion': 'failure', 'run_attempt': 2}
                raise AssertionError('Unexpected request ' + route)
        accepted, reasons = formal.resolve_products(Transport(), self.source, ABI, self.context())
        self.assertEqual(accepted, {})
        self.assertIn('latest-producer-attempt-not-successful', reasons)
        self.assertFalse(any('/attempts/' in route for route in requests))

    def test_missing_artifact_is_build_miss(self):
        _, _, run, _, _ = self.fixture(); source = self.source
        class Transport:
            def get(self, route):
                if route == '/commits/main': return {'sha': source['sha']}
                if route.startswith('/compare/'): return {'status': 'identical', 'merge_base_commit': {'sha': source['sha']}}
                if '/workflows/' in route: return {'workflow_runs': [{'id': 10, 'run_attempt': 2}]}
                if route == '/actions/runs/10': return run
                if '/artifacts?' in route: return {'artifacts': []}
                raise AssertionError(route)
        accepted, reasons = formal.resolve_products(Transport(), self.source, ABI, self.context())
        self.assertEqual(accepted, {})
        self.assertIn('artifact-missing-or-expired', reasons)


if __name__ == '__main__': unittest.main()
