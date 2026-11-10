import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execFileSync } from 'node:child_process';
import { prCandidatePlan, assertPrCandidateSource } from './prepare-pr-candidate-plan.mjs';
import { releaseCandidatePlanPath } from './v2-release-inventory.mjs';
import { assertSealPreflight } from './compose-v2-release-seal.mjs';
import { rustRegistryReuse } from './v2-rust-registry-reuse.mjs';

const head = 'a'.repeat(40), merge = 'b'.repeat(40), tree = 'c'.repeat(40), base = 'd'.repeat(40);
function fixture() {
  const npm = { primary: [{ name: 'core', version: '2.0.0', sourceSha256: 'new-source' }], payloads: [] };
  const cargo = [{ name: 'farmfe_core', version: '2.2.0', sourceSha256: 'current', packageInputsSha256: 'complete-inputs' }];
  const checkout = { testedCommit: merge, workflowSHA: merge, sourceCommit: head, testedTree: tree, eventName: 'pull_request', parents: [base, head] };
  const plan = prCandidatePlan({ base: { abi: '0.5.4', npm: { primary: [] }, publication: { old: true } }, npm, cargo, checkout });
  return { plan, checkout, receipt: plan.validation, current: { commit: merge, tree, npm } };
}
test('PR test inventory reflects current source, retains versions, and does not inherit release selection', () => {
  const { plan, receipt, current } = fixture();
  assertPrCandidateSource(plan, receipt, current);
  assert.equal(plan.npm.primary[0].version, '2.0.0'); assert.equal(plan.cargo[0].version, '2.2.0');
  assert.equal(plan.publication, undefined); assert.equal(plan.preparation.mode, 'pr-validation');
  assert.deepEqual(rustRegistryReuse(plan), []);
  // Pending Changesets and PR-head/main-side tree differences are not release prerequisites.
  assert.notEqual(plan.validation.sourceCommit, current.commit);
  const result = { candidateArchivesVerified: true, externalSdkVerified: true, prCandidateVerified: true,
    readyToPublish: false, stableVersionStateReady: false };
  assertSealPreflight(plan, result);
  assert.throws(() => assertSealPreflight({ preparation: { mode: 'changesets' } }, result));
  for (const field of ['candidateArchivesVerified', 'externalSdkVerified', 'prCandidateVerified'])
    assert.throws(() => assertSealPreflight(plan, { ...result, [field]: false }));
  assert.throws(() => assertSealPreflight(plan, { ...result, readyToPublish: true }));
});
test('PR source receipts reject wrong checkout, tree, inventory and publication deltas', () => {
  const { plan, receipt, current } = fixture();
  for (const changed of [{ ...current, commit: head }, { ...current, tree: base }, { ...current, npm: { primary: [], payloads: [] } }])
    assert.throws(() => assertPrCandidateSource(plan, receipt, changed));
  assert.throws(() => assertPrCandidateSource({ ...plan, publication: {} }, receipt, current));
  assert.throws(() => assertPrCandidateSource(plan, { ...receipt, sourceCommit: base }, current));
});
test('PR plan override cannot enter main publication scope or escape its fixed output path', () => {
  const root = '/fixture';
  const env = { FARM_PR_CANDIDATE_PLAN: '.release-work/pr-candidate-plan.json', GITHUB_ACTIONS: 'true', GITHUB_JOB: 'seal',
    GITHUB_EVENT_NAME: 'pull_request', GITHUB_REPOSITORY: 'farm-fe/farm', GITHUB_WORKFLOW_REF: 'farm-fe/farm/.github/workflows/ci.yaml@refs/pull/1/merge' };
  assert.equal(releaseCandidatePlanPath(root, env), path.resolve(root, env.FARM_PR_CANDIDATE_PLAN));
  assert.equal(releaseCandidatePlanPath(root, { ...env, GITHUB_EVENT_NAME: 'workflow_dispatch', GITHUB_REF: 'refs/heads/main' }), path.resolve(root, env.FARM_PR_CANDIDATE_PLAN));
  for (const changed of [{ GITHUB_EVENT_NAME: 'push' }, { GITHUB_JOB: 'producer' }, { FARM_RELEASE_CANDIDATE_PLAN: 'release-work/candidate-plan.json' },
    { FARM_PR_CANDIDATE_PLAN: '../outside.json' }, { GITHUB_WORKFLOW_REF: 'farm-fe/farm/.github/workflows/incremental-release.yaml@refs/heads/main' }])
    assert.throws(() => releaseCandidatePlanPath(root, { ...env, ...changed }));
  const preflight = fs.readFileSync(new URL('./v2-release-preflight.mjs', import.meta.url), 'utf8');
  assert.match(preflight, /!prCandidate \|\| !process\.argv\.includes\('--publication'\)/);
  assert.match(preflight, /pendingChangesets === 0/);
  assert.match(preflight, /prCandidate \? receipt\.testedCommit : receipt\.sourceCommit/);
});

test('real Git merge with pending Changesets and changed main product tree is a valid PR candidate', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-pr-candidate-merge-'));
  const git = (...args) => execFileSync('git', args, { cwd: directory, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  try {
    git('init', '-b', 'main'); git('config', 'user.name', 'Fixture'); git('config', 'user.email', 'fixture@example.invalid');
    fs.mkdirSync(path.join(directory, 'packages/core'), { recursive: true });
    fs.writeFileSync(path.join(directory, 'packages/core/source.js'), 'export const main = 1;');
    git('add', '.'); git('commit', '-m', 'fixture base');
    git('checkout', '-b', 'pr');
    fs.mkdirSync(path.join(directory, '.changeset'));
    fs.writeFileSync(path.join(directory, '.changeset/pending.md'), '---\n"core": patch\n---\nActual PR change.\n');
    fs.writeFileSync(path.join(directory, 'packages/core/pr.js'), 'export const pr = 1;');
    git('add', '.'); git('commit', '-m', 'fixture PR'); const head = git('rev-parse', 'HEAD');
    git('checkout', 'main');
    fs.writeFileSync(path.join(directory, 'packages/core/source.js'), 'export const main = 2;');
    git('add', '.'); git('commit', '-m', 'fixture main');
    git('merge', '--no-ff', 'pr', '-m', 'fixture tested merge');
    const commit = git('rev-parse', 'HEAD'), tree = git('rev-parse', 'HEAD^{tree}');
    assert.notEqual(git('rev-parse', `${head}^{tree}`), tree);
    assert.throws(() => git('diff', '--quiet', head, 'HEAD', '--', 'packages'));
    const { plan: stale } = fixture();
    const npm = { primary: [{ name: 'core', version: '2.0.0', sourceSha256: 'actual-merge-inputs' }], payloads: [] };
    const plan = prCandidatePlan({ base: stale, npm, cargo: stale.cargo, checkout: { sourceCommit: head, testedCommit: commit,
      testedTree: tree, workflowSHA: commit, eventName: 'pull_request', parents: git('rev-list', '--parents', '-n', '1', commit).split(' ').slice(1) } });
    assertPrCandidateSource(plan, plan.validation, { commit, tree, npm });
    assert.ok(fs.existsSync(path.join(directory, '.changeset/pending.md')));
    assert.equal(git('status', '--porcelain'), '');
    assert.equal(stale.npm.primary[0].sourceSha256, 'new-source', 'release inventory was not rewritten');
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});
