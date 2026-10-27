import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { aggregate, compatible, command } from './performance-report.mjs';

test('failed and missing samples do not become zero-duration measurements', () => {
  const cells = aggregate([
    { scored: false, cacheMode: 'cold', success: true, durationMs: 999 },
    { scored: true, cacheMode: 'cold', success: true, durationMs: 10 },
    { scored: true, cacheMode: 'cold', success: true, durationMs: 30 },
    { scored: true, cacheMode: 'cold', success: false, durationMs: null }
  ]);
  assert.equal(cells[0].medianMs, 20);
  assert.equal(cells[0].iqrMs, 10);
  assert.equal(cells[0].validSamples, 2);
  assert.equal(cells[1].medianMs, null);
});
test('baseline requires matching explicit protocol, fixture and toolchain dimensions', () => {
  const metadata = { protocol: 'v1', fixtureSha256: 'abc', node: '22', pnpm: '9', os: 'linux', arch: 'x64', lockSha256: 'def' };
  assert.ok(compatible(metadata, { ...metadata }));
  assert.equal(compatible(metadata, { ...metadata, arch: 'arm64' }), false);
  assert.equal(compatible({}, {}), false);
});
test('failed child command is visible and timeout terminates the owned child tree', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-test-'));
  try {
    await assert.rejects(command(process.execPath, ['-e', 'process.exit(7)'], { log: path.join(directory, 'error.log') }), /Command failed/);
    await assert.rejects(command(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { log: path.join(directory, 'timeout.log'), timeoutMs: 100 }), /timedOut/);
  } finally { await fs.rm(directory, { recursive: true }); }
});
test('performance job is non-blocking, follows Examples and pins artifact names to SHA', async () => {
  const source = await fs.readFile(new URL('../.github/workflows/ci.yaml', import.meta.url), 'utf8');
  const job = source.split('\n  performance-report:')[1].split('\n  type-check:')[0];
  assert.match(job, /needs: \[examples-test\]/);
  assert.match(job, /continue-on-error: true/);
  assert.match(job, /timeout-minutes: 20/);
  assert.equal((job.match(/os: .*latest/g) ?? []).length, 3);
  assert.doesNotMatch(job, /cargo |pattern:/);
  for (const line of job.split('\n').filter(line => line.trim().startsWith('name:') && line.includes('plugin-'))) assert.ok(line.includes('github.sha'));
});
test('setup failure still produces invalid structured evidence', async () => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-perf-evidence-'));
  try {
    const script = fileURLToPath(new URL('./performance-evidence.mjs', import.meta.url));
    for (const stage of ['init', 'finalize']) await command(process.execPath, [script, stage], { cwd: directory, log: path.join(directory, 'test.log') });
    const report = JSON.parse(await fs.readFile(path.join(directory, 'performance-results/raw.json'), 'utf8'));
    assert.equal(report.status, 'invalid');
    assert.equal(report.publishable, false);
    assert.deepEqual(report.records, []);
    for (const file of ['summary.json', 'samples.csv', 'run.log']) await fs.access(path.join(directory, 'performance-results', file));
  } finally { await fs.rm(directory, { recursive: true }); }
});
