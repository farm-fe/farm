import test from 'node:test';
import assert from 'node:assert/strict';
import { compactReport, unavailableReport, unavailableReason, platforms } from './aggregate-performance-reports.mjs';
import fs from 'node:fs/promises';
import { validateSummary, renderComment } from './performance-pr-reporter.mjs';
const cell = value => ({ cacheMode: 'cold', samples: 3, validSamples: 3, medianMs: value, iqrMs: 1 });
test('missing artifacts after skipped sampling produce explicit unavailable reports', async () => {
  const reason = unavailableReason(new Error('ENOENT'), { samplingResult: 'skipped', downloadOutcome: 'failure' });
  assert.match(reason, /Sampling job: skipped/);
  assert.match(reason, /Artifact download: failure/);
  const reports = platforms.map(([, os, arch]) => unavailableReport(os, arch, 'a'.repeat(40), 'b'.repeat(40), reason));
  assert.equal(reports.length, 3);
  for (const report of reports) {
    assert.equal(report.status, 'invalid'); assert.deepEqual(report.cells, []);
    assert.ok(report.diagnostics.includes(reason));
  }
  const workflow = await fs.readFile(new URL('../.github/workflows/ci.yaml', import.meta.url), 'utf8');
  const summary = workflow.split('\n  performance-summary:')[1].split('\n  type-check:')[0];
  assert.match(summary, /id: performance-inputs\r?\n\s+continue-on-error: true/);
  assert.match(summary, /PERFORMANCE_SAMPLING_RESULT:/);
  assert.match(summary, /run: node scripts\/aggregate-performance-reports.mjs/);
});
test('Farm base/head cells remain separate and comparison fields are absent', () => {
  const report = compactReport({ metadata: {}, status: 'complete', cells: [cell(100)], comparison: { mode: 'paired', baseCells: [cell(120)] } });
  assert.equal(report.cells[0].medianMs, 100);
  assert.equal(report.comparison.baseCells[0].medianMs, 120);
  assert.deepEqual(Object.keys(report.comparison).sort(), ['baseCells', 'mode']);
});

test('old mixed-tool protocol is rejected instead of relabeled as Farm-only history', () => {
  assert.throws(() => validateSummary(JSON.stringify({ schemaVersion: 2, repositoryId: 1, runId: 2, runAttempt: 1, event: 'pull_request', headSHA: 'a'.repeat(40), baseSHA: 'b'.repeat(40), prNumber: 3, protocol: 'commerce-business-2048-supervised-v1', reports: [] })));
});

test('all three missing platforms remain explicit invalid evidence with reasons', () => {
  const summary = { schemaVersion: 2, repositoryId: 1, runId: 2, runAttempt: 1, event: 'pull_request', headSHA: 'a'.repeat(40), baseSHA: 'b'.repeat(40), prNumber: 3, protocol: 'commerce-business-2048-farm-supervised-v2',
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
test('missing baseline and history stay absent rather than zero', () => {
  const report = compactReport({ status: 'invalid', cells: [] });
  assert.equal(report.comparison.mode, 'current-only');
  for (const key of ['baseCells']) assert.deepEqual(report.comparison[key], []);
  assert.deepEqual(report.history, []);
});
test('Darwin resource failure retains observed producer identity and the actual reason', () => {
  const report = compactReport({ status: 'invalid', failureKind: 'resource-unavailable', metadata: { os: 'darwin', arch: 'arm64', revision: 'c'.repeat(40), baseSHA: 'b'.repeat(40) },
    errors: ['Available memory probe unavailable; observed free pages are not available memory'] });
  const summary = { schemaVersion: 2, repositoryId: 1, runId: 2, runAttempt: 1, event: 'pull_request', headSHA: 'a'.repeat(40), baseSHA: 'b'.repeat(40), prNumber: 3, protocol: 'commerce-business-2048-farm-supervised-v2', reports: [report] };
  validateSummary(JSON.stringify(summary));
  assert.equal(report.metadata.revision, 'c'.repeat(40));
  assert.ok(report.diagnostics.includes('Failure kind: resource-unavailable'));
  assert.match(renderComment(summary, 'farm-fe/farm'), /free pages/);
});

test('invalid producer receipts cannot expose build-only timings as valid head/base metrics', () => {
  const report = compactReport({ status: 'invalid', cells: [cell(123)], comparison: { mode: 'paired', baseCells: [cell(456)] } });
  for (const row of [...report.cells, ...report.comparison.baseCells]) {
    assert.equal(row.validSamples, 0); assert.equal(row.medianMs, null); assert.equal(row.iqrMs, null);
  }
  const rendered = renderComment({ ...report, runId: 1, headSHA: 'a'.repeat(40) }, 'farm-fe/farm');
  assert.doesNotMatch(rendered, /123.0 ms|456.0 ms/);
  assert.match(rendered, /Unavailable/);
});
