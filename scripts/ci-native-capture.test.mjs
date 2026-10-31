// cspell:words RUSTFLAGS
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { captureRuntime, captureProducts, captureProductProof, proofDirectory } from './ci-native-capture.mjs';

test('runtime capture measures only bounded tools and never publishes arbitrary environment', () => {
  const calls = [];
  const record = captureRuntime({ env: { FARM_BUILD_ABI: 'linux-x64-gnu', FARM_BUILD_TARGET: 'x86_64-unknown-linux-gnu', FARM_BUILD_PROFILE: 'ci', RUSTFLAGS: '-Copt-level=3', GH_TOKEN: 'secret-fixture' }, run(executable, args, options) {
    calls.push([executable, args]); assert.equal(options.shell, false); assert.equal(options.timeout, 15000);
    return { status: 0, stdout: 'measured tool version' };
  } });
  assert.deepEqual(calls.map(row => row[0]), ['rustc', 'cargo', 'rustup']);
  assert.equal(record.linker, null); assert.equal(record.completeMeasuredContext, false); assert.equal(record.reuseAuthorized, false);
  assert.ok(!JSON.stringify(record).includes('secret-fixture')); assert.match(record.flagsHash, /^[a-f0-9]{64}$/);
});

test('a failed context probe remains unavailable and never becomes reusable', () => {
  const record = captureRuntime({ env: {}, run: () => ({ status: 1, stdout: '', error: { code: 'ENOENT' } }) });
  assert.equal(record.errors.length, 3); assert.equal(record.measuredToolsComplete, false); assert.equal(record.reuseAuthorized, false);
});

test('producer records require every exact native file and preserve its measured byte hash', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-native-shadow-'));
  const abi = 'linux-x64-gnu';
  const inventory = { npm: { primary: [{ name: '@farmfe/core', manifestPath: 'packages/core/package.json' }], payloads: [{ name: '@farmfe/core-linux-x64-gnu', owner: '@farmfe/core', platform: abi }] } };
  try {
    const file = path.join(root, 'packages/core/binding/farm.linux-x64-gnu.node'); fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, 'synthetic native bytes');
    const record = captureProducts({ cwd: root, env: { FARM_BUILD_ABI: abi, FARM_BUILD_PROFILE: 'ci' }, inventory });
    assert.equal(record.records.length, 1); assert.equal(record.records[0].bytes, 22); assert.match(record.records[0].sha256, /^[a-f0-9]{64}$/);
    assert.equal(record.reuseAuthorized, false); assert.equal(record.acceptanceInherited, false);
    fs.rmSync(file); assert.throws(() => captureProducts({ cwd: root, env: { FARM_BUILD_ABI: abi }, inventory }));
  } finally { fs.rmSync(root, { recursive: true }); }
});

test('host product evidence never writes the container runtime directory', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-native-host-proof-')), abi = 'linux-x64-gnu';
  try {
    const runtime = proofDirectory(root, abi, 'runtime'), host = proofDirectory(root, abi, 'products');
    assert.notEqual(runtime, host); fs.mkdirSync(runtime, {recursive: true});
    const measured = Buffer.from('{"profile":"release","measuredToolsComplete":true}');
    fs.writeFileSync(path.join(runtime, 'runtime.json'), measured);
    const binary = path.join(root, 'packages/core/binding', `farm.${abi}.node`);
    fs.mkdirSync(path.dirname(binary), {recursive: true}); fs.writeFileSync(binary, 'native fixture');
    const inventory = {npm: {primary: [{name: '@farmfe/core', manifestPath: 'packages/core/package.json'}],
      payloads: [{name: '@farmfe/core-linux-x64-gnu', owner: '@farmfe/core', platform: abi}]}};
    const writes = [], io = {...fs, writeFileSync(file, ...args) {
      assert.ok(!file.startsWith(runtime + path.sep), 'Container runtime directory is read only');
      writes.push(file); return fs.writeFileSync(file, ...args);
    }};
    const env = {FARM_BUILD_ABI: abi, FARM_BUILD_PROFILE: 'release'};
    const record = captureProductProof({cwd: root, env, inventory, io});
    assert.equal(record.profile, 'release'); assert.equal(record.records.length, 1);
    assert.deepEqual(fs.readFileSync(path.join(host, 'runtime.json')), measured);
    assert.deepEqual(fs.readFileSync(path.join(runtime, 'runtime.json')), measured);
    assert.ok(writes.every(file => path.dirname(file) === host));
    assert.equal(JSON.parse(fs.readFileSync(path.join(host, 'products.json'))).records[0].sha256, record.records[0].sha256);
    const count = writes.length;
    assert.deepEqual(captureProductProof({cwd: root, env, inventory, io}), record);
    assert.equal(writes.length, count);
    fs.writeFileSync(binary, 'changed native fixture');
    assert.throws(() => captureProductProof({cwd: root, env, inventory, io}), /Existing native evidence differs/);
    assert.throws(() => proofDirectory(root, '../elsewhere', 'products'));
    assert.throws(() => proofDirectory(root, abi, 'unknown'));
  } finally { fs.rmSync(root, {recursive: true}); }
});

test('both native workflows capture and upload the same host-owned product namespace', () => {
  for (const name of ['rust-build-user.yaml', 'rust-build-release.yaml']) {
    const source = fs.readFileSync(new URL('../.github/workflows/' + name, import.meta.url), 'utf8');
    assert.ok(source.includes('--products .ci-native-host-proof/${{ matrix.settings.abi }}/products.json --output .ci-native-host-proof/${{ matrix.settings.abi }}/source.json'));
    assert.ok(source.includes('path: .ci-native-host-proof/${{ matrix.settings.abi }}/*.json'));
    assert.ok(!source.includes('--products .ci-native-proof/'));
    assert.ok(source.includes('name: native-shadow-${{ github.sha }}-${{ matrix.settings.abi }}-${{ github.run_attempt }}'));
  }
});


test('release proof requires exactly the selected owners and every selected file', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-native-selection-'));
  const abi = 'linux-x64-gnu';
  const inventory = {npm: {primary: [{name: '@farmfe/core', manifestPath: 'packages/core/package.json'}, {name: '@farmfe/plugin-react', manifestPath: 'rust-plugins/react/package.json'}],
    payloads: [{name: 'core-payload', owner: '@farmfe/core', platform: abi}, {name: 'react-payload', owner: '@farmfe/plugin-react', platform: abi}]}};
  try {
    const files = ['packages/core/binding/farm.linux-x64-gnu.node', 'rust-plugins/react/npm/linux-x64-gnu/index.farm'];
    for (const file of files) {fs.mkdirSync(path.dirname(path.join(root, file)), {recursive: true}); fs.writeFileSync(path.join(root, file), 'native fixture');}
    const capture = owners => captureProducts({cwd: root, inventory, env: {FARM_BUILD_ABI: abi, FARM_BUILD_PROFILE: 'release', FARM_RELEASE_NATIVE_OWNERS: JSON.stringify(owners)}});
    assert.deepEqual(capture(['@farmfe/core']).records.map(row => row.owner), ['@farmfe/core']);
    assert.deepEqual(capture(['@farmfe/plugin-react']).records.map(row => row.owner), ['@farmfe/plugin-react']);
    assert.deepEqual(capture([]).records, []);
    fs.rmSync(path.join(root, files[1])); assert.throws(() => capture(['@farmfe/plugin-react']));
    assert.equal(capture(['@farmfe/core']).records.length, 1);
    assert.throws(() => capture(['unknown-owner']));
  } finally {fs.rmSync(root, {recursive: true});}
});

test('release capture and exact product upload fail closed while CI remains shadow', () => {
  for (const name of ['rust-build-user.yaml', 'rust-build-release.yaml']) {
    const source = fs.readFileSync(new URL('../.github/workflows/' + name, import.meta.url), 'utf8');
    const capture = source.split('- name: Capture exact native product bytes')[1].split('- name: Capture committed source ledger')[0];
    assert.ok(capture.includes("continue-on-error: ${{ inputs.profile != 'release' }}"));
    assert.ok(capture.includes('node scripts/ci-native-capture.mjs products'));
    const upload = source.split('- name: Preserve required release product proof')[1].split('- name: Preserve bounded shadow')[0];
    assert.ok(upload.includes("if: ${{ inputs.profile == 'release' }}"));
    assert.ok(!upload.includes('continue-on-error:')); assert.ok(upload.includes('if-no-files-found: error'));
    assert.ok(upload.includes('path: .ci-native-host-proof/${{ matrix.settings.abi }}/products.json'));
    assert.ok(!upload.includes('*.json'));
    const shadow = source.split('- name: Preserve bounded shadow')[1].split('- name: Preserve actual cell')[0];
    assert.ok(shadow.includes("always() && inputs.profile != 'release'"));
    assert.ok(shadow.includes('continue-on-error: true'));
    assert.ok(source.includes('FARM_RELEASE_NATIVE_OWNERS: ${{ inputs.native_owners }}'));
  }
});
