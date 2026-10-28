import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { nativeArtifactInventory, nativeBuildSupport } from './native-artifact-inventory.mjs';
import { assembleNativeArtifacts } from './assemble-v2-native-artifacts.mjs';
test('current manifests retain nine ABIs, Tailwind and declared Docs targets', () => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const plan = nativeArtifactInventory(root), support = nativeBuildSupport(root);
  assert.equal(plan.npm.payloads.length, (support.regularPlugins.length + 2) * support.abis.length + support.docsPlugins.length * support.docsAbis.length);
  assert.equal(new Set(plan.npm.payloads.map(row => row.platform)).size, 9);
  assert.equal(plan.npm.payloads.filter(row => row.owner === '@farmfe/plugin-tailwindcss').length, 9);
  assert.equal(plan.npm.payloads.filter(row => row.owner === '@farmfe/plugin-docs').length, 5);
  const ci = fs.readFileSync(new URL('../.github/workflows/ci.yaml', import.meta.url), 'utf8');
  for (const name of ['check-core-artifacts:', 'check-create-farm-rust-artifacts:', 'check-plugin-artifacts:']) assert.ok(!ci.includes(name));
});
test('discovery follows manifest versions and aggregates missing files before copying', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-native-inventory-'));
  try {
    const owner = path.join(root, 'packages/core'); fs.mkdirSync(path.join(owner, 'npm/linux-x64-gnu'), { recursive: true });
    const write = version => {
      fs.writeFileSync(path.join(owner, 'package.json'), JSON.stringify({ name: '@farmfe/core', version, napi: { targets: ['x86_64-unknown-linux-gnu'] }, optionalDependencies: { '@farmfe/core-linux-x64-gnu': version } }));
      fs.writeFileSync(path.join(owner, 'npm/linux-x64-gnu/package.json'), JSON.stringify({ name: '@farmfe/core-linux-x64-gnu', version, os: ['linux'], cpu: ['x64'], libc: ['glibc'], main: 'farm.node' }));
    };
    write('2.0.0');
    const support = { abis: ['linux-x64-gnu'], regularPlugins: [], docsPlugins: [], docsAbis: [] };
    assert.equal(nativeArtifactInventory(root, support).npm.payloads[0].version, '2.0.0');
    for (const [field, value] of [['version', '1.0.0'], ['name', '@wrong/payload'], ['os', ['win32']], ['cpu', ['arm64']], ['libc', ['musl']]]) {
      const file = path.join(owner, 'npm/linux-x64-gnu/package.json');
      const payload = JSON.parse(fs.readFileSync(file));
      fs.writeFileSync(file, JSON.stringify({ ...payload, [field]: value }));
      assert.throws(() => nativeArtifactInventory(root, support), /mismatch/);
      write('2.0.0');
    }
    const ownerFile = path.join(owner, 'package.json');
    const manifest = JSON.parse(fs.readFileSync(ownerFile));
    for (const optionalDependencies of [{}, { '@farmfe/core-linux-x64-gnu': '1.0.0' }]) {
      fs.writeFileSync(ownerFile, JSON.stringify({ ...manifest, optionalDependencies }));
      assert.throws(() => nativeArtifactInventory(root, support), /optional dependency/);
    }
    write('2.9.9'); const plan = nativeArtifactInventory(root, support); assert.equal(plan.npm.payloads[0].version, '2.9.9');
    fs.mkdirSync(path.join(root, 'pool'));
    plan.npm.payloads.push({ ...plan.npm.payloads[0], name: 'second-missing' });
    assert.throws(() => assembleNativeArtifacts(root, path.join(root, 'pool'), 'a'.repeat(40), plan), error => error instanceof AggregateError && error.errors.length === 2);
    assert.ok(!fs.existsSync(path.join(owner, 'npm/linux-x64-gnu/farm.node')));
  } finally { fs.rmSync(root, { recursive: true }); }
});
