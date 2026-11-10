// cspell:words XDG NOSYSTEM
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { executionPlan } from './ci-suite-execution.mjs';
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
  write('Cargo.toml', '[workspace]\nmembers=[]\n');
  write('pnpm-workspace.yaml', 'packages:\n  - packages/*\n  - examples/*\n');
  for (const name of ['core','cli','runtime','runtime-plugin-hmr','runtime-plugin-import-meta']) write(`packages/${name}/package.json`, JSON.stringify({name:'@farmfe/'+name}));
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

test('leaf changes still execute all discovered tests and reject empty success', () => {
  const fixture = model();
  try {
    fixture.write('examples/react/src/main.ts', 'changed'); fixture.git('add', '.'); fixture.git('commit', '--quiet', '-m', 'leaf');
    const data = readGitSuiteFiles(fixture.directory);
    const inventory = discoverSuiteInventory(data.files, suitePolicy, data.sourceIdentity, data.exampleDirectories);
    for (const cache of ['hit', 'miss']) {
      const browser = executionPlan(inventory, 'examples-browser', fixture.directory, { FARM_NATIVE_CACHE: cache });
      assert.deepEqual(browser.ids, ['examples-browser:examples/react', 'examples-browser:examples/two']);
      assert.throws(() => checkBrowserExecution(inventory, new Map([['react', [{passed:true}]]]), ['react'], browser));
      const ts = executionPlan(inventory, 'ts', fixture.directory, { FARM_NATIVE_CACHE: cache });
      assert.deepEqual(ts.ids, ['ts:packages/a/a.spec.ts']);
      assert.throws(() => checkVitestExecution(inventory, {success:true,testResults:[],numTotalTests:0,numPassedTests:0}, fixture.directory, value=>value, ts));
    }
  } finally { fixture.close(); }
});

test('actual workflow executes full suites without historical selection', () => {
  const workflow = fs.readFileSync(path.join(root, '.github/workflows/ci.yaml'), 'utf8');
  const primary = fs.readFileSync(path.join(root, '.github/workflows/ci-primary-platform.yaml'), 'utf8');
  assert.doesNotMatch(workflow, /suite_selection_hash|ci-suite-baseline.py --output/);
  assert.ok(workflow.includes('ci-suite-execution.mjs --collect ci-gate-suites'));
  assert.doesNotMatch(primary, /selection-only|FARM_SUITE_SELECTION_HASH/);
});

// Exercise real Git discovery, including mode-only changes; all tests run.
for (const scenario of ['noop', 'add', 'edit', 'delete', 'rename', 'executable', 'symlink',
  'rename-to-product', 'rename-from-product', 'lockfile', 'cargo-lock', 'workflow', 'public-docs', 'mdx', 'config', 'runtime', 'hmr', 'import-meta', 'mixed-react']) {
  test(`committed full-test discovery: ${scenario}`, () => {
    const fixture = model();
    const note = 'docs/experiences/notes.md';
    const commit = message => { fixture.git('add', '-A'); fixture.git('commit', '--quiet', '-m', message); };
    try {
      fixture.write(note, 'baseline notes\n');
      fixture.write('examples/react/src/main.ts', 'baseline source\n');
      commit('seed');
      const baselineSHA = fixture.git('rev-parse', 'HEAD').trim();
      const baselineTree = fixture.git('rev-parse', 'HEAD^{tree}').trim();
      if (scenario === 'add') fixture.write('docs/experiences/new.md', 'new notes\n');
      else if (scenario === 'edit') fixture.write(note, 'updated notes\n');
      else if (scenario === 'delete') fixture.git('rm', note);
      else if (scenario === 'rename') fixture.git('mv', note, 'docs/experiences/renamed.md');
      else if (scenario === 'executable') { fixture.git('update-index', '--chmod=+x', note); }
      else if (scenario === 'symlink') { fs.unlinkSync(path.join(fixture.directory, note)); fs.symlinkSync('../../package.json', path.join(fixture.directory, note)); }
      else if (scenario === 'rename-to-product') fixture.git('mv', note, 'new-source.ts');
      else if (scenario === 'rename-from-product') fixture.git('mv', 'examples/react/src/main.ts', 'docs/experiences/source.md');
      else if (scenario === 'mixed-react') { fixture.write(note, 'updated notes\n'); fixture.write('examples/react/src/main.ts', 'updated source\n'); }
      else if (scenario !== 'noop') fixture.write({lockfile: 'pnpm-lock.yaml', 'cargo-lock': 'Cargo.lock', workflow: '.github/workflows/ci.yaml', 'public-docs': 'website/docs/guide.md', mdx: 'docs/experiences/component.mdx', config: 'examples/react/farm.config.ts', runtime: 'packages/runtime/src/index.ts',
        hmr: 'packages/runtime-plugin-hmr/src/index.ts', 'import-meta': 'packages/runtime-plugin-import-meta/src/index.ts'}[scenario], 'changed\n');
      if (scenario === 'executable') fixture.git('commit', '--quiet', '-m', scenario);
      else if (scenario !== 'noop') commit(scenario);
      const data = readGitSuiteFiles(fixture.directory);
      const inventory = discoverSuiteInventory(data.files, suitePolicy, data.sourceIdentity, data.exampleDirectories);
      for (const kind of ['ts', 'examples-build', 'examples-browser']) {
        const cold=executionPlan(inventory,kind,fixture.directory,{});
        const warm=executionPlan(inventory,kind,fixture.directory,{FARM_SUITE_SELECTION_HASH:'old-proof',FARM_NATIVE_CACHE:'hit'});
        assert.equal(cold.mode,'all');assert.deepEqual(warm,cold);assert.ok(cold.ids.length>0);
      }
    } finally { fixture.close(); }
  });
}
