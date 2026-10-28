import test from 'node:test';
import assert from 'node:assert/strict';
import { publicFetchManifest, bindCandidateChecksums, lockedPackages } from './prepare-sdk-frozen-lock.mjs';
const block = (name, version, checksum) => `[[package]]\nname = "${name}"\nversion = "${version}"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "${checksum}"\n`;
test('public cache recipe never requests unpublished selected SDK versions', () => {
  const lock = block('farmfe_core', '2.2.0', 'a'.repeat(64)) + block('serde', '1.0.228', 'b'.repeat(64));
  const manifest = publicFetchManifest({ cargo: [{ name: 'farmfe_core', version: '2.2.0' }] }, lock);
  assert.doesNotMatch(manifest, /farmfe_core/); assert.match(manifest, /package = "serde", version = "=1.0.228"/);
});
test('candidate binding changes only selected archive checksums, preserving public versions and bytes', () => {
  const lock = block('farmfe_core', '2.2.0', 'a'.repeat(64)) + block('serde', '1.0.228', 'b'.repeat(64));
  const bound = bindCandidateChecksums(lock, { selected: [{ name: 'farmfe_core', version: '2.2.0', sha256: 'c'.repeat(64) }], reused: [] });
  assert.equal(lockedPackages(bound)[0].checksum, 'c'.repeat(64));
  assert.ok(bound.includes(block('serde', '1.0.228', 'b'.repeat(64))));
  assert.throws(() => bindCandidateChecksums(lock, { selected: [{ name: 'missing', version: '2.0.0', sha256: 'c'.repeat(64) }], reused: [] }), /does not cover/);
});
