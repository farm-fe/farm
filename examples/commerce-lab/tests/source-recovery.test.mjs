import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { restoreSourceProbes } from '../benchmark/source-recovery.mjs';

test('recovery restores known edits and preserves unknown concurrent edits', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-recovery-'));
  const backups = new Map([['a.ts', 'original A'], ['b.ts', 'original B'], ['c.ts', 'original C']]);
  await fs.writeFile(path.join(root, 'a.ts'), 'probe A');
  await fs.writeFile(path.join(root, 'b.ts'), 'human B');
  await fs.writeFile(path.join(root, 'c.ts'), 'probe C');
  const result = await restoreSourceProbes(root, backups, [['a.ts', 'original', 'probe'], ['b.ts', 'original', 'probe'], ['c.ts', 'original', 'probe']]);
  assert.deepEqual(result.restored, ['a.ts', 'c.ts']);
  assert.deepEqual(result.conflicts, ['b.ts']);
  assert.deepEqual(result.errors, []);
  assert.equal(await fs.readFile(path.join(root, 'b.ts'), 'utf8'), 'human B');
  assert.equal(await fs.readFile(path.join(root, 'c.ts'), 'utf8'), 'original C');
});

test('missing files and escaped paths fail without stopping other recovery', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-recovery-'));
  const result = await restoreSourceProbes(root, new Map([['missing.ts', 'source'], ['../escaped.ts', 'source']]), []);
  assert.equal(result.errors.length, 2);
});
