// cspell:words XDG NOSYSTEM
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { calculateSelection, executionPlan } from './ci-suite-selection.mjs';
import { readGitSuiteFiles, discoverSuiteInventory, suitePolicy } from './ci-suite-inventory.mjs';
import { checkBrowserExecution, checkVitestExecution } from './ci-suite-execution.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
function model() {
  const directory = fs.realpathSync.native(fs.mkdtempSync(path.join(os.tmpdir(), 'farm-suite-selection-')));
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !/^GIT_/i.test(key)));
  Object.assign(env, { GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: path.join(directory, 'empty-config'),
    XDG_CONFIG_HOME: directory, HOME: directory,
    GIT_AUTHOR_NAME: 'Suite Fixture', GIT_AUTHOR_EMAIL: 'fixture@example.invalid', GIT_COMMITTER_NAME: 'Suite Fixture', GIT_COMMITTER_EMAIL: 'fixture@example.invalid' });
  fs.writeFileSync(env.GIT_CONFIG_GLOBAL, '');
  const git = (...args) => execFileSync('git', args, { cwd: directory, env, encoding: 'utf8', timeout: 10000 });
  git('init', '--quiet');
  assert.equal(fs.realpathSync.native(git('rev-parse', '--show-toplevel').trim()), directory);
  const write = (file, body) => { const target = path.join(directory, file); fs.mkdirSync(path.dirname(target), { recursive: true }); fs.writeFileSync(target, body); };
  write('package.json', JSON.stringify({ devDependencies: { vitest: '2.0.4' } }));
  for (const file of Object.keys(suitePolicy.inputs)) write(file, fs.readFileSync(path.join(root, file)));
  for (const name of ['react', 'two']) {
    write(`examples/${name}/package.json`, JSON.stringify({ name, scripts: { build: 'node build.mjs' } }));
    write(`examples/${name}/index.html`, '<html></html>');
  }
  write('packages/a/package.json', JSON.stringify({ name: 'a' }));
  write('packages/a/a.spec.ts', 'test source');
  git('add', '.'); git('commit', '--quiet', '-m', 'baseline');
  const baselineSHA = git('rev-parse', 'HEAD').trim(), baselineTree = git('rev-parse', 'HEAD^{tree}').trim();
  return { directory, git, write, baselineSHA, baselineTree, close: () => fs.rmSync(directory, { recursive: true, force: true }) };
}

test('real isolated Git leaf change selects actual runner files and proves empty TS explicitly', () => {
  const fixture = model();
  try {
    fixture.write('examples/react/src/main.ts', 'changed'); fixture.git('add', '.'); fixture.git('commit', '--quiet', '-m', 'leaf');
    const identity = { kind: 'independently-verified-ci-suite-baseline', sourceSHA: fixture.git('rev-parse', 'HEAD').trim(),
      sourceTree: fixture.git('rev-parse', 'HEAD^{tree}').trim(), baselineSHA: fixture.baselineSHA, baselineTree: fixture.baselineTree,
      repositoryId: 1, runId: 2, attempt: 1, baselineEvidence: { artifactId: 7 } };
    const selection = calculateSelection(identity, fixture.directory);
    assert.equal(selection.decision.full, false);
    assert.deepEqual(selection.decision.sets['examples-browser'].ids, ['examples-browser:examples/react']);
    const folder = path.join(fixture.directory, 'ci-suite-selection'); fs.mkdirSync(folder);
    const bytes = Buffer.from(JSON.stringify(selection)); fs.writeFileSync(path.join(folder, 'selection.json'), bytes);
    const env = { FARM_SUITE_SELECTION_HASH: crypto.createHash('sha256').update(bytes).digest('hex'), GITHUB_SHA: identity.sourceSHA,
      GITHUB_RUN_ID: '2', GITHUB_RUN_ATTEMPT: '1', GITHUB_REPOSITORY_ID: '1' };
    const data = readGitSuiteFiles(fixture.directory), inventory = discoverSuiteInventory(data.files, suitePolicy, data.sourceIdentity, data.exampleDirectories);
    const browser = executionPlan(inventory, 'examples-browser', fixture.directory, env);
    const receipt = checkBrowserExecution(inventory, new Map([['react', [{ passed: true, skipped: false }]]]), ['react'], browser);
    assert.deepEqual(receipt.executedIds, ['examples-browser:examples/react']); assert.equal(receipt.executedCases, 1);
    assert.throws(() => checkBrowserExecution(inventory, new Map(), [], browser));
    const ts = executionPlan(inventory, 'ts', fixture.directory, env);
    assert.equal(ts.mode, 'not-applicable');
    assert.equal(checkVitestExecution(inventory, { success: true, testResults: [], numTotalTests: 0, numPassedTests: 0 }, fixture.directory, value => value, ts).disposition, 'not-applicable');
    assert.throws(() => executionPlan(inventory, 'ts', fixture.directory, { ...env, GITHUB_RUN_ATTEMPT: '2' }));
    fs.writeFileSync(path.join(folder, 'selection.json'), '{}');
    assert.throws(() => executionPlan(inventory, 'ts', fixture.directory, env));
  } finally { fixture.close(); }
});

test('actual workflow binds selection to all platforms and stable suite-count gate', () => {
  const workflow = fs.readFileSync(path.join(root, '.github/workflows/ci.yaml'), 'utf8');
  const primary = fs.readFileSync(path.join(root, '.github/workflows/ci-primary-platform.yaml'), 'utf8');
  assert.equal(workflow.split('suite_selection_hash: ${{ needs.type-check.outputs.suite_selection_hash }}').length - 1, 3);
  assert.ok(workflow.includes('ci-suite-baseline.py --output ci-suite-selection/selection.json'));
  assert.ok(workflow.includes('ci-suite-execution.mjs --collect ci-gate-suites'));
  assert.equal(primary.split('FARM_SUITE_SELECTION_HASH: ${{ inputs.suite_selection_hash }}').length - 1, 3);
});
