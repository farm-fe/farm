import test from 'node:test';
import assert from 'node:assert/strict';
import { publicConsumerManifest, assertPublicRustLock } from './verify-published-v2-consumers.mjs';
test('public registry consumer uses exact selected stable versions without workspace overrides', () => {
  const plan = { npm: { primary: ['@farmfe/core', '@farmfe/cli', '@farmfe/docs', '@farmfe/docs-theme-default'].map(name => ({ name, version: name.includes('docs') ? '0.0.1' : '2.0.0' })) } };
  const manifest = publicConsumerManifest(plan);
  assert.equal(manifest.dependencies['@farmfe/docs'], '0.0.1');
  assert.doesNotMatch(JSON.stringify(manifest), /workspace:|overrides|file:/);
  plan.npm.primary[0].version = '2.0.0-beta.1'; assert.throws(() => publicConsumerManifest(plan));
});
test('public SDK proof rejects path, different source, wrong immutable bytes and removed dependencies', () => {
  const row = { name: 'farmfe_core', version: '2.2.0', sha256: 'a'.repeat(64) };
  const lock = `[[package]]\nname = "farmfe_core"\nversion = "2.2.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "${row.sha256}"\n`;
  assertPublicRustLock({ rust: [row] }, lock);
  for (const changed of [lock.replace(row.sha256, 'b'.repeat(64)), lock.replace('registry+', 'path+'), lock + '\nname = "tardar"\n']) assert.throws(() => assertPublicRustLock({ rust: [row] }, changed));
});
