import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { gzipSync } from 'node:zlib';
import { verifyCompression } from '../benchmark/artifacts.mjs';

test('artifact verification requires every eligible gzip and rejects corrupted output', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-artifacts-'));
  const source = Buffer.from('const product = 7;\n'.repeat(100));
  await fs.writeFile(path.join(directory, 'main.js'), source);
  await assert.rejects(verifyCompression(directory), /Missing gzip/);
  await fs.writeFile(path.join(directory, 'main.js.gz'), gzipSync(source));
  const result = await verifyCompression(directory);
  assert.equal(result.verifiedGzipFiles, 1);
  await fs.writeFile(path.join(directory, 'main.js.gz'), gzipSync('incorrect'));
  await assert.rejects(verifyCompression(directory), /differs/);
});

test('artifact verification rejects orphaned gzip resources', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-orphan-'));
  await fs.writeFile(path.join(directory, 'missing.js.gz'), gzipSync('missing'));
  await assert.rejects(verifyCompression(directory), /orphan/i);
});
