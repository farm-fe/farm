import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { rustRegistryReuse } from './v2-rust-registry-reuse.mjs';
import { sealRustPackages } from './seal-v2-rust-packages.mjs';

const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const macros = [
  { name: 'farmfe_macro_cache_item', version: '2.1.0', sha256: digest(Buffer.from('original registry cache macro fixture')) },
  { name: 'farmfe_macro_plugin', version: '2.0.0', sha256: digest(Buffer.from('original registry plugin macro fixture')) },
];
const proof = file => ({ status: 'passed', workspacePatch: false, registryPackage: file === 'published-macro-compat.json' ? macros[0] : macros[1] });

test('first-stable and future reuse share exact SDK membership, identity and checksum requirements', () => {
  const plan = { cargo: macros };
  assert.deepEqual(rustRegistryReuse(plan, { readEvidence: proof }), macros);
  const future = { preparation: { mode: 'changesets' }, cargo: macros, publication: { reuseRust: macros } };
  assert.deepEqual(rustRegistryReuse(future), macros);
  assert.throws(() => rustRegistryReuse({ ...future, publication: { reuseRust: [...macros, macros[0]] } }), /Duplicate/);
  assert.throws(() => rustRegistryReuse({ ...future, publication: { reuseRust: [{ ...macros[0], version: '9.0.0' }] } }), /outside exact SDK/);
  assert.throws(() => rustRegistryReuse({ ...future, publication: { reuseRust: [{ ...macros[0], sha256: 'bad' }] } }));
  assert.throws(() => rustRegistryReuse(plan, { readEvidence: file => ({ ...proof(file), workspacePatch: true }) }), /Public registry compatibility/);
});

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-rust-reuse-'));
  fs.mkdirSync(path.join(root, 'docs/releases'), { recursive: true });
  fs.mkdirSync(path.join(root, 'docs/release-evidence'), { recursive: true });
  fs.writeFileSync(path.join(root, 'docs/releases/v2-candidate.json'), JSON.stringify({ cargo: macros }));
  for (const name of ['published-macro-compat.json', 'published-plugin-macro-compat.json']) fs.writeFileSync(path.join(root, 'docs/release-evidence', name), JSON.stringify(proof(name)));
  const cargoHome = path.join(root, 'cargo-home'), cache = path.join(cargoHome, 'registry/cache/index.crates.io-fixture');
  return { root, cargoHome, cache };
}
const options = f => ({ root: f.root, env: { CARGO_HOME: f.cargoHome }, sourceCommit: 'a'.repeat(40), resources: { freeRamGiB: 8, freeDiskGiB: 20 } });

test('cold or corrupt registry cache writes a failed receipt without running Cargo or repackaging a macro', () => {
  const f = fixture();
  try {
    const output = path.join(f.root, 'missing-cache');
    const missing = sealRustPackages({ ...options(f), output });
    assert.equal(missing.status, 'package-failed'); assert.equal(missing.failureStage, 'reuse-registry-archives');
    assert.match(missing.error, /Missing verified immutable registry archive/);
    assert.equal(JSON.parse(fs.readFileSync(path.join(output, 'receipt.json'))).status, 'package-failed');
    assert.equal(missing.command, undefined); assert.deepEqual(missing.records, []);
    fs.mkdirSync(f.cache, { recursive: true });
    fs.writeFileSync(path.join(f.cache, `${macros[0].name}-${macros[0].version}.crate`), 'corrupt bytes');
    const corrupt = sealRustPackages({ ...options(f), output: path.join(f.root, 'corrupt-cache') });
    assert.equal(corrupt.status, 'package-failed'); assert.match(corrupt.error, /Immutable registry archive differs/);
    assert.equal(corrupt.compiled, false); assert.equal(corrupt.published, false); assert.equal(corrupt.command, undefined);
  } finally { fs.rmSync(f.root, { recursive: true }); }
});

test('verified original registry bytes are copied unchanged and excluded from the package command', () => {
  const f = fixture();
  try {
    fs.mkdirSync(f.cache, { recursive: true });
    const bytes = [Buffer.from('original registry cache macro fixture'), Buffer.from('original registry plugin macro fixture')];
    macros.forEach((row, index) => fs.writeFileSync(path.join(f.cache, `${row.name}-${row.version}.crate`), bytes[index]));
    const output = path.join(f.root, 'sealed'), receipt = sealRustPackages({ ...options(f), output });
    assert.equal(receipt.status, 'packaged-not-compiled'); assert.equal(receipt.command, null);
    assert.equal(receipt.reusedRegistry.length, 2); assert.deepEqual(receipt.records, []);
    receipt.reusedRegistry.forEach((row, index) => assert.deepEqual(fs.readFileSync(path.join(output, row.file)), bytes[index]));
    assert.equal(receipt.compiled, false); assert.equal(receipt.published, false);
  } finally { fs.rmSync(f.root, { recursive: true }); }
});
