import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { inspectSdkMetadata } from './run-sdk-consumer.mjs';
import { sdkConsumerManifest } from './v2-rust-registry-reuse.mjs';

const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const readJson = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const normalized = file => fs.readFileSync(file, 'utf8').replaceAll('\r\n', '\n');

// A successful process exit or a self-reported list of crate versions alone is
// insufficient. Re-read the actual lock, metadata and sealed registry bytes.
export function verifySdkConsumer(plan, proofFile, fixtureFile) {
  const directory = path.dirname(path.resolve(proofFile));
  const sdk = readJson(proofFile);
  assert.equal(sdk.status, 'passed', 'SDK test is not complete');
  assert.equal(sdk.phase, 'complete'); assert.equal(sdk.exitCode, 0);
  assert.equal(sdk.sourceMode, 'sealed-local-registry');
  assert.equal(sdk.workspacePatches, false); assert.equal(sdk.published, false);
  assert.match(sdk.sourceCommit, /^[a-f0-9]{40}$/);
  assert.equal(sdk.planSha256, hash(JSON.stringify(plan)));
  assert.equal(sdk.log, 'sdk-test.log', 'Unexpected SDK log path');
  const log = fs.readFileSync(path.join(directory, sdk.log));
  assert.equal(hash(log), sdk.logSha256, 'SDK log checksum mismatch');
  assert.match(log.toString('utf8'), /test result: ok\. 3 passed; 0 failed/);
  const manifest = path.join(directory, 'consumer.Cargo.toml');
  const lockFile = path.join(directory, 'consumer.Cargo.lock');
  assert.equal(normalized(manifest), sdkConsumerManifest(plan, normalized(fixtureFile)), 'SDK fixture differs from reviewed consumer');
  assert.doesNotMatch(normalized(manifest), /^\s*\[patch|\bpath\s*=/m);
  const materialization = readJson(path.join(directory, 'materialization.json'));
  assert.equal(materialization.planSha256, sdk.planSha256);
  assert.equal(materialization.sourceCommit, sdk.sourceCommit);
  assert.equal(materialization.workspacePatches, false);
  const expected = [...materialization.selected, ...materialization.reused];
  assert.deepEqual(expected.map(row => `${row.name}@${row.version}`).sort(), plan.cargo.map(row => `${row.name}@${row.version}`).sort(), 'SDK archive coverage differs');
  const registry = path.resolve(directory, materialization.registry);
  if (!path.isAbsolute(materialization.registry)) {
    const relative = path.relative(directory, registry);
    assert.ok(!relative.startsWith('..') && !path.isAbsolute(relative), 'Portable SDK registry escapes evidence directory');
  }
  for (const record of expected) {
    const archive = fs.readFileSync(path.join(registry, `${record.name}-${record.version}.crate`));
    assert.equal(hash(archive), record.sha256, `SDK registry bytes differ: ${record.name}`);
  }
  const metadata = readJson(path.join(directory, 'sdk-metadata.json'));
  const packages = inspectSdkMetadata(metadata, materialization, normalized(lockFile));
  assert.deepEqual(packages, sdk.packages, 'SDK metadata differs from the completed test receipt');
  return { verified: true, sourceMode: sdk.sourceMode, sourceCommit: sdk.sourceCommit,
    packageCount: packages.length, planSha256: sdk.planSha256,
    logSha256: sdk.logSha256, manifestSha256: hash(normalized(manifest)), lockSha256: hash(normalized(lockFile)),
    metadataSha256: hash(fs.readFileSync(path.join(directory, 'sdk-metadata.json'))),
    coreArchiveSha256: packages.find(pkg => pkg.name === 'farmfe_core')?.sha256,
    publishedRegistryConsumer: false };
}
