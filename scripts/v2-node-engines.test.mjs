import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { collectInstalledDependencies, evaluateNodeEngines } from './v2-node-engines.mjs';

async function fixture(t) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-engines-test-'));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const manifest = async (relative, value) => {
    const directory = path.join(root, relative);
    await fs.mkdir(directory, { recursive: true });
    await fs.writeFile(path.join(directory, 'package.json'), JSON.stringify(value));
  };
  return { root, manifest };
}

test('required engine failures block while incompatible optional packages are reported separately', () => {
  const result = evaluateNodeEngines([{ name: 'tool', engine: '^22.20 || >=24.12' }, { name: 'optional', engine: '>=99', optional: true }], ['20.19.0', '22.20.0', '24.12.0']);
  assert.deepEqual(result.map(row => row.satisfiesInstalledRanges), [false, true, true]);
  assert.deepEqual(result[0].offenders.map(row => row.name), ['tool']);
  assert.deepEqual(result[1].optionalOffenders.map(row => row.name), ['optional']);
});

test('uses nested versions, excludes dev dependencies, and propagates optional subtree status', async t => {
  const f = await fixture(t);
  await f.manifest('.', { name: 'consumer', engines: { node: '>=20' }, dependencies: { core: '1' }, optionalDependencies: { addon: '1' }, devDependencies: { ignored: '1' } });
  await f.manifest('node_modules/core', { name: 'core', dependencies: { child: '1' } });
  await f.manifest('node_modules/core/node_modules/child', { name: 'child', version: '1', engines: { node: '>=20' } });
  await f.manifest('node_modules/child', { name: 'child', version: '2', engines: { node: '>=99' } });
  await f.manifest('node_modules/addon', { name: 'addon', dependencies: { missingLeaf: '1' } });
  const graph = collectInstalledDependencies(f.root);
  assert.equal(graph.packages.find(item => item.name === 'child').version, '1');
  assert.equal(graph.packages.find(item => item.name === 'consumer').engine, '>=20');
  assert.deepEqual(graph.missing.map(({ name, optional }) => ({ name, optional })), [{ name: 'missingLeaf', optional: true }]);
});

test('revisits shared optional modules when also reached through a required path', async t => {
  const f = await fixture(t);
  await f.manifest('.', { name: 'consumer', dependencies: { shared: '1' }, optionalDependencies: { optionalRoot: '1' } });
  await f.manifest('node_modules/optionalRoot', { name: 'optionalRoot', dependencies: { shared: '1' } });
  await f.manifest('node_modules/shared', { name: 'shared', dependencies: { absent: '1', optionalRoot: '1' } });
  const graph = collectInstalledDependencies(f.root, ['optionalRoot', 'shared']);
  assert.equal(graph.packages.find(item => item.name === 'shared').optional, false);
  assert.equal(graph.missing.find(item => item.name === 'absent').optional, false);
  assert.equal(graph.packages.filter(item => item.name === 'shared').length, 1);
});
