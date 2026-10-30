// cspell:words cksum
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


test('all new SDK archives use the explicit public registry without archive locks or repackaging public macros', () => {
  const f = fixture();
  try {
    const plan = JSON.parse(fs.readFileSync(new URL('../docs/releases/v2-candidate.json', import.meta.url)));
    const selected = plan.cargo.filter(row => !macros.some(macro => macro.name === row.name));
    assert.equal(selected.length, 26); assert.equal(plan.cargo.length, 28);
    fs.writeFileSync(path.join(f.root, 'docs/releases/v2-candidate.json'), JSON.stringify(plan));
    fs.mkdirSync(f.cache, { recursive: true });
    const originals = [Buffer.from('original registry cache macro fixture'), Buffer.from('original registry plugin macro fixture')];
    macros.forEach((row, index) => fs.writeFileSync(path.join(f.cache, `${row.name}-${row.version}.crate`), originals[index]));
    const output = path.join(f.root, 'all-sdk'); let calls = 0;
    const receipt = sealRustPackages({ ...options(f), output, packageRunner(executable, args, settings) {
      calls++;
      assert.deepEqual(args, ['+nightly-2026-04-10', 'package', '--no-verify', '--exclude-lockfile', '--registry', 'crates-io', '--offline', '--locked', '--allow-dirty', ...selected.flatMap(row => ['-p', row.name])]);
      assert.equal(settings.cwd, f.root); assert.equal(settings.timeout, 120000);
      const directory = path.join(settings.env.CARGO_TARGET_DIR, 'package'); fs.mkdirSync(directory, { recursive: true });
      for (const row of selected) fs.writeFileSync(path.join(directory, `${row.name}-${row.version}.crate`), `candidate ${row.name}`);
      return { status: 0, stdout: '', stderr: '' };
    } });
    assert.equal(calls, 1); assert.equal(receipt.status, 'packaged-not-compiled');
    assert.deepEqual(receipt.records.map(row => row.name), selected.map(row => row.name));
    for (const row of receipt.records) assert.equal(row.sha256, digest(fs.readFileSync(path.join(output, row.file))));
    receipt.reusedRegistry.forEach((row, index) => assert.deepEqual(fs.readFileSync(path.join(output, row.file)), originals[index]));
    assert.equal(receipt.compiled, false); assert.equal(receipt.published, false);
  } finally { fs.rmSync(f.root, { recursive: true }); }
});

test('locked prefetch precedes archive preparation and isolated frozen SDK validation remains mandatory', () => {
  const workflow = fs.readFileSync(new URL('../.github/workflows/v2-candidate-seal.yaml', import.meta.url), 'utf8');
  const fetch = workflow.indexOf('cargo +nightly-2026-04-10 fetch --locked');
  const packageStep = workflow.indexOf('node scripts/seal-v2-rust-packages.mjs');
  const registry = workflow.indexOf('node scripts/sdk-candidate-registry.mjs');
  const bind = workflow.indexOf('node scripts/prepare-sdk-frozen-lock.mjs bind-candidate');
  const consumer = workflow.indexOf('node scripts/run-sdk-consumer.mjs');
  assert.ok(fetch >= 0 && fetch < packageStep && packageStep < registry && registry < bind && bind < consumer);
  assert.ok(workflow.includes("FARM_SDK_FROZEN_LOCK: '1'"));
  const registryScript = fs.readFileSync(new URL('./sdk-candidate-registry.mjs', import.meta.url), 'utf8');
  assert.ok(registryScript.includes('target/package/tmp-registry/index'));
  assert.ok(registryScript.includes('metadata.cksum !== record.sha256'));
});
