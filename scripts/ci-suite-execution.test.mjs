import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import crypto from 'node:crypto';
import { checkVitestExecution, checkBrowserExecution, receiptIdentity, validateCollectedSuites } from './ci-suite-execution.mjs';
import { makeSuiteExecutionPlan } from './ci-suite-inventory.mjs';

function inventory(kind) {
  const row = { schemaVersion: 1, protocol: 'farm-suite-discovery-v1', fileEnumerationComplete: true, selectionReady: false,
    fullOnlyReasons: [], sourceIdentity: { commit: 'a'.repeat(40), tree: 'b'.repeat(40) },
    items: [{ id: kind === 'ts' ? 'ts:packages/a/a.spec.ts' : 'examples-browser:examples/react', kind, requiresActualCases: true }] };
  return { ...row, inventoryHash: crypto.createHash('sha256').update(JSON.stringify(row)).digest('hex') };
}

test('actual Vitest files and cases must match committed nonempty discovery', () => {
  const report = { success: true, numTotalTests: 2, numPassedTests: 2,
    testResults: [{ name: 'packages/a/a.spec.ts', status: 'passed', assertionResults: [{ status: 'passed' }, { status: 'passed' }] }] };
  const result = checkVitestExecution(inventory('ts'), report, path.resolve('source'), value => value);
  assert.equal(result.executedCases, 2); assert.equal(result.executedIds.length, 1);
  for (const change of [row => { row.testResults = []; }, row => { row.testResults[0].assertionResults = []; },
    row => { row.testResults[0].assertionResults[0].status = 'pending'; }, row => { row.numPassedTests = 1; },
    row => { row.testResults[0].name = '../other/a.spec.ts'; }]) {
    const bad = structuredClone(report); change(bad);
    assert.throws(() => checkVitestExecution(inventory('ts'), bad, path.resolve('source'), value => value));
  }
});

test('browser terminal cases cannot be inferred from a positive worker exit', () => {
  const selected = inventory('examples-browser');
  const rows = new Map([['react', [{ passed: true, skipped: false }]]]);
  assert.equal(checkBrowserExecution(selected, rows, ['react']).executedCases, 1);
  assert.throws(() => checkBrowserExecution(selected, new Map(), ['react']), /Missing browser/);
  assert.throws(() => checkBrowserExecution(selected, new Map([['react', []]]), ['react']), /Missing browser/);
  assert.throws(() => checkBrowserExecution(selected, new Map([['react', [{ passed: true, skipped: true }]]]), ['react']), /skipped/);
  assert.throws(() => checkBrowserExecution(selected, rows, []), /discovery differs/);
  rows.set('worker-failure', [{ passed: false }]);
  assert.throws(() => checkBrowserExecution(selected, rows, ['react']), /fatal or failed/);
});

test('runner identity records actual Git source and current Actions attempt', () => {
  const env = { GITHUB_ACTIONS: 'true', GITHUB_RUN_ID: '1', GITHUB_RUN_ATTEMPT: '2', GITHUB_REPOSITORY_ID: '3',
    GITHUB_SHA: 'a'.repeat(40), GITHUB_JOB: 'ts-test', GITHUB_WORKFLOW_REF: 'farm-fe/farm/.github/workflows/ci.yaml@refs/pull/2330/merge',
    GITHUB_EVENT_NAME: 'pull_request', FARM_SUITE_ABI: 'win32-x64-msvc' };
  assert.equal(receiptIdentity(inventory('ts'), env).run.runAttempt, 2);
  assert.throws(() => receiptIdentity(inventory('ts'), { ...env, GITHUB_SHA: 'c'.repeat(40) }), /checkout differs/);
  assert.throws(() => receiptIdentity(inventory('ts'), { ...env, GITHUB_ACTIONS: 'false' }), /only in Actions/);
});

test('complete collection requires every platform and original per-file case counts', () => {
  const original = inventory('ts'); delete original.inventoryHash;
  original.items.push({ id: 'examples-browser:examples/react', kind: 'examples-browser', requiresActualCases: true });
  const combined = { ...original, inventoryHash: crypto.createHash('sha256').update(JSON.stringify(original)).digest('hex') };
  const env = { GITHUB_ACTIONS: 'true', GITHUB_RUN_ID: '1', GITHUB_RUN_ATTEMPT: '2', GITHUB_REPOSITORY_ID: '3',
    GITHUB_SHA: 'a'.repeat(40), GITHUB_JOB: 'performance-summary', GITHUB_WORKFLOW_REF: 'farm-fe/farm/.github/workflows/ci.yaml@refs/pull/2330/merge',
    GITHUB_EVENT_NAME: 'pull_request' };
  const rows = ['linux-x64-gnu', 'darwin-arm64', 'win32-x64-msvc'].flatMap(abi => ['ts', 'examples-browser'].map(kind => {
    const plan = makeSuiteExecutionPlan(combined, kind);
    const data = { ...receiptIdentity(combined, { ...env, FARM_SUITE_ABI: abi, GITHUB_JOB: kind === 'ts' ? 'ts-test' : 'examples-test' }),
      schemaVersion: 1, receiptKind: 'actual-suite-execution-not-byte-reuse', kind, inventoryHash: combined.inventoryHash, planHash: plan.planHash,
      selectedIds: plan.ids, executedIds: plan.ids, executedCases: 1, caseCounts: plan.ids.map(id => ({ id, executedCases: 1 })), failed: 0, skipped: 0 };
    return { artifactName: `farm-ci-suite-${kind === 'ts' ? 'ts' : 'browser'}-${env.GITHUB_SHA}-${abi}-2`, data };
  }));
  assert.equal(validateCollectedSuites(combined, rows, env).cells.length, 6);
  assert.throws(() => validateCollectedSuites(combined, rows.slice(1), env), /All three platforms/);
  const duplicate = structuredClone(rows); duplicate[1] = duplicate[0];
  assert.throws(() => validateCollectedSuites(combined, duplicate, env), /Duplicate or foreign/);
  const forged = structuredClone(rows); forged[0].data.executedCases = 100;
  assert.throws(() => validateCollectedSuites(combined, forged, env), /case totals differ/);
  const old = structuredClone(rows); old[0].data.run.runAttempt = 1;
  assert.throws(() => validateCollectedSuites(combined, old, env));
});
