// A current-checkout test inventory, never publication or version approval.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import { root, npmInventory, cargoInventory, releaseCandidatePlanPath, validationEvent } from './v2-release-inventory.mjs';
import { assertCandidateCheckout } from './pack-v2-npm-candidates.mjs';

export function prCandidatePlan({ base, npm, cargo, checkout }) {
  assertCandidateCheckout(checkout);
  assert.match(checkout.testedTree, /^[a-f0-9]{40}$/);
  return { schemaVersion: 1, status: 'pr-test-only-not-publication-approval', abi: base.abi,
    preparation: { mode: 'pr-validation' }, validation: { sourceCommit: checkout.sourceCommit,
      testedCommit: checkout.testedCommit, testedTree: checkout.testedTree },
    npm, cargo, counts: { primary: npm.primary.length, payloads: npm.payloads.length, total: npm.primary.length + npm.payloads.length } };
}
export function assertPrCandidateSource(plan, receipt, { commit, tree, npm }) {
  assert.equal(plan.preparation?.mode, 'pr-validation');
  assert.equal(plan.status, 'pr-test-only-not-publication-approval');
  assert.equal(plan.publication, undefined, 'PR test plan cannot select a publication delta');
  assert.deepEqual(plan.npm, npm, 'PR candidate npm inputs differ from actual checkout');
  assert.equal(plan.validation?.testedCommit, commit); assert.equal(plan.validation?.testedTree, tree);
  for (const key of ['sourceCommit', 'testedCommit', 'testedTree']) assert.equal(receipt[key], plan.validation[key]);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.ok(process.env.FARM_PR_CANDIDATE_PLAN, 'Explicit PR-only plan destination required');
  const output = releaseCandidatePlanPath(root);
  const event = validationEvent();
  const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
  const testedCommit = git('rev-parse', 'HEAD');
  const metadata = JSON.parse(execFileSync('cargo', ['+nightly-2026-04-10', 'metadata', '--locked', '--offline', '--no-deps', '--format-version', '1'], { cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 ** 2 }));
  const plan = prCandidatePlan({ base: JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json'))),
    npm: npmInventory(), cargo: cargoInventory(metadata), checkout: { testedCommit, workflowSHA: process.env.GITHUB_SHA,
      sourceCommit: event.pull_request.head.sha, testedTree: git('rev-parse', 'HEAD^{tree}'), eventName: process.env.GITHUB_EVENT_NAME,
      parents: git('rev-list', '--parents', '-n', '1', testedCommit).split(' ').slice(1) } });
  fs.writeFileSync(output, JSON.stringify(plan, null, 2) + '\n', { flag: 'wx' });
  console.log(JSON.stringify({ mode: plan.preparation.mode, counts: plan.counts, cargo: plan.cargo.length, publicationApproved: false }));
}
