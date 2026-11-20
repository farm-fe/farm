import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import test from 'node:test';
import { archiveFiles, sha256 } from './release-archive.mjs';
import { buildIdentity, corePeer, owner, prepareMfBundle, readBundleInputs, readMfBundleForTest, targets, version } from './prepare-mf-single-package.mjs';

const project = 'rust-plugins/module-federation';
const repo = new URL('../', import.meta.url);
function nativeHeader(abi) {
  const bytes = Buffer.alloc(128);
  if (abi === 'darwin-arm64') {
    bytes.writeUInt32LE(0xfeedfacf); bytes.writeUInt32LE(0x100000c, 4);
  } else if (abi === 'linux-x64-gnu') {
    Buffer.from([127, 69, 76, 70, 2, 1]).copy(bytes); bytes.writeUInt16LE(62, 18);
  } else {
    bytes.write('MZ'); bytes.writeUInt32LE(64, 60); bytes.write('PE\0\0', 64); bytes.writeUInt16LE(0x8664, 68);
  }
  return bytes;
}
function fixture(t) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'farm-mf-single-')));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const directory = path.join(root, 'repo'), input = path.join(root, 'native'), output = path.join(root, 'out');
  for (const file of ['Cargo.lock', 'rust-toolchain.toml', 'LICENSE',
    ...['package.json', 'scripts/bundled-index.mjs', 'scripts/native-platform.mjs', 'BUNDLED-README.md', 'BUNDLED-THIRD-PARTY-NOTICES.md'].map(name => project + '/' + name)]) {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.copyFileSync(new URL(file, repo), path.join(directory, file));
  }
  const git = (...args) => execFileSync('git', args, { cwd: directory, stdio: 'ignore' });
  git('init'); git('add', '.'); git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-m', 'source fixture');
  const identity = buildIdentity(directory);
  for (const [abi, target] of Object.entries(targets)) {
    const folder = path.join(input, abi), bytes = nativeHeader(abi);
    fs.mkdirSync(folder, { recursive: true });
    fs.writeFileSync(path.join(folder, 'index.farm'), bytes);
    const legal = Buffer.from(JSON.stringify({ kind: 'locked-cargo-license-review-input',
      target, reviewComplete: false, components: [{ name: 'Fixture', declaration: 'MIT' }] }));
    fs.writeFileSync(path.join(folder, 'legal-inputs.json'), legal);
    fs.writeFileSync(path.join(folder, 'receipt.json'), JSON.stringify({
      schemaVersion: 1, kind: 'mf-bundle-build-input-not-publication-authorization', ...identity,
      abi, target, profile: 'release', binarySha256: sha256(bytes), binarySize: bytes.length, legalInputsSha256: sha256(legal),
    }));
  }
  return { root, directory, input, output, identity };
}

test('packs one exact public name, three embedded assets and no source or scripts', async t => {
  const f = fixture(t), receipt = prepareMfBundle(f);
  assert.equal(receipt.name, owner);
  assert.equal(receipt.version, version);
  assert.equal(receipt.licenseReviewComplete, false);
  assert.equal(receipt.publicationAuthorized, false);
  assert.equal(receipt.published, false);
  assert.equal(receipt.native.length, 3);
  assert.deepEqual(readMfBundleForTest(f.directory, f.output), receipt);
  const files = archiveFiles(fs.readFileSync(path.join(f.output, receipt.file)));
  const manifest = JSON.parse(files.get('package/package.json'));
  assert.deepEqual(manifest.peerDependencies, { '@farmfe/core': corePeer });
  for (const key of ['scripts', 'optionalDependencies', 'publishConfig', 'private']) assert.equal(manifest[key], undefined);
  const installed = path.join(f.root, 'node_modules', owner);
  for (const [name, bytes] of files) {
    const location = path.join(installed, name.slice(8));
    fs.mkdirSync(path.dirname(location), { recursive: true });
    fs.writeFileSync(location, bytes);
  }
  const runtime = path.join(f.root, 'node_modules/@module-federation/runtime');
  fs.mkdirSync(runtime, { recursive: true });
  fs.writeFileSync(path.join(runtime, 'package.json'), JSON.stringify({ main: 'index.js' }));
  fs.writeFileSync(path.join(runtime, 'index.js'), '');
  const entry = pathToFileURL(path.join(installed, 'scripts/index.mjs')).href;
  for (const abi of Object.keys(targets)) {
    const [os, arch] = abi.split('-');
    const program = `Object.defineProperty(process, 'platform', { value: ${JSON.stringify(os)} });
Object.defineProperty(process, 'arch', { value: ${JSON.stringify(arch)} });
process.report.getReport = () => ({header: {glibcVersionRuntime:'2.39'}});
const {default: federation} = await import(${JSON.stringify(entry)});
console.log(JSON.stringify(federation({name:'fixture'})));`;
    const actual = JSON.parse(execFileSync(process.execPath, ['--input-type=module', '--eval', program], { encoding: 'utf8' }));
    assert.equal(actual[0], path.join(installed, 'native', abi, 'index.farm'));
    assert.equal(actual[1].name, 'fixture');
    assert.equal(actual[1].runtimePath, path.join(runtime, 'index.js'));
    fs.renameSync(path.join(installed, 'native', abi, 'index.farm'), path.join(installed, 'native', abi, 'missing.farm'));
    assert.throws(() => execFileSync(process.execPath, ['--input-type=module', '--eval', program], { encoding: 'utf8', stdio: 'pipe' }), /Missing bundled Federation binary/);
  }
});

test('rejects missing target, changed bytes, wrong architecture or foreign source', t => {
  const f = fixture(t);
  const abi = 'darwin-arm64', folder = path.join(f.input, abi), receiptPath = path.join(folder, 'receipt.json');
  const receipt = JSON.parse(fs.readFileSync(receiptPath));
  const write = row => fs.writeFileSync(receiptPath, JSON.stringify(row));
  write({ ...receipt, sourceCommit: 'a'.repeat(40) });
  assert.throws(() => readBundleInputs(f.identity, f.input), /receipt differs/);
  write(receipt);
  const binary = path.join(folder, 'index.farm'), bytes = nativeHeader(abi);
  bytes[127] = 1; fs.writeFileSync(binary, bytes);
  assert.throws(() => readBundleInputs(f.identity, f.input), /receipt differs/);
  fs.writeFileSync(binary, nativeHeader('linux-x64-gnu'));
  assert.throws(() => readBundleInputs(f.identity, f.input), /architecture differs/);
  fs.renameSync(folder, path.join(f.root, abi));
  assert.throws(() => readBundleInputs(f.identity, f.input), /three-ABI artifact set/);
});

test('rejects dirty tracked source before producing output', t => {
  const f = fixture(t);
  fs.appendFileSync(path.join(f.directory, 'Cargo.lock'), '\n# stale\n');
  assert.throws(() => prepareMfBundle(f), /Tracked source differs/);
  assert.equal(fs.existsSync(f.output), false);
});

test('sealed candidate recheck rejects bit flips', t => {
  const f = fixture(t), receipt = prepareMfBundle(f);
  const file = path.join(f.output, receipt.file), bytes = fs.readFileSync(file);
  bytes[bytes.length - 1] ^= 1;
  fs.writeFileSync(file, bytes);
  assert.throws(() => readMfBundleForTest(f.directory, f.output), /Expected values to be strictly equal/);
});
