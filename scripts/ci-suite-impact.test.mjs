import test from 'node:test';
import assert from 'node:assert/strict';
import { directoryImpactRules } from './ci-directory-impact-rules.mjs';
import { suiteImpact } from './ci-suite-impact.mjs';

const files = {
  'packages/a/package.json': JSON.stringify({ name: 'a' }),
  'js-plugins/b/package.json': JSON.stringify({ name: 'b', dependencies: { a: 'workspace:*' } }),
  'examples/one/package.json': JSON.stringify({ name: 'one', dependencies: { b: 'workspace:*' } }),
  'examples/two/package.json': JSON.stringify({ name: 'two' })
};
const inventory = { fileEnumerationComplete: true, inventoryHash: 'a'.repeat(64), fullOnlyReasons: [], items: [
  { kind: 'ts', id: 'ts:packages/a/a.spec.ts', file: 'packages/a/a.spec.ts' },
  { kind: 'ts', id: 'ts:js-plugins/b/b.spec.ts', file: 'js-plugins/b/b.spec.ts' },
  ...['one', 'two'].flatMap(name => ['examples-build', 'examples-browser'].map(kind => ({ kind, id: `${kind}:examples/${name}`, directory: `examples/${name}` })))
] };
const decide = (changes, before = files, after = files) => suiteImpact({ before, after, changes, inventory,
  auditedOwners: ['packages/a', 'js-plugins/b', 'examples/one', 'examples/two'] });

test('leaf Examples select exact build and browser sets without unrelated TS', () => {
  const plan = decide([{ path: 'examples/one/src/main.ts' }]);
  assert.equal(plan.full, false);
  assert.deepEqual(plan.sets.ts, { mode: 'not-applicable', ids: [] });
  assert.deepEqual(plan.sets['examples-browser'].ids, ['examples-browser:examples/one']);
  assert.deepEqual(plan.sets['examples-build'].ids, ['examples-build:examples/one']);
  assert.equal(plan.executionAuthorized, false);
});
test('JS reverse closure and old graph deletions retain affected consumers', () => {
  const after = structuredClone(files);
  after['examples/one/package.json'] = JSON.stringify({ name: 'one' });
  const plan = decide([{ oldPath: 'packages/a/src/old.ts', path: null }], files, after);
  assert.deepEqual(plan.affectedOwners, ['examples/one', 'js-plugins/b', 'packages/a']);
  assert.deepEqual(plan.sets.ts.ids, ['ts:js-plugins/b/b.spec.ts', 'ts:packages/a/a.spec.ts']);
  assert.deepEqual(plan.sets['examples-browser'].ids, ['examples-browser:examples/one']);
});
test('cross owner rename includes both ends and unknown inputs require all', () => {
  assert.deepEqual(decide([{ oldPath: 'examples/one/src/x.ts', path: 'examples/two/src/x.ts' }]).sets['examples-build'].ids,
    ['examples-build:examples/one', 'examples-build:examples/two']);
  for (const path of ['pnpm-lock.yaml', 'scripts/build.mjs', 'unknown.ts']) assert.equal(decide([{ path }]).full, true);
});
test('unresolved workspace aliases and duplicate owners never select partially', () => {
  const alias = structuredClone(files);
  alias['js-plugins/b/package.json'] = JSON.stringify({ name: 'b', dependencies: { a: 'workspace:other@*' } });
  assert.equal(decide([{ path: 'examples/one/x.ts' }], files, alias).full, true);
  alias['js-plugins/b/package.json'] = JSON.stringify({ name: 'b', dependencies: { a: 'npm:other@1.0.0' } });
  assert.ok(decide([{ path: 'examples/one/src/x.ts' }], files, alias).reasons.includes('unresolved-package-alias'));
  const duplicate = { ...files, 'packages/duplicate/package.json': JSON.stringify({ name: 'a' }) };
  assert.equal(decide([{ path: 'examples/one/x.ts' }], duplicate, duplicate).full, true);
});
test('shared runtime changes execute every related suite and report missing roots', () => {
  const runtime = { ...files, 'packages/runtime/package.json': JSON.stringify({ name: '@farmfe/runtime' }) };
  const result = decide([{ path: 'packages/runtime/src/index.ts' }], runtime, runtime);
  assert.equal(result.full, true);
  assert.ok(result.reasons.includes('required-shared-root-missing'));
  assert.equal(result.sets.ts.mode, 'all');
});


test('documentation exemption requires complete ordinary Git metadata for both sides', () => {
  const file = 'docs/experiences/notes.md';
  const entry = {file, mode: '100644', type: 'blob'};
  for (const row of [{path: file}, {path: file, beforeEntry: null, afterEntry: null},
    {path: file, beforeEntry: entry, afterEntry: {...entry, mode: '100755'}},
    {path: file, beforeEntry: {...entry, mode: '120000'}, afterEntry: null},
    {path: file, beforeEntry: null, afterEntry: {...entry, file: 'other.md'}}]) assert.equal(decide([row]).full, true);
  assert.equal(decide([{path: file, beforeEntry: entry, afterEntry: entry}]).full, false);
});

for (const input of ['package.json', 'farm.config.ts', 'public/icon.svg', '.env', 'tests/fixtures/input.js', 'src/removed.ts']) {
  test(`directory ownership includes manifest, config and resources: ${input}`, () => {
    const result = decide([{path: 'js-plugins/b/' + input}]);
    assert.equal(result.full, false);
    assert.deepEqual(result.affectedOwners, ['examples/one', 'js-plugins/b']);
    assert.deepEqual(result.sets['examples-build'].ids, ['examples-build:examples/one']);
    assert.ok(result.explanations.some(row => row.owner === 'js-plugins/b' && row.reason === 'directory-owner'));
    assert.ok(result.explanations.some(row => row.owner === 'examples/one' && row.reason === 'dependent'));
  });
}
for (const version of ['file:../../packages/a', 'link:../../packages/a', 'workspace:../../packages/a', 'workspace:a@*', 'npm:a@1.0.0']) {
  for (const field of ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies']) {
    test(`validation follows ${field} ${version} aliases`, () => {
      const snapshot = {...files, 'js-plugins/b/package.json': JSON.stringify({name: 'b', [field]: {alias: version}})};
      const result = decide([{path: 'packages/a/src/input.ts'}], snapshot, snapshot);
      assert.equal(result.full, false);
      assert.deepEqual(result.affectedOwners, ['examples/one', 'js-plugins/b', 'packages/a']);
    });
  }
}
test('global rules and unknown directory inputs report explicit full-execution reasons', () => {
  for (const file of ['scripts/new-recipe.mjs', '.github/workflows/new.yaml', 'Cargo.lock', 'crates/compiler/src/lib.rs']) {
    const result = decide([{path: file}]);
    assert.equal(result.full, true);
    assert.ok(result.explanations.some(row => row.path === file && row.reason === 'global'));
  }
  const result = decide([{path: 'unclassified/generated.txt'}]);
  assert.equal(result.full, true);
  assert.ok(result.explanations.some(row => row.reason === 'unknown-full'));
});

test('directory rule inventory is canonical, unique and records implicit edges', () => {
  const valid = value => /^[a-zA-Z0-9._/-]+$/.test(value) && !value.startsWith('/') && !value.split('/').includes('..');
  assert.equal(new Set(directoryImpactRules.global).size, directoryImpactRules.global.length);
  assert.ok(directoryImpactRules.global.every(valid));
  for (const rule of directoryImpactRules.dependencies) {
    assert.ok(valid(rule.consumer));
    assert.ok(rule.dependencies.length > 0 && rule.dependencies.every(valid));
    assert.equal(new Set(rule.dependencies).size, rule.dependencies.length);
  }
  assert.ok(directoryImpactRules.global.includes('examples/ssr-utils/'));
  const postcss = directoryImpactRules.dependencies.find(row => row.consumer === 'js-plugins/postcss');
  assert.ok(postcss.dependencies.includes('examples/vite-adapter-vue'));
});

test('missing declared directory dependencies cannot silently reduce validation', () => {
  const incomplete = {...files, 'js-plugins/postcss/package.json': JSON.stringify({name: 'postcss'})};
  const result = decide([{path: 'examples/one/src/main.ts'}], incomplete, incomplete);
  assert.equal(result.full, true);
  assert.ok(result.reasons.includes('declared-directory-dependency-missing'));
});
