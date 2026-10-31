import test from 'node:test';
import assert from 'node:assert/strict';
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
