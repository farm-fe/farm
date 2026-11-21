import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {validationDispatches} from './prepare-v2-release-intent.mjs';
import {waitForVersionPrHead} from './changesets-pr-head.mjs';
const read=file=>fs.readFileSync(new URL('../'+file,import.meta.url),'utf8');

test('ordinary PR CI tests current candidates without a publication reference',()=>{
  const ci=read('.github/workflows/ci.yaml');
  assert.match(ci,/  pull_request:/);
  assert.doesNotMatch(ci,/release_pr|release_base/);
  assert.doesNotMatch(ci,/release-pr-validation|release-candidate-seal:/);
  assert.doesNotMatch(ci,/profile: release|FARM_BUILD_PROFILE: release/);
  assert.match(ci,/uses: \.\/.github\/workflows\/v2-candidate-seal.yaml/);
  assert.equal(fs.existsSync(new URL('../.github/workflows/release-pr-validation.yaml',import.meta.url)),false);
  assert.equal(fs.existsSync(new URL('./release-pr-validation.mjs',import.meta.url)),false);
  const dispatches=validationDispatches(2330,'a'.repeat(40));
  assert.deepEqual(dispatches.map(row=>row.workflow),['ci.yaml','rust-test.yaml','lint.yaml','code-spell-check.yaml']);
});

test('fixed-main release builds and freezes bytes then verifies registry read-back without tests',()=>{
  const release=read('.github/workflows/incremental-release.yaml');
  assert.doesNotMatch(release,/  pull_request:|pr-validation-scope:|pr-release-reference:|release-pr-validation/);
  assert.match(release,/branches: \[main\]/);assert.match(release,/default: false/);
  assert.ok(release.includes("github.ref == 'refs/heads/main' && (github.event_name == 'push' || github.event_name == 'workflow_dispatch')"));
  assert.ok(release.includes("assert os.environ['GITHUB_WORKFLOW_SHA'] == os.environ['GITHUB_SHA']"));
  for(const stage of ['  producer:','  publisher:','  public-proof:','  finalize:'])assert.ok(release.includes(stage),stage);
  assert.doesNotMatch(release,/run-sdk-consumer|seal-v2-rust-packages|CARGO_REGISTRY_TOKEN/);
  assert.match(release,/seal-npm/);
  assert.doesNotMatch(release,/release-candidate-consumers|release-public-consumers|verify-v2-npm-consumer|verify-published-v2-consumers/);
  assert.match(release,/incremental-release-record.py registry/);
  assert.ok(read('.github/workflows/v2-candidate-seal.yaml').includes('scripts/verify-v2-npm-consumer.mjs'));
  assert.ok(release.includes("github.ref == 'refs/heads/main' && needs.source.outputs.version_ready == 'true'"));
  assert.ok(release.includes("github.event_name == 'push' || (github.event_name == 'workflow_dispatch' && inputs.publish == true)"));
  assert.ok(release.includes("needs.sign-frozen-release.result == 'success'"));
  assert.match(release,/node scripts\/automatic-release-source\.mjs/);
  assert.match(release,/group: farm-independent-release\n  cancel-in-progress: false/);
  assert.equal((read('.github/workflows/release-native-build.yaml').match(/profile: release/g)||[]).length,2);
});

test('Docs remains a CI quality lane using formal recipes, not a release reference',()=>{
  const docs=read('.github/workflows/farm-docs.yaml');
  assert.match(docs,/FARM_BUILD_PROFILE: ci/);assert.match(docs,/scripts\/docs-native-build.mjs/);assert.match(docs,/scripts\/docs-native-products.py consume/);
  assert.doesNotMatch(docs,/run build:rs\n|release-profile reference/);
  assert.match(docs,/windows-latest/);assert.match(docs,/ubuntu-24.04/);
});


const repo = 'farm-fe/farm', number = 2373;
const base = 'a'.repeat(40), previous = 'b'.repeat(40), expected = 'c'.repeat(40), other = 'd'.repeat(40);
const makePR = head => ({ number, state: 'open', title: 'chore: release packages',
  base: { ref: 'main', sha: base, repo: { full_name: repo } },
  head: { ref: 'changeset-release/main', sha: head, repo: { full_name: repo } } });

function fixture({ prs = [makePR(previous), makePR(expected)], remote = { base, head: expected } } = {}) {
  let reads = 0, refReads = 0, waits = 0;
  const input = { repository: repo, number, base, previous, expected, maxAttempts: 3,
    readPR: () => prs[Math.min(reads++, prs.length - 1)],
    readRefs: () => { refReads++; return remote; },
    wait: async () => { waits++; } };
  return { input, counts: () => [reads, refReads, waits] };
}

test('expected remote head with only the previous PR head visible can converge', async () => {
  const f = fixture();
  assert.equal((await waitForVersionPrHead(f.input)).head.sha, expected);
  assert.deepEqual(f.counts(), [2, 2, 1]);
});

test('exact published seal is accepted without waiting', async () => {
  const f = fixture({ prs: [makePR(expected)] });
  await waitForVersionPrHead(f.input);
  assert.deepEqual(f.counts(), [1, 1, 0]);
});

test('stale PR head visibility stops after the bounded number of attempts', async () => {
  const f = fixture({ prs: [makePR(previous)] });
  await assert.rejects(waitForVersionPrHead(f.input), /did not converge/);
  assert.deepEqual(f.counts(), [3, 3, 2]);
});

test('third head, moved remote branch, or moved main aborts without a retry', async () => {
  for (const options of [{ prs: [makePR(other)] }, { remote: { base, head: other } }, { remote: { base: other, head: expected } }]) {
    const f = fixture(options);
    await assert.rejects(waitForVersionPrHead(f.input), /moved concurrently/);
    assert.equal(f.counts()[2], 0);
  }
});

test('wrong or changed PR identity fails closed and does not retry API errors', async () => {
  for (const mutate of [
    pr => { pr.number++; }, pr => { pr.state = 'closed'; }, pr => { pr.title = 'another PR'; },
    pr => { pr.base.sha = other; }, pr => { pr.base.ref = 'stable'; }, pr => { pr.base.repo.full_name = 'fork/farm'; },
    pr => { pr.head.ref = 'unreviewed'; }, pr => { pr.head.repo.full_name = 'fork/farm'; },
  ]) {
    const pr = makePR(previous); mutate(pr);
    const f = fixture({ prs: [pr] });
    await assert.rejects(waitForVersionPrHead(f.input), /identity changed|moved concurrently/);
    assert.equal(f.counts()[2], 0);
  }
  const f = fixture();
  f.input.readPR = () => { throw new Error('REST unavailable'); };
  await assert.rejects(waitForVersionPrHead(f.input), /REST unavailable/);
  assert.equal(f.counts()[2], 0);
});

test('version preparation checks head visibility before receipt and again after upload before dispatch', () => {
  const source = read('scripts/prepare-v2-release-intent.mjs');
  assert.match(source, /const published = await confirmVersionPrHead[\s\S]*?writeVersionReceipt\(published\)/);
  const workflow = read('.github/workflows/release.yaml');
  assert.match(workflow, /Upload version-only provenance[\s\S]*?--check-dispatch\)[\s\S]*?gh workflow run/);
});
