import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { exampleNativeInput } from './example-native-inputs.mjs';
test('a foreign platform file does not satisfy this host and input presence is not ABI execution', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-example-native-'));
  try {
    const foreign = path.join(directory, 'npm/darwin-arm64'); fs.mkdirSync(foreign, { recursive: true }); fs.writeFileSync(path.join(foreign, 'index.farm'), 'fixture only');
    const host = { platform: 'linux', arch: 'x64' };
    assert.equal(exampleNativeInput(directory, 'linux-x64-gnu', host), null);
    assert.throws(() => exampleNativeInput(directory, 'darwin-arm64', host), /does not match/);
    const local = path.join(directory, 'npm/linux-x64-gnu'); fs.mkdirSync(local); fs.writeFileSync(path.join(local, 'index.farm'), 'fixture only');
    assert.match(exampleNativeInput(directory, 'linux-x64-gnu', host).verification, /actual ABI requires native execution/);
  } finally { fs.rmSync(directory, { recursive: true }); }
});
