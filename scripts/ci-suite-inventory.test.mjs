import test from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import { discoverSuiteInventory, makeSuiteExecutionPlan, validateSuiteExecution, canonicalSuiteFile, runtimePrerequisites, suitePolicy, readGitSuiteFiles } from './ci-suite-inventory.mjs';

const hash = text => crypto.createHash('sha256').update(text).digest('hex');

test('explicit ArcGIS preview exclusion is pinned to actual source and carried in its plan', () => {
  const {files, policy} = fixture(), file = 'examples/arcgis/e2e.spec.mjs';
  files[file] = fs.readFileSync(new URL('../' + file, import.meta.url), 'utf8');
  files['examples/arcgis/package.json'] = '{"name":"arcgis","scripts":{"build":"farm build"}}';
  const actual = discoverSuiteInventory(files, policy);
  const plan = makeSuiteExecutionPlan(actual, 'examples-browser');
  assert.deepEqual(plan.excludedCases, [{id: 'examples-browser:examples/arcgis', name: 'run preview',
    reason: 'Existing explicit preview exclusion; run start remains required.', sourceDigest: suitePolicy.browserCaseExclusions[file].sourceDigest}]);
  files[file] += '\n// changed declaration\n';
  assert.throws(() => discoverSuiteInventory(files, policy), /exclusion source changed/);
});

test('committed discovery verifies bounded Git blobs and rejects wrong types or invalid UTF8', () => {
  const oid = 'e'.repeat(40), body = Buffer.from('{"title":"中文 🚀"}');
  const archive = Buffer.concat([Buffer.from(`${oid} blob ${body.length}\n`), body, Buffer.from('\n')]);
  const reader = (data, mode = '100644') => (args, options) => {
    if (args[0] === 'rev-parse') return (args[1] === 'HEAD' ? 'a' : 'b').repeat(40) + '\n';
    if (args[0] === 'ls-tree') return Buffer.from(`${mode} blob ${oid}\tpackage.json\0`);
    assert.deepEqual(args, ['cat-file', '--batch']); assert.equal(options.input, oid + '\n');
    return data;
  };
  const result = readGitSuiteFiles('fixture', reader(archive));
  assert.equal(result.files['package.json'], body.toString('utf8'));
  assert.equal(result.sourceIdentity.commit, 'a'.repeat(40));
  assert.equal(result.sourceIdentity.trackedInputsOnly, true);
  assert.throws(() => readGitSuiteFiles('fixture', reader(archive, '120000')));
  assert.throws(() => readGitSuiteFiles('fixture', reader(archive.subarray(0, -1))));
  assert.throws(() => readGitSuiteFiles('fixture', reader(Buffer.concat([archive, Buffer.from('extra')]))));
  const invalid = Buffer.concat([Buffer.from(`${oid} blob 1\n`), Buffer.from([255]), Buffer.from('\n')]);
  assert.throws(() => readGitSuiteFiles('fixture', reader(invalid)));
});

test('reader preserves README-only Examples and rejects redirected or conflicting source identities', () => {
  const oid = 'e'.repeat(40), other = 'f'.repeat(40), body = Buffer.from('{"devDependencies":{"vitest":"2.0.4"}}');
  const baseReader = (extra, wrongTree = false) => (args) => {
    if (args[0] === 'rev-parse') return args[1] === 'HEAD' ? 'a'.repeat(40) : (wrongTree && args[1] !== 'HEAD^{tree}' ? 'c' : 'b').repeat(40);
    if (args[0] === 'ls-tree') return Buffer.from(`100644 blob ${oid}\tpackage.json\0${extra}`);
    assert.deepEqual(args, ['cat-file', '--batch']);
    return Buffer.concat([Buffer.from(`${oid} blob ${body.length}\n`), body, Buffer.from('\n')]);
  };
  const read = readGitSuiteFiles('fixture', baseReader(`100644 blob ${other}\texamples/notes/README.md\0`));
  assert.deepEqual(read.exampleDirectories, ['notes']);
  assert.equal(read.files['examples/notes/README.md'], undefined);
  const inventory = discoverSuiteInventory(read.files, { ...suitePolicy, inputs: {} }, read.sourceIdentity, read.exampleDirectories);
  for (const kind of ['examples-build', 'examples-browser']) assert.ok(inventory.excluded.some(row => row.directory === 'examples/notes' && row.kind === kind));
  for (const extra of [`120000 blob ${other}\texamples/linked\0`, `160000 commit ${other}\texamples/external\0`, `100644 blob ${other}\texamples/Notes/README.md\0${`100644 blob ${other}\texamples/notes/other.md\0`}`, `100644 blob ${other}\texamples/cafe\u0301/README.md\0`]) assert.throws(() => readGitSuiteFiles('fixture', baseReader(extra)));
  assert.throws(() => readGitSuiteFiles('fixture', baseReader('', true)), /commit and HEAD tree differ/);
});

function fixture() {
  const files = {
    'package.json': JSON.stringify({ devDependencies: { vitest: '2.0.4' } }),
    'vitest.config.ts': "include: ['packages/**/*.spec.ts', 'js-plugins/**/*.spec.ts']",
    'packages/core/package.json': '{"name":"@farmfe/core"}',
    'packages/core/tests/example.spec.ts': 'test("real case", () => {})',
    'packages/core/dist/excluded.spec.ts': '',
    'js-plugins/demo/package.json': '{"name":"demo"}',
    'js-plugins/demo/example.spec.ts': '',
    'examples/demo/package.json': '{"name":"example","scripts":{"build":"farm build"},"dependencies":{"@farmfe/docs":"workspace:*","@farmfe/plugin-demo":"workspace:*"}}',
    'examples/demo/index.html': '<html></html>',
    'examples/demo/e2e.spec.mjs': 'startAndTest()',
    'examples/empty/README.md': 'No runnable entry',
    'examples/issues1433/index.html': '<html></html>',
    'examples/nestjs/e2e.spec.mjs': 'watchAndTest()',
    'examples/smoke/index.html': '<html></html>',
    'packages/docs/package.json': '{"name":"@farmfe/docs","dependencies":{"@farmfe/plugin-docs":"workspace:*"}}',
    'rust-plugins/demo/package.json': '{"name":"@farmfe/plugin-demo"}'
  };
  const policy = { ...suitePolicy, inputs: { 'vitest.config.ts': hash(files['vitest.config.ts']) } };
  return { files, policy, inventory: discoverSuiteInventory(files, policy) };
}

test('discovery records actual spec files owners exclusions and separate Examples/create-farm suites', () => {
  const { inventory } = fixture();
  assert.equal(inventory.schemaVersion, 1); assert.equal(inventory.selectionReady, false); assert.equal(inventory.executionEnabled, false);
  assert.equal(inventory.fileEnumerationComplete, true); assert.equal(inventory.ownerMappingComplete, false);
  assert.equal(inventory.dependencyMappingComplete, false); assert.equal(inventory.discoveryComplete, false);
  assert.deepEqual(inventory.items.filter(row => row.kind === 'ts').map(row => row.file).sort(), ['js-plugins/demo/example.spec.ts', 'packages/core/tests/example.spec.ts']);
  assert.equal(inventory.items.find(row => row.id === 'ts:packages/core/tests/example.spec.ts').owner.manifest, 'packages/core/package.json');
  assert.ok(inventory.excluded.some(row => row.file === 'packages/core/dist/excluded.spec.ts' && row.reason === 'vitest-default-exclude'));
  assert.ok(inventory.excluded.some(row => row.directory === 'examples/issues1433' && row.kind === 'examples-browser'));
  assert.ok(inventory.items.some(row => row.id === 'examples-browser:examples/nestjs' && row.mode === 'explicit-spec'));
  assert.ok(inventory.items.some(row => row.id === 'examples-browser:examples/smoke' && row.mode === 'default-start-preview-smoke'));
  assert.equal(inventory.items.filter(row => row.kind === 'create-farm').length, 8);
  const example = inventory.items.find(row => row.id === 'examples-build:examples/demo');
  assert.deepEqual(example.prerequisites.native, ['demo', 'docs']);
  assert.deepEqual(example.prerequisites.public, ['docs', 'docs-theme-default']);
  assert.deepEqual(example.prerequisites.runtime, runtimePrerequisites);
  assert.ok(runtimePrerequisites.includes('packages/runtime-plugin-import-meta'));
});

test('duplicate package names and unreviewed config do not authorize selective discovery', () => {
  const { files, policy } = fixture();
  files['examples/other/package.json'] = files['examples/demo/package.json'];
  const inventory = discoverSuiteInventory(files, policy);
  assert.ok(inventory.fullOnlyReasons.some(row => row.code === 'ambiguous-package-name'));
  assert.equal(inventory.ownerMappingComplete, false);
  assert.throws(() => makeSuiteExecutionPlan(inventory, 'ts', { mode: 'selected', ids: [inventory.items.find(row => row.kind === 'ts').id] }), /requires all/);
  files['vitest.config.ts'] += '\n// changed discovery policy';
  assert.throws(() => makeSuiteExecutionPlan(discoverSuiteInventory(files, policy), 'ts'), /incomplete/);
});

test('coverage requires every planned nonempty terminal result and real cases', () => {
  const complete = fixture();
  for (const name of ['nestjs', 'smoke']) complete.files[`examples/${name}/package.json`] = JSON.stringify({ name: `example-${name}`, scripts: { build: 'farm build' } });
  const inventory = discoverSuiteInventory(complete.files, complete.policy), plan = makeSuiteExecutionPlan(inventory, 'ts');
  assert.deepEqual(inventory.fullOnlyReasons, []);
  assert.throws(() => makeSuiteExecutionPlan(inventory, 'ts', { mode: 'selected', ids: [plan.ids[0]] }), /not authorized/);
  const results = plan.ids.map(id => ({ id, inventoryHash: inventory.inventoryHash, planHash: plan.planHash, status: 'executed-success', skipped: false, executedCases: 1 }));
  assert.deepEqual(validateSuiteExecution(inventory, plan, results), { coverageVerified: true, impactAuthorizationVerified: false, publicationAuthorized: false, skipAuthorized: false, reuseAuthorized: false });
  for (const invalid of [[], results.slice(1), [results[0], results[0]], results.map(row => ({ ...row, executedCases: 0 })), results.map(row => ({ ...row, skipped: true })), results.map(row => ({ ...row, status: 'executed-failure' })), results.map(row => ({ ...row, inventoryHash: 'old' })), results.map(row => ({ ...row, id: 'ts:../escape' }))]) assert.throws(() => validateSuiteExecution(inventory, plan, invalid));
  assert.throws(() => makeSuiteExecutionPlan({ ...inventory, items: [] }, 'ts'));
  const { files, policy } = fixture();
  delete files['packages/core/tests/example.spec.ts']; delete files['js-plugins/demo/example.spec.ts'];
  assert.throws(() => makeSuiteExecutionPlan(discoverSuiteInventory(files, policy), 'ts'), /Empty/);
  for (const file of ['../escape', 'packages//demo.spec.ts', '/absolute', 'packages/./demo.spec.ts', 'C:/escape', 'packages\\escape']) assert.throws(() => canonicalSuiteFile(file));
});

test('build browser and template evidence cannot substitute for one another', () => {
  const { inventory } = fixture();
  const build = makeSuiteExecutionPlan(inventory, 'examples-build'), browser = makeSuiteExecutionPlan(inventory, 'examples-browser'), templates = makeSuiteExecutionPlan(inventory, 'create-farm');
  assert.ok(build.ids.every(id => id.startsWith('examples-build:')));
  assert.ok(browser.ids.every(id => id.startsWith('examples-browser:')));
  assert.ok(templates.ids.every(id => id.startsWith('create-farm:')));
  const emptyCases = templates.ids.map(id => ({ id, inventoryHash: inventory.inventoryHash, planHash: templates.planHash, status: 'executed-success', skipped: false, executedCases: 0 }));
  assert.throws(() => validateSuiteExecution(inventory, templates, emptyCases), /No-tests/);
});
