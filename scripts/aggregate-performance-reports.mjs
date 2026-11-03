import { execFileSync } from 'node:child_process';
import { measurementResolver, measurementPlatforms } from './performance-attempts.mjs';
import { githubClient } from './performance-cohorts.mjs';
import { qualifiedPerformanceCells, completePerformanceCells } from './performance-metrics.mjs';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { validateSummary } from './performance-pr-reporter.mjs';
import { validationEvent } from './v2-release-inventory.mjs';
import assert from 'node:assert/strict';

export function measurementIdentity(event,environment=process.env){
  if(environment.GITHUB_EVENT_NAME==='push'){
    assert.equal(environment.GITHUB_REF,'refs/heads/main');
    assert.equal(event.after,environment.GITHUB_SHA);
    assert.match(event.after,/^[a-f0-9]{40}$/);assert.match(event.before,/^[a-f0-9]{40}$/);
    assert.notEqual(event.before,'0'.repeat(40));
    return {event:'push',headSHA:event.after,baseSHA:event.before,prNumber:null};
  }
  assert.ok(event.pull_request&&Number.isSafeInteger(event.number)&&event.number>0,'Exact PR measurement identity required');
  return {event:environment.GITHUB_EVENT_NAME === 'workflow_dispatch' ? 'workflow_dispatch' : 'pull_request',headSHA:event.pull_request.head.sha,baseSHA:event.pull_request.base.sha,prNumber:event.number};
}

export function compactReport(report) {
  const metadata = Object.fromEntries(['os', 'arch', 'revision', 'baseSHA', 'node', 'pnpm', 'fixtureSha256', 'lockSha256', 'runnerImage'].map(key => [key, report.metadata?.[key] ?? null]));
  return { metadata, status: report.status, cells: qualifiedPerformanceCells(report.status, report.cells ?? []),
    comparison: { mode: report.comparison?.mode ?? 'current-only', baseCells: report.comparison?.mode === 'paired' ? qualifiedPerformanceCells(report.status, report.comparison.baseCells ?? []) : [] },
    history: (report.status === 'complete' ? report.history?.entries ?? [] : []).filter(row => row.status === 'complete' && completePerformanceCells(row.cells)).slice(0, 10).map(row => ({ headSHA: row.sourceHeadSHA, runId: row.runId,
      medianMs: row.cells.find(cell => cell.cacheMode === 'cold')?.medianMs })).filter(row => Number.isFinite(row.medianMs) && row.medianMs > 0),
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
  const event = process.env.GITHUB_EVENT_NAME==='push' ? JSON.parse(await fs.readFile(process.env.GITHUB_EVENT_PATH,'utf8')) : validationEvent();
  const summary = { schemaVersion: 3, repositoryId: event.repository.id, runId: Number(process.env.GITHUB_RUN_ID), runAttempt: Number(process.env.GITHUB_RUN_ATTEMPT),
    ...measurementIdentity(event),
    protocol: 'commerce-business-2048-farm-supervised-v2', reports: [], measurementSources: [] };
  const failures = [];
  assert.equal(execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),process.env.GITHUB_SHA,'Aggregation checkout differs');
  let resolve, resolutionError;
  try {
    resolve = await measurementResolver({api:githubClient(), repository:process.env.GITHUB_REPOSITORY, summary, producerSHA:process.env.GITHUB_SHA,
      testedTree:execFileSync('git',['rev-parse','HEAD^{tree}'],{encoding:'utf8'}).trim()});
  } catch(error) { resolutionError = error; }
  for (const platform of measurementPlatforms) {
    const [abi, os, arch] = platform;
    const name = `performance-${process.env.GITHUB_SHA}-${abi}-attempt-${summary.runAttempt}`;
    try {
      if (resolutionError) throw resolutionError;
      const {raw, source} = await resolve(platform);
      const report = compactReport(raw);
      if (report.metadata.os !== os || report.metadata.arch !== arch) throw new Error('Platform identity mismatch');
      validateSummary(JSON.stringify({ ...summary, reports: [report], measurementSources:[source] }));
      summary.reports.push(report);
      summary.measurementSources.push(source);
    } catch (error) {
      failures.push(`${name}: ${error.message}`);
      summary.measurementSources.push(null);
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
