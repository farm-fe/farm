import assert from 'node:assert/strict';
import test from 'node:test';
import { summarize, quantile, toCsv } from '../benchmark/statistics.mjs';

test('quantiles use linear interpolation over sorted samples', () => {
  assert.equal(quantile([40, 10, 30, 20], 0.5), 25);
  assert.equal(quantile([40, 10, 30, 20], 0.25), 17.5);
  assert.equal(quantile([7], 0.95), 7);
  assert.equal(quantile([], 0.5), null);
  assert.throws(() => quantile([1], 2), /probability/);
});

test('failures are counted and never treated as zero-duration successes', () => {
  const records = [
    ...[10, 20, 30, 40].map(durationMs => ({ group: 'js', scenario: 'hmr-svg', scale: 'baseline', cacheMode: 'warm', durationMs, success: true })),
    { group: 'js', scenario: 'hmr-svg', scale: 'baseline', cacheMode: 'warm', durationMs: 30000, success: false, error: 'timeout' },
  ];
  const [result] = summarize(records);
  assert.equal(result.samples, 5);
  assert.equal(result.validSamples, 4);
  assert.equal(result.failures, 1);
  assert.equal(result.failureRate, 0.2);
  assert.deepEqual(result.durationMs, { min: 10, q1: 17.5, median: 25, q3: 32.5, iqr: 15, p95: 38.5, max: 40 });
  assert.equal(result.sufficientSamples, false);
});

test('groups, scales and cache modes remain separate; all-failed cells have no timing', () => {
  const cells = summarize([
    { group: 'js', scenario: 'build', scale: 'baseline', cacheMode: 'cold', success: false, durationMs: 8 },
    { group: 'js', scenario: 'build', scale: 'large', cacheMode: 'cold', success: true, durationMs: 20 },
    { group: 'rust', scenario: 'build', scale: 'baseline', cacheMode: 'warm', success: true, durationMs: 10 },
  ]);
  assert.equal(cells.length, 3);
  assert.equal(cells.find(cell => cell.failures === 1).durationMs, null);
  assert.throws(() => summarize([{ group: 'js', scenario: 'build', success: true, durationMs: -1 }]), /durationMs/);
});

test('CSV preserves raw failures, commas, quotes and line breaks', () => {
  const result = toCsv([{ runId: '1', success: false, error: 'bad,"input"\nretry' }], ['runId', 'success', 'error']);
  assert.equal(result, 'runId,success,error\r\n1,false,"bad,""input""\nretry"\r\n');
});
