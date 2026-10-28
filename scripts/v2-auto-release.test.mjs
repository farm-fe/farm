import test from 'node:test';
import assert from 'node:assert/strict';
import { digest, validateTrigger, validateCandidateRun, validateArchiveProvenance, immutablePublish, orchestrate, validateFinalRelease, selectRequiredRuns, requiredWorkflows, requiredCiJobs, validateRequiredCiJobs, confirmRegistryChecksum, verifyRegistryArchive } from './v2-auto-release.mjs';
const intent = { schemaVersion: 1, pullRequest: 2329, baseBranch: 'main', sourceCommit: 'a'.repeat(40), tag: 'v2.0.0', requiredWorkflows, requiredCiJobs, performanceRequired: false, abi: '0.5.4', rustCoreVersion: '2.2.0' };
const bytes = JSON.stringify(intent);
const event = { action: 'closed', number: 2329, pull_request: { merged: true, base: { ref: 'main' }, head: { sha: 'a'.repeat(40) }, merge_commit_sha: 'b'.repeat(40) } };

test('every functional lane including integrated Docs is required despite informational performance', () => {
  const jobs = requiredCiJobs.map(name => ({ name, status: 'completed', conclusion: 'success' }));
  validateRequiredCiJobs([...jobs, { name: 'Commerce Performance', status: 'completed', conclusion: 'failure' }]);
  for (const name of requiredCiJobs) {
    assert.throws(() => validateRequiredCiJobs(jobs.filter(job => job.name !== name)), /required CI job/);
    assert.throws(() => validateRequiredCiJobs(jobs.map(job => job.name === name ? { ...job, conclusion: 'skipped' } : job)), /not successful/);
  }
});
test('missing or stale independent workflow runs do not become a green final release', () => {
  const run = { id: 1, head_sha: event.pull_request.head.sha, path: '.github/workflows/rust-test.yaml', event: 'pull_request', pull_requests: [{ number: 2329, head: { sha: event.pull_request.head.sha } }] };
  assert.equal(selectRequiredRuns([run], [run.path], run.head_sha, 2329)[0].id, 1);
  assert.throws(() => selectRequiredRuns([run], [run.path, '.github/workflows/ci.yaml'], run.head_sha, 2329), /absent/);
  assert.throws(() => selectRequiredRuns([run], [run.path], 'c'.repeat(40), 2329), /absent/);
  assert.equal(selectRequiredRuns([run, { ...run, id: 2, conclusion: 'failure' }], [run.path], run.head_sha, 2329)[0].id, 2);
});
test('head, synthetic test merge and final merge are distinct identities with proven equal trees', () => {
  const testedCommit = 'c'.repeat(40);
  const receipt = { sourceCommit: event.pull_request.head.sha, testedCommit, testedTree: 'tree' };
  validateArchiveProvenance(receipt, event, testedCommit, 'tree', 'tree', 'tree');
  assert.throws(() => validateArchiveProvenance({ ...receipt, sourceCommit: testedCommit }, event, testedCommit, 'tree', 'tree', 'tree'));
  assert.throws(() => validateArchiveProvenance({ ...receipt, testedCommit: event.pull_request.head.sha }, event, testedCommit, 'tree', 'tree', 'tree'));
  assert.throws(() => validateArchiveProvenance(receipt, event, testedCommit, 'tree', 'tree', 'different'), /Final merged tree/);
});
test('only exact reviewed release merge is accepted', () => {
  validateTrigger(event, intent, bytes, digest(bytes));
  for (const patch of [{ action: 'opened' }, { number: 2330 }, { pull_request: { ...event.pull_request, merged: false } }, { pull_request: { ...event.pull_request, base: { ref: 'next' } } }]) assert.throws(() => validateTrigger({ ...event, ...patch }, intent, bytes, digest(bytes)));
  assert.throws(() => validateTrigger(event, intent, bytes + ' ', digest(bytes)), /intent changed/);
});
test('existing immutable version requires checksum AND compatibility evidence', async () => {
  const api = { lookup: async () => 'a', verifyReuse: async () => {}, publish: async () => assert.fail('must never overwrite') };
  await assert.rejects(immutablePublish({ name: 'core', sha256: 'b' }, api), /checksum mismatch/);
  await assert.rejects(immutablePublish({ name: 'core', sha256: 'a' }, api), /reuse evidence/);
  await immutablePublish({ name: 'core', sha256: 'a', reuseEvidence: { archiveSha256: 'a', compatible: true } }, api);
});
test('stale PR, unsuccessful CI and different tested merge tree fail closed', () => {
  const candidateIntent = { seal: { workflowPath: '.github/workflows/e2e.yaml' } };
  const ci = { path: candidateIntent.seal.workflowPath, event: 'pull_request', conclusion: 'success', pull_requests: [{ number: 2329, head: { sha: event.pull_request.head.sha } }] };
  validateCandidateRun(ci, candidateIntent, event, 'same', 'same');
  assert.throws(() => validateCandidateRun({ ...ci, pull_requests: [] }, candidateIntent, event, 'same', 'same'), /Stale/);
  assert.throws(() => validateCandidateRun({ ...ci, conclusion: 'failure' }, candidateIntent, event, 'same', 'same'));
  assert.throws(() => validateCandidateRun(ci, candidateIntent, event, 'old', 'new'), /tree differs/);
});
test('retry after partially successful Rust upload verifies bytes and reuse', async () => {
  let published = false;
  const record = { name: 'core', sha256: 'a', reuseEvidence: { archiveSha256: 'a', compatible: true } };
  const api = { lookup: async () => published ? 'a' : null, publish: async () => { published = true; throw Error('connection lost'); }, verifySealedCandidate: async () => {} };
  await assert.rejects(immutablePublish(record, api, 'publish'), /connection lost/);
  await immutablePublish(record, api, 'publish');
});
test('reuse never creates an absent version and resumed new versions require the candidate gate', async () => {
  const record = { name: 'core', sha256: 'a' };
  await assert.rejects(immutablePublish(record, { lookup: async () => null, publish: async () => assert.fail('must not publish') }), /reuse version is absent/);
  await assert.rejects(immutablePublish(record, { lookup: async () => 'a' }, 'publish'), /sealed candidate validation/);
  await assert.rejects(immutablePublish(record, { lookup: async () => 'a', verifySealedCandidate: async () => { throw Error('SDK missing'); } }, 'publish'), /SDK missing/);
});
test('successful upload propagation retries only reads, with a bound; wrong bytes fail immediately', async () => {
  let reads = 0, sleeps = 0;
  await confirmRegistryChecksum({ sha256: 'a' }, async () => ++reads < 3 ? null : 'a', async () => { sleeps++; });
  assert.equal(reads, 3); assert.equal(sleeps, 2);
  reads = 0;
  await assert.rejects(confirmRegistryChecksum({ sha256: 'a' }, async () => { reads++; return 'b'; }, async () => {}), /differ/);
  assert.equal(reads, 1);
  await assert.rejects(confirmRegistryChecksum({ sha256: 'a' }, async () => null, async () => {}), /stop and inspect/);
});
test('registry archive verification is streamed, byte-bounded and content checked', async () => {
  await verifyRegistryArchive(new Response('archive'), digest('archive'), 7);
  await assert.rejects(verifyRegistryArchive(new Response('too long'), digest('archive'), 7), /exceeds/);
  await assert.rejects(verifyRegistryArchive(new Response('short'), digest('archive'), 7), /truncated/);
  await assert.rejects(verifyRegistryArchive(new Response('changed'), digest('archive'), 7), /differs/);
});
test('final Release retries reuse only the same immutable tag and provenance', () => {
  const sha = 'a'.repeat(40), hash = 'b'.repeat(64);
  const release = { draft: false, prerelease: false, body: `<!-- farm-release-intent:${hash} -->` };
  assert.equal(validateFinalRelease(null, null, sha, hash), 'create');
  assert.equal(validateFinalRelease(sha, null, sha, hash), 'create');
  assert.equal(validateFinalRelease(sha, release, sha, hash), 'already-complete');
  assert.throws(() => validateFinalRelease('c'.repeat(40), release, sha, hash), /different source/);
  assert.throws(() => validateFinalRelease(sha, { ...release, body: 'unrelated release' }, sha, hash), /different provenance/);
  assert.throws(() => validateFinalRelease(sha, { ...release, prerelease: true }, sha, hash));
});
for (const failure of ['prepare', 'sourceGate', 'rust', 'publicationGate', 'npm', 'registryVerify']) test(`no GitHub Release after ${failure} failure; retry traverses every gate`, async () => {
  const calls = [];
  let fail = true;
  const api = Object.fromEntries(['prepare', 'sourceGate', 'rust', 'publicationGate', 'npm', 'registryVerify', 'release'].map(name => [name, async () => { calls.push(name); if (name === failure && fail) throw Error(name === 'prepare' ? 'Stale candidate CI' : name); }]));
  await assert.rejects(orchestrate(api));
  assert.ok(!calls.includes('release'));
  fail = false; calls.length = 0;
  await orchestrate(api);
  assert.deepEqual(calls, ['prepare', 'sourceGate', 'rust', 'publicationGate', 'npm', 'registryVerify', 'release']);
});
