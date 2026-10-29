import test from 'node:test';
import assert from 'node:assert/strict';
import { publicConsumerManifest, assertPublicRustLock, cleanupConsumerScratch } from './verify-published-v2-consumers.mjs';
test('public registry consumer uses exact selected stable versions without workspace overrides', () => {
  const plan = { npm: { primary: ['@farmfe/core', '@farmfe/cli', '@farmfe/docs', '@farmfe/docs-theme-default', 'create-farm'].map(name => ({ name, version: name.includes('docs') ? '0.0.1' : '2.0.0' })) } };
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

test('first-stable consumer installs every primary, not just four fixed support packages', () => {
  const support = ['@farmfe/core', '@farmfe/cli', '@farmfe/docs', '@farmfe/docs-theme-default', 'create-farm'];
  const names = [...support, ...Array.from({ length: 46 }, (_, i) => `@farmfe/plugin-${i}`)];
  const plan = { npm: { primary: names.map(name => ({ name, version: '2.0.0' })) } };
  assert.equal(Object.keys(publicConsumerManifest(plan).dependencies).length, 51);
  plan.publication = { npm: { primary: [plan.npm.primary[5]], payloads: [] } };
  assert.deepEqual(Object.keys(publicConsumerManifest(plan).dependencies).sort(), [...support, names[5]].sort());
});

test('normal and confirmed failure cleanup remove only owned scratch; unconfirmed timeout retains it', async () => {
  for (const report of [{ status: 'passed' }, { status: 'failed', cleanupConfirmed: true }]) {
    const removals = [];
    await cleanupConsumerScratch('/owned/scratch', report, async (...args) => removals.push(args));
    assert.equal(report.scratchRemoved, true); assert.deepEqual(removals, [['/owned/scratch', { recursive: true, force: true }]]);
  }
  const report = { status: 'failed', cleanupConfirmed: false, ownedPid: 42, cleanupError: 'Access denied' };
  await cleanupConsumerScratch('/owned/scratch', report, async () => { throw new Error('Must not delete unconfirmed live inputs'); });
  assert.equal(report.scratchRemoved, false); assert.equal(report.status, 'failed');
  assert.match(report.scratchRetainedReason, /unconfirmed/); assert.equal(report.ownedPid, 42);
});

test('scratch removal failure remains visible and does not claim success', async () => {
  const report = { status: 'passed' };
  await cleanupConsumerScratch('/owned/scratch', report, async () => { throw new Error('busy'); });
  assert.equal(report.status, 'failed'); assert.equal(report.scratchRemoved, false);
  assert.match(report.scratchRetainedReason, /busy/);
});

test('both consumer CLIs propagate final cleanup failure as a nonzero exit', async () => {
  const fs = await import('node:fs/promises');
  for (const name of ['verify-published-v2-consumers.mjs', 'verify-v2-npm-consumer.mjs']) {
    const source = await fs.readFile(new URL(name, import.meta.url), 'utf8');
    assert.match(source, /await cleanupConsumerScratch\(scratch, (?:report|result)\);\s+if \((?:report|result).status !== 'passed'\) process.exitCode = 1/);
  }
});
