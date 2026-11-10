"""Manual fixed-package read-only diagnostic; never authorizes publication."""
# cspell:words whoami importlib npmjs
import importlib.util
import json
import os
from pathlib import Path
import urllib.error

spec = importlib.util.spec_from_file_location('diagnostic_publisher', Path(__file__).with_name('incremental-release-publish.py'))
publisher = importlib.util.module_from_spec(spec); spec.loader.exec_module(publisher)
PACKAGE = {'kind': 'npm', 'name': '@farmfe/cli', 'version': '2.0.0'}


def diagnose(environment, writer_factory=publisher.RegistryWriter):
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
