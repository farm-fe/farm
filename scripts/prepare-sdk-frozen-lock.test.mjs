import test from 'node:test';
import assert from 'node:assert/strict';
import { publicFetchManifest, bindCandidateChecksums, lockedPackages } from './prepare-sdk-frozen-lock.mjs';
const block = (name, version, checksum) => `[[package]]\nname = "${name}"\nversion = "${version}"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\nchecksum = "${checksum}"\n`;
const macros = [
  { name: 'farmfe_macro_cache_item', version: '2.1.0', sha256: 'c'.repeat(64) },
  { name: 'farmfe_macro_plugin', version: '2.0.0', sha256: 'd'.repeat(64) },
];
const firstStable = { cargo: [{ name: 'farmfe_core', version: '2.2.0' }, ...macros] };
const proof = file => ({ status: 'passed', workspacePatch: false, registryPackage: file === 'published-macro-compat.json' ? macros[0] : macros[1] });

test('cold public cache recipe includes both original first-stable macros but never unpublished SDK versions', () => {
  const lock = block('farmfe_core', '2.2.0', 'a'.repeat(64)) + block('serde', '1.0.228', 'b'.repeat(64)) + macros.map(row => block(row.name, row.version, row.sha256)).join('');
  const manifest = publicFetchManifest(firstStable, lock, { readEvidence: proof });
  assert.doesNotMatch(manifest, /farmfe_core/); assert.match(manifest, /package = "serde", version = "=1.0.228"/);
  assert.match(manifest, /package = "farmfe_macro_cache_item", version = "=2.1.0"/);
  assert.match(manifest, /package = "farmfe_macro_plugin", version = "=2.0.0"/);
  assert.equal(manifest.match(/farmfe_macro_cache_item/g).length, 1);
});

test('future recipe reuses only exact reviewed registry versions and rejects contradictory SDK lock checksums', () => {
  const plan = { preparation: { mode: 'changesets' }, cargo: firstStable.cargo, publication: { reuseRust: macros } };
  const lock = block('serde', '1.0.228', 'b'.repeat(64));
  const manifest = publicFetchManifest(plan, lock, { readEvidence: () => assert.fail('Future mode must not use first-stable evidence') });
  assert.match(manifest, /package = "farmfe_macro_plugin", version = "=2.0.0"/);
  assert.doesNotMatch(manifest, /farmfe_core/);
  assert.throws(() => publicFetchManifest(plan, lock + block(macros[0].name, macros[0].version, 'e'.repeat(64))), /checksum differs/);
  assert.throws(() => publicFetchManifest(firstStable, lock, { readEvidence: file => ({ ...proof(file), status: 'failed' }) }), /proof required/);
});

test('candidate binding changes only selected archive checksums, preserving public versions and bytes', () => {
  const lock = block('farmfe_core', '2.2.0', 'a'.repeat(64)) + block('serde', '1.0.228', 'b'.repeat(64));
  const bound = bindCandidateChecksums(lock, { selected: [{ name: 'farmfe_core', version: '2.2.0', sha256: 'c'.repeat(64) }], reused: [] });
  assert.equal(lockedPackages(bound)[0].checksum, 'c'.repeat(64));
  assert.ok(bound.includes(block('serde', '1.0.228', 'b'.repeat(64))));
  assert.throws(() => bindCandidateChecksums(lock, { selected: [{ name: 'missing', version: '2.0.0', sha256: 'c'.repeat(64) }], reused: [] }), /does not cover/);
});
