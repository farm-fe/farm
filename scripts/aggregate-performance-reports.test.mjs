import test from 'node:test';
import assert from 'node:assert/strict';
import { compactReport } from './aggregate-performance-reports.mjs';
const cell = value => ({ cacheMode: 'cold', samples: 3, validSamples: 3, medianMs: value, iqrMs: 1 });
test('full Commerce and common-subset measurements retain separate identities', () => {
  const raw = { metadata: {}, status: 'complete', cells: [cell(100)], comparison: { mode: 'paired', baseCells: [cell(120)] },
    vite: { farmCells: [cell(10)], viteCells: [cell(20)] } };
  const report = compactReport(raw);
  assert.equal(report.cells[0].medianMs, 100);
  assert.equal(report.comparison.baseCells[0].medianMs, 120);
  assert.equal(report.comparison.farmSubsetCells[0].medianMs, 10);
  assert.equal(report.comparison.viteCells[0].medianMs, 20);
});
test('missing baseline, subset and history stay absent rather than zero', () => {
  const report = compactReport({ status: 'invalid', cells: [] });
  assert.equal(report.comparison.mode, 'current-only');
  for (const key of ['baseCells', 'farmSubsetCells', 'viteCells']) assert.deepEqual(report.comparison[key], []);
  assert.deepEqual(report.history, []);
});
