// cspell:words SDKROOT
// cspell:words RUSTFLAGS esac
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { captureRuntime, captureProducts, captureProductProof, proofDirectory } from './ci-native-capture.mjs';

test('runtime capture measures only bounded tools and never publishes arbitrary environment', () => {
  const calls = [];
  const record = captureRuntime({ platform: 'linux', env: { FARM_BUILD_ABI: 'linux-x64-gnu', FARM_BUILD_TARGET: 'x86_64-unknown-linux-gnu', FARM_BUILD_PROFILE: 'ci', RUSTFLAGS: '-Copt-level=3', GH_TOKEN: 'secret-fixture' }, run(executable, args, options) {
    calls.push([executable, args]); assert.equal(options.shell, false); assert.equal(options.timeout, 15000);
    return { status: 0, stdout: 'measured tool version' };
  } });
  assert.deepEqual(calls.map(row => row[0]), ['rustc', 'cargo', 'rustup']);
  assert.equal(record.linker, null); assert.equal(record.completeMeasuredContext, false); assert.equal(record.reuseAuthorized, false);
  assert.ok(!JSON.stringify(record).includes('secret-fixture')); assert.match(record.flagsHash, /^[a-f0-9]{64}$/);
});

test('a failed context probe remains unavailable and never becomes reusable', () => {
  const record = captureRuntime({ platform: 'linux', env: {}, run: () => ({ status: 1, stdout: '', error: { code: 'ENOENT' } }) });
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


test('bounded tool probes cannot auto-install or mutate the real build environment', () => {
  for (const inherited of [undefined, '0', '1']) {
    const env = { FARM_BUILD_TARGET: 'x86_64-unknown-linux-gnu', CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER: 'gcc' };
    if (inherited !== undefined) env.RUSTUP_AUTO_INSTALL = inherited;
    const original = {...env}, calls = [];
    const record = captureRuntime({ platform: 'linux', env, run(executable, args, options) {
      calls.push(executable);
      assert.notEqual(options.env, env);
      assert.equal(options.env.RUSTUP_AUTO_INSTALL, '0');
      assert.equal(options.timeout, 15000);
      // An absent compiler is unavailable evidence, not permission to install.
      return executable === 'gcc' ? {status: 0, stdout: 'gcc fixture'} : {status: 1, stdout: ''};
    } });
    assert.deepEqual(calls, ['rustc', 'cargo', 'rustup', 'gcc']);
    assert.deepEqual(env, original);
    assert.equal(record.rustc, null); assert.equal(record.cargo, null);
    assert.equal(record.errors.length, 3);
    assert.equal(record.measuredToolsComplete, false); assert.equal(record.reuseAuthorized, false);
  }
});


test('real native build resolves its toolchain before bounded shadow probes', () => {
  const source = fs.readFileSync(new URL('./ci-build-rust-artifacts.sh', import.meta.url), 'utf8');
  const phase = source.slice(source.indexOf('if [ "${FARM_NATIVE_PROOF_MODE:-off}"'));
  const setup = phase.indexOf('cargo --version');
  const probe = phase.indexOf('node scripts/ci-native-capture.mjs runtime');
  assert.ok(setup >= 0 && probe > setup);
  assert.ok(source.startsWith('#!/usr/bin/env sh\nset -eu'));
  assert.doesNotMatch(phase.slice(setup, probe), /cargo --version[^\n]*(?:\|\||&)/);
  assert.doesNotMatch(source, /RUSTUP_AUTO_INSTALL=/);
});


test('native shell stops failed toolchain setup before capture or compilation', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-native-setup-'));
  try {
    const bin = path.join(root, 'bin'), log = path.join(root, 'events'); fs.mkdirSync(bin);
    for (const [name, body] of Object.entries({
      cargo: 'echo setup >> "$PROBE_EVENTS"; exit "$SETUP_EXIT"',
      node: 'case "$1" in *ci-native-capture.mjs) echo probe >> "$PROBE_EVENTS" ;; esac',
      npm: 'echo build >> "$PROBE_EVENTS"'
    })) fs.writeFileSync(path.join(bin, name), '#!/bin/sh\n' + body + '\n', {mode: 0o755});
    const source = fileURLToPath(new URL('./ci-build-rust-artifacts.sh', import.meta.url));
    const shell = fs.readFileSync(source, 'utf8');
    const plugins = shell.match(/PLUGINS="([a-z -]+)"/)[1].split(' ');
    plugins.push('ssr-css', 'syntax-highlight', 'docs', 'docs-snapshot', 'docs-seo', 'docs-snippets');
    for (const folder of ['packages/core', 'packages/create-farm', ...plugins.map(name => 'rust-plugins/' + name)]) fs.mkdirSync(path.join(root, folder), {recursive: true});
    for (const status of [17, 0]) {
      fs.writeFileSync(log, '');
      const result = spawnSync('sh', [source], {cwd: root, encoding: 'utf8', timeout: 10000,
        env: {...process.env, PATH: bin + path.delimiter + process.env.PATH, PROBE_EVENTS: log, SETUP_EXIT: String(status),
          FARM_BUILD_ABI: 'darwin-arm64', FARM_BUILD_PROFILE: 'ci', FARM_NATIVE_PROOF_MODE: 'shadow', FARM_RELEASE_NATIVE_OWNERS: ''}});
      assert.equal(result.status, status, result.stderr);
      const events = fs.readFileSync(log, 'utf8').trim().split('\n');
      if (status) assert.deepEqual(events, ['setup']);
      else { assert.deepEqual(events.slice(0, 3), ['setup', 'probe', 'build']); assert.equal(events.filter(x => x === 'build').length, plugins.length + 2); }
    }
  } finally {fs.rmSync(root, {recursive: true, force: true});}
});

test('CI strip override is measured in the recipe fingerprint', () => {
  const run = () => ({status: 0, stdout: 'tool'});
  const before = captureRuntime({env: {FARM_BUILD_ABI: 'darwin-arm64', FARM_BUILD_PROFILE: 'ci'}, run});
  const after = captureRuntime({env: {FARM_BUILD_ABI: 'darwin-arm64', FARM_BUILD_PROFILE: 'ci', CARGO_PROFILE_CI_STRIP: 'none'}, run});
  assert.notEqual(before.flagsHash, after.flagsHash);
});

test('native recipe leaves CI-only strip overrides unset for both formal roles', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-native-strip-'));
  try {
    const source = fs.readFileSync(new URL('./ci-build-rust-artifacts.sh', import.meta.url), 'utf8');
    const bin = path.join(root, 'bin'); fs.mkdirSync(bin);
    fs.writeFileSync(path.join(bin, 'node'), '#!/bin/sh\nexit 0\n', {mode: 0o755});
    // Execute the actual setup section; stop before tool probes or build commands.
    const setup = source.slice(0, source.indexOf('if [ "${FARM_NATIVE_PROOF_MODE'));
    for (const [abi, profile, expected] of [['darwin-arm64', 'ci', 'unset'], ['darwin-x64', 'ci', 'unset'], ['darwin-arm64', 'release', 'unset'], ['linux-x64-gnu', 'ci', 'unset']]) {
      const env = {...process.env, PATH: bin + path.delimiter + process.env.PATH, FARM_BUILD_ABI: abi, FARM_BUILD_PROFILE: profile, FARM_RELEASE_NATIVE_OWNERS: ''};
      delete env.CARGO_PROFILE_CI_STRIP;
      const result = spawnSync('sh', ['-c', setup + '\nprintf "%s" "${CARGO_PROFILE_CI_STRIP-unset}"'], {cwd: root, env, encoding: 'utf8'});
      assert.equal(result.status, 0, result.stderr); assert.equal(result.stdout, expected);
    }
  } finally {fs.rmSync(root, {recursive: true, force: true});}
});

test('formal compatibility binds Apple deployment and C compiler flags',()=>{
 const capture=extra=>captureRuntime({platform:'darwin',env:{FARM_BUILD_ABI:'darwin-arm64',FARM_BUILD_TARGET:'aarch64-apple-darwin',...extra},run:()=>({status:0,stdout:'measured'})});
 const base=capture({});
 for(const [key,a,b]of [['MACOSX_DEPLOYMENT_TARGET','11.0','14.0'],['SDKROOT','/sdk/one','/sdk/two'],['CFLAGS','-O1','-O2'],['CARGO_BUILD_JOBS','2','4']]) {
  assert.notEqual(capture({[key]:a}).formalFlagsHash,capture({[key]:b}).formalFlagsHash,key);
  assert.deepEqual(capture({[key]:a}).unboundBuildVariables,[]);
 }
 assert.deepEqual(capture({CFLAGS_aarch64_apple_darwin:'-O2'}).unboundBuildVariables,['CFLAGS_aarch64_apple_darwin']);
 assert.equal(base.errors.length,0);
});
