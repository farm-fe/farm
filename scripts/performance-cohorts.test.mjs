import test from 'node:test';
import assert from 'node:assert/strict';
import { discoverBaseCohort, loadComparableHistory, nativePlugins, readSafeZip, githubClient, historyCompatibilityKeys } from './performance-cohorts.mjs';
const revision = 'a'.repeat(40), merge = 'b'.repeat(40);
const run = { id: 7, repository: { full_name: 'farm/farm' }, head_repository: { full_name: 'farm/farm' }, event: 'pull_request', conclusion: 'success', path: '.github/workflows/ci.yaml', head_sha: revision };
const artifacts = prefix => ['', ...nativePlugins.map(name => `-plugin-${name}`)].map((suffix, id) => ({ id: id + 1, name: `${prefix}-linux-x64-gnu${suffix}` }));
test('exact base PR head cohort is executable, fork runs are excluded', async () => {
  const api = async route => route.includes('/artifacts') ? { artifacts: artifacts(revision) } : { workflow_runs: [run] };
  assert.equal((await discoverBaseCohort({ repository: 'farm/farm', revision, abi: 'linux-x64-gnu', api })).mode, 'paired');
  const forkApi = async () => ({ workflow_runs: [{ ...run, head_repository: { full_name: 'fork/farm' } }] });
  assert.equal((await discoverBaseCohort({ repository: 'farm/farm', revision, abi: 'linux-x64-gnu', api: forkApi })).mode, 'current-only');
});
test('merge artifact prefix requires independently equal source trees', async () => {
  const makeApi = same => async route => route.includes('/git/commits/') ? { tree: { sha: route.endsWith(revision) || same ? 'tree' : 'other' } } : route.includes('/artifacts') ? { artifacts: artifacts(merge) } : { workflow_runs: [run] };
  assert.equal((await discoverBaseCohort({ repository: 'farm/farm', revision, abi: 'linux-x64-gnu', api: makeApi(false) })).mode, 'current-only');
  assert.equal((await discoverBaseCohort({ repository: 'farm/farm', revision, abi: 'linux-x64-gnu', api: makeApi(true) })).producerSHA, merge);
});
test('history distinguishes missing and expired without creating samples', async () => {
  const api = async route => route.includes('/artifacts') ? { artifacts: [{ name: `performance-${merge}-linux-x64-gnu`, expired: true }] } : { workflow_runs: [run] };
  const result = await loadComparableHistory({ repository: 'farm/farm', abi: 'linux-x64-gnu', metadata: {}, api });
  assert.deepEqual(result.entries, []); assert.equal(result.excluded[0].reason, 'expired');
});
test('ZIP parser rejects malformed and overlarge archives', () => {
  assert.throws(() => readSafeZip(Buffer.from('bad')));
  assert.throws(() => readSafeZip(Buffer.alloc(129 * 1024 * 1024)));
});
test('complete cohorts beyond the first 100 artifacts are found by bounded pagination', async () => {
  const api = async route => route.includes('/artifacts')
    ? route.includes('&page=2') ? { total_count: 123, artifacts: artifacts(revision) }
      : { total_count: 123, artifacts: Array.from({ length: 100 }, (_, id) => ({ id, name: `unrelated-${id}` })) }
    : { workflow_runs: [run] };
  assert.equal((await discoverBaseCohort({ repository: 'farm/farm', revision, abi: 'linux-x64-gnu', api })).mode, 'paired');
});
test('PR-only CI for a main merge resolves the associated PR and still proves producer tree', async () => {
  const api = async route => {
    if (route.includes('/commits/') && route.includes('/pulls?')) return [{ merged_at: '2026-10-27', merge_commit_sha: revision, head: { sha: merge, repo: { full_name: 'farm/farm' } } }];
    if (route.includes('/git/commits/')) return { tree: { sha: 'equal-tree' } };
    if (route.includes('/artifacts')) return { artifacts: artifacts(merge) };
    return { workflow_runs: route.includes(`head_sha=${revision}`) ? [] : [{ ...run, head_sha: merge }] };
  };
  const cohort = await discoverBaseCohort({ repository: 'farm/farm', revision, abi: 'linux-x64-gnu', api });
  assert.equal(cohort.mode, 'paired');
  assert.equal(cohort.revision, revision);
  assert.equal(cohort.producerSHA, merge);
});

function historyZip(text) {
  const body = Buffer.from(text), filename = Buffer.from('summary.json');
  const local = Buffer.alloc(30); local.writeUInt32LE(0x04034b50); local.writeUInt16LE(filename.length, 26);
  const directory = Buffer.alloc(46); directory.writeUInt32LE(0x02014b50); directory.writeUInt32LE(body.length, 20); directory.writeUInt32LE(body.length, 24); directory.writeUInt16LE(filename.length, 28);
  const end = Buffer.alloc(22); end.writeUInt32LE(0x06054b50); end.writeUInt16LE(1, 8); end.writeUInt16LE(1, 10); end.writeUInt32LE(local.length + filename.length + body.length, 16);
  return Buffer.concat([local, filename, body, directory, filename, end]);
}

test('invalid acceptance and incomplete metrics never enter compatible history', async () => {
  const metadata = Object.fromEntries(historyCompatibilityKeys.map(key => [key, 'same-fixture-cohort']));
  const cells = ['cold', 'warm'].map(cacheMode => ({ cacheMode, samples: 3, validSamples: 3, medianMs: 100, iqrMs: 0 }));
  for (const status of ['invalid', 'partial', 'complete']) {
    const report = { status, metadata: { ...metadata, revision }, cells: status === 'complete' ? [{ ...cells[0], validSamples: 0, medianMs: null, iqrMs: null }, cells[1]] : cells };
    const api = async route => route.endsWith('/zip') ? historyZip(JSON.stringify(report)) : route.includes('/artifacts') ? { artifacts: [{ id: 1, name: `performance-${revision}-linux-x64-gnu` }] } : { workflow_runs: [run] };
    const result = await loadComparableHistory({ repository: 'farm/farm', abi: 'linux-x64-gnu', metadata, api, currentRunId: 8 });
    assert.deepEqual(result.entries, []); assert.equal(result.excluded[0].reason, 'invalid');
  }
});

test('metadata history requests share the full supervised deadline rather than the retired 18-minute cap', async () => {
  const old = process.env.FARM_PERFORMANCE_STARTED_AT;
  try {
    process.env.FARM_PERFORMANCE_STARTED_AT = String(Date.now() - 20 * 60 * 1000);
    const client = githubClient({ token: 'fixture-only', fetchImpl: async () => Response.json({ bounded: true }) });
    assert.deepEqual(await client('/fixture'), { bounded: true });
  } finally { if (old === undefined) delete process.env.FARM_PERFORMANCE_STARTED_AT; else process.env.FARM_PERFORMANCE_STARTED_AT = old; }
});
