import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { writeJsonCheckpoint } from '../benchmark/report-checkpoint.mjs';

test('a failed checkpoint preserves the last complete report', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-checkpoint-'));
  const target = path.join(root, 'run.json');
  await fs.writeFile(target, JSON.stringify({ records: [1] }));
  const diskFull = Object.assign(new Error('disk full'), { code: 'ENOSPC' });
  await assert.rejects(writeJsonCheckpoint(target, { records: [1, 2] }, {
    ...fs,
    async open(...args) {
      const handle = await fs.open(...args);
      return {
        async writeFile() { throw diskFull; },
        sync: () => handle.sync(),
        close: () => handle.close(),
      };
    },
  }), { code: 'ENOSPC' });
  assert.deepEqual(JSON.parse(await fs.readFile(target, 'utf8')), { records: [1] });
  assert.deepEqual(await fs.readdir(root), ['run.json']);
});

test('a complete checkpoint replaces the report with valid JSON', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-checkpoint-'));
  const target = path.join(root, 'run.json');
  await writeJsonCheckpoint(target, { records: [] });
  await writeJsonCheckpoint(target, { records: [1, 2], error: 'server exited' });
  assert.deepEqual(JSON.parse(await fs.readFile(target, 'utf8')), { records: [1, 2], error: 'server exited' });
  assert.deepEqual(await fs.readdir(root), ['run.json']);
});
