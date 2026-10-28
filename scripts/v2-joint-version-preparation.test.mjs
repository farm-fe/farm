import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { assertVersionOnlyManifest, assertNewRustVersion, cargoChangeset, synchronizeNativePayloads, validateCargoPreparation } from './prepare-cargo-version-proposal.mjs';
import { changesetsPublication } from './v2-release-inventory.mjs';

test('one real Core patch Changeset generates an official owner bump then synchronizes platform wrappers', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-official-version-'));
  try {
    const write = (file, value) => { const target = path.join(directory, file); fs.mkdirSync(path.dirname(target), { recursive: true }); fs.writeFileSync(target, typeof value === 'string' ? value : JSON.stringify(value)); };
    write('package.json', { name: 'version-fixture', private: true, workspaces: ['packages/*'] });
    write('.changeset/config.json', { changelog: false, commit: false, access: 'public', baseBranch: 'main', updateInternalDependencies: 'patch', ignore: [] });
    write('.changeset/core-patch.md', '---\n"@farmfe/core": patch\n---\n\nCore patch fixture.\n');
    write('packages/core/package.json', { name: '@farmfe/core', version: '2.0.0', optionalDependencies: { '@farmfe/core-linux-x64-gnu': '2.0.0' } });
    write('packages/payload/package.json', { name: '@farmfe/core-linux-x64-gnu', version: '2.0.0' });
    const cliDirectory = path.dirname(fs.realpathSync(fileURLToPath(new URL('../node_modules/@changesets/cli/package.json', import.meta.url))));
    const cli = JSON.parse(fs.readFileSync(path.join(cliDirectory, 'package.json')));
    execFileSync(process.execPath, [path.resolve(cliDirectory, typeof cli.bin === 'string' ? cli.bin : Object.values(cli.bin)[0]), 'version'], { cwd: directory, timeout: 30000, encoding: 'utf8' });
    const read = file => JSON.parse(fs.readFileSync(path.join(directory, file)));
    assert.equal(read('packages/core/package.json').version, '2.0.1');
    assert.equal(read('packages/payload/package.json').version, '2.0.0');
    synchronizeNativePayloads(directory, { npm: { primary: [{ name: '@farmfe/core', version: '2.0.0', manifestPath: 'packages/core/package.json' }], payloads: [{ name: '@farmfe/core-linux-x64-gnu', owner: '@farmfe/core', manifestPath: 'packages/payload/package.json' }] } });
    assert.equal(read('packages/payload/package.json').version, '2.0.1');
    assert.equal(read('packages/core/package.json').optionalDependencies['@farmfe/core-linux-x64-gnu'], '2.0.1');
    assert.ok(!fs.existsSync(path.join(directory, '.changeset/core-patch.md')));
    assert.ok(fs.readFileSync(path.join(directory, 'packages/core/CHANGELOG.md'), 'utf8').includes('## 2.0.1'));
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});

test('Cargo proposal permits stable version edits but rejects source or dependency rewrites', () => {
  const before = '[package]\nname = "example"\nversion = "2.2.0"\n[dependencies]\nother = { version = "2.1.0", path = "../other" }\n';
  assertVersionOnlyManifest(before, before.replace('2.2.0', '2.2.1').replace('2.1.0', '2.1.1'));
  assert.throws(() => assertVersionOnlyManifest(before, before.replace('../other', '../unreviewed')), /more than stable version/);
  assert.throws(() => assertVersionOnlyManifest(before, before.replace('2.2.0', '2.2.1-beta.0')), /more than stable version/);
  assert.equal(cargoChangeset(['@farmfe/core', '@farmfe/plugin-example', '@farmfe/core']).match(/"@farmfe\/core": patch/g).length, 1);
  assert.throws(() => cargoChangeset(['@farmfe/plugin-example']));
  assertNewRustVersion('2.2.0', '2.2.1');
  assert.throws(() => assertNewRustVersion('2.2.0', '2.1.9'), /advance/);
  assert.throws(() => assertNewRustVersion('2.2.0', '3.0.0'), /major version/);
});

test('joint Cargo receipt binds exact proposal files and rejects source, stale base and unversioned consumers', () => {
  const names = ['farmfe_compiler', 'farmfe_core', 'farmfe_ecosystem_glob', 'farmfe_toolkit'];
  const manifestPaths = { farmfe_compiler: 'crates/compiler/Cargo.toml', farmfe_core: 'crates/core/Cargo.toml', farmfe_ecosystem_glob: 'rust-ecosystem/glob/Cargo.toml', farmfe_toolkit: 'crates/toolkit/Cargo.toml' };
  const metadata = { packages: names.map(name => ({ name, version: name === 'farmfe_core' ? '2.2.1' : '2.2.0', manifest_path: path.resolve(manifestPaths[name]), dependencies: [], publish: null })) };
  metadata.packages.push({ name: 'farmfe_node', version: '2.0.0', manifest_path: path.resolve('crates/node/Cargo.toml'), dependencies: [{ name: 'farmfe_core', path: path.resolve('crates/core'), kind: null }] });
  const baseline = { abi: '0.5.4', npm: { primary: [{ name: '@farmfe/core', manifestPath: 'packages/core/package.json' }], payloads: [{ owner: '@farmfe/core' }] }, cargo: names.map(name => ({ name, version: '2.2.0', manifestPath: manifestPaths[name] })) };
  const record = { schemaVersion: 1, baseSHA: 'a'.repeat(40), headSHA: 'b'.repeat(40), branch: 'release-plz-example', files: ['crates/core/Cargo.toml'], versions: [{ name: 'farmfe_core', version: '2.2.1' }], requiredNpm: ['@farmfe/core'], changeset: cargoChangeset(['@farmfe/core']) };
  const before = '[package]\nname = "farmfe_core"\nversion = "2.2.0"\n', after = before.replace('2.2.0', '2.2.1');
  const options = { baseSHA: record.baseSHA, changedFiles: () => record.files, readGitFile: sha => sha === record.baseSHA ? before : after,
    readCurrent: file => file.endsWith('lib.rs') ? 'pub const VERSION: &str = "0.5.4";' : after };
  assert.deepEqual(validateCargoPreparation(record, baseline, metadata, options), record.files);
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, baseSHA: 'c'.repeat(40) }));
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, changedFiles: () => ['crates/core/src/lib.rs'] }), /file set drift/);
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, readCurrent: () => 'changed' }), /content differs/);
  assert.throws(() => validateCargoPreparation({ ...record, requiredNpm: [] }, baseline, metadata, options));
});

test('official owner bump synchronizes every native manifest and exact optional dependency only for changed owners', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-version-contract-'));
  try {
    const write = (file, value) => fs.writeFileSync(path.join(directory, file), JSON.stringify(value));
    write('core.json', { name: '@farmfe/core', version: '2.0.1', optionalDependencies: { payload: '2.0.0', payload2: '2.0.0' } });
    write('payload.json', { name: 'payload', version: '2.0.0' });
    write('payload2.json', { name: 'payload2', version: '2.0.0' });
    write('js.json', { name: 'js', version: '1.0.0' });
    const originalJs = fs.readFileSync(path.join(directory, 'js.json'));
    const baseline = { npm: { primary: [{ name: '@farmfe/core', version: '2.0.0', manifestPath: 'core.json' }, { name: 'js', version: '1.0.0', manifestPath: 'js.json' }], payloads: [
      { owner: '@farmfe/core', name: 'payload', manifestPath: 'payload.json' }, { owner: '@farmfe/core', name: 'payload2', manifestPath: 'payload2.json' }
    ] } };
    synchronizeNativePayloads(directory, baseline);
    const read = file => JSON.parse(fs.readFileSync(path.join(directory, file)));
    assert.deepEqual(read('core.json').optionalDependencies, { payload: '2.0.1', payload2: '2.0.1' });
    assert.equal(read('payload.json').version, '2.0.1'); assert.equal(read('payload2.json').version, '2.0.1');
    assert.deepEqual(fs.readFileSync(path.join(directory, 'js.json')), originalJs);
    write('payload.json', { name: 'wrong', version: '2.0.0' });
    assert.throws(() => synchronizeNativePayloads(directory, baseline), /identity changed/);
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});

test('ABI changes cannot reuse an old plugin binary; independent Node-API generator stays outside this cohort', () => {
  const old = { abi: '0.5.4', npm: { primary: ['@farmfe/core', '@farmfe/plugin-example', 'create-farm'].map(name => ({ name, version: '2.0.0' })), payloads: ['@farmfe/core', '@farmfe/plugin-example', 'create-farm'].map(owner => ({ name: owner + '-platform', owner, version: '2.0.0' })) }, cargo: [] };
  const next = structuredClone(old); next.abi = '0.5.5'; next.npm.primary[0].version = '2.0.1'; next.npm.payloads[0].version = '2.0.1';
  const official = [{ name: '@farmfe/core', newVersion: '2.0.1' }];
  assert.throws(() => changesetsPublication(next, old, official), /ABI change requires a new native owner version/);
  next.npm.primary[1].version = '2.0.1'; next.npm.payloads[1].version = '2.0.1'; official.push({ name: '@farmfe/plugin-example', newVersion: '2.0.1' });
  const delta = changesetsPublication(next, old, official);
  assert.equal(delta.counts.total, 4); assert.ok(!delta.npm.primary.some(row => row.name === 'create-farm'));
});
