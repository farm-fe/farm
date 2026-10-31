import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { readGitSuiteFiles, discoverSuiteInventory, makeSuiteExecutionPlan, suitePolicy } from './ci-suite-inventory.mjs';
import { resolveNpmCli } from './v2-npm-cli.mjs';
import { command } from './performance-report.mjs';
import { executionPlan } from './ci-suite-selection.mjs';
export { executionPlan } from './ci-suite-selection.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
export function actualSuiteInventory(directory = root) {
  const { files, sourceIdentity, exampleDirectories } = readGitSuiteFiles(directory);
  const inventory = discoverSuiteInventory(files, suitePolicy, sourceIdentity, exampleDirectories);
  assert.equal(inventory.fileEnumerationComplete, true, 'Suite file enumeration must be complete');
  return inventory;
}

export function receiptIdentity(inventory, env = process.env) {
  assert.equal(env.GITHUB_ACTIONS, 'true', 'Execution receipts are created only in Actions');
  for (const key of ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_REPOSITORY_ID']) assert.match(env[key] ?? '', /^[1-9][0-9]*$/);
  assert.equal(inventory.sourceIdentity.commit, env.GITHUB_SHA, 'Actual suite checkout differs');
  assert.match(inventory.sourceIdentity.tree, /^[a-f0-9]{40}$/);
  assert.ok(env.GITHUB_JOB && env.GITHUB_WORKFLOW_REF, 'Actual suite workflow identity missing');
  assert.ok(['linux-x64-gnu', 'darwin-arm64', 'win32-x64-msvc'].includes(env.FARM_SUITE_ABI), 'Actual primary suite ABI missing');
  return { sourceSHA: inventory.sourceIdentity.commit, sourceTree: inventory.sourceIdentity.tree,
    run: { repositoryId: Number(env.GITHUB_REPOSITORY_ID), runId: Number(env.GITHUB_RUN_ID), runAttempt: Number(env.GITHUB_RUN_ATTEMPT), event: env.GITHUB_EVENT_NAME },
    jobKey: env.GITHUB_JOB, workflowRef: env.GITHUB_WORKFLOW_REF, abi: env.FARM_SUITE_ABI, chain: 'ci', acceptanceInherited: false };
}

export function checkVitestExecution(inventory, report, directory = root, canonical = fs.realpathSync.native, plan = makeSuiteExecutionPlan(inventory, 'ts')) {
  assert.equal(report.success, true, 'Vitest did not succeed');
  assert.ok(Array.isArray(report.testResults) && report.testResults.length === plan.ids.length, 'Missing Vitest file result');
  const realRoot = canonical(directory);
  const results = report.testResults.map(row => {
    assert.equal(typeof row.name, 'string');
    const relative = path.relative(realRoot, canonical(path.resolve(directory, row.name))).replaceAll('\\', '/');
    assert.ok(relative && !relative.startsWith('../') && !path.isAbsolute(relative), 'Vitest result escaped source');
    const id = 'ts:' + relative;
    assert.ok(plan.ids.includes(id), 'Unexpected Vitest spec result');
    assert.equal(row.status, 'passed', 'Vitest file failed or skipped');
    assert.ok(Array.isArray(row.assertionResults) && row.assertionResults.length > 0, 'No-tests success is not execution');
    assert.ok(row.assertionResults.every(test => test.status === 'passed'), 'Selected Vitest cases failed or skipped');
    return { id, executedCases: row.assertionResults.length };
  });
  assert.equal(new Set(results.map(row => row.id)).size, plan.ids.length, 'Duplicate Vitest file result');
  const executedCases = results.reduce((sum, row) => sum + row.executedCases, 0);
  assert.equal(report.numTotalTests, executedCases, 'Vitest case count differs');
  assert.equal(report.numPassedTests, executedCases, 'Vitest passed count differs');
  return { inventoryHash: inventory.inventoryHash, planHash: plan.planHash, kind: 'ts', disposition: plan.ids.length ? 'executed' : 'not-applicable', selectedIds: plan.ids,
    executedIds: results.map(row => row.id).sort(), caseCounts: results, executedCases, failed: 0, skipped: 0 };
}

export function checkBrowserExecution(inventory, allResults, selectedExamples, plan = makeSuiteExecutionPlan(inventory, 'examples-browser')) {
  assert.ok(allResults instanceof Map && Array.isArray(selectedExamples), 'Actual browser results missing');
  const requested = selectedExamples.map(name => 'examples-browser:examples/' + name).sort();
  assert.deepEqual(requested, plan.ids, 'Browser discovery differs from committed suite inventory');
  assert.ok([...allResults.values()].flat().every(row => row.passed), 'Browser runner has a fatal or failed result');
  let executedCases = 0;
  const executedIds = [], caseCounts = [];
  for (const name of selectedExamples) {
    const rows = allResults.get(name);
    assert.ok(Array.isArray(rows) && rows.length > 0, 'Missing browser terminal cases');
    assert.ok(rows.every(row => row.passed === true && row.skipped !== true), 'Selected browser cases failed or skipped');
    executedCases += rows.length; executedIds.push('examples-browser:examples/' + name);
    caseCounts.push({ id: 'examples-browser:examples/' + name, executedCases: rows.length });
  }
  assert.ok(executedCases > 0 || plan.mode === 'not-applicable' && plan.ids.length === 0, 'Empty browser execution cannot establish a baseline');
  return { inventoryHash: inventory.inventoryHash, planHash: plan.planHash, kind: 'examples-browser', disposition: plan.ids.length ? 'executed' : 'not-applicable', selectedIds: plan.ids,
    executedIds: executedIds.sort(), caseCounts, executedCases, failed: 0, skipped: 0 };
}

export function saveSuiteReceipt(file, inventory, data, env = process.env, directory = root) {
  const output = path.resolve(directory, file), base = path.resolve(directory, 'ci-suite-results');
  assert.equal(path.dirname(output), base, 'Receipt must be a direct member of the CI results directory');
  assert.match(path.basename(output), /^[a-z0-9.-]+\.json$/);
  const identity = receiptIdentity(inventory, env);
  if (data.kind === 'examples-browser') {
    const build = JSON.parse(fs.readFileSync(path.join(base, 'build.json'), 'utf8'));
    const plan = executionPlan(inventory, 'examples-build', directory, env);
    assert.equal(build.sourceSHA, identity.sourceSHA); assert.equal(build.sourceTree, identity.sourceTree);
    assert.deepEqual(build.run, identity.run); assert.equal(build.abi, identity.abi);
    assert.equal(build.inventoryHash, inventory.inventoryHash); assert.equal(build.planHash, plan.planHash);
    assert.deepEqual(build.selectedIds, plan.ids); assert.deepEqual(build.executedIds, plan.ids);
    assert.equal(build.failed, 0); assert.equal(build.skipped, 0);
    data = { ...data, buildExecution: build };
  }
  const record = { ...data, ...identity, schemaVersion: 1, receiptKind: 'actual-suite-execution-not-byte-reuse' };
  fs.mkdirSync(base, { recursive: true });
  assert.ok(!fs.lstatSync(base).isSymbolicLink(), 'Receipt directory redirects');
  assert.equal(fs.realpathSync.native(base), path.join(fs.realpathSync.native(directory), 'ci-suite-results'), 'Receipt directory escaped source');
  if (fs.existsSync(output)) assert.ok(fs.lstatSync(output).isFile() && !fs.lstatSync(output).isSymbolicLink(), 'Receipt file redirects');
  fs.writeFileSync(output, JSON.stringify(record, null, 2) + '\n');
  return record;
}

export async function runTypeScriptEvidence({ directory = root, env = process.env, run = command } = {}) {
  const inventory = actualSuiteInventory(directory), output = 'ci-suite-results/ts.json';
  const plan = executionPlan(inventory, 'ts', directory, env);
  if (plan.mode === 'not-applicable') {
    const data = checkVitestExecution(inventory, { success: true, testResults: [], numTotalTests: 0, numPassedTests: 0 }, directory, fs.realpathSync.native, plan);
    return saveSuiteReceipt(output, inventory, data, env, directory);
  }
  const raw = path.join(directory, 'ci-suite-results', 'vitest.json'), log = path.join(directory, 'ci-suite-results', 'vitest.log');
  fs.mkdirSync(path.dirname(raw), { recursive: true });
  assert.ok(!fs.existsSync(raw) && !fs.existsSync(path.join(directory, output)), 'Stale suite result exists');
  const filters = plan.mode === 'all' ? [] : plan.ids.map(id => id.slice(3));
  await run(process.execPath, [resolveNpmCli(), 'run', 'test', '--', ...filters, '--reporter=json', '--outputFile=' + raw],
    { cwd: directory, env, log, timeoutMs: 30 * 60 * 1000, phase: 'typescript-actual-suite-results' });
  assert.ok(fs.statSync(raw).size <= 16 * 1024 ** 2, 'Vitest report exceeds bound');
  const data = checkVitestExecution(inventory, JSON.parse(fs.readFileSync(raw, 'utf8')), directory, fs.realpathSync.native, plan);
  return saveSuiteReceipt(output, inventory, data, env, directory);
}

export function validateCollectedSuites(inventory, rows, env = process.env) {
  const expected = receiptIdentity(inventory, { ...env, FARM_SUITE_ABI: 'linux-x64-gnu' });
  const abis = ['linux-x64-gnu', 'darwin-arm64', 'win32-x64-msvc'];
  assert.equal(rows.length, 6, 'All three platforms require TS and browser evidence');
  const seen = new Set();
  for (const row of rows) {
    const key = row.data.kind + ':' + row.data.abi;
    assert.ok(abis.includes(row.data.abi) && ['ts', 'examples-browser'].includes(row.data.kind) && !seen.has(key), 'Duplicate or foreign suite cell');
    seen.add(key);
    assert.equal(row.data.schemaVersion, 1); assert.equal(row.data.chain, 'ci');
    assert.equal(row.data.receiptKind, 'actual-suite-execution-not-byte-reuse');
    assert.equal(row.data.sourceSHA, expected.sourceSHA); assert.equal(row.data.sourceTree, expected.sourceTree);
    assert.deepEqual(row.data.run, expected.run); assert.equal(row.data.acceptanceInherited, false);
    const label = row.data.kind === 'ts' ? 'ts' : 'browser';
    assert.equal(row.data.jobKey, row.data.kind === 'ts' ? 'ts-test' : 'examples-test');
    assert.equal(row.data.workflowRef, expected.workflowRef);
    assert.equal(row.artifactName, `farm-ci-suite-${label}-${expected.sourceSHA}-${row.data.abi}-${expected.run.runAttempt}`, 'Suite artifact namespace differs');
    const plan = executionPlan(inventory, row.data.kind, root, env);
    assert.equal(row.data.inventoryHash, inventory.inventoryHash); assert.equal(row.data.planHash, plan.planHash);
    assert.deepEqual(row.data.selectedIds, plan.ids); assert.deepEqual(row.data.executedIds, plan.ids);
    if (env.FARM_SUITE_SELECTION_HASH && row.data.kind === 'examples-browser') {
      const buildPlan = executionPlan(inventory, 'examples-build', root, env), build = row.data.buildExecution;
      assert.ok(build, 'Selected browser evidence requires actual build coverage');
      assert.equal(build.sourceSHA, expected.sourceSHA); assert.equal(build.sourceTree, expected.sourceTree); assert.deepEqual(build.run, expected.run);
      assert.equal(build.abi, row.data.abi); assert.equal(build.inventoryHash, inventory.inventoryHash); assert.equal(build.planHash, buildPlan.planHash);
      assert.deepEqual(build.selectedIds, buildPlan.ids); assert.deepEqual(build.executedIds, buildPlan.ids);
      assert.equal(build.failed, 0); assert.equal(build.skipped, 0);
    }
    if (plan.ids.length === 0) {
      assert.equal(plan.mode, 'not-applicable'); assert.equal(row.data.disposition, 'not-applicable');
      assert.equal(row.data.executedCases, 0); assert.deepEqual(row.data.caseCounts, []);
    }
    assert.equal(row.data.failed, 0); assert.equal(row.data.skipped, 0);
    assert.ok(Array.isArray(row.data.caseCounts) && row.data.caseCounts.length === plan.ids.length, 'Per-file case evidence missing');
    assert.equal(new Set(row.data.caseCounts.map(item => item.id)).size, plan.ids.length);
    let count = 0;
    for (const item of row.data.caseCounts) {
      assert.ok(plan.ids.includes(item.id) && Number.isSafeInteger(item.executedCases) && item.executedCases > 0, 'Invalid semantic case evidence');
      count += item.executedCases;
    }
    assert.equal(count, row.data.executedCases, 'Suite case totals differ');
  }
  return { schemaVersion: 1, kind: 'complete-ci-suite-counts-not-build-bytes', ...expected,
    cells: rows.map(row => ({ ...row.data, artifactName: row.artifactName })), acceptanceInherited: false };
}

export function collectSuites(input, output, directory = root, env = process.env) {
  const inventory = actualSuiteInventory(directory), rows = [];
  assert.ok(fs.statSync(input).isDirectory() && !fs.lstatSync(input).isSymbolicLink(), 'Suite input directory redirects');
  const artifacts = fs.readdirSync(input);
  assert.equal(artifacts.length, 6, 'Missing or extra suite artifact');
  for (const artifactName of artifacts) {
    const folder = path.join(input, artifactName);
    assert.ok(fs.lstatSync(folder).isDirectory() && !fs.lstatSync(folder).isSymbolicLink(), 'Suite artifact directory redirects');
    const files = fs.readdirSync(folder);
    assert.ok(files.length === 1 && ['ts.json', 'browser.json'].includes(files[0]), 'Unexpected suite artifact members');
    const file = path.join(folder, files[0]), info = fs.lstatSync(file);
    assert.ok(info.isFile() && !info.isSymbolicLink() && info.size > 0 && info.size <= 1024 ** 2, 'Suite receipt is invalid or exceeds bound');
    rows.push({ artifactName, data: JSON.parse(fs.readFileSync(file, 'utf8')) });
  }
  const result = validateCollectedSuites(inventory, rows, env);
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, JSON.stringify(result, null, 2) + '\n');
  return result;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const action = process.argv[2] === '--collect'
    ? Promise.resolve().then(() => collectSuites(process.argv[3], process.argv[4])) : runTypeScriptEvidence();
  action.then(row => console.log(JSON.stringify({ kind: row.kind, files: row.executedIds?.length ?? null, cases: row.executedCases ?? null, cells: row.cells?.length ?? null })))
    .catch(() => { console.error('Actual suite execution or count validation failed'); process.exitCode = 1; });
}
