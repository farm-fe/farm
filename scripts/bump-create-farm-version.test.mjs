import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { synchronizeTemplateDependencies } from './bump-create-farm-version.mjs';

async function fixture(t) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-template-version-test-'));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const manifest = async (relative, value) => {
    const file = path.join(root, relative, 'package.json');
    await fs.mkdir(path.dirname(file), { recursive: true });
    await fs.writeFile(file, JSON.stringify(value));
    return file;
  };
  return { root, manifest };
}

test('updates both template families and all dependency sections using actual owning versions', async t => {
  const f = await fixture(t);
  await f.manifest('packages/core', { name: '@farmfe/core', version: '2.0.0' });
  await f.manifest('js-plugins/dts', { name: '@farmfe/js-plugin-dts', version: '0.5.1' });
  await f.manifest('rust-plugins/react', { name: '@farmfe/plugin-react', version: '1.7.3' });
  const first = await f.manifest('crates/create-farm-rs/templates/react', { dependencies: { '@farmfe/core': '^1.0.0', unrelated: '^3' }, optionalDependencies: { '@farmfe/plugin-react': '*' } });
  const second = await f.manifest('packages/create-farm-plugin/templates/js', { devDependencies: { '@farmfe/js-plugin-dts': '^2.0.0' }, peerDependencies: { '@farmfe/core': '*' } });
  assert.equal((await synchronizeTemplateDependencies(f.root)).length, 2);
  const a = JSON.parse(await fs.readFile(first, 'utf8'));
  const b = JSON.parse(await fs.readFile(second, 'utf8'));
  assert.equal(a.dependencies['@farmfe/core'], '^2.0.0');
  assert.equal(a.dependencies.unrelated, '^3');
  assert.equal(a.optionalDependencies['@farmfe/plugin-react'], '^1.7.3');
  assert.equal(b.devDependencies['@farmfe/js-plugin-dts'], '^0.5.1');
  assert.equal(b.peerDependencies['@farmfe/core'], '^2.0.0');
  assert.deepEqual(await synchronizeTemplateDependencies(f.root), []);
});

test('validates all owned dependencies before writing an earlier valid template', async t => {
  const f = await fixture(t);
  await f.manifest('packages/core', { name: '@farmfe/core', version: '2.0.0' });
  const first = await f.manifest('crates/create-farm-rs/templates/react', { dependencies: { '@farmfe/core': '^1.0.0' } });
  await f.manifest('packages/create-farm-plugin/templates/js', { devDependencies: { '@farmfe/unknown': '*' } });
  const before = await fs.readFile(first, 'utf8');
  await assert.rejects(synchronizeTemplateDependencies(f.root), /Unknown template dependency @farmfe\/unknown/);
  assert.equal(await fs.readFile(first, 'utf8'), before);
});
