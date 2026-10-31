import test from 'node:test';
import assert from 'node:assert/strict';
import { inspectSdkMetadata } from './run-sdk-consumer.mjs';
test('SDK proof binds actual registry source IDs and exact locked archive hashes', () => {
  const packageRecord = { name: 'farmfe_core', version: '2.2.0', sha256: 'a'.repeat(64) };
  const source = { selected: [packageRecord], reused: [] };
  const metadata = { workspace_members: ['consumer'], packages: [{ ...packageRecord, id: 'registry-core', source: 'registry+https://github.com/rust-lang/crates.io-index' }] };
  const lock = `[[package]]\nname = "farmfe_core"\nversion = "2.2.0"\nchecksum = "${packageRecord.sha256}"\n`;
  assert.equal(inspectSdkMetadata(metadata, source, lock).length, 1);
  assert.throws(() => inspectSdkMetadata({ ...metadata, packages: [{ ...metadata.packages[0], source: null }] }, source, lock));
  assert.throws(() => inspectSdkMetadata(metadata, source, lock.replace(packageRecord.sha256, 'b'.repeat(64))));
  assert.throws(() => inspectSdkMetadata({ ...metadata, packages: [...metadata.packages, { name: 'tardar' }] }, source, lock));
});
test('only reviewed third-party historical utility is allowed beside exact V2; own binding still fails', () => {
  const current = { name: 'farmfe_utils', version: '2.1.1', sha256: 'a'.repeat(64) };
  const source = 'registry+https://github.com/rust-lang/crates.io-index';
  const metadata = { workspace_members: ['consumer'], packages: [
    { ...current, id: 'utils-v2', source }, { name: 'farmfe_utils', version: '0.1.6', id: 'utils-old', source },
    { name: 'enhanced-magic-string', version: '0.1.0', id: 'magic', source }
  ], resolve: { nodes: [{ id: 'magic', deps: [{ pkg: 'utils-old' }] }] } };
  const lock = `[[package]]\nname = "farmfe_utils"\nversion = "2.1.1"\nchecksum = "${current.sha256}"\n`;
  const records = inspectSdkMetadata(metadata, { selected: [current], reused: [] }, lock);
  assert.equal(records[0].version, '2.1.1');
  assert.equal(records[0].compatibilityTransitives[0].version, '0.1.6');
  metadata.resolve.nodes.push({ id: 'utils-v2', deps: [{ pkg: 'utils-old' }] });
  assert.throws(() => inspectSdkMetadata(metadata, { selected: [current], reused: [] }, lock), /must not directly bind/);
});
