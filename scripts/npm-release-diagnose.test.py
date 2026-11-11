"""Default diagnostic path with mocked process/HTTP boundaries; no registry access."""
# cspell:words whoami importlib npmjs USERCONFIG GLOBALCONFIG
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock
import urllib.error

spec = importlib.util.spec_from_file_location('npm_diagnostic_contract', Path(__file__).with_name('npm-release-diagnose.py'))
diagnostic = importlib.util.module_from_spec(spec); spec.loader.exec_module(diagnostic)


class Contracts(unittest.TestCase):
    def setUp(self):
        self.actual_temporary = tempfile.TemporaryDirectory
        self.fixture = self.actual_temporary(prefix='farm-diagnostic-test-')
        self.addCleanup(self.fixture.cleanup)
        self.node, self.cli = [Path(self.fixture.name) / name for name in ['node', 'npm-cli.js']]
        for file in [self.node, self.cli]: file.write_text('fixture only')
        self.environment = {'GITHUB_ACTIONS': 'true', 'GITHUB_REPOSITORY': 'farm-fe/farm',
            'GITHUB_REF': 'refs/heads/main', 'GITHUB_EVENT_NAME': 'workflow_dispatch',
            'GITHUB_WORKFLOW_REF': 'farm-fe/farm/.github/workflows/npm-release-diagnose.yaml@refs/heads/main',
            'GITHUB_SHA': 'a' * 40, 'GITHUB_WORKFLOW_SHA': 'a' * 40, 'NPM_TOKEN': 'private-token-fixture'}
        self.paths, self.commands, self.requests = [], [], []
        self.resolve = SimpleNamespace(returncode=0, stdout=json.dumps({'node': str(self.node), 'cli': str(self.cli)}))
        self.whoami = SimpleNamespace(returncode=0, stdout=b'"private-username-fixture"')
        self.public = {'name': '@farmfe/cli', 'version': '2.0.0'}

    def temporary(self, **options):
        result = self.actual_temporary(**options)
        self.paths.append(Path(result.name))
        return result

    def command(self, argv, **options):
        env = options['env']
        config = Path(env['NPM_CONFIG_USERCONFIG'])
        global_config = Path(env['NPM_CONFIG_GLOBALCONFIG'])
        self.assertEqual(stat.S_IMODE(config.stat().st_mode), 0o600)
        self.assertEqual(stat.S_IMODE(global_config.stat().st_mode), 0o600)
        self.assertEqual(global_config.read_text(), '')
        self.assertNotIn(self.environment['NPM_TOKEN'], config.read_text())
        self.assertNotEqual(options['cwd'], str(Path.cwd()))
        self.assertNotEqual(options['cwd'], env['HOME'])
        self.assertEqual(list(Path(options['cwd']).iterdir()), [])
        self.assertTrue(Path(env['HOME']).is_dir())
        self.assertFalse(options['shell'])
        self.commands.append((list(argv), {**options, 'env': dict(env)}, config.read_text()))
        if '--input-type=module' in argv:
            self.assertNotIn('NODE_AUTH_TOKEN', env)
            self.assertNotIn('_authToken', config.read_text())
            self.assertIn('import {resolveNpmCli} from "file:///', argv[-1])
            self.assertNotIn('resolvePublisherNpmCli', argv[-1])
            self.assertEqual(options['timeout'], 15)
            result = self.resolve
        else:
            self.assertEqual(argv, [str(self.node), str(self.cli), 'whoami', '--json', '--registry=https://registry.npmjs.org'])
            self.assertEqual(options['timeout'], 30)
            self.assertEqual(env['NODE_AUTH_TOKEN'], self.environment['NPM_TOKEN'])
            self.assertIn('//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}', config.read_text())
            result = self.whoami
        if isinstance(result, BaseException): raise result
        return result

    def public_open(self, request, **options):
        self.requests.append(request)
        self.assertEqual(request.full_url, 'https://registry.npmjs.org/%40farmfe%2Fcli/2.0.0')
        self.assertEqual(request.get_method(), 'GET')
        self.assertIsNone(request.data)
        self.assertFalse(any(key.lower() == 'authorization' for key in request.headers))
        self.assertNotIn('private-', repr(request.headers))
        self.assertLessEqual(options['timeout'], 30)
        if isinstance(self.public, BaseException): raise self.public
        return io.BytesIO(json.dumps(self.public).encode())

    def execute(self, environment=None):
        opener = SimpleNamespace(open=self.public_open)
        with mock.patch.object(diagnostic.subprocess, 'run', side_effect=self.command), \
             mock.patch.object(diagnostic.tempfile, 'TemporaryDirectory', side_effect=self.temporary), \
             mock.patch.object(diagnostic.publisher.entry.evidence.registry_module.urllib.request,
                               'build_opener', return_value=opener), \
             mock.patch.object(diagnostic.publisher, 'RegistryWriter', side_effect=AssertionError('Publisher must not be used')):
            return diagnostic.diagnose(self.environment if environment is None else environment)

    def assert_clean(self):
        self.assertEqual(len(self.paths), 1)
        self.assertTrue(all(not path.exists() for path in self.paths))

    def test_default_path_uses_legacy_token_without_oidc_and_only_whoami_and_public_get(self):
        report = self.execute()
        self.assertTrue(report['authenticated'])
        self.assertTrue(report['targetExists'])
        self.assertEqual(report['package'], '@farmfe/cli@2.0.0')
        self.assertFalse(report['publicationPermissionVerified'])
        self.assertFalse(report['publicationExecuted'])
        self.assertNotIn('private-', json.dumps(report))
        self.assertEqual(len(self.commands), 2)
        self.assertEqual(len(self.requests), 1)
        self.assert_clean()

    def test_ambient_oidc_auth_and_loader_configuration_never_enter_children_or_public_reads(self):
        hostile = {'ACTIONS_ID_TOKEN_REQUEST_URL': 'https://example.invalid/oidc',
            'ACTIONS_ID_TOKEN_REQUEST_TOKEN': 'private-oidc-fixture', 'GH_TOKEN': 'private-gh-fixture',
            'NODE_AUTH_TOKEN': 'private-wrong-token', 'NODE_OPTIONS': '--require /bad/inject.js',
            'NODE_PATH': '/bad/modules', 'npm_execpath': '/bad/npm',
            'NPM_CONFIG_USERCONFIG': '/bad/npmrc', 'npm_config_registry': 'https://example.invalid/npm',
            'npm_config_script_shell': '/bad/shell', 'HOME': '/bad/home'}
        report = self.execute({**self.environment, **hostile})
        self.assertTrue(report['authenticated'])
        for _, options, _ in self.commands:
            for key, value in hostile.items(): self.assertNotEqual(options['env'].get(key), value, key)
            self.assertNotIn('NPM_TOKEN', options['env'])
            self.assertEqual(options['env']['npm_config_ignore_scripts'], 'true')
        self.assert_clean()

    def test_authentication_failures_are_redacted_without_retry_and_still_read_public_target(self):
        failures = [SimpleNamespace(returncode=1, stdout=b'private-token-fixture',
                    stderr=b'Authorization: private-token-fixture\nnpm error code E401\nprivate-username-fixture'),
            subprocess.TimeoutExpired(['node', 'whoami'], 30, output=b'private-token-fixture', stderr=b'private-username-fixture'),
            subprocess.CalledProcessError(1, ['node', 'whoami'], output=b'private-token-fixture', stderr=b'npm error code E401'),
            OSError('private-token-fixture')]
        for failure in failures:
            with self.subTest(failure=type(failure).__name__):
                self.whoami = failure
                self.paths.clear(); self.commands.clear(); self.requests.clear()
                report = self.execute()
                self.assertFalse(report['authenticated'])
                self.assertTrue(report['targetExists'])
                self.assertIn('operation=whoami', report['authenticationError'])
                self.assertNotIn('private-', json.dumps(report))
                self.assertEqual(len(self.commands), 2)
                self.assertEqual(len(self.requests), 1)
                self.assert_clean()

    def test_invalid_or_oversized_identity_is_not_reported(self):
        for output in [b'{private-token-fixture', b'["private-token-fixture"]', b'""', b'"' + b'x' * 4096 + b'"']:
            with self.subTest(size=len(output)):
                self.whoami = SimpleNamespace(returncode=0, stdout=output)
                self.paths.clear(); self.commands.clear(); self.requests.clear()
                report = self.execute()
                self.assertFalse(report['authenticated'])
                self.assertEqual(report['authenticationError'], 'Authentication response invalid')
                self.assertNotIn('private-', json.dumps(report))
                self.assert_clean()

    def test_public_absence_and_errors_do_not_infer_publication_permission(self):
        for value in [urllib.error.HTTPError('fixed', 404, 'private-token-fixture', {}, None),
                      urllib.error.HTTPError('fixed', 403, 'private-token-fixture', {}, None),
                      {'name': '@other/pkg', 'version': '2.0.0'}]:
            with self.subTest(value=type(value).__name__):
                self.public = value
                self.paths.clear(); self.commands.clear(); self.requests.clear()
                report = self.execute()
                self.assertFalse(report['publicationPermissionVerified'])
                self.assertFalse(report['publicationExecuted'])
                self.assertNotIn('private-', json.dumps(report))
                if isinstance(value, urllib.error.HTTPError) and value.code == 404:
                    self.assertIs(report['targetExists'], False)
                else:
                    self.assertIsNone(report['targetExists'])
                    self.assertEqual(report['registryError'], 'Public registry read failed')
                self.assert_clean()

    def test_fixed_manual_main_workflow_guards_reject_before_creating_client(self):
        changes = {'GITHUB_ACTIONS': 'false', 'GITHUB_REPOSITORY': 'other/farm',
            'GITHUB_REF': 'refs/heads/topic', 'GITHUB_EVENT_NAME': 'push',
            'GITHUB_WORKFLOW_REF': 'farm-fe/farm/.github/workflows/other.yaml@refs/heads/main',
            'GITHUB_SHA': 'a' * 39, 'GITHUB_WORKFLOW_SHA': 'b' * 40}
        for key, value in changes.items():
            with self.subTest(key=key):
                factory = mock.Mock()
                with self.assertRaisesRegex(ValueError, 'exact manual main workflow'):
                    diagnostic.diagnose({**self.environment, key: value}, factory)
                factory.assert_not_called()
        with self.assertRaisesRegex(ValueError, 'exact manual main workflow'):
            diagnostic.diagnose({**self.environment, 'GITHUB_SHA': '', 'GITHUB_WORKFLOW_SHA': ''}, mock.Mock())

    def test_missing_or_invalid_token_does_not_fall_back_to_oidc_or_ambient_auth(self):
        for token in ['', None, 'private-token\nfixture', 'private-token\0fixture']:
            with self.subTest(token_type=type(token).__name__):
                env = {**self.environment, 'NPM_TOKEN': token, 'NODE_AUTH_TOKEN': 'private-ambient-token',
                    'ACTIONS_ID_TOKEN_REQUEST_URL': 'https://example.invalid/oidc',
                    'ACTIONS_ID_TOKEN_REQUEST_TOKEN': 'private-oidc-token'}
                with self.assertRaisesRegex(ValueError, 'diagnostic credential missing or invalid'):
                    self.execute(env)
                self.assertEqual(self.commands, [])
                self.assertEqual(self.requests, [])
                self.assertEqual(self.paths, [])

    def test_setup_failure_removes_private_config_and_redacts_error(self):
        for result in [SimpleNamespace(returncode=0, stdout='private-token-fixture'),
                       SimpleNamespace(returncode=0, stdout=json.dumps({'node': '/missing/private-token-fixture', 'cli': str(self.cli)})),
                       OSError('private-token-fixture')]:
            with self.subTest(result=type(result).__name__):
                self.resolve = result
                self.paths.clear(); self.commands.clear(); self.requests.clear()
                with self.assertRaises(ValueError) as raised:
                    self.execute()
                self.assertNotIn('private-', str(raised.exception))
                self.assertEqual(len(self.commands), 1)
                self.assertEqual(self.requests, [])
                self.assert_clean()
        original = Path.write_text
        def fail_config(file, *args, **options):
            if file.name == 'global.npmrc': raise OSError('private-token-fixture')
            return original(file, *args, **options)
        self.paths.clear(); self.commands.clear()
        with mock.patch.object(Path, 'write_text', fail_config):
            with self.assertRaisesRegex(ValueError, 'Diagnostic CLI setup failed'):
                self.execute()
        self.assertEqual(self.commands, [])
        self.assert_clean()

    def test_diagnostic_client_has_no_write_api_and_rejects_other_npm_operations(self):
        client = diagnostic.DiagnosticNpmReader(self.environment, '.', run_command=mock.Mock())
        with client:
            for operation in ['publish', 'dist-tag', 'install', 'exec']:
                with self.assertRaisesRegex(ValueError, 'Read-only diagnostic operation required'):
                    client.npm_command(operation, diagnostic.PACKAGE, [])
            client.command.assert_not_called()
            self.assertFalse(hasattr(client, 'publish'))
            self.assertFalse(hasattr(client, 'promote'))
        self.assertEqual(client.environment, {})
        with self.assertRaisesRegex(ValueError, 'closed'): client.npm()

    def test_diagnostic_workflow_remains_manual_fixed_and_without_oidc_or_write_permissions(self):
        workflow = (Path(__file__).parents[1] / '.github/workflows/npm-release-diagnose.yaml').read_text()
        self.assertIn('workflow_dispatch:', workflow)
        self.assertIn("github.repository == 'farm-fe/farm'", workflow)
        self.assertIn("github.ref == 'refs/heads/main'", workflow)
        self.assertIn('ref: ${{ github.sha }}', workflow)
        self.assertIn('contents: read', workflow)
        self.assertIn('NPM_TOKEN: ${{ secrets.NPM_TOKEN }}', workflow)
        for forbidden in ['id-token:', 'write', 'inputs:', 'dist-tag', 'pull_request:', 'push:', 'npm install', 'pnpm install']:
            self.assertNotIn(forbidden, workflow)


if __name__ == '__main__': unittest.main()
