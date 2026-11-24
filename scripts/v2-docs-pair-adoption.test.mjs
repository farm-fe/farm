import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import test from 'node:test';
import { prepareRetainedDocsBaseline } from './docs-pair-adoption.mjs';
import { changesetsPublication, npmInventory } from './v2-release-inventory.mjs';

const baseSha = 'f62d30bac2f61908f46cd3e0a424a943873462fa';
const baseDigest = 'd2c1195e070c3b188b112dcd7fb62359265ab7978da753624ae6bc515fa012b1';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const pairs = ['@farmfe/docs', '@farmfe/docs-theme-default'];

function fixture() {
  // This is the real historical plan, not a synthetic claimed release.
  const file = 'docs/releases/v2-candidate.json';
  const checkout = fs.readFileSync(file);
  const frozenPlanBytes = hash(checkout) === baseDigest ? checkout : execFileSync('git', ['show', `${baseSha}:${file}`]);
  const historical = JSON.parse(frozenPlanBytes);
  const inventory = npmInventory();
  const checked = {};
  const inputs = pairs.map(name => {
    const prior = historical.npm.primary.find(row => row.name === name);
    const manifest = JSON.parse(fs.readFileSync(prior.manifestPath));
    // Model the pending transition on pre-change branches as well as #2381.
    manifest.version = '2.0.0';
    const raw = Buffer.from(JSON.stringify(manifest));
    checked[prior.manifestPath] = raw;
    const actual = inventory.primary.find(row => row.name === name);
    actual.version = manifest.version;
    actual.manifestSha256 = hash(raw);
    return {name, manifestPath: prior.manifestPath, manifestSha256: hash(raw)};
  });
  const deltaBytes = Buffer.from(JSON.stringify({schemaVersion: 1, kind: 'pending-docs-source-only',
    baseInventorySha256: hash(frozenPlanBytes), packages: inputs}));
  const sourceSHA = 'a'.repeat(40), sourceTree = 'b'.repeat(40);
  const expected = pairs.map((name, i) => ({name, version: '2.0.0', archiveBytes: 2 + i, archiveSha256: String(i + 3).repeat(64),
    integrity: 'sha512-' + Buffer.alloc(64, i + 9).toString('base64')}));
  const sealed = {schemaVersion: 1, kind: 'docs-frozen-pair-v1', sourceSHA, sourceTree, packages: expected};
  const observed = {schemaVersion: 1, kind: 'docs-public-retain-only-v1', sourceSHA, sourceTree,
    packages: expected.map(row => ({...row, resolved: `https://registry.npmjs.org/${row.name}/-/${row.name.split('/')[1]}-2.0.0.tgz`,
      metadataSha256: '5'.repeat(64)}))};
  let reads = 0, publicChecks = 0;
  const args = {frozenPlanBytes, deltaBytes, inventory, trustedProducer: {sourceSHA, sourceTree, frozen: sealed},
    readFixedSource: async (sha, tree, path) => {
      reads++; assert.equal(sha, sourceSHA); assert.equal(tree, sourceTree); return checked[path];
    },
    verifyPublic: async ({expected, sourceSHA: sha, sourceTree: tree}) => {
      publicChecks++; assert.equal(sha, sourceSHA); assert.equal(tree, sourceTree); assert.deepEqual(expected, sealed); return observed;
    }};
  return {args, observed, checked, historical, counter: () => ({reads, publicChecks})};
}

test('readback permits only a retained Docs pair and a subsequent official Core delta', async () => {
  const f = fixture();
  const {baseline, adoption} = await prepareRetainedDocsBaseline(f.args);
  assert.equal(adoption.publicationAuthorized, false);
  assert.equal(baseline.publication, undefined);
  assert.equal(baseline.preparation, undefined);
  assert.equal(adoption.retained.length, 2);
  assert.deepEqual(baseline.cargo, f.historical.cargo);
  assert.deepEqual(baseline.npm.payloads, f.historical.npm.payloads);
  assert.deepEqual(f.counter(), {reads: 2, publicChecks: 1});
  const unchanged = rows => rows.filter(row => !pairs.includes(row.name));
  assert.deepEqual(unchanged(baseline.npm.primary), unchanged(f.historical.npm.primary));
  const next = structuredClone(baseline);
  next.publicationScope = 'npm';
  const oldCore = next.npm.primary.find(row => row.name === '@farmfe/core');
  oldCore.version = '2.0.2';
  for (const payload of next.npm.payloads.filter(row => row.owner === oldCore.name)) payload.version = oldCore.version;
  const official = [{name: oldCore.name, newVersion: oldCore.version}];
  assert.throws(() => changesetsPublication(next, f.historical, official), /Version not generated.*@farmfe\/docs/);
  const selected = changesetsPublication(next, baseline, official);
  assert.equal(selected.releaseIdentity, 'v2.0.2');
  assert.deepEqual(selected.npm.primary.map(row => row.name), ['@farmfe/core']);
  assert.ok(selected.npm.payloads.every(row => row.owner === '@farmfe/core'));
});

test('stale, unrelated or one-sided source declarations cannot reach the public reader', async () => {
  for (const mutate of [
    f => { f.args.frozenPlanBytes = Buffer.concat([f.args.frozenPlanBytes, Buffer.from(' ')]); },
    f => { f.args.deltaBytes = Buffer.from(JSON.stringify({...JSON.parse(f.args.deltaBytes), packages: []})); },
    f => { const d = JSON.parse(f.args.deltaBytes); d.packages[0].name = '@farmfe/core'; f.args.deltaBytes = Buffer.from(JSON.stringify(d)); },
    f => { f.args.trustedProducer.frozen.sourceTree = 'e'.repeat(40); },
    f => { f.checked['packages/docs/package.json'] = Buffer.from('{}'); },
    f => { f.args.inventory.primary.find(row => row.name === pairs[1]).manifestSha256 = 'f'.repeat(64); },
    f => { f.args.inventory.payloads.push({owner: pairs[0]}); }
  ]) {
    const f = fixture(); mutate(f);
    await assert.rejects(prepareRetainedDocsBaseline(f.args));
    assert.equal(f.counter().publicChecks, 0);
  }
});

test('a missing, untrusted or swapped public pair cannot become an adoption', async () => {
  for (const mutate of [
    f => { f.args.verifyPublic = () => { throw new Error('exact theme not public'); }; },
    f => { f.observed.packages.pop(); },
    f => { f.observed.packages[0].archiveSha256 = 'f'.repeat(64); },
    f => { f.observed.packages[0].resolved = f.observed.packages[0].resolved + '?unreviewed'; },
    f => { f.observed.sourceSHA = f.observed.sourceTree; },
    f => { f.observed.kind = 'unchecked'; }
  ]) {
    const f = fixture(); mutate(f);
    await assert.rejects(prepareRetainedDocsBaseline(f.args));
  }
});

// This file is discovered by the existing release-contract runner. Hosted
// checks exercise only offline proof fixtures, not build, credentials or npm.
test('run the read-only exact pair proof contracts without a registry', () => {
  execFileSync('python3', ['-B', 'scripts/docs-pair-registry-proof.test.py'], {timeout: 15_000});
  execFileSync(process.execPath, ['--check', 'scripts/farm-docs-build-frozen-pair.mjs'], {timeout: 15_000});
  execFileSync('bash', ['-n', 'scripts/farm-docs-test-frozen-pair.sh'], {timeout: 15_000});
});
