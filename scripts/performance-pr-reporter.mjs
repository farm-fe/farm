import {observeAttempts,actualJob} from './ci-attempt-producers.mjs';
import { qualifiedPerformanceCells, completePerformanceCells } from './performance-metrics.mjs';
import fs from 'node:fs/promises';
import { inflateRawSync } from 'node:zlib';
import { pathToFileURL } from 'node:url';

export const marker = '<!-- farm-performance-report:v1 -->';
const sha = value => typeof value === 'string' && /^[a-f0-9]{40}$/.test(value);
const integer = value => Number.isSafeInteger(value) && value > 0;
const exact = (value, keys) => value && typeof value === 'object' && !Array.isArray(value) && Object.keys(value).sort().join() === [...keys].sort().join();
const assert = (condition, message) => { if (!condition) throw new Error(message); };
function cells(value) {
  assert(Array.isArray(value) && value.length <= 2, 'Invalid cells');
  const seen = new Set();
  for (const cell of value) {
    assert(exact(cell, ['cacheMode', 'samples', 'validSamples', 'medianMs', 'iqrMs']), 'Invalid cell schema');
    assert(['cold', 'warm'].includes(cell.cacheMode) && !seen.has(cell.cacheMode), 'Invalid cache mode');
    seen.add(cell.cacheMode);
    assert(Number.isInteger(cell.samples) && cell.samples >= 0 && cell.samples <= 100 && Number.isInteger(cell.validSamples) && cell.validSamples >= 0 && cell.validSamples <= cell.samples, 'Invalid sample counts');
    for (const key of ['medianMs', 'iqrMs']) assert(cell[key] === null || (Number.isFinite(cell[key]) && cell[key] >= 0 && cell[key] <= 86400000), 'Invalid timing');
    assert(cell.validSamples ? cell.medianMs !== null && cell.iqrMs !== null : cell.medianMs === null && cell.iqrMs === null, 'Inconsistent timing');
  }
}
export function validateSummary(bytes) {
  assert(Buffer.byteLength(bytes) <= 65536, 'Summary exceeds 64 KiB');
  const value = JSON.parse(bytes);
  const sourceKeys = value.schemaVersion === 3 ? ['measurementSources'] : [];
  assert(exact(value, [...sourceKeys, 'schemaVersion', 'repositoryId', 'runId', 'runAttempt', 'event', 'headSHA', 'baseSHA', 'prNumber', 'protocol', 'reports']), 'Invalid summary schema');
  assert([2,3].includes(value.schemaVersion) && integer(value.runAttempt) && value.protocol === 'commerce-business-2048-farm-supervised-v2' && (value.schemaVersion === 3 ? ['pull_request','push','workflow_dispatch'] : ['pull_request','push']).includes(value.event), 'Invalid protocol');
  assert(integer(value.repositoryId) && integer(value.runId) && (['pull_request','workflow_dispatch'].includes(value.event) ? integer(value.prNumber) : value.prNumber===null) && sha(value.headSHA) && sha(value.baseSHA), 'Invalid identity');
  assert(Array.isArray(value.reports) && value.reports.length > 0 && value.reports.length <= 3, 'Invalid reports');
  const systems = new Set();
  for (const report of value.reports) {
  assert(exact(report, ['metadata', 'status', 'cells', 'comparison', 'history', 'diagnostics']), 'Invalid report schema');
  assert(Array.isArray(report.diagnostics) && report.diagnostics.length <= 30 && report.diagnostics.every(item => typeof item === 'string' && item.length <= 500 && !/[\x00-\x1f]/.test(item)), 'Invalid diagnostics');
  const metadata = report.metadata;
  assert(exact(metadata, ['os', 'arch', 'revision', 'baseSHA', 'node', 'pnpm', 'fixtureSha256', 'lockSha256', 'runnerImage']), 'Invalid metadata schema');
  assert(['linux', 'darwin', 'win32'].includes(metadata.os) && ['x64', 'arm64'].includes(metadata.arch) && !systems.has(metadata.os), 'Invalid platform');
  systems.add(metadata.os);
  assert(sha(metadata.revision) && metadata.baseSHA === value.baseSHA, 'Invalid report revision');
  for (const key of ['node', 'pnpm', 'runnerImage']) assert((report.status === 'invalid' && metadata[key] === null) || (typeof metadata[key] === 'string' && /^[A-Za-z0-9._+() -]{1,100}$/.test(metadata[key])), 'Invalid runtime metadata');
  for (const key of ['fixtureSha256', 'lockSha256']) assert((report.status === 'invalid' && metadata[key] === null) || (typeof metadata[key] === 'string' && /^[a-f0-9]{64}$/.test(metadata[key])), 'Invalid fingerprint');
  validateReport(report);
  }
  if (value.schemaVersion === 3) {
    assert(Array.isArray(value.measurementSources) && value.measurementSources.length === value.reports.length, 'Missing per-platform measurement sources');
    for (const [index, source] of value.measurementSources.entries()) {
      const report = value.reports[index];
      if (source === null) { assert(report.status === 'invalid', 'Complete report lacks producer proof'); continue; }
      assert(exact(source, [...('nativeKind' in source ? ['nativeKind'] : []), 'abi','runId','runAttempt','jobId','artifactId','artifactDigest','rawSummarySha256','producerSHA','testedTree','profile','workflowPath','nativeAttempt','nativeJobId','nativeArtifactId','nativeArtifactDigest','harnessSha256']), 'Invalid measurement source schema');
      assert(!('nativeKind' in source) || ['build','materialization'].includes(source.nativeKind), 'Invalid native source kind');
      const abi = {linux:'linux-x64-gnu',darwin:'darwin-arm64',win32:'win32-x64-msvc'}[report.metadata.os];
      assert(source.abi === abi && source.runId === value.runId && source.profile === 'ci' && source.workflowPath === '.github/workflows/ci.yaml', 'Measurement chain differs');
      for (const key of ['runAttempt','nativeAttempt']) assert(integer(source[key]) && source[key] <= value.runAttempt, 'Future measurement attempt');
      for (const key of ['jobId','artifactId','nativeJobId','nativeArtifactId']) assert(integer(source[key]), 'Invalid measurement producer ID');
      for (const key of ['artifactDigest','nativeArtifactDigest']) assert(/^sha256:[a-f0-9]{64}$/.test(source[key]), 'Invalid measurement archive digest');
      for (const key of ['rawSummarySha256','harnessSha256']) assert(/^[a-f0-9]{64}$/.test(source[key]), 'Invalid measurement input hash');
      assert(sha(source.testedTree) && source.producerSHA === report.metadata.revision, 'Measurement tree or producer differs');
    }
  }
  return value;
}
function validateReport(value) {
  assert(['complete', 'partial', 'invalid'].includes(value.status), 'Invalid status');
  cells(value.cells);
  assert(exact(value.comparison, ['mode', 'baseCells']) && ['paired', 'current-only'].includes(value.comparison.mode), 'Invalid comparison');
  cells(value.comparison.baseCells);
  if (value.comparison.mode === 'current-only') assert(value.comparison.baseCells.length === 0, 'Current-only report must not contain base samples');
  if (value.status === 'complete') {
    assert(completePerformanceCells(value.cells), 'Complete report requires three valid cold/warm head samples');
    if (value.comparison.mode === 'paired') assert(completePerformanceCells(value.comparison.baseCells), 'Complete paired report requires three valid cold/warm base samples');
  }
  if (value.status !== 'complete') assert([...value.cells, ...value.comparison.baseCells].every(cell => cell.validSamples === 0 && cell.medianMs === null && cell.iqrMs === null), 'Unqualified report must not expose valid timing metrics');
  assert(Array.isArray(value.history) && value.history.length <= 20, 'Invalid history');
  if (value.status !== 'complete') assert(value.history.length === 0, 'Unqualified report must not expose comparable history');
  for (const row of value.history) assert(exact(row, ['headSHA', 'runId', 'medianMs']) && sha(row.headSHA) && integer(row.runId) && Number.isFinite(row.medianMs) && row.medianMs > 0 && row.medianMs <= 86400000, 'Invalid history row');
}

// Parse one bounded ZIP entry without extracting paths or executing artifact content.
export function readSummaryZip(zip) {
  assert(zip.length <= 131072, 'Artifact exceeds 128 KiB');
  let end = zip.length - 22;
  while (end >= Math.max(0, zip.length - 65557) && zip.readUInt32LE(end) !== 0x06054b50) end--;
  assert(end >= 0 && zip.readUInt16LE(end + 10) === 1 && zip.readUInt16LE(end + 8) === 1, 'Expected one ZIP entry');
  const pos = zip.readUInt32LE(end + 16);
  assert(pos + 46 <= end && zip.readUInt32LE(pos) === 0x02014b50, 'Invalid ZIP directory');
  const method = zip.readUInt16LE(pos + 10), size = zip.readUInt32LE(pos + 20), outputSize = zip.readUInt32LE(pos + 24);
  const nameLength = zip.readUInt16LE(pos + 28), local = zip.readUInt32LE(pos + 42);
  assert(zip.subarray(pos + 46, pos + 46 + nameLength).toString() === 'summary.json' && outputSize <= 65536 && size <= 131072, 'Invalid artifact entry');
  assert(local + 30 <= pos && zip.readUInt32LE(local) === 0x04034b50 && !(zip.readUInt16LE(local + 6) & 1), 'Invalid ZIP entry');
  const start = local + 30 + zip.readUInt16LE(local + 26) + zip.readUInt16LE(local + 28);
  assert(start + size <= pos && [0, 8].includes(method), 'Invalid ZIP compression');
  const data = zip.subarray(start, start + size);
  const result = method === 8 ? inflateRawSync(data, { maxOutputLength: 65536 }) : data;
  assert(result.length === outputSize, 'Invalid ZIP size');
  return result.toString('utf8');
}

export function verifyIdentity(summary, run, pr, repositoryId) {
  assert((run.run_attempt ?? 1) === summary.runAttempt, 'Run attempt mismatch');
  assert(run.repository.id === repositoryId && summary.repositoryId === repositoryId && run.id === summary.runId && run.event === 'pull_request' && run.name === 'E2E Tests' && run.status === 'completed', 'Run identity mismatch');
  assert(run.head_sha === summary.headSHA && pr.number === summary.prNumber && pr.state === 'open' && pr.head.sha === summary.headSHA && pr.base.sha === summary.baseSHA && pr.base.repo.id === repositoryId, 'Stale or mismatched PR');
  assert(run.head_repository.id === pr.head.repo.id && run.pull_requests.some(item => item.number === pr.number && item.head.sha === summary.headSHA), 'Run does not belong to PR');
}
export function renderComment(summary, repository) {
  if (summary.reports) return `${marker}\n<!-- farm-performance-run:${summary.runId}:${summary.runAttempt} -->\n## Performance report\n\n[Run ${summary.runId}, attempt ${summary.runAttempt}](https://github.com/${repository}/actions/runs/${summary.runId}) - Head: \`${summary.headSHA}\`\n\nPlatforms are reported separately. Aggregation attempt does not relabel earlier measurements.\n\n${summary.reports.map((report,index) => `### ${report.metadata.os} / ${report.metadata.arch}\n\n${summary.measurementSources?.[index] ? `Measurement attempt ${summary.measurementSources[index].runAttempt}; job ${summary.measurementSources[index].jobId}; artifact ${summary.measurementSources[index].artifactId}.\n\n` : ''}${renderComment({ ...summary, ...report, reports: null }, repository).split('## Performance report\n')[1]}`).join('\n\n')}`;
  const accepted = summary.status === 'complete' && completePerformanceCells(summary.cells)
    && (summary.comparison.mode !== 'paired' || completePerformanceCells(summary.comparison.baseCells));
  const timing = cell => cell && cell.medianMs !== null ? `${cell.medianMs.toFixed(1)} ms (IQR ${cell.iqrMs.toFixed(1)}; ${cell.validSamples}/${cell.samples})` : 'Unavailable';
  const history = accepted
    ? (summary.history ?? []).filter(row => sha(row.headSHA) && integer(row.runId) && Number.isFinite(row.medianMs) && row.medianMs > 0 && row.medianMs <= 86400000) : [];
  const diagnostics = (summary.diagnostics ?? []).map(reason => `- ${reason.replace(/[&<>`*_\[\]\\@]/g, character => `&#${character.charCodeAt(0)};`)}`).join('\n');
  const rows = ['cold', 'warm'].map(mode => `| ${mode} | ${timing(qualifiedPerformanceCells(accepted ? 'complete' : 'invalid', summary.cells).find(c => c.cacheMode === mode))} | ${timing(qualifiedPerformanceCells(accepted ? 'complete' : 'invalid', summary.comparison.baseCells).find(c => c.cacheMode === mode))} |`);

  return `${marker}\n## Performance report\n\nStatus: **${summary.status}**; comparison: **${summary.comparison.mode}**\n\n| Cache | Farm head | Farm base |\n|---|---|---|\n${rows.join('\n')}\n\n${diagnostics}\n\n[Run ${summary.runId}](https://github.com/${repository}/actions/runs/${summary.runId}); head: \`${summary.headSHA}\`\n\nMeasurements are untrusted CI data, informational only; three samples are not formal certification.\n\n### Recent comparable cold builds\n${history.length ? history.map(row => `- \`${row.headSHA.slice(0, 12)}\`: ${row.medianMs.toFixed(1)} ms ([run](https://github.com/${repository}/actions/runs/${row.runId}))`).join('\n') : 'No compatible history available.'}`;
}

export async function report({ token, repository, event, api = 'https://api.github.com', fetcher = fetch }) {
  assert(/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository), 'Invalid repository');
  const request = async (route, options = {}) => {
    const response = await fetcher(`${api}/repos/${repository}${route}`, { ...options, headers: { Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28', ...options.headers } });
    assert(response.ok, `GitHub request failed: ${response.status}`); return response.json();
  };
  const runId = event.workflow_run.id;
  assert(integer(runId), 'Invalid triggering run');
  const run = await request(`/actions/runs/${runId}`);
  assert(run.repository.id === event.repository.id && run.name === 'E2E Tests' && run.status === 'completed', 'Invalid triggering run');
  if(run.event!=='pull_request')return {skipped:'Only PR runs may update a PR comment'};
  let aggregationAttempt = run.run_attempt ?? 1;
  if (aggregationAttempt > 1) aggregationAttempt = actualJob(await observeAttempts(request, '', run), 'Aggregate informational performance report').attempt;
  const matches = await findSummaryArtifacts(request, runId, aggregationAttempt);
  assert(matches.length === 1 && matches[0].size_in_bytes <= 131072, 'Missing or ambiguous summary artifact');
  const response = await fetcher(`${api}/repos/${repository}/actions/artifacts/${matches[0].id}/zip`, { headers: { Authorization: `Bearer ${token}` } });
  assert(response.ok, 'Artifact download failed');
  const reader = response.body.getReader(); const chunks = []; let length = 0;
  for (;;) { const { done, value } = await reader.read(); if (done) break; length += value.length; if (length > 131072) { await reader.cancel(); throw new Error('Artifact too large'); } chunks.push(Buffer.from(value)); }
  const summary = validateSummary(readSummaryZip(Buffer.concat(chunks)));
  const pr = await request(`/pulls/${summary.prNumber}`); verifyIdentity(summary, {...run,run_attempt:aggregationAttempt}, pr, event.repository.id);
  const comments = [];
  for (let page = 1; page <= 100; page++) {
    const batch = await request(`/issues/${pr.number}/comments?per_page=100&page=${page}`); comments.push(...batch);
    if (batch.length < 100) break;
    assert(page < 100, 'Too many comments');
  }
  const existing = comments.filter(c => c.user.login === 'github-actions[bot]' && c.user.type === 'Bot' && c.body.startsWith(marker));
  assert(existing.length <= 1, 'Multiple reporter comments require manual reconciliation');
  const previousRun = existing.length && /<!-- farm-performance-run:(\d+)(?::(\d+))? -->/.exec(existing[0].body);
  if (previousRun && (Number(previousRun[1]) > summary.runId || (Number(previousRun[1]) === summary.runId && Number(previousRun[2] ?? 1) >= summary.runAttempt))) return { skipped: 'An equal or newer run already reported' };
  // Recheck immediately before writing; workflow concurrency serializes reporter runs.
  const freshRun = await request(`/actions/runs/${runId}`);
  assert((freshRun.run_attempt ?? 1) === (run.run_attempt ?? 1), 'Run attempt moved before comment update');
  verifyIdentity(summary, {...freshRun,run_attempt:aggregationAttempt}, await request(`/pulls/${pr.number}`), event.repository.id);
  const body = JSON.stringify({ body: renderComment(summary, repository) });
  return request(existing.length ? `/issues/comments/${existing[0].id}` : `/issues/${pr.number}/comments`, { method: existing.length ? 'PATCH' : 'POST', headers: { 'Content-Type': 'application/json' }, body });
}
export async function findSummaryArtifacts(request, runId, attempt) {
  const matches = [];
  // Bound traversal, but do not assume the summary precedes 246 native files.
  for (let page = 1; page <= 20; page++) {
    const batch = await request(`/actions/runs/${runId}/artifacts?per_page=100&page=${page}`);
    assert(Array.isArray(batch.artifacts), 'Invalid artifact list');
    matches.push(...batch.artifacts.filter(item => item.name === `performance-pr-summary-${attempt}` && !item.expired));
    if (batch.artifacts.length < 100) return matches;
  }
  throw new Error('Artifact pagination exceeds limit');
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const event = JSON.parse(await fs.readFile(process.env.GITHUB_EVENT_PATH, 'utf8'));
  await report({ token: process.env.GITHUB_TOKEN, repository: process.env.GITHUB_REPOSITORY, event });
}
