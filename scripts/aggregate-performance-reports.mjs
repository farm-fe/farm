import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { validateSummary } from './performance-pr-reporter.mjs';
import { validationEvent } from './v2-release-inventory.mjs';

export function compactReport(report) {
  const metadata = Object.fromEntries(['os', 'arch', 'revision', 'baseSHA', 'node', 'pnpm', 'fixtureSha256', 'lockSha256', 'runnerImage'].map(key => [key, report.metadata?.[key] ?? null]));
  return { metadata, status: report.status, cells: report.cells ?? [],
    comparison: { mode: report.comparison?.mode ?? 'current-only', baseCells: report.comparison?.mode === 'paired' ? report.comparison.baseCells : [],
      farmSubsetCells: report.vite?.farmCells ?? [], viteCells: report.vite?.viteCells ?? [], subsetProtocol: 'commerce-catalog-quote-subset-v1' },
    history: (report.history?.entries ?? []).slice(0, 10).map(row => ({ headSHA: row.sourceHeadSHA, runId: row.runId,
      medianMs: row.cells.find(cell => cell.cacheMode === 'cold')?.medianMs })).filter(row => Number.isFinite(row.medianMs)),
    diagnostics: [report.failureKind ? `Failure kind: ${report.failureKind}` : null, report.comparison?.reason, report.history?.reason, report.history?.error,
      ...(report.history?.excluded ?? []).map(row => typeof row === 'string' ? row : `History run ${row.runId}: ${row.reason}${row.detail ? ` (${row.detail})` : ''}`), ...(report.errors ?? [])]
      .filter(value => typeof value === 'string' && value.length).slice(0, 30).map(value => value.replace(/[\x00-\x1f]/g, ' ').slice(0, 500)) };
}
export const platforms = [['linux-x64-gnu', 'linux', 'x64'], ['darwin-arm64', 'darwin', 'arm64'], ['win32-x64-msvc', 'win32', 'x64']];
export function unavailableReport(os, arch, revision, baseSHA, reason) {
  const report = compactReport({ status: 'invalid', cells: [], errors: [reason] });
  report.metadata = { os, arch, revision, baseSHA, node: null, pnpm: null, fixtureSha256: null, lockSha256: null, runnerImage: null };
  return report;
}
export function unavailableReason(error, { samplingResult, downloadOutcome } = {}) {
  const context = [];
  if (samplingResult) context.push(`Sampling job: ${samplingResult}`);
  if (downloadOutcome) context.push(`Artifact download: ${downloadOutcome}`);
  return [...context, `Missing or invalid platform evidence: ${error.message}`].join('; ');
}
async function main() {
  const event = validationEvent();
  const directory = path.resolve('.performance-reports');
  const summary = { schemaVersion: 2, repositoryId: event.repository.id, runId: Number(process.env.GITHUB_RUN_ID), runAttempt: Number(process.env.GITHUB_RUN_ATTEMPT), event: 'pull_request',
    headSHA: event.pull_request.head.sha, baseSHA: event.pull_request.base.sha, prNumber: event.number,
    protocol: 'commerce-business-2048-rust-build-v2', reports: [] };
  const failures = [];
  for (const [abi, os, arch] of platforms) {
    const name = `performance-${process.env.GITHUB_SHA}-${abi}-attempt-${summary.runAttempt}`;
    try {
      const file = path.join(directory, name, 'summary.json');
      if ((await fs.stat(file)).size > 2 * 1024 * 1024) throw new Error('Oversized raw summary');
      const raw = JSON.parse(await fs.readFile(file, 'utf8'));
      if (raw.metadata?.revision !== process.env.GITHUB_SHA) throw new Error('Producer SHA mismatch');
      const report = compactReport(raw);
      if (report.metadata.os !== os || report.metadata.arch !== arch) throw new Error('Platform identity mismatch');
      validateSummary(JSON.stringify({ ...summary, reports: [report] }));
      summary.reports.push(report);
    } catch (error) {
      failures.push(`${name}: ${error.message}`);
      summary.reports.push(unavailableReport(os, arch, process.env.GITHUB_SHA, summary.baseSHA,
        unavailableReason(error, { samplingResult: process.env.PERFORMANCE_SAMPLING_RESULT, downloadOutcome: process.env.PERFORMANCE_DOWNLOAD_OUTCOME })));
    }
  }
  validateSummary(JSON.stringify(summary));
  await fs.mkdir('performance-pr-summary', { recursive: true });
  await fs.writeFile('performance-pr-summary/summary.json', JSON.stringify(summary));
  if (failures.length) {
    console.error(failures.join('\n'));
    if (process.env.GITHUB_STEP_SUMMARY) await fs.appendFile(process.env.GITHUB_STEP_SUMMARY, `\nIncomplete performance platforms:\n\n${failures.map(row => `- ${row}`).join('\n')}\n`);
    process.exitCode = 1;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
