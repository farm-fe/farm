import test from 'node:test';
import assert from 'node:assert/strict';
import { discoverBaseCohort, loadComparableHistory, nativePlugins, readSafeZip } from './performance-cohorts.mjs';
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
