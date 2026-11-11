# cspell:words retagged readback nonterminal
"""Offline temporary publication contracts; no real credentials or registry writes."""
# cspell:words importlib npmjs USERCONFIG GLOBALCONFIG ENEEDAUTH
import copy
import gzip
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import shutil
import sys
import subprocess
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock
import urllib.error

spec = importlib.util.spec_from_file_location('temporary_publication_tests', Path(__file__).with_name('temporary-new-packages.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def archive(name, **fields):
    metadata = {'name': name, 'version': '0.0.1', 'repository': 'https://github.com/farm-fe/farm', **fields}
    stream = io.BytesIO()
    with tarfile.open(fileobj=stream, mode='w', format=tarfile.USTAR_FORMAT) as output:
        body = json.dumps(metadata).encode()
        member = tarfile.TarInfo('package/package.json')
        member.size = len(body)
        output.addfile(member, io.BytesIO(body))
    return gzip.compress(stream.getvalue(), mtime=0)


class RegistryVisibility(unittest.TestCase):
    def test_publication_readback_polls_without_republishing(self):
        writer = mock.Mock()
        writer.verify.side_effect = [False, False, True]
        delays = []
        self.assertTrue(module.wait_for_registry(writer, {'name': 'test'}, delays.append))
        self.assertEqual(delays, [1, 2])
        writer.publish.assert_not_called()

    def test_polling_is_bounded_and_propagates_lookup_errors(self):
        writer = mock.Mock()
        writer.verify.return_value = False
        delays = []
        self.assertFalse(module.wait_for_registry(writer, {'name': 'test'}, delays.append))
        self.assertEqual(delays, [1, 2, 4, 8, 15])
        writer.verify.side_effect = OSError('registry unavailable')
        with self.assertRaises(OSError):
            module.wait_for_registry(writer, {'name': 'test'}, delays.append)
        writer.publish.assert_not_called()


class FakeRegistry:
    def __init__(self, bodies):
        self.bodies = bodies
        self.existing = set()
        self.writes, self.reads = [], []
        self.failed, self.lost, self.read_fail, self.race, self.conflicts = set(), set(), set(), set(), set()
        self.calls = {}

    def exists(self, row):
        name = row['name']
        self.calls[name] = self.calls.get(name, 0) + 1
        if name in self.read_fail:
            raise OSError('private-sentinel')
        if name in self.race and self.calls[name] == 2:
            self.existing.add(name)
        return name in self.existing

    def publish(self, row, body):
        name = row['name']
        assert body == self.bodies[name]
        assert name not in self.existing
        self.writes.append(name)
        if name in self.failed:
            raise module.publisher.npm_failure('publish', row, code=1,
                stderr='npm error code ENEEDAUTH\nprivate-sentinel')
        self.existing.add(name)
        if name in self.lost:
            raise TimeoutError('private-sentinel')

    def verify(self, row):
        self.reads.append(row['name'])
        if row['name'] in self.conflicts:
            raise ValueError('Immutable registry version has different bytes')
        return row['name'] in self.existing


class PublicationContracts(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='farm-temporary-publish-test-')
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        (self.directory / 'archives').mkdir()
        self.bodies, rows = {}, []
        for index, name in enumerate(sorted(module.ALLOWLIST)):
            body = archive(name)
            self.bodies[name] = body
            row = {'kind': 'npm', 'name': name, 'version': '0.0.1', 'archive': str(index) + '.tgz',
                   'archiveBytes': len(body), 'archiveSha256': hashlib.sha256(body).hexdigest()}
            rows.append(row)
            (self.directory / 'archives' / row['archive']).write_bytes(body)
        self.manifest = {'sourceSHA': 'a' * 40, 'packages': rows}
        self.registry = FakeRegistry(self.bodies)
        self.history = []

    def execute(self, manifest=None):
        report = module.publish_new_packages(self.manifest if manifest is None else manifest,
            self.directory, self.registry, lambda value: self.history.append(copy.deepcopy(value)))
        self.assertFalse(any(row.get('success') for row in self.history[:-1]))
        return report, {row['name']: row for row in report['packages']}

    def test_allowlist_is_exactly_48_fixed_new_names(self):
        platforms = ['darwin-arm64', 'darwin-x64', 'linux-arm64-gnu', 'linux-x64-gnu', 'win32-x64-msvc']
        plugins = ['docs', 'docs-seo', 'docs-snapshot', 'docs-snippets', 'ssr-css', 'syntax-highlight']
        expected = {'@farmfe/docs', '@farmfe/docs-theme-default'}
        expected |= {'@farmfe/plugin-' + plugin + suffix for plugin in plugins
                     for suffix in [''] + ['-' + platform for platform in platforms]}
        expected |= {'@farmfe/plugin-vue-jsx' + suffix for suffix in [''] + ['-' + platform for platform in
                     platforms + ['linux-arm64-musl', 'linux-x64-musl', 'win32-arm64-msvc', 'win32-ia32-msvc']]}
        self.assertEqual(module.ALLOWLIST, expected)
        self.assertEqual(len(expected), 48)

    def test_only_allowlisted_missing_names_publish_original_bytes(self):
        self.manifest['packages'].append({'kind': 'npm', 'name': '@farmfe/core', 'version': '2.0.0'})
        report, rows = self.execute()
        self.assertTrue(report['success'])
        self.assertEqual(set(self.registry.writes), module.ALLOWLIST)
        self.assertEqual(len(self.registry.writes), 48)
        self.assertTrue(all(row['registryBytesVerified'] for row in rows.values()))

    def test_existing_names_are_never_republished_or_retagged(self):
        self.registry.existing.update(module.ALLOWLIST)
        report, rows = self.execute()
        self.assertTrue(report['success'])
        self.assertEqual(self.registry.writes, [])
        self.assertTrue(all(row['status'] == 'already-published' for row in rows.values()))

    def test_name_created_between_checks_is_skipped(self):
        self.registry.race.update(module.ALLOWLIST)
        report, _ = self.execute()
        self.assertTrue(report['success'])
        self.assertEqual(self.registry.writes, [])
        self.assertTrue(all(count == 2 for count in self.registry.calls.values()))

    def test_existing_conflicting_bytes_are_not_overwritten_or_counted_as_success(self):
        name = sorted(module.ALLOWLIST)[0]
        self.registry.existing.add(name)
        self.registry.conflicts.add(name)
        report, rows = self.execute()
        self.assertFalse(report['success'])
        self.assertEqual(rows[name]['status'], 'failed')
        self.assertFalse(rows[name]['uploadAttempted'])
        self.assertNotIn(name, self.registry.writes)

    def test_read_errors_are_not_absence_and_independent_names_continue(self):
        name = sorted(module.ALLOWLIST)[0]
        self.registry.read_fail.add(name)
        report, rows = self.execute()
        self.assertFalse(report['success'])
        self.assertEqual(rows[name]['status'], 'failed')
        self.assertFalse(rows[name]['uploadAttempted'])
        self.assertEqual(len(self.registry.writes), 47)
        self.assertNotIn('private-sentinel', json.dumps(report))

    def test_failed_upload_is_not_retried_and_diagnostic_is_redacted(self):
        name = sorted(module.ALLOWLIST)[0]
        self.registry.failed.add(name)
        report, rows = self.execute()
        self.assertFalse(report['success'])
        self.assertEqual(self.registry.writes.count(name), 1)
        self.assertTrue(rows[name]['outcomeUncertain'])
        self.assertIn('ENEEDAUTH', json.dumps(report))
        self.assertNotIn('private-sentinel', json.dumps(report))

    def test_lost_response_reads_back_bytes_but_does_not_claim_call_success(self):
        name = sorted(module.ALLOWLIST)[0]
        self.registry.lost.add(name)
        report, rows = self.execute()
        self.assertFalse(report['success'])
        self.assertEqual(self.registry.writes.count(name), 1)
        self.assertEqual(rows[name]['status'], 'failed')
        self.assertFalse(rows[name]['outcomeUncertain'])
        self.assertTrue(rows[name]['registryBytesVerified'])
        self.assertNotIn('private-sentinel', json.dumps(report))

    def test_partial_set_duplicates_and_wrong_versions_reject_before_writes(self):
        variants = []
        missing = copy.deepcopy(self.manifest)
        missing['packages'].pop()
        variants.append(missing)
        duplicate = copy.deepcopy(self.manifest)
        duplicate['packages'].append(copy.deepcopy(duplicate['packages'][0]))
        variants.append(duplicate)
        version = copy.deepcopy(self.manifest)
        version['packages'][0]['version'] = '0.0.2'
        variants.append(version)
        for candidate in variants:
            with self.subTest(candidate=candidate['packages'][0]['version']), self.assertRaises(ValueError):
                self.execute(candidate)
        self.assertEqual(self.registry.writes, [])

    def test_changed_local_archive_never_reaches_writer(self):
        row = self.manifest['packages'][0]
        (self.directory / 'archives' / row['archive']).write_bytes(b'changed')
        report, rows = self.execute()
        self.assertFalse(report['success'])
        self.assertFalse(rows[row['name']]['uploadAttempted'])
        self.assertNotIn(row['name'], self.registry.writes)


class CredentialContracts(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='farm-temporary-auth-test-')
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.node, self.cli = self.directory / 'node', self.directory / 'npm-cli.js'
        for path in [self.node, self.cli]:
            path.write_text('offline executable fixture')
        self.calls = []
        self.token = 'private-bypass-token-fixture'
        self.name = sorted(module.ALLOWLIST)[0]
        self.body = archive(self.name)
        self.row = {'kind': 'npm', 'name': self.name, 'version': '0.0.1', 'archive': 'fixture.tgz',
                    'archiveBytes': len(self.body), 'archiveSha256': hashlib.sha256(self.body).hexdigest()}
        self.environment = {'GITHUB_ACTIONS': 'true', 'PATH': os.defpath,
            'NODE_AUTH_TOKEN': 'private-ambient-fixture', 'NODE_OPTIONS': '--require /bad/loader',
            'NODE_PATH': '/bad/modules', 'GH_TOKEN': 'private-github-fixture',
            'NPM_TOKEN': 'private-old-token', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN': 'private-oidc-fixture',
            'ACTIONS_ID_TOKEN_REQUEST_URL': 'https://example.invalid/oidc',
            'NPM_CONFIG_USERCONFIG': '/bad/config', 'npm_config_registry': 'https://example.invalid/npm'}

    def command(self, argv, **options):
        env = options['env']
        self.calls.append((list(argv), {**options, 'env': dict(env)}))
        config = Path(env['NPM_CONFIG_USERCONFIG'])
        self.assertEqual(stat.S_IMODE(config.stat().st_mode), 0o600)
        self.assertNotIn(self.token, config.read_text())
        self.assertEqual(Path(env['NPM_CONFIG_GLOBALCONFIG']).read_text(), '')
        self.assertFalse(options.get('shell', False))
        for key in ['GH_TOKEN', 'NPM_TOKEN', 'NODE_OPTIONS', 'NODE_PATH', 'ACTIONS_ID_TOKEN_REQUEST_TOKEN',
                    'ACTIONS_ID_TOKEN_REQUEST_URL', 'npm_config_registry']:
            self.assertNotIn(key, env)
        self.assertNotIn(self.token, repr(argv))
        if '--input-type=module' in argv:
            self.assertNotIn('NODE_AUTH_TOKEN', env)
            return SimpleNamespace(returncode=0, stdout=json.dumps({'node': str(self.node), 'cli': str(self.cli)}))
        self.assertEqual(env['NODE_AUTH_TOKEN'], self.token)
        self.assertEqual(argv[:3], [str(self.node), str(self.cli), 'publish'])
        self.assertEqual(Path(argv[3]).read_bytes(), self.body)
        self.assertIn('--ignore-scripts', argv)
        self.assertIn('--registry=https://registry.npmjs.org', argv)
        self.assertNotIn('--provenance', argv)
        self.assertIn('--provenance=false', argv)
        self.assertEqual(argv[argv.index('--tag') + 1], 'farm-release-0.0.1')
        self.assertNotIn('dist-tag', argv)
        self.assertNotEqual(options['cwd'], str(Path.cwd()))
        return SimpleNamespace(returncode=0, stdout=b'', stderr=b'')

    def writer(self):
        return module.NewPackageWriter(self.token, self.directory)

    def test_secret_only_reaches_npm_publish_and_scratch_is_removed(self):
        with mock.patch.dict(os.environ, self.environment, clear=True), \
             mock.patch.object(module.subprocess, 'run', side_effect=self.command):
            writer = self.writer()
            scratch = Path(writer.scratch.name)
            try:
                writer.publish(self.row, self.body)
            finally:
                writer.close()
        self.assertFalse(scratch.exists())
        self.assertIsNone(writer.token)
        self.assertEqual(len(self.calls), 2)

    def test_real_readback_constructor_preserves_actions_read_authority(self):
        # Mock only the external lookup; keep the constructor's Actions gate real.
        with mock.patch.dict(os.environ, self.environment, clear=True), \
             mock.patch.object(module.subprocess, 'run', side_effect=self.command), \
             mock.patch.object(module.publisher.RegistryWriter, 'lookup', return_value=self.row['archiveSha256']):
            writer = self.writer()
            try:
                self.assertTrue(writer.verify(self.row))
            finally:
                writer.close()

    def test_archive_requiring_provenance_is_rejected_without_token_subprocess(self):
        body = archive(self.name, publishConfig={'provenance': True})
        row = {**self.row, 'archiveBytes': len(body), 'archiveSha256': hashlib.sha256(body).hexdigest()}
        with mock.patch.dict(os.environ, self.environment, clear=True), \
             mock.patch.object(module.subprocess, 'run', side_effect=self.command):
            writer = self.writer()
            try:
                with self.assertRaises(ValueError):
                    writer.publish(row, body)
            finally:
                writer.close()
        self.assertEqual(len(self.calls), 1)

    def test_npm_timeout_is_redacted_without_retry(self):
        with mock.patch.dict(os.environ, self.environment, clear=True), \
             mock.patch.object(module.subprocess, 'run', side_effect=self.command):
            writer = self.writer()
            try:
                with mock.patch.object(module.subprocess, 'run', side_effect=subprocess.TimeoutExpired(
                        ['npm'], 120, output=b'private-sentinel', stderr=b'npm error code ENEEDAUTH\nprivate-sentinel')) as command:
                    with self.assertRaises(module.publisher.NpmCommandFailure) as caught:
                        writer.publish(self.row, self.body)
                    self.assertEqual(command.call_count, 1)
                    self.assertIn('timeout', str(caught.exception))
                    self.assertNotIn('private-sentinel', str(caught.exception))
            finally:
                writer.close()

    def test_name_absence_requires_404_and_matching_name_is_required(self):
        writer = object.__new__(module.NewPackageWriter)
        writer.reader = SimpleNamespace(json=mock.Mock())
        writer.reader.json.return_value = {'name': self.name}
        self.assertTrue(writer.exists(self.row))
        writer.reader.json.return_value = {'name': '@farmfe/other'}
        with self.assertRaises(ValueError):
            writer.exists(self.row)
        for code in [404, 401, 403, 429, 500]:
            writer.reader.json.side_effect = urllib.error.HTTPError('https://registry.npmjs.org/', code, 'fixture', {}, None)
            if code == 404:
                self.assertFalse(writer.exists(self.row))
            else:
                with self.assertRaises(ValueError):
                    writer.exists(self.row)

    def test_missing_and_multiline_tokens_reject_before_subprocess(self):
        with mock.patch.object(module.subprocess, 'run') as command:
            for token in [None, '', 'token\nsecret', 'token\rsecret', 'token\0secret']:
                with self.subTest(token=repr(token)), self.assertRaises(ValueError):
                    module.NewPackageWriter(token, self.directory)
            command.assert_not_called()


class ScopeContracts(unittest.TestCase):
    def test_wrong_manual_runtime_rejects_before_candidate_read(self):
        env = {'GITHUB_ACTIONS': 'true', 'GITHUB_REPOSITORY': 'farm-fe/farm',
               'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REF': 'refs/heads/main',
               'GITHUB_WORKFLOW_REF': 'farm-fe/farm/' + module.WORKFLOW + '@refs/heads/main',
               'GITHUB_SHA': 'a' * 40, 'GITHUB_WORKFLOW_SHA': 'a' * 40}
        for key, value in [('GITHUB_ACTIONS', 'false'), ('GITHUB_REPOSITORY', 'fork/farm'),
                           ('GITHUB_EVENT_NAME', 'push'), ('GITHUB_REF', 'refs/heads/other'),
                           ('GITHUB_WORKFLOW_REF', 'farm-fe/farm/.github/workflows/other.yaml@refs/heads/main'),
                           ('GITHUB_WORKFLOW_SHA', 'b' * 40)]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                module.verify_candidate(Path('/nonexistent/offline-candidate'), {**env, key: value})

    def test_workflow_is_dispatch_only_read_only_and_serialized_with_automatic_release(self):
        workflow = (Path(__file__).parent.parent / module.WORKFLOW).read_text()
        self.assertIn('workflow_dispatch:', workflow)
        self.assertNotIn('\n  push:', workflow)
        self.assertNotIn('pull_request', workflow)
        self.assertNotIn('id-token:', workflow)
        self.assertNotIn('contents: write', workflow)
        self.assertNotIn('actions: write', workflow)
        self.assertIn('group: farm-independent-release', workflow)
        self.assertIn('cancel-in-progress: false', workflow)
        self.assertIn('persist-credentials: false', workflow)
        self.assertEqual(workflow.count('secrets.NPM_BYPASS_2FA_TOKEN'), 1)
        self.assertIn('if: always()', workflow)
        self.assertIn('artifact-ids: ' + str(module.CANDIDATE_ARTIFACT), workflow)
        self.assertIn('run-id: ' + str(module.PRODUCER_RUN), workflow)
        self.assertIn('merge-multiple: true', workflow)
        self.assertNotIn('inputs.producer_', workflow)


class ProducerProofContracts(unittest.TestCase):
    def setUp(self):
        # Existing complete source/plan/seal fixture; only external signature/API
        # observations are substituted. The real ReleaseVerifier stays in path.
        fixture_module = module.load('temporary_source_fixture', 'incremental-release-entry.test.py')
        self.fixture = fixture_module.Contracts()
        self.fixture.setUp()
        self.addCleanup(shutil.rmtree, self.fixture.root)
        self.fixture.run.update(event='push', conclusion='failure')
        self.fixture.manifest['releaseValidation'] = 'archive-integrity-v1'
        self.fixture.seal['manifestHash'] = module.entry.chain.digest(self.fixture.manifest)
        old_get = self.fixture.transport.get.side_effect
        def get(route):
            if route.startswith('/compare/'):
                return {'status': 'identical', 'base_commit': {'sha': self.fixture.source['sha']},
                        'merge_base_commit': {'sha': self.fixture.source['sha']}}
            return old_get(route)
        self.fixture.transport.get.side_effect = get
        self.verifier = module.OriginalProducerVerifier(self.fixture.transport,
            lambda sha: copy.deepcopy(self.fixture.source), receipt_provider=self.fixture.receipt_fixture)

    def verify(self):
        with mock.patch.object(module.entry.execution, 'jobs_for', return_value=self.fixture.jobs):
            return self.verifier.verify(self.fixture.manifest, self.fixture.root, phase='prepublish')

    def test_failed_overall_run_still_requires_complete_original_source_and_seal(self):
        proof = self.verify()
        self.assertTrue(proof['prepublicationVerified'])
        self.assertFalse(proof['publicationExecuted'])

    def test_failed_producer_or_nonterminal_overall_run_is_rejected(self):
        self.fixture.jobs[0]['conclusion'] = 'failure'
        with self.assertRaises(ValueError):
            self.verify()
        self.fixture.jobs[0]['conclusion'] = 'success'
        self.fixture.run['status'] = 'in_progress'
        with self.assertRaises(ValueError):
            self.verify()

    def test_bad_seal_changed_bytes_and_filtered_manifest_are_rejected(self):
        self.fixture.seal['manifestHash'] = 'f' * 64
        with self.assertRaises(ValueError):
            self.verify()
        self.fixture.seal['manifestHash'] = module.entry.chain.digest(self.fixture.manifest)
        row = self.fixture.manifest['packages'][0]
        path = self.fixture.root / row['archive']
        body = path.read_bytes()
        path.write_bytes(b'changed')
        with self.assertRaises(ValueError):
            self.verify()
        path.write_bytes(body)
        self.fixture.manifest['packages'].pop()
        with self.assertRaises(ValueError):
            self.verify()

    def test_foreign_repository_dispatch_source_and_untrusted_ancestry_rejected(self):
        original = copy.deepcopy(self.fixture.run)
        for key, value in [('event', 'workflow_dispatch'), ('head_branch', 'other'),
                           ('path', '.github/workflows/ci.yaml'), ('head_sha', 'f' * 40),
                           ('head_repository', {'id': 987})]:
            self.fixture.run.clear()
            self.fixture.run.update(copy.deepcopy(original))
            self.fixture.run[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.verify()
        self.fixture.run.clear()
        self.fixture.run.update(original)
        old_get = self.fixture.transport.get.side_effect
        self.fixture.transport.get.side_effect = lambda route: {'status': 'diverged'} if route.startswith('/compare/') else old_get(route)
        with self.assertRaises(ValueError):
            self.verify()


class CandidateBindingContracts(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='farm-temporary-binding-test-')
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        rows = [{'kind': 'npm', 'name': name, 'version': '0.0.1', 'archiveBytes': 10,
                 'archiveSha256': 'a' * 64} for name in sorted(module.ALLOWLIST)]
        identities = ['npm:' + row['name'] for row in rows]
        self.manifest = {'chain': 'release', 'profile': 'release', 'publicationScope': 'npm',
            'sourceSHA': module.PRODUCER_SOURCE, 'sourceTree': 'b' * 40, 'planHash': 'c' * 64,
            'producerRunId': module.PRODUCER_RUN, 'producerAttempt': module.PRODUCER_ATTEMPT,
            'baseline': None, 'packages': rows, 'family': rows, 'publish': identities, 'rebuild': identities}
        self.environment = {'GITHUB_ACTIONS': 'true', 'GITHUB_REPOSITORY': 'farm-fe/farm',
            'GITHUB_EVENT_NAME': 'workflow_dispatch', 'GITHUB_REF': 'refs/heads/main',
            'GITHUB_WORKFLOW_REF': 'farm-fe/farm/' + module.WORKFLOW + '@refs/heads/main',
            'GITHUB_SHA': 'd' * 40, 'GITHUB_WORKFLOW_SHA': 'd' * 40, 'GH_TOKEN': 'offline-read-token'}
        self.binding = ({'digest': module.CANDIDATE_DIGEST}, {'run_attempt': module.PRODUCER_ATTEMPT}, {})
        self.proof = {'prepublicationVerified': True}

    def verify(self):
        (self.directory / 'manifest.json').write_text(json.dumps(self.manifest))
        finder = mock.Mock(return_value=self.binding)
        with mock.patch.object(module.entry.evidence.transport_module, 'GitHubReadOnly', return_value='offline-api'), \
             mock.patch.object(module.entry.evidence, 'source_reader', return_value={'sha': module.PRODUCER_SOURCE}), \
             mock.patch.object(module, 'load', return_value=SimpleNamespace(find_frozen_binding=finder)), \
             mock.patch.object(module.OriginalProducerVerifier, 'verify', return_value=self.proof) as verifier:
            result = module.verify_candidate(self.directory, self.environment)
            finder.assert_called_once_with('offline-api', {'sha': module.PRODUCER_SOURCE}, 'frozen',
                module.PRODUCER_RUN, module.PRODUCER_ATTEMPT, module.CANDIDATE_ARTIFACT, 'candidates')
            self.assertEqual(len(verifier.call_args.args[0]['packages']), len(self.manifest['packages']))
            self.assertEqual(verifier.call_args.kwargs['phase'], 'prepublish')
            return result

    def test_fixed_original_artifact_uses_signed_binding_and_full_verifier(self):
        self.assertEqual(self.verify(), self.manifest)

    def test_alternate_producer_source_run_and_attempt_rejected(self):
        original = copy.deepcopy(self.manifest)
        for key, value in [('sourceSHA', 'f' * 40), ('producerRunId', 123), ('producerAttempt', 2)]:
            self.manifest = copy.deepcopy(original)
            self.manifest[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.verify()

    def test_candidate_digest_original_attempt_and_preflight_cannot_be_substituted(self):
        original = self.binding
        self.binding = ({'digest': 'sha256:' + 'f' * 64}, original[1], {})
        with self.assertRaises(ValueError):
            self.verify()
        self.binding = (original[0], {'run_attempt': 2}, {})
        with self.assertRaises(ValueError):
            self.verify()
        self.binding = original
        self.proof = {'prepublicationVerified': False}
        with self.assertRaises(ValueError):
            self.verify()


class MainContracts(unittest.TestCase):
    def test_deadline_closes_writer_and_leaves_failure_report(self):
        with tempfile.TemporaryDirectory(prefix='farm-temporary-deadline-test-') as directory:
            output = Path(directory) / 'result.json'
            writer = SimpleNamespace(close=mock.Mock())
            with mock.patch.dict(os.environ, {'NPM_BYPASS_2FA_TOKEN': 'private-fixture', 'CONFIRM_PUBLICATION': 'true'}, clear=True), \
                 mock.patch.object(sys, 'argv', ['temporary', '--candidate', directory, '--output', str(output)]), \
                 mock.patch.object(module, 'verify_candidate', return_value={}), \
                 mock.patch.object(module, 'NewPackageWriter', return_value=writer), \
                 mock.patch.object(module, 'publish_new_packages', side_effect=module.best.PublicationInterrupted('private-fixture')), \
                 mock.patch('builtins.print'):
                self.assertEqual(module.main(), 1)
            writer.close.assert_called_once()
            report = json.loads(output.read_text())
            self.assertFalse(report['success'])
            self.assertFalse(report['complete'])
            self.assertEqual(len(report['packages']), 48)
            self.assertNotIn('private-fixture', output.read_text())

    def test_token_detached_before_proof_and_failure_saves_safe_complete_inventory(self):
        with tempfile.TemporaryDirectory(prefix='farm-temporary-main-test-') as directory:
            output = Path(directory) / 'result.json'
            secret = 'private-bypass-main-fixture'
            def reject(candidate, environment):
                self.assertNotIn('NPM_BYPASS_2FA_TOKEN', environment)
                raise ValueError(secret)
            with mock.patch.dict(os.environ, {'NPM_BYPASS_2FA_TOKEN': secret, 'CONFIRM_PUBLICATION': 'true'}, clear=True), \
                 mock.patch.object(sys, 'argv', ['temporary', '--candidate', directory, '--output', str(output)]), \
                 mock.patch.object(module, 'verify_candidate', side_effect=reject), \
                 mock.patch.object(module, 'NewPackageWriter') as writer, \
                 mock.patch('builtins.print') as message:
                self.assertEqual(module.main(), 1)
                writer.assert_not_called()
            report = json.loads(output.read_text())
            self.assertFalse(report['success'])
            self.assertFalse(report['complete'])
            self.assertEqual({row['name'] for row in report['packages']}, module.ALLOWLIST)
            self.assertNotIn(secret, output.read_text())
            self.assertNotIn(secret, str(message.call_args_list))


if __name__ == '__main__':
    unittest.main()
