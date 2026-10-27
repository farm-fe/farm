import test from 'node:test';
import assert from 'node:assert/strict';
import { validateSummary, verifyIdentity, renderComment, marker, readSummaryZip, report, findSummaryArtifacts } from './performance-pr-reporter.mjs';

const head = 'a'.repeat(40), base = 'b'.repeat(40);
const cell = { cacheMode: 'cold', samples: 5, validSamples: 5, medianMs: 100, iqrMs: 10 };
const fixture = () => ({ schemaVersion: 2, repositoryId: 1, runId: 2, runAttempt: 1, event: 'pull_request', headSHA: head, baseSHA: base, prNumber: 3, protocol: 'commerce-baseline-rust-build-v1', reports: [{ metadata: { os: 'linux', arch: 'x64', revision: 'c'.repeat(40), baseSHA: base, node: 'v22.4.0', pnpm: '9.4.0', fixtureSha256: 'a'.repeat(64), lockSha256: 'b'.repeat(64), runnerImage: 'ubuntu24' }, status: 'complete', diagnostics: [], cells: [cell], comparison: { mode: 'paired', baseCells: [cell], viteCells: [cell], farmSubsetCells: [cell], subsetProtocol: 'commerce-catalog-quote-subset-v1' }, history: [{ headSHA: head, runId: 2, medianMs: 100 }] }] });
const run = { id: 2, repository: { id: 1 }, event: 'pull_request', name: 'E2E Tests', status: 'completed', head_sha: head, head_repository: { id: 9 }, pull_requests: [{ number: 3, head: { sha: head } }] };
const pr = { number: 3, state: 'open', head: { sha: head, repo: { id: 9 } }, base: { sha: base, repo: { id: 1 } } };
function zip(text, name = 'summary.json') {
  const body = Buffer.from(text), filename = Buffer.from(name);
  const local = Buffer.alloc(30); local.writeUInt32LE(0x04034b50); local.writeUInt16LE(filename.length, 26);
  const directory = Buffer.alloc(46); directory.writeUInt32LE(0x02014b50); directory.writeUInt32LE(body.length, 20); directory.writeUInt32LE(body.length, 24); directory.writeUInt16LE(filename.length, 28);
  const end = Buffer.alloc(22); end.writeUInt32LE(0x06054b50); end.writeUInt16LE(1, 8); end.writeUInt16LE(1, 10); end.writeUInt32LE(local.length + filename.length + body.length, 16);
  return Buffer.concat([local, filename, body, directory, filename, end]);
}
test('strict valid envelope retains per-platform measurements and merge revision', () => {
  assert.deepEqual(validateSummary(JSON.stringify(fixture())), fixture());
  const body = renderComment(fixture(), 'farm-fe/farm');
  assert.equal(body.split(marker).length, 2);
  assert.match(body, /linux \/ x64/); assert.match(body, /100.0 ms/);
});
test('rejects unknown fields, unsafe text, excess payload, invalid numbers and duplicate platforms', () => {
  const cases = [v => { v.extra = 1; }, v => { v.reports[0].metadata.os = '<script>'; }, v => { v.reports[0].cells[0] = { ...cell, medianMs: -1 }; }, v => { v.reports.push(v.reports[0]); }, v => { v.reports[0].comparison.mode = 'trusted'; }];
  for (const mutate of cases) { const v = fixture(); mutate(v); assert.throws(() => validateSummary(JSON.stringify(v))); }
  assert.throws(() => validateSummary(' '.repeat(65537)));
});
test('fork identity works only with API-confirmed matching PR and SHA', () => {
  verifyIdentity(fixture(), run, pr, 1);
  for (const changed of [{ ...pr, head: { ...pr.head, sha: base } }, { ...pr, state: 'closed' }, { ...pr, number: 4 }, { ...pr, base: { ...pr.base, sha: head } }]) assert.throws(() => verifyIdentity(fixture(), run, changed, 1));
  assert.throws(() => verifyIdentity(fixture(), { ...run, pull_requests: [] }, pr, 1));
  assert.throws(() => verifyIdentity(fixture(), { ...run, repository: { id: 8 } }, pr, 1));
});
test('bounded ZIP parser rejects path traversal, extra entries and oversized archives', () => {
  assert.equal(readSummaryZip(zip('{}')), '{}');
  assert.throws(() => readSummaryZip(zip('{}', '../summary.json')));
  assert.throws(() => readSummaryZip(Buffer.alloc(131073)));
  const multiple = zip('{}'); multiple.writeUInt16LE(2, multiple.length - 12); assert.throws(() => readSummaryZip(multiple));
});
test('reporter updates the single bot comment and rechecks freshness immediately before write', async () => {
  const writes = []; let reads = 0;
  const fetcher = async (url, options = {}) => {
    if (options.method) { writes.push({ url, ...options }); return Response.json({ id: 4 }); }
    if (url.endsWith('/zip')) return new Response(zip(JSON.stringify(fixture())));
    if (url.includes('/artifacts?')) return Response.json({ artifacts: [{ name: 'performance-pr-summary-1', id: 5, size_in_bytes: 5000 }] });
    if (url.includes('/comments?')) return Response.json([{ id: 4, user: { login: 'github-actions[bot]', type: 'Bot' }, body: marker }]);
    if (url.includes('/pulls/')) { reads++; return Response.json(pr); }
    return Response.json(run);
  };
  await report({ token: 'test', repository: 'farm-fe/farm', event: { workflow_run: { id: 2 }, repository: { id: 1 } }, fetcher });
  assert.equal(reads, 2); assert.equal(writes.length, 1); assert.equal(writes[0].method, 'PATCH');
  reads = 0; writes.length = 0;
  const staleFetcher = async (url, options) => url.includes('/pulls/') && reads++ > 0 ? Response.json({ ...pr, head: { ...pr.head, sha: base } }) : fetcher(url, options);
  await assert.rejects(report({ token: 'test', repository: 'farm-fe/farm', event: { workflow_run: { id: 2 }, repository: { id: 1 } }, fetcher: staleFetcher }));
  assert.equal(writes.length, 0);
  const newerFetcher = async (url, options) => url.includes('/comments?') ? Response.json([{ id: 4, user: { login: 'github-actions[bot]', type: 'Bot' }, body: `${marker}\n<!-- farm-performance-run:99 -->` }]) : fetcher(url, options);
  const result = await report({ token: 'test', repository: 'farm-fe/farm', event: { workflow_run: { id: 2 }, repository: { id: 1 } }, fetcher: newerFetcher });
  assert.match(result.skipped, /newer/); assert.equal(writes.length, 0);
});


test('summary artifact after 246 native artifacts is found on page three', async () => {
  const pages = [];
  const found = await findSummaryArtifacts(async route => {
    const page = Number(new URL(`https://api.invalid${route}`).searchParams.get('page'));
    pages.push(page);
    return { artifacts: page < 3 ? Array.from({ length: 100 }, () => ({ name: 'native' })) : [{ name: 'performance-pr-summary-2', id: 8 }] };
  }, 2, 2);
  assert.deepEqual(pages, [1, 2, 3]); assert.equal(found[0].id, 8);
});
test('same run new attempt replaces old partial; old attempts cannot overwrite it', async () => {
  const value = fixture(); value.runAttempt = 2;
  const writes = [];
  const fetcher = async (url, options = {}) => {
    if (options.method) { writes.push(options); return Response.json({ id: 4 }); }
    if (url.endsWith('/zip')) return new Response(zip(JSON.stringify(value)));
    if (url.includes('/artifacts?')) return Response.json({ artifacts: [{ name: 'performance-pr-summary-2', id: 5, size_in_bytes: 5000 }] });
    if (url.includes('/comments?')) return Response.json([{ id: 4, user: { login: 'github-actions[bot]', type: 'Bot' }, body: `${marker}\n<!-- farm-performance-run:2:1 -->` }]);
    if (url.includes('/pulls/')) return Response.json(pr);
    return Response.json({ ...run, run_attempt: 2 });
  };
  await report({ token: 'test', repository: 'farm-fe/farm', event: { workflow_run: { id: 2 }, repository: { id: 1 } }, fetcher });
  assert.equal(writes.length, 1);
  assert.match(JSON.parse(writes[0].body).body, /farm-performance-run:2:2/);
  assert.throws(() => verifyIdentity(value, { ...run, run_attempt: 3 }, pr, 1), /attempt/);
});
