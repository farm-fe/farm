import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { npmInventory, changesetsPublication } from './v2-release-inventory.mjs';
import fs from 'node:fs';
import path from 'node:path';
import { root } from './v2-release-inventory.mjs';

test('candidate scope includes SDK generator and approved Vuetify patch', () => {
  const inventory = npmInventory();
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  assert.deepEqual(inventory, plan.npm);
  if (plan.preparation?.mode !== 'changesets') {
    assert.equal(inventory.primary.length, 51);
    assert.equal(inventory.payloads.length, 246);
    assert.equal(inventory.primary.find(pkg => pkg.name === 'create-farm-plugin').version, '2.0.0');
    assert.equal(inventory.primary.find(pkg => pkg.name === '@farmfe/js-plugin-vuetify').version, '0.0.2');
  }
  assert.ok(!inventory.primary.some(pkg => pkg.name === '@farmfe/js-plugin-copy'));
  for (const payload of inventory.payloads) {
    assert.equal(payload.version, inventory.primary.find(pkg => pkg.name === payload.owner).version);
    const platform = JSON.parse(fs.readFileSync(path.join(root, payload.manifestPath)));
    const owner = inventory.primary.find(pkg => pkg.name === payload.owner);
    const parent = JSON.parse(fs.readFileSync(path.join(root, owner.manifestPath)));
    assert.equal(platform.version, owner.version, `Platform version drift: ${payload.name}`);
    assert.equal(parent.optionalDependencies[payload.name], owner.version, `Optional dependency drift: ${payload.name}`);
  }
});
const source = 'a'.repeat(64);
const baseline = () => ({ npm: { primary: [
  { name: '@farmfe/core', version: '2.0.0', sourceSha256: source },
  { name: '@farmfe/js-plugin-example', version: '1.0.0', sourceSha256: source }
], payloads: [{ name: '@farmfe/core-linux-x64-gnu', owner: '@farmfe/core', version: '2.0.0' }] }, cargo: [{ name: 'farmfe_core', version: '2.2.0', sourceSha256: source }] });
test('JS-only Changesets delta leaves Core, Rust and native versions immutable', () => {
  const old = baseline(), family = structuredClone(old);
  family.npm.primary[1].version = '1.0.1';
  family.npm.primary[1].sourceSha256 = 'b'.repeat(64);
  const delta = changesetsPublication(family, old, [{ name: '@farmfe/js-plugin-example', newVersion: '1.0.1' }]);
  assert.equal(delta.counts.total, 1); assert.equal(delta.cargo.length, 0); assert.equal(delta.npm.payloads.length, 0);
  assert.match(delta.releaseIdentity, /^packages-[a-f0-9]{16}$/);
});
test('Core/native versions and source changes without version bumps fail closed', () => {
  const old = baseline(), family = structuredClone(old);
  family.npm.primary[0].version = '2.0.1';
  const releases = [{ name: '@farmfe/core', newVersion: '2.0.1' }];
  assert.throws(() => changesetsPublication(family, old, releases), /owner\/payload version drift/);
  family.npm.payloads[0].version = '2.0.1';
  assert.equal(changesetsPublication(family, old, releases).releaseIdentity, 'v2.0.1');
  family.cargo[0].sourceSha256 = 'b'.repeat(64);
  assert.throws(() => changesetsPublication(family, old, releases), /Rust source changed without a version bump/);
  family.cargo[0].version = '2.2.1';
  assert.equal(changesetsPublication(family, old, releases).cargo.length, 1);
  assert.throws(() => changesetsPublication(family, old, []), /official Changesets/);
});
test('source consistency cannot be mistaken for publication approval', () => {
  const run = (...args) => spawnSync(process.execPath, [new URL('./v2-release-preflight.mjs', import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1'), ...args], { encoding: 'utf8' });
  assert.equal(run().status, 0);
  const publication = run('--publication');
  assert.equal(publication.status, 1);
  assert.equal(JSON.parse(publication.stdout).readyToPublish, false);
});
