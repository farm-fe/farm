import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { validateSummary } from './performance-pr-reporter.mjs';

export function compactReport(report) {
  const metadata = Object.fromEntries(['os', 'arch', 'revision', 'baseSHA', 'node', 'pnpm', 'fixtureSha256', 'lockSha256', 'runnerImage'].map(key => [key, report.metadata?.[key]]));
  return { metadata, status: report.status, cells: report.cells,
    comparison: { mode: report.comparison?.mode ?? 'current-only', baseCells: report.comparison?.mode === 'paired' ? report.comparison.baseCells : [],
      farmSubsetCells: report.vite?.farmCells ?? [], viteCells: report.vite?.viteCells ?? [], subsetProtocol: 'commerce-catalog-quote-subset-v1' },
    history: (report.history?.entries ?? []).slice(0, 10).map(row => ({ headSHA: row.sourceHeadSHA, runId: row.runId,
      medianMs: row.cells.find(cell => cell.cacheMode === 'cold')?.medianMs })).filter(row => Number.isFinite(row.medianMs)) };
}
async function main() {
  const event = JSON.parse(await fs.readFile(process.env.GITHUB_EVENT_PATH, 'utf8'));
  const directory = path.resolve('.performance-reports');
  const summary = { schemaVersion: 1, repositoryId: event.repository.id, runId: Number(process.env.GITHUB_RUN_ID), event: 'pull_request',
    headSHA: event.pull_request.head.sha, baseSHA: event.pull_request.base.sha, prNumber: event.number,
    protocol: 'commerce-baseline-rust-build-v1', reports: [] };
  const failures = [];
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    if (!entry.isDirectory() || !new RegExp(`^performance-${process.env.GITHUB_SHA}-(linux-x64-gnu|darwin-arm64|win32-x64-msvc)$`).test(entry.name)) throw new Error('Unexpected report directory');
    try {
      const file = path.join(directory, entry.name, 'summary.json');
      if ((await fs.stat(file)).size > 2 * 1024 * 1024) throw new Error('Oversized raw summary');
      const raw = JSON.parse(await fs.readFile(file, 'utf8'));
      if (raw.metadata?.revision !== process.env.GITHUB_SHA) throw new Error('Producer SHA mismatch');
      const report = compactReport(raw);
      validateSummary(JSON.stringify({ ...summary, reports: [report] }));
      summary.reports.push(report);
    } catch (error) { failures.push(`${entry.name}: ${error.message}`); }
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
