// cspell:words RUSTFLAGS
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { captureRuntime, captureProducts } from './ci-native-capture.mjs';

test('runtime capture measures only bounded tools and never publishes arbitrary environment', () => {
  const calls = [];
  const record = captureRuntime({ env: { FARM_BUILD_ABI: 'linux-x64-gnu', FARM_BUILD_TARGET: 'x86_64-unknown-linux-gnu', FARM_BUILD_PROFILE: 'ci', RUSTFLAGS: '-Copt-level=3', GH_TOKEN: 'secret-fixture' }, run(executable, args, options) {
    calls.push([executable, args]); assert.equal(options.shell, false); assert.equal(options.timeout, 15000);
    return { status: 0, stdout: 'measured tool version' };
  } });
  assert.deepEqual(calls.map(row => row[0]), ['rustc', 'cargo']);
  assert.equal(record.linker, null); assert.equal(record.completeMeasuredContext, false); assert.equal(record.reuseAuthorized, false);
  assert.ok(!JSON.stringify(record).includes('secret-fixture')); assert.match(record.flagsHash, /^[a-f0-9]{64}$/);
});

test('a failed context probe remains unavailable and never becomes reusable', () => {
  const record = captureRuntime({ env: {}, run: () => ({ status: 1, stdout: '', error: { code: 'ENOENT' } }) });
  assert.equal(record.errors.length, 2); assert.equal(record.measuredToolsComplete, false); assert.equal(record.reuseAuthorized, false);
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
