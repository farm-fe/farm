import assert from 'node:assert/strict';
import test from 'node:test';
import { refreshCandidateSourceInventory } from './refresh-v2-source-inventory.mjs';

const original = () => ({
  status: 'candidate-not-publication-approval',
  preparation: { mode: 'first-stable', receipt: 'frozen-versions.json' },
  abi: '0.5.4',
  npm: { primary: [{ name: '@farmfe/core', version: '2.0.0', sourceSha256: 'a'.repeat(64) }], payloads: [] },
  cargo: [{ name: 'farmfe_plugin_runtime', version: '2.2.0', sourceSha256: 'b'.repeat(64), packageInputsSha256: 'c'.repeat(64) }],
  counts: { primary: 1, payloads: 0, total: 1 }
});
const pending = [{ name: '@farmfe/core', oldVersion: '2.0.0', newVersion: '2.0.1', type: 'patch' }];

test('source refresh rejects changed immutable Cargo versions instead of blessing new hashes', () => {
  const baseline = original(), cargo = structuredClone(baseline.cargo);
  cargo[0].sourceSha256 = 'd'.repeat(64);
  assert.throws(() => refreshCandidateSourceInventory(baseline, { npm: baseline.npm, cargo, pending, requiredOwners: ['@farmfe/core'] }), /Rust source changed without a version bump/);
  cargo[0].version = '2.1.9';
  assert.throws(() => refreshCandidateSourceInventory(baseline, { npm: baseline.npm, cargo, pending, requiredOwners: ['@farmfe/core'] }), /advance the immutable version/);
});

test('new Cargo versions require pending official Changesets for every affected native owner', () => {
  const baseline = original(), cargo = structuredClone(baseline.cargo), npm = structuredClone(baseline.npm);
  cargo[0].version = '2.2.1'; cargo[0].sourceSha256 = 'd'.repeat(64);
  npm.primary[0].sourceSha256 = 'e'.repeat(64);
  const options = { npm, cargo, pending, requiredOwners: ['@farmfe/core'] };
  for (const releases of [[], [{ ...pending[0], type: 'none' }], [{ ...pending[0], newVersion: '2.0.0' }]]) {
    assert.throws(() => refreshCandidateSourceInventory(baseline, { ...options, pending: releases }), /Pending Changeset required/);
  }
  const refreshed = refreshCandidateSourceInventory(baseline, options);
  assert.equal(refreshed.cargo[0].version, '2.2.1');
  assert.equal(refreshed.npm.primary[0].version, '2.0.0', 'Only official Changesets consumes pending npm versions');
  assert.deepEqual(refreshed.preparation, baseline.preparation, 'Never rewrite publication preparation evidence');
  assert.equal(refreshed.status, 'candidate-not-publication-approval');
  assert.equal(refreshed.publication, undefined);
  assert.throws(() => refreshCandidateSourceInventory(baseline, { ...options, requiredOwners: ['@farmfe/core', '@farmfe/plugin-worker'] }), /Pending Changeset required/);
  assert.deepEqual(baseline, original(), 'Do not mutate the prior version evidence');
});

test('unchanged native owner source still needs a Changeset when its compiler dependency changes', () => {
  const baseline = original();
  baseline.npm.primary.push({ name: '@farmfe/plugin-worker', version: '2.0.0', sourceSha256: 'a'.repeat(64) });
  const cargo = structuredClone(baseline.cargo); cargo[0].version = '2.2.1';
  const options = { npm: baseline.npm, cargo, pending, requiredOwners: ['@farmfe/core', '@farmfe/plugin-worker'] };
  assert.throws(() => refreshCandidateSourceInventory(baseline, options), /Pending Changeset required.*plugin-worker/);
  assert.doesNotThrow(() => refreshCandidateSourceInventory(baseline, {
    ...options, pending: [...pending, { ...pending[0], name: '@farmfe/plugin-worker' }]
  }));
});

test('fixture and workspace-lock fingerprints may refresh without inventing Cargo version changes', () => {
  const baseline = original(), cargo = structuredClone(baseline.cargo);
  cargo[0].packageInputsSha256 = 'f'.repeat(64);
  assert.deepEqual(refreshCandidateSourceInventory(baseline, { npm: baseline.npm, cargo, pending: [], requiredOwners: [] }).cargo, cargo);
});

test('source refresh cannot widen the release family or consume npm versions', () => {
  const baseline = original();
  assert.throws(() => refreshCandidateSourceInventory(baseline, { npm: baseline.npm, cargo: [...baseline.cargo, { name: 'farmfe_plugin_module_federation', version: '0.0.0', publish: [] }], pending: [], requiredOwners: [] }), /Cargo release scope changed/);
  const npm = structuredClone(baseline.npm); npm.primary[0].version = '2.0.1';
  assert.throws(() => refreshCandidateSourceInventory(baseline, { npm, cargo: baseline.cargo, pending, requiredOwners: [] }), /npm versions require official version preparation/);
  assert.throws(() => refreshCandidateSourceInventory({ ...baseline, publication: {} }, { npm: baseline.npm, cargo: baseline.cargo, pending: [], requiredOwners: [] }), /prepared publication/);
});
