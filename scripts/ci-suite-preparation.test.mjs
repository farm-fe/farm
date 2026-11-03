import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { preparationCommands, executePreparation } from './ci-suite-preparation.mjs';
const files = Object.fromEntries([
  ['js-plugins/a', {name: 'a', scripts: {build: 'node build.mjs'}}],
  ['js-plugins/b', {name: 'b', devDependencies: {alias: 'file:../a'}, scripts: {build: 'node build.mjs'}}],
  ['examples/one', {name: 'one', dependencies: {b: 'workspace:*'}}]
].map(([directory, pkg]) => [directory + '/package.json', JSON.stringify(pkg)]));
const inventory = {inventoryHash: 'hash', items: [
  {id: 'ts:b', kind: 'ts', file: 'js-plugins/b/test.spec.ts'},
  {id: 'examples-build:one', kind: 'examples-build', directory: 'examples/one'}
]};
const plan = (kind, ids) => ({kind, ids, inventoryHash: 'hash'});
test('selected TS and Examples prepare alias prerequisites in dependency order', () => {
  for (const kind of ['ts', 'examples-build']) {
    const commands = preparationCommands(files, inventory, plan(kind, [kind === 'ts' ? 'ts:b' : 'examples-build:one']));
    assert.deepEqual(commands.slice(-2).map(row => row.directory), ['js-plugins/a', 'js-plugins/b']);
    assert.deepEqual(commands.find(row => row.directory === 'packages/core').args, ['run', 'build:cjs']);
  }
  assert.deepEqual(preparationCommands(files, inventory, plan('ts', [])), []);
  assert.ok(preparationCommands(files, inventory, plan('examples-build', [])).some(row => row.directory === 'packages/cli'));
  assert.throws(() => preparationCommands(files, inventory, plan('ts', ['unknown'])));
});
test('actual executor fails before dependents and rejects altered manifests', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'suite-preparation-'));
  try {
    for (const [file, body] of Object.entries(files)) {fs.mkdirSync(path.dirname(path.join(root, file)), {recursive: true}); fs.writeFileSync(path.join(root, file), body);}
    fs.writeFileSync(path.join(root, 'js-plugins/a/build.mjs'), "import fs from 'node:fs'; fs.writeFileSync('../ready', 'ok');");
    fs.writeFileSync(path.join(root, 'js-plugins/b/build.mjs'), "import fs from 'node:fs'; if(fs.readFileSync('../ready','utf8') !== 'ok') process.exit(1);");
    const commands = preparationCommands(files, inventory, plan('ts', ['ts:b'])).slice(-2);
    const run = (_, args, options) => execFileSync(process.execPath, ['build.mjs'], options);
    executePreparation(commands, files, root, run);
    fs.writeFileSync(path.join(root, 'js-plugins/a/build.mjs'), 'process.exit(3)');
    let calls = 0;
    assert.throws(() => executePreparation(commands, files, root, (...args) => {calls++; run(...args);}));
    assert.equal(calls, 1);
    fs.writeFileSync(path.join(root, 'js-plugins/a/package.json'), '{}');
    assert.throws(() => executePreparation(commands, files, root, run), /manifest changed/);
  } finally {fs.rmSync(root, {recursive: true, force: true});}
});

test('unresolved graph falls back to the established full plugin preparation recipe', () => {
  const unresolved = {...files, 'js-plugins/b/package.json': JSON.stringify({name: 'b', dependencies: {unknown: 'workspace:*'}, scripts: {build: 'node build.mjs'}})};
  const commands = preparationCommands(unresolved, inventory, {...plan('ts', ['ts:b']), mode: 'all'});
  assert.deepEqual(commands.slice(-2).map(row => row.directory), ['js-plugins/a', 'js-plugins/b']);
  assert.ok(commands.every(row => row.reason === 'unresolved-graph-full-recipe'));
  assert.throws(() => preparationCommands(unresolved, inventory, {...plan('ts', ['ts:b']), mode: 'selected'}));
});

test('ordinary private package prerequisites are not dropped from the forward build closure', () => {
  const snapshot = {...files,
    'packages/helper/package.json': JSON.stringify({name: 'helper', private: true, scripts: {build: 'node build.mjs'}}),
    'js-plugins/a/package.json': JSON.stringify({name: 'a', dependencies: {helper: 'workspace:*'}, scripts: {build: 'node build.mjs'}})};
  const commands = preparationCommands(snapshot, inventory, plan('ts', ['ts:b']));
  assert.deepEqual(commands.slice(-3).map(row => row.directory), ['packages/helper', 'js-plugins/a', 'js-plugins/b']);
});
