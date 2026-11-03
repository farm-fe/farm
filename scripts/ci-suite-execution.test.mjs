import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import {stripTypeScriptTypes} from 'node:module';
import { checkVitestExecution, checkBrowserExecution, receiptIdentity, validateCollectedSuites, vitestFailureSummary, readVitestFailureSummary, runVitestCommand, typeScriptChildEnvironment } from './ci-suite-execution.mjs';
import { makeSuiteExecutionPlan } from './ci-suite-inventory.mjs';

function inventory(kind) {
  const row = { schemaVersion: 1, protocol: 'farm-suite-discovery-v1', fileEnumerationComplete: true, selectionReady: false,
    fullOnlyReasons: [], sourceIdentity: { commit: 'a'.repeat(40), tree: 'b'.repeat(40) },
    items: [{ id: kind === 'ts' ? 'ts:packages/a/a.spec.ts' : 'examples-browser:examples/react', kind, requiresActualCases: true }] };
  return { ...row, inventoryHash: crypto.createHash('sha256').update(JSON.stringify(row)).digest('hex') };
}

test('actual Farm env source exposes harness variables unless the TS child is isolated', () => {
  const source = fs.readFileSync(new URL('../packages/core/src/config/env.ts', import.meta.url), 'utf8');
  const code = stripTypeScriptTypes(source).replace(/^import[\s\S]*?;[ \t]*(?:\r?\n)?/gm, '').replace(/^export /gm, '');
  const create = new Function('fs', 'path', 'parse', 'expand', 'arraify', 'normalizePath', 'tryStatSync', 'process', code + '\nreturn loadEnv;');
  const load = env => create({}, path, () => ({}), () => {}, value => Array.isArray(value) ? value : [value], value => value, () => null, {env})('development', '/empty-fixture');
  const parent = {FARM_SUITE_ABI: 'linux-x64-gnu', FARM_SUITE_SELECTION_HASH: '', FARM_SUITE_RESULTS_FILE: 'ci-suite-results/ts.json', FARM_DISABLE_CACHE: ''};
  assert.deepEqual(load(parent), parent);
  assert.deepEqual(load(typeScriptChildEnvironment(parent)), {FARM_DISABLE_CACHE: ''});
  assert.equal(parent.FARM_SUITE_ABI, 'linux-x64-gnu');
});

test('actual TS child invocation keeps product env and parent evidence while removing only harness controls', async () => {
  const parent = {FARM_SUITE_ABI: 'linux-x64-gnu', FARM_SUITE_SELECTION_HASH: 'bound hash', FARM_DISABLE_CACHE: 'true', VITE_FEATURE: 'kept'};
  let observed;
  await runVitestCommand({run: async (_file, _args, options) => {observed = options.env;}, npm: 'npm-cli.js', filters: [],
    raw: 'not-written.json', directory: '.', env: parent, log: 'not-written.log'});
  assert.equal(observed.FARM_SUITE_ABI, undefined); assert.equal(observed.FARM_SUITE_SELECTION_HASH, undefined);
  assert.equal(observed.FARM_DISABLE_CACHE, 'true'); assert.equal(observed.VITE_FEATURE, 'kept');
  assert.equal(parent.FARM_SUITE_SELECTION_HASH, 'bound hash');
});

test('failed Vitest child reports the actual bounded assertion and preserves the original failure', async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-vitest-diagnostic-'));
  try {
    const raw = path.join(directory, 'vitest.json'), output = [], failure = new Error('fake child exit');
    const env = {GH_TOKEN: 'sentinel-private-value'};
    const report = {success: false, numFailedTests: 1, numFailedTestSuites: 1, numTotalTests: 3, testResults: [{name: path.join(directory, 'packages/a.spec.ts'), status: 'failed',
      assertionResults: [{status: 'failed', fullName: 'actual failing case', failureMessages: ['Expected 1 to equal 2; sentinel-private-value ::warning::']}]}]};
    let observed;
    await assert.rejects(runVitestCommand({npm: 'npm-cli.js', filters: [], raw, directory, env, log: path.join(directory, 'vitest.log'), emit: line => output.push(line),
      run: async (_executable, args) => { observed = args; fs.writeFileSync(raw, JSON.stringify(report)); throw failure; }}), error => error === failure);
    assert.ok(observed.includes('--reporter=default')); assert.ok(observed.includes('--reporter=json'));
    assert.ok(output[0].includes('actual failing case')); assert.ok(output[0].includes('Expected 1 to equal 2'));
    assert.ok(!output.join('').includes(env.GH_TOKEN)); assert.ok(!output.join('').includes('::warning::'));
    const receipt = JSON.parse(fs.readFileSync(path.join(directory, 'ts-failure.json'), 'utf8'));
    assert.equal(receipt.kind, 'vitest-failure-diagnostic-not-acceptance'); assert.equal(receipt.failedTests, 1);
    assert.equal(receipt.failures[0].file, 'packages/a.spec.ts');
    fs.writeFileSync(raw, '{invalid'); assert.equal(readVitestFailureSummary(raw, directory).unavailable, 'missing-or-invalid-report');
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});

test('Vitest diagnostics bound collection errors, paths and messages without claiming execution', () => {
  const directory = path.resolve('source');
  const result = vitestFailureSummary({numFailedTests: -1, testResults: Array.from({length: 40}, () => ({status: 'failed', name: path.resolve('outside.spec.ts'), message: 'x'.repeat(10000)}))}, directory);
  assert.equal(result.failedTests, null); assert.equal(result.failures.length, 12);
  assert.equal(result.failures[0].file, '[outside-source]'); assert.equal(result.failures[0].message.length, 1800); assert.equal(result.truncated, true);
});

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

test('only source-declared browser exclusions are separate from actual executed cases', () => {
  const original = inventory('examples-browser'); delete original.inventoryHash;
  const exclusion = {id: original.items[0].id, name: 'run preview', reason: 'Reviewed source exclusion', sourceDigest: 'd'.repeat(64)};
  original.items[0].excludedCases = [exclusion];
  const selected = {...original, inventoryHash: crypto.createHash('sha256').update(JSON.stringify(original)).digest('hex')};
  const rows = [{fullName: 'run start', passed: true, skipped: false}, {fullName: 'run preview', passed: true, skipped: true}];
  const result = checkBrowserExecution(selected, new Map([['react', rows]]), ['react']);
  assert.equal(result.executedCases, 1); assert.equal(result.caseCounts[0].executedCases, 1); assert.equal(result.skipped, 0);
  assert.deepEqual(result.excludedCases, [exclusion]);
  for (const changed of [[rows[0]], [rows[1]], [...rows, rows[1]], [rows[0], {...rows[1], fullName: 'unknown skip'}]]) {
    assert.throws(() => checkBrowserExecution(selected, new Map([['react', changed]]), ['react']));
  }
  assert.throws(() => checkBrowserExecution(inventory('examples-browser'), new Map([['react', rows]]), ['react']), /reviewed exclusion/);
  const extra = new Map([['react', rows], ['unknown-example', [{passed: true, skipped: true}]]]);
  assert.throws(() => checkBrowserExecution(selected, extra, ['react']), /Unknown or undeclared/);
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
