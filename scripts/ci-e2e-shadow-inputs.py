"""E2E producer inventory for the shared strict original-attempt reader."""
# cspell:words importlib
import importlib.util
import json
import re
from pathlib import Path

spec = importlib.util.spec_from_file_location('shared_shadow_inputs', Path(__file__).with_name('ci-unit-shadow-inputs.py'))
shared = importlib.util.module_from_spec(spec); spec.loader.exec_module(shared)
impact = shared.impact
PREFIX = {'linux-x64-gnu': 'Primary Linux', 'darwin-arm64': 'Primary macOS', 'win32-x64-msvc': 'Primary Windows'}
CELLS = [
    ('native:ci:' + abi, (PREFIX[abi] + ' / native / Build and Upload User Artifacts - ' if abi in PREFIX else 'Build and Upload Release Artifacts - ') + abi, 'build', abi)
    for abi in impact.TARGETS
]
for abi, runner in shared.execution.PLATFORMS.items():
    CELLS.extend([
        ('examples:' + abi, PREFIX[abi] + ' / Examples Test (' + runner + ')', 'examples-test', abi),
        ('ts-tests:' + abi, PREFIX[abi] + ' / Typescript Test (' + runner + ')', 'ts-test', abi),
        ('platform-contracts:' + abi, 'Platform safety contracts (' + runner + ')', 'platform-contract-tests', runner),
    ])
    if abi != 'darwin-arm64':
        CELLS.append(('docs:ci:' + abi, PREFIX[abi] + ' / Docs CI quality checks / Docs / ' + runner, 'docs', abi))
CELLS.extend([
    ('types', 'Type Check', 'type-check', 'single'),
    ('contracts', 'Release source contracts', 'release-contract-tests', 'single'),
    ('native-assembly', 'Check declared native inputs (not ABI execution)', 'check-native-candidate-assembly', 'single'),
    ('execution-baseline', 'Record verified CI execution baseline', 'execution-baseline', 'single'),
])


PRIMARY_CELLS = {kind + ':' + abi for abi in PREFIX for kind in ['ts-tests', 'examples']}


def primary_upload_outputs(environment):
    # Only fixed workflow needs outputs, never receipt or artifact self-assertions.
    result = {}
    patterns = {'artifact_id': r'[1-9][0-9]*', 'artifact_digest': r'[a-f0-9]{64}', 'run_attempt': r'[1-9][0-9]*'}
    for abi, platform in [('linux-x64-gnu','LINUX'), ('darwin-arm64','MACOS'), ('win32-x64-msvc','WINDOWS')]:
        row = json.loads(environment['CI_SHADOW_UPLOADS_' + platform])
        expected = {label + '_' + prefix + field for label in ['ts','browser'] for prefix in ['', 'shadow_'] for field in patterns}
        shared.check(isinstance(row, dict) and set(row) == expected, 'Missing or unexpected primary shadow upload outputs')
        shared.check(all(isinstance(row[label + '_' + prefix + field], str) and re.fullmatch(pattern, row[label + '_' + prefix + field])
                         for label in ['ts','browser'] for prefix in ['', 'shadow_'] for field, pattern in patterns.items()), 'Malformed primary upload outputs')
        for label, kind in [('ts','ts-tests'), ('browser','examples')]:
            output = {field:row.get(label + '_shadow_' + field) for field in patterns}
            shared.check(all(isinstance(output[field], str) and re.fullmatch(pattern, output[field]) for field, pattern in patterns.items()), 'Malformed or missing shadow upload outputs')
            result[kind + ':' + abi] = output
    return result


def resolve(transport, run, identity, *, upload_outputs=None):
    if upload_outputs is not None:
        shared.check(set(upload_outputs) == PRIMARY_CELLS, 'Shadow upload output inventory differs')
    return shared.resolve(transport, run, identity, scope='e2e', cells=CELLS, upload_outputs=upload_outputs)


def current(environment):
    return shared.current(environment, scope='e2e', cells=CELLS, upload_outputs=primary_upload_outputs(environment))
