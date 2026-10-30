import { latestPublicationDecision, verifyLatestTarget, ensureLatestTarget } from './release.mjs';
import test from 'node:test';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import { digest, validateTrigger, validateCandidateRun, validateArchiveProvenance, immutablePublish, orchestrate, validateFinalRelease, selectRequiredRuns, requiredWorkflows, requiredCiJobs, validateRequiredCiJobs, confirmRegistryChecksum, verifyRegistryArchive, performancePolicy, validatePerformanceEvidence } from './v2-auto-release.mjs';
const intent = { schemaVersion: 1, pullRequest: 2329, baseBranch: 'main', sourceCommit: 'a'.repeat(40), tag: 'v2.0.0', requiredWorkflows, requiredCiJobs, performanceRequired: true, abi: '0.5.4', rustCoreVersion: '2.2.0' };
const bytes = JSON.stringify(intent);
const event = { action: 'closed', number: 2329, pull_request: { merged: true, base: { ref: 'main' }, head: { sha: 'a'.repeat(40) }, merge_commit_sha: 'b'.repeat(40) } };

test('future version PR validates its own intent and explicitly dispatched final head', () => {
  const future = { ...intent, preparationMode: 'changesets', pullRequest: 2400, tag: 'packages-' + 'c'.repeat(16), requiredWorkflows: requiredWorkflows.filter(file => !file.endsWith('lint-pr-title.yml')) };
  const futureEvent = { ...event, number: 2400 };
  const content = JSON.stringify(future);
  validateTrigger(futureEvent, future, content, digest(content));
  const run = { id: 2, path: '.github/workflows/ci.yaml', event: 'workflow_dispatch', conclusion: 'success', head_sha: event.pull_request.head.sha };
  assert.equal(selectRequiredRuns([run], [run.path], run.head_sha, 2400, { allowDispatch: true })[0].id, 2);
  assert.throws(() => selectRequiredRuns([run], [run.path], 'd'.repeat(40), 2400, { allowDispatch: true }), /absent/);
  validateCandidateRun(run, { ...future, seal: { workflowPath: run.path } }, futureEvent, 'same', 'same');
  assert.throws(() => validateCandidateRun(run, { ...future, seal: { workflowPath: run.path } }, futureEvent, 'tested', 'merged'), /tree differs/);
  assert.throws(() => validateTrigger(futureEvent, future, content, '0'.repeat(64)), /intent changed/);
});

test('every functional, candidate-consumer and valid performance lane is required before publication', () => {
  const jobs = requiredCiJobs.map(name => ({ name, status: 'completed', conclusion: 'success' }));
  validateRequiredCiJobs(jobs);
  assert.equal(performancePolicy.regressionThreshold, null);
  assert.equal(performancePolicy.requiredForPublication, true);
  for (const name of requiredCiJobs) {
    assert.throws(() => validateRequiredCiJobs(jobs.filter(job => job.name !== name)), /required CI job/);
    assert.throws(() => validateRequiredCiJobs(jobs.map(job => job.name === name ? { ...job, conclusion: 'skipped' } : job)), /not successful/);
  }
});
test('platform workflow prefixes preserve exact functional release jobs without ambiguous matches', () => {
  const platforms = new Map([['ubuntu-24.04', 'Primary Linux'], ['macos-latest', 'Primary macOS'], ['windows-latest', 'Primary Windows']]);
  const jobs = requiredCiJobs.map(name => {
    for (const [os, prefix] of platforms) {
      if (name === `Examples Test (${os})` || name === `Typescript Test (${os})`) return { name: `${prefix} / ${name}`, status: 'completed', conclusion: 'success' };
    }
    return { name, status: 'completed', conclusion: 'success' };
  });
  validateRequiredCiJobs(jobs);
  const example = jobs.find(job => job.name === 'Primary Linux / Examples Test (ubuntu-24.04)');
  assert.throws(() => validateRequiredCiJobs([...jobs, example]), /ambiguous/);
  assert.throws(() => validateRequiredCiJobs(jobs.map(job => job === example ? { ...job, conclusion: 'skipped' } : job)), /not successful/);
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

test('publication requires all three strict Farm-only reports from the sealed producer and exact attempt', () => {
  const producerSHA = 'c'.repeat(40), runId = 4, runAttempt = 2;
  const samples = ['cold', 'warm'].map(cacheMode => ({ cacheMode, samples: 3, validSamples: 3, medianMs: 100, iqrMs: 0 }));
  const summary = { schemaVersion: 2, protocol: performancePolicy.protocol, event: 'pull_request', repositoryId: 1, runId, runAttempt,
    headSHA: event.pull_request.head.sha, baseSHA: 'd'.repeat(40), prNumber: 2329,
    reports: performancePolicy.platforms.map(([os, arch]) => ({ metadata: { os, arch, revision: producerSHA, baseSHA: 'd'.repeat(40), node: 'v22.20.0', pnpm: '9.4.0', fixtureSha256: 'e'.repeat(64), lockSha256: 'f'.repeat(64), runnerImage: 'ci' },
      status: 'complete', cells: samples, comparison: { mode: 'current-only', baseCells: [] }, history: [], diagnostics: [] })) };
  const expected = { repositoryId: 1, runId, runAttempt, headSHA: summary.headSHA, baseSHA: summary.baseSHA, prNumber: 2329, producerSHA };
  validatePerformanceEvidence(JSON.stringify(summary), expected);
  const mutations = [value => value.reports.pop(), value => { value.reports[0].status = 'invalid'; }, value => { value.reports[0].cells = []; }, value => { value.reports[0].metadata.revision = summary.headSHA; }, value => { value.runAttempt = 1; }, value => { value.protocol = 'commerce-business-2048-supervised-v1'; }, value => { value.reports[1].metadata.arch = 'x64'; }];
  for (const mutate of mutations) { const value = JSON.parse(JSON.stringify(summary)); mutate(value); assert.throws(() => validatePerformanceEvidence(JSON.stringify(value), expected)); }
  assert.throws(() => validateTrigger(event, { ...intent, performanceRequired: false }, JSON.stringify({ ...intent, performanceRequired: false }), digest(JSON.stringify({ ...intent, performanceRequired: false }))), /requires complete/);
});

test('stable latest tags are read back and never deliberately replace a higher version', async () => {
  const record = { name: '@farmfe/core', version: '2.0.0' };
  assert.equal(latestPublicationDecision(null, '2.0.0'), 'advance');
  assert.equal(latestPublicationDecision('2.0.0-beta.1', '2.0.0'), 'advance');
  assert.equal(latestPublicationDecision('2.1.0', '2.0.0'), 'preserve-higher');
  assert.equal(latestPublicationDecision('2.1.0-beta.1', '2.0.0'), 'preserve-higher');
  assert.equal(latestPublicationDecision('2.0.0+build.7', '2.0.0+other'), 'verified');
  for (const invalid of ['02.0.0', '2.0.0-01', '2.0.0-', '2.0.0+', '2.0.0.1']) assert.throws(() => latestPublicationDecision(invalid, '2.0.0'));
  assert.throws(() => latestPublicationDecision('bad', '2.0.0'));
  assert.throws(() => latestPublicationDecision('1.0.0', '2.0.0-beta.1'));
  assert.throws(() => verifyLatestTarget(record, '1.9.0'), /has not reached/);
  let latest = '2.1.0', writes = 0;
  assert.deepEqual(await ensureLatestTarget(record, { readLatest: async () => latest, writeLatest: async () => { writes++; } }), { action: 'preserve-higher', observed: '2.1.0' });
  assert.equal(writes, 0);
  latest = '1.9.0';
  assert.deepEqual(await ensureLatestTarget(record, { readLatest: async () => latest, writeLatest: async () => { writes++; latest = '2.0.0'; } }), { action: 'verified', observed: '2.0.0' });
  assert.equal(writes, 1);
  await ensureLatestTarget(record, { readLatest: async () => latest, writeLatest: async () => assert.fail('idempotent tag must not move') });
  await assert.rejects(ensureLatestTarget(record, { readLatest: async () => '1.9.0', writeLatest: async () => {} }), /has not reached/);
});

test('publication entry module graph has no installed workspace dependency requirement', () => {
  const visited = new Set();
  function visit(file) {
    if (visited.has(file)) return; visited.add(file);
    const source = fs.readFileSync(file, 'utf8');
    for (const match of source.matchAll(/(?:^|\n)import\s+(?:[^'";]+?\s+from\s+)?['"]([^'"]+)['"]/g)) {
      const specifier = match[1];
      if (specifier.startsWith('node:')) continue;
      assert.ok(specifier.startsWith('./') || specifier.startsWith('../'), `Publication requires installed package ${specifier} from ${file}`);
      visit(path.resolve(path.dirname(file), specifier));
    }
  }
  visit(fileURLToPath(new URL('./v2-auto-release.mjs', import.meta.url)));
});
