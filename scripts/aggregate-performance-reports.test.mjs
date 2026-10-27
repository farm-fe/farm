import test from 'node:test';
import assert from 'node:assert/strict';
import { compactReport, unavailableReport, platforms } from './aggregate-performance-reports.mjs';
import { validateSummary, renderComment } from './performance-pr-reporter.mjs';
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

test('all three missing platforms remain explicit invalid evidence with reasons', () => {
  const summary = { schemaVersion: 2, repositoryId: 1, runId: 2, runAttempt: 1, event: 'pull_request', headSHA: 'a'.repeat(40), baseSHA: 'b'.repeat(40), prNumber: 3, protocol: 'commerce-baseline-rust-build-v1',
    reports: platforms.map(([, os, arch]) => unavailableReport(os, arch, 'c'.repeat(40), 'b'.repeat(40), 'Report artifact missing')) };
  validateSummary(JSON.stringify(summary));
  assert.deepEqual(summary.reports.map(row => row.metadata.os), ['linux', 'darwin', 'win32']);
  assert.ok(summary.reports.every(row => row.status === 'invalid' && row.metadata.fixtureSha256 === null));
  const text = renderComment(summary, 'farm-fe/farm');
  assert.equal(text.match(/Report artifact missing/g).length, 3);
});
test('base and history exclusions survive compaction and render as escaped text', () => {
  const report = compactReport({ status: 'invalid', cells: [], comparison: { reason: 'Base lacks protocol' }, history: { excluded: [{ runId: 4, reason: 'expired' }], error: 'History unavailable' }, errors: ['<script>'] });
  assert.ok(report.diagnostics.includes('Base lacks protocol'));
  assert.ok(report.diagnostics.includes('History run 4: expired'));
  assert.ok(report.diagnostics.includes('History unavailable'));
  const text = renderComment({ ...report, runId: 2, headSHA: 'a'.repeat(40) }, 'farm-fe/farm');
  assert.doesNotMatch(text, /<script>/); assert.match(text, /&#60;script&#62;/);
});
test('missing baseline, subset and history stay absent rather than zero', () => {
  const report = compactReport({ status: 'invalid', cells: [] });
  assert.equal(report.comparison.mode, 'current-only');
  for (const key of ['baseCells', 'farmSubsetCells', 'viteCells']) assert.deepEqual(report.comparison[key], []);
  assert.deepEqual(report.history, []);
});
