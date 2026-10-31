import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fingerprintNativeFile } from '../benchmark/native-provenance.mjs';

test('native identity follows exact file bytes and preserves real path', async () => {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-native-'));
  const file = path.join(dir, 'fixture.farm');
  await fs.writeFile(file, 'first artifact');
  const first = await fingerprintNativeFile(file);
  assert.equal(first.path, await fs.realpath(file));
  assert.equal(first.bytes, 14);
  assert.match(first.sha256, /^[a-f0-9]{64}$/);
  await fs.writeFile(file, 'second artifact');
  assert.notEqual((await fingerprintNativeFile(file)).sha256, first.sha256);
  await assert.rejects(fingerprintNativeFile(path.join(dir, 'missing.farm')), { code: 'ENOENT' });
});
