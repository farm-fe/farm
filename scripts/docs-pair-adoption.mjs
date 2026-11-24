/** Build an adoption baseline after an independently verified public readback. */
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

const pairs = Object.freeze({
  '@farmfe/docs': 'packages/docs/package.json',
  '@farmfe/docs-theme-default': 'packages/docs-theme-default/package.json'
});
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const proofDigest = bytes => /^([a-f0-9]{64})$/.test(bytes ?? '');
const gitDigest = bytes => /^([a-f0-9]{40})$/.test(bytes ?? '');
const sort = rows => [...rows].sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0);
const historicalPlan = 'd2c1195e070c3b188b112dcd7fb62359265ab7978da753624ae6bc515fa012b1';
const keys = Object.keys(pairs).sort();
function exactPair(rows) {
  assert.ok(Array.isArray(rows));
  const ordered = sort(rows);
  assert.deepEqual(ordered.map(row => row.name), keys, 'Exactly both reviewed Docs packages required');
  return ordered;
}

function archive(rows, { metadata = false } = {}) {
  return exactPair(rows).map(row => {
    assert.deepEqual(Object.keys(row).sort(), [
      'archiveBytes', 'archiveSha256', 'integrity', ...(metadata ? ['metadataSha256'] : []), 'name', ...(metadata ? ['resolved'] : []), 'version'
    ], 'Unexpected pair archive evidence fields');
    assert.equal(row.version, '2.0.0');
    assert.ok(Number.isSafeInteger(row.archiveBytes) && row.archiveBytes > 0 && row.archiveBytes <= 100 * 1024 ** 2);
    assert.ok(proofDigest(row.archiveSha256));
    if (metadata) {
      assert.ok(proofDigest(row.metadataSha256));
      const leaf = row.name.split('/')[1];
      assert.equal(row.resolved, `https://registry.npmjs.org/${row.name}/-/${leaf}-2.0.0.tgz`, 'Public Docs archive origin differs');
    }
    assert.match(row.integrity, /^sha512-[A-Za-z0-9+/]{86}==$/);
    const raw = Buffer.from(row.integrity.slice(7), 'base64');
    assert.equal(raw.length, 64);
    assert.equal(raw.toString('base64'), row.integrity.slice(7));
    return {name: row.name, version: row.version, archiveBytes: row.archiveBytes,
      archiveSha256: row.archiveSha256, integrity: row.integrity};
  });
}

/**
 * This function may consume the independently authenticated producer seal and
 * public-install proof only. It does not authenticate an artifact ID or infer
 * trust from JSON; consumers must authenticate the callbacks before calling.
 * No writes, network calls, authorization, or mutation of the original plan.
 */
export async function prepareRetainedDocsBaseline({
  frozenPlanBytes, deltaBytes, inventory, readFixedSource, trustedProducer, verifyPublic
}) {
  assert.ok(Buffer.isBuffer(frozenPlanBytes) && Buffer.isBuffer(deltaBytes));
  const baseHash = digest(frozenPlanBytes);
  assert.equal(baseHash, historicalPlan, 'Frozen baseline differs');
  const source = JSON.parse(deltaBytes);
  assert.deepEqual(Object.keys(source).sort(), ['baseInventorySha256', 'kind', 'packages', 'schemaVersion']);
  assert.equal(source.schemaVersion, 1);
  assert.equal(source.kind, 'pending-docs-source-only');
  assert.equal(source.baseInventorySha256, baseHash);
  assert.ok(gitDigest(trustedProducer.sourceSHA) && gitDigest(trustedProducer.sourceTree));
  const sealed = trustedProducer.frozen;
  assert.deepEqual(Object.keys(sealed).sort(), ['kind', 'packages', 'schemaVersion', 'sourceSHA', 'sourceTree']);
  assert.equal(sealed.kind, 'docs-frozen-pair-v1');
  assert.equal(sealed.schemaVersion, 1);
  assert.equal(sealed.sourceSHA, trustedProducer.sourceSHA);
  assert.equal(sealed.sourceTree, trustedProducer.sourceTree);
  const expected = archive(sealed.packages);
  const baseline = JSON.parse(frozenPlanBytes);
  const next = new Map();
  for (const row of exactPair(source.packages)) {
    assert.deepEqual(Object.keys(row).sort(), ['manifestPath', 'manifestSha256', 'name']);
    const manifestPath = pairs[row.name];
    assert.equal(row.manifestPath, manifestPath);
    const original = baseline.npm.primary.find(item => item.name === row.name);
    const observed = inventory.primary.filter(item => item.name === row.name);
    assert.equal(observed.length, 1);
    const [item] = observed;
    assert.deepEqual(Object.keys(item).sort(), ['manifestPath', 'manifestSha256', 'name', 'sourceSha256', 'version']);
    assert.equal(original?.version, '0.0.2');
    assert.equal(original.manifestPath, manifestPath);
    assert.equal(item.manifestPath, manifestPath);
    assert.equal(item.version, '2.0.0');
    assert.ok(proofDigest(item.manifestSha256) && proofDigest(item.sourceSha256));
    const raw = await readFixedSource(trustedProducer.sourceSHA, trustedProducer.sourceTree, manifestPath);
    assert.ok(Buffer.isBuffer(raw));
    assert.equal(digest(raw), row.manifestSha256);
    const manifest = JSON.parse(raw);
    assert.equal(digest(JSON.stringify(manifest)), item.manifestSha256);
    assert.equal(manifest.name, row.name);
    assert.equal(manifest.version, '2.0.0');
    assert.equal(manifest.private ?? false, false);
    assert.ok(!inventory.payloads.some(payload => payload.owner === row.name), 'Docs source has unexpected native payload');
    next.set(row.name, {...item});
  }
  // The trust boundary is explicit. Implementations check the pinned original
  // bytes against an actual independent public npm install and archive reads.
  const publicProof = await verifyPublic({expected: sealed, sourceSHA: trustedProducer.sourceSHA, sourceTree: trustedProducer.sourceTree});
  assert.deepEqual(Object.keys(publicProof).sort(), ['kind', 'packages', 'schemaVersion', 'sourceSHA', 'sourceTree']);
  assert.equal(publicProof.kind, 'docs-public-retain-only-v1');
  assert.equal(publicProof.schemaVersion, 1);
  assert.equal(publicProof.sourceSHA, sealed.sourceSHA);
  assert.equal(publicProof.sourceTree, sealed.sourceTree);
  assert.deepEqual(archive(publicProof.packages, {metadata: true}), expected, 'Public pair differs from the independently sealed bytes');
  const { publication, preparation, ...family } = baseline;
  return {baseline: {...family, npm: {...family.npm, primary: family.npm.primary.map(item => next.get(item.name) ?? item)}},
    adoption: {kind: 'docs-retained-baseline-proposal-v1', historicalPlanSha256: baseHash,
      sourceSHA: sealed.sourceSHA, sourceTree: sealed.sourceTree, publicProofSha256: digest(JSON.stringify(publicProof)),
      retained: expected, publicationAuthorized: false }};
}
