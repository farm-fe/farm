import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { npmInventory } from './v2-release-inventory.mjs';

test('candidate scope includes SDK generator and approved Vuetify patch', () => {
  const inventory = npmInventory();
  assert.equal(inventory.primary.length, 51);
  assert.equal(inventory.payloads.length, 246);
  assert.equal(inventory.primary.find(pkg => pkg.name === 'create-farm-plugin').version, '2.0.0');
  assert.equal(inventory.primary.find(pkg => pkg.name === '@farmfe/js-plugin-vuetify').version, '0.0.2');
  assert.ok(!inventory.primary.some(pkg => pkg.name === '@farmfe/js-plugin-copy'));
  for (const payload of inventory.payloads) {
    assert.equal(payload.version, inventory.primary.find(pkg => pkg.name === payload.owner).version);
  }
});
test('source consistency cannot be mistaken for publication approval', () => {
  const run = (...args) => spawnSync(process.execPath, [new URL('./v2-release-preflight.mjs', import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, '$1'), ...args], { encoding: 'utf8' });
  assert.equal(run().status, 0);
  const publication = run('--publication');
  assert.equal(publication.status, 1);
  assert.equal(JSON.parse(publication.stdout).readyToPublish, false);
});
