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
