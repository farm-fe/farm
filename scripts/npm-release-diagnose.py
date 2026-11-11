"""Manual fixed-package read-only diagnostic; never authorizes publication."""
# cspell:words whoami importlib npmjs USERCONFIG GLOBALCONFIG USERPROFILE TMPDIR
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import urllib.error

spec = importlib.util.spec_from_file_location('diagnostic_publisher', Path(__file__).with_name('incremental-release-publish.py'))
publisher = importlib.util.module_from_spec(spec); spec.loader.exec_module(publisher)
PACKAGE = {'kind': 'npm', 'name': '@farmfe/cli', 'version': '2.0.0'}


class DiagnosticNpmReader:
    """Legacy-token authentication probe, deliberately separate from publication.

    The only registry operations exposed here are whoami and public reads. This
    does not restore token authentication to the OIDC-only release writer.
    """
    def __init__(self, environment, directory, run_command=None):
        self.environment = dict(environment)
        self.command = subprocess.run if run_command is None else run_command
        self.reader = publisher.entry.evidence.registry_module.RegistryReadOnly(
            environment={'GITHUB_ACTIONS': environment.get('GITHUB_ACTIONS')})
        self._npm, self._scratch, self._cwd = None, None, None
        self._closed = False

    def __enter__(self):
        publisher.check(not self._closed, 'Diagnostic reader is closed')
        return self

    def __exit__(self, *exc):
        self.close()

    def close(self):
        self._closed = True
        if self._npm is not None:
            self._npm[2].clear()
            self._npm = None
        self.environment.clear()
        if self._scratch is not None:
            self._scratch.cleanup()
            self._scratch = None
        self._cwd = None

    def _run(self, operation, row, argv, **options):
        try:
            result = self.command(argv, shell=False, **options)
        except subprocess.TimeoutExpired as error:
            raise publisher.npm_failure(operation, row, timed_out=True,
                stdout=error.output, stderr=error.stderr) from None
        except subprocess.CalledProcessError as error:
            raise publisher.npm_failure(operation, row, code=error.returncode,
                stdout=error.stdout, stderr=error.stderr) from None
        except (OSError, ValueError):
            raise publisher.npm_failure(operation, row) from None
        if result.returncode != 0:
            raise publisher.npm_failure(operation, row, code=result.returncode,
                stdout=getattr(result, 'stdout', None), stderr=getattr(result, 'stderr', None)) from None
        return result

    def npm(self):
        publisher.check(not self._closed, 'Diagnostic reader is closed')
        if self._npm is None:
            token = self.environment.get('NPM_TOKEN')
            publisher.check(isinstance(token, str) and bool(token)
                            and not any(value in token for value in ['\0', '\r', '\n']),
                            'Existing npm diagnostic credential missing or invalid')
            self._scratch = tempfile.TemporaryDirectory(prefix='farm-npm-diagnostic-')
            scratch = Path(self._scratch.name)
            try:
                home, cwd, temporary = [scratch / name for name in ['home', 'work', 'tmp']]
                for directory in [home, cwd, temporary]: directory.mkdir(mode=0o700)
                self._cwd = str(cwd)
                config, global_config = scratch / 'user.npmrc', scratch / 'global.npmrc'
                config.write_text('registry=https://registry.npmjs.org/\n', encoding='utf-8')
                global_config.write_text('', encoding='utf-8')
                for file in [config, global_config]: file.chmod(0o600)
                # No OIDC credentials, loader options, ambient auth or npm config.
                base = {'PATH': self.environment.get('PATH', os.environ.get('PATH', os.defpath)),
                        'HOME': str(home), 'USERPROFILE': str(home),
                        'TMPDIR': str(temporary), 'TMP': str(temporary), 'TEMP': str(temporary),
                        'NPM_CONFIG_USERCONFIG': str(config), 'NPM_CONFIG_GLOBALCONFIG': str(global_config),
                        'npm_config_cache': str(scratch / 'cache'),
                        'npm_config_ignore_scripts': 'true', 'npm_config_update_notifier': 'false'}
                module = json.dumps(Path(__file__).with_name('v2-npm-cli.mjs').resolve().as_uri())
                probe = ('import {resolveNpmCli} from ' + module + ';'
                         'console.log(JSON.stringify({node:process.execPath,cli:resolveNpmCli()}));')
                result = self._run('resolve-cli', None,
                    [self.environment.get('FARM_NODE_EXECUTABLE', 'node'), '--input-type=module', '-e', probe],
                    env=base, cwd=self._cwd, capture_output=True, text=True, timeout=15)
                publisher.check(isinstance(result.stdout, str) and len(result.stdout) <= 8192,
                                'Diagnostic CLI response invalid')
                selected = json.loads(result.stdout)
                node, cli = selected['node'], selected['cli']
                for file in [node, cli]:
                    publisher.check(isinstance(file, str) and Path(file).is_absolute() and Path(file).is_file(),
                                    'Controlled diagnostic executable missing')
                # The token itself is never written to disk or passed to a probe.
                config.write_text('registry=https://registry.npmjs.org/\n'
                                  '//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}\n', encoding='utf-8')
                self._npm = (node, cli, {**base, 'NODE_AUTH_TOKEN': token})
            except BaseException as error:
                self.close()
                if isinstance(error, publisher.NpmCommandFailure): raise
                if isinstance(error, Exception): raise ValueError('Diagnostic CLI setup failed') from None
                raise
        return self._npm

    def npm_command(self, operation, row, argv, **_options):
        publisher.check(operation == 'whoami' and row == PACKAGE, 'Read-only diagnostic operation required')
        node, cli, env = self.npm()
        expected = [node, cli, 'whoami', '--json', '--registry=https://registry.npmjs.org']
        publisher.check(argv == expected, 'Fixed diagnostic command required')
        return self._run('whoami', PACKAGE, expected, env=env, cwd=self._cwd,
                         capture_output=True, timeout=30)


def diagnose(environment, writer_factory=DiagnosticNpmReader):
    publisher.check(environment.get('GITHUB_ACTIONS') == 'true'
                    and environment.get('GITHUB_REPOSITORY') == 'farm-fe/farm'
                    and environment.get('GITHUB_REF') == 'refs/heads/main'
                    and environment.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'
                    and environment.get('GITHUB_WORKFLOW_REF') == 'farm-fe/farm/.github/workflows/npm-release-diagnose.yaml@refs/heads/main'
                    and environment.get('GITHUB_SHA') == environment.get('GITHUB_WORKFLOW_SHA')
                    and publisher.re.fullmatch('[a-f0-9]{40}', environment.get('GITHUB_SHA', '')),
                    'Diagnostic requires the exact manual main workflow')
    report = {'operation': 'read-only-npm-diagnostic', 'package': '@farmfe/cli@2.0.0',
              'authenticated': False, 'targetExists': None, 'publicationPermissionVerified': False,
              'publicationExecuted': False}
    with writer_factory(environment, '.') as writer:
        node, cli, env = writer.npm()
        try:
            result = writer.npm_command('whoami', PACKAGE,
                [node, cli, 'whoami', '--json', '--registry=https://registry.npmjs.org'],
                env=env, capture_output=True, timeout=30)
            # Username is only shape-checked in memory, never logged or saved.
            publisher.check(len(result.stdout) <= 4096, 'Authentication response exceeds bound')
            username = json.loads(result.stdout)
            publisher.check(isinstance(username, str) and bool(username), 'Invalid authentication response')
            report['authenticated'] = True
        except publisher.NpmCommandFailure as error:
            report['authenticationError'] = str(error)
        except Exception:
            report['authenticationError'] = 'Authentication response invalid'
        try:
            value = writer.reader.json('https://registry.npmjs.org/%40farmfe%2Fcli/2.0.0')
            publisher.check(value.get('name') == PACKAGE['name'] and value.get('version') == PACKAGE['version'], 'Registry identity differs')
            report['targetExists'] = True
        except urllib.error.HTTPError as error:
            if error.code == 404: report['targetExists'] = False
            else: report['registryError'] = 'Public registry read failed'
        except Exception:
            report['registryError'] = 'Public registry read failed'
    return report


if __name__ == '__main__':
    try:
        report = diagnose(os.environ)
        print(json.dumps(report, sort_keys=True))
        if not report['authenticated'] or report['targetExists'] is None: raise SystemExit(1)
    except Exception:
        # No traceback/config paths/environment or arbitrary exception text.
        print(json.dumps({'operation': 'read-only-npm-diagnostic', 'error': 'Diagnostic setup failed', 'publicationExecuted': False}))
        raise SystemExit(1) from None
