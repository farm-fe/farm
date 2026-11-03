"""E2E producer inventory for the shared strict original-attempt reader."""
# cspell:words importlib
import importlib.util
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
        ('performance:' + abi, 'Commerce Performance (' + runner + ')', 'performance-report', abi),
        ('performance-contracts:' + abi, 'Performance report contract (' + runner + ')', 'performance-contract-tests', runner),
    ])
    if abi != 'darwin-arm64':
        CELLS.append(('docs:ci:' + abi, PREFIX[abi] + ' / Docs CI quality checks / Docs / ' + runner, 'docs', abi))
CELLS.extend([
    ('types', 'Type Check', 'type-check', 'single'),
    ('contracts', 'Release source contracts', 'release-contract-tests', 'single'),
    ('native-assembly', 'Check declared native inputs (not ABI execution)', 'check-native-candidate-assembly', 'single'),
    ('performance-summary', 'Aggregate informational performance report', 'performance-summary', 'single'),
])


def resolve(transport, run, identity):
    return shared.resolve(transport, run, identity, scope='e2e', cells=CELLS)


def current(environment):
    return shared.current(environment, scope='e2e', cells=CELLS)
