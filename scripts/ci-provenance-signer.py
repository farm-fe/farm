"""Isolated main CI/Unit signer preparation; signing is a pinned Action step.

No tested-source checkout, dependency install, build, cache, or caller-selected
policy is executed here. Producer runners remain trusted execution environments.
"""
# cspell:words importlib
import importlib.util
import json
import os
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('signer_bindings', ROOT / 'ci-attested-bindings.py')
bindings = importlib.util.module_from_spec(spec); spec.loader.exec_module(bindings)
check = bindings.check
REPOSITORY = 'farm-fe/farm'
REPOSITORY_ID = 507542208
SIGNER = '.github/workflows/ci-provenance-signer.yaml'
CALLERS = {'ci':'.github/workflows/ci.yaml', 'unit':'.github/workflows/rust-test.yaml'}


def cells(scope):
    check(scope in CALLERS, 'Unknown signer scope')
    result = {'planner':{'producer':'Incremental ' + ('CI' if scope == 'ci' else 'Unit CI') + ' plan (shadow only)',
                         'uploadStep':'Upload fixed source impact plan', 'artifactName':'impact-plan-{sourceSHA}-{originAttempt}'}}
    if scope == 'ci':
        result['execution-baseline'] = {'producer':'Record verified CI execution baseline',
            'uploadStep':'Upload trusted CI execution baseline', 'artifactName':'farm-ci-execution-baseline-{runId}-{originAttempt}'}
        for abi, prefix, platform in [('linux-x64-gnu','Primary Linux','ubuntu-24.04'), ('darwin-arm64','Primary macOS','macos-latest'), ('win32-x64-msvc','Primary Windows','windows-latest')]:
            for label, title, upload in [('ts','Typescript','TypeScript'), ('browser','Examples','browser')]:
                result[label + ':' + abi] = {'producer':prefix + ' / ' + title + ' Test (' + platform + ')',
                    'uploadStep':'Preserve actual ' + upload + ' discovery and execution counts',
                    'artifactName':'farm-ci-suite-' + label + '-{sourceSHA}-' + abi + '-{originAttempt}'}
    return result


def identity(environment, scope):
    check(scope in CALLERS, 'Unknown signer scope')
    revision = environment.get('SIGNER_WORKFLOW_SHA', '')
    source = environment.get('GITHUB_SHA', '')
    check(re.fullmatch(r'[a-f0-9]{40}', revision) and re.fullmatch(r'[a-f0-9]{40}', source), 'Exact signer and source revisions required')
    check(environment.get('GITHUB_ACTIONS') == 'true' and environment.get('GITHUB_EVENT_NAME') == 'push'
          and environment.get('GITHUB_REF') == 'refs/heads/main'
          and environment.get('GITHUB_REPOSITORY') == REPOSITORY
          and environment.get('GITHUB_REPOSITORY_ID') == str(REPOSITORY_ID)
          and environment.get('GITHUB_JOB') == 'sign'
          and environment.get('RUNNER_ENVIRONMENT') == 'github-hosted'
          and environment.get('SIGNER_WORKFLOW_REF') == REPOSITORY + '/' + SIGNER + '@' + revision
          and environment.get('GITHUB_WORKFLOW_REF') == REPOSITORY + '/' + CALLERS[scope] + '@refs/heads/main',
          'Only the fixed main push signer context is accepted')
    for key in ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT']:
        check(re.fullmatch(r'[1-9][0-9]*', environment.get(key, '')), 'Invalid signer run identity')
    check(int(environment['GITHUB_RUN_ATTEMPT']) <= 32, 'Signer attempt bound exceeded')
    return {'repository':REPOSITORY, 'repositoryId':REPOSITORY_ID, 'sourceSHA':source,
            'signerWorkflow':SIGNER, 'signerSHA':revision, 'callerWorkflow':CALLERS[scope],
            'runId':int(environment['GITHUB_RUN_ID']), 'attempt':int(environment['GITHUB_RUN_ATTEMPT']), 'cells':cells(scope)}


def prepare(transport, environment, scope, outputs, templates):
    expected = identity(environment, scope)
    source = transport.get('/git/commits/' + expected['sourceSHA'])
    check(source.get('sha') == expected['sourceSHA'] and re.fullmatch(r'[a-f0-9]{40}', source.get('tree', {}).get('sha', '')), 'Signer source tree is unavailable')
    expected['sourceTree'] = source['tree']['sha']
    return bindings.build_manifest_from_fixed_outputs(transport, expected, outputs, templates)


def main():
    scope = os.environ.get('SIGNER_SCOPE', '')
    identity(os.environ, scope)  # Fail before constructing any API transport.
    # These bytes are part of the pinned signer checkout, not caller inputs.
    directory = ROOT / 'ci-provenance-templates' / scope
    index = bindings.strict_json((directory / 'index.json').read_bytes())
    check(isinstance(index, list) and index and len(index) == len(set(index)), 'Invalid reviewed template inventory')
    templates = {}
    for path in index:
        check(isinstance(path, str) and path.startswith(('.github/', 'scripts/')) and '..' not in path.split('/'), 'Invalid reviewed template path')
        file = directory / path
        check(file.is_file() and not file.is_symlink(), 'Reviewed template is missing')
        templates[path] = file.read_bytes()
    spec = importlib.util.spec_from_file_location('signer_read_transport', ROOT / 'ci-native-evidence.py')
    api = importlib.util.module_from_spec(spec); spec.loader.exec_module(api)
    transport = api.GitHubReadOnly(REPOSITORY, os.environ.get('GH_TOKEN', ''))
    outputs = bindings.strict_json(os.environ.get('SIGNER_UPLOAD_BINDINGS', '').encode())
    body = prepare(transport, os.environ, scope, outputs, templates)
    destination = Path('ci-provenance-output')
    destination.mkdir()  # Never reuse a previous invocation's output directory.
    (destination / 'ci-bindings.json').write_bytes(body)
    (destination / 'predicate.json').write_text(json.dumps({'schemaVersion':1,'kind':'fixed-workflow-upload-bindings'}), encoding='utf-8')


if __name__ == '__main__':
    main()
