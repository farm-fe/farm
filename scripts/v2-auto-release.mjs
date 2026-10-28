// cspell:words npmjs
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { verifyExistingNpm } from './release.mjs';

export const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export const requiredWorkflows = ['ci.yaml', 'rust-test.yaml', 'lint.yaml', 'code-spell-check.yaml', 'lint-pr-title.yml'].map(name => `.github/workflows/${name}`);
export const requiredCiJobs = ['Release source contracts', 'Check declared native inputs (not ABI execution)', 'Type Check & Changesets',
  ...['ubuntu-latest', 'macos-latest', 'windows-latest'].flatMap(os => [`Examples Test (${os})`, `Typescript Test (${os})`]),
  'Docs / windows-latest', 'Docs / ubuntu-latest', 'Seal complete npm and Rust candidates'];
export function validateRequiredCiJobs(jobs) {
  for (const name of requiredCiJobs) {
    const matches = jobs.filter(job => job.name === name || job.name.endsWith(` / ${name}`));
    assert.equal(matches.length, 1, `Missing or ambiguous required CI job: ${name}`);
    assert.equal(matches[0].status, 'completed', `Required CI job not complete: ${name}`);
    assert.equal(matches[0].conclusion, 'success', `Required CI job not successful: ${name}`);
  }
}
const root = fileURLToPath(new URL('../', import.meta.url));
export function validateTrigger(event, intent, bytes, expectedHash) {
  assert.equal(event.action, 'closed');
  assert.ok(Number.isSafeInteger(intent.pullRequest) && intent.pullRequest > 0, 'Reviewed release PR number is required');
  assert.equal(event.number, intent.pullRequest);
  assert.equal(event.pull_request?.merged, true);
  assert.equal(event.pull_request?.base?.ref, 'main');
  assert.equal(intent.schemaVersion, 1);
  assert.equal(intent.baseBranch, 'main');
  assert.deepEqual(intent.requiredWorkflows, requiredWorkflows, 'Required release workflows changed');
  assert.deepEqual(intent.requiredCiJobs, requiredCiJobs, 'Required functional CI jobs changed');
  assert.equal(intent.performanceRequired, false, 'Exploratory performance is not a publication certification');
  assert.equal(intent.abi, '0.5.4'); assert.equal(intent.rustCoreVersion, '2.2.0');
  assert.equal(digest(bytes), expectedHash, 'Reviewed release intent changed');
  assert.match(event.pull_request.merge_commit_sha, /^[a-f0-9]{40}$/);
  assert.match(event.pull_request.head.sha, /^[a-f0-9]{40}$/);
  assert.match(intent.tag, /^v2\.\d+\.\d+$/);
}
export function validateCandidateRun(ci, intent, event, testedTree, mergeTree) {
  assert.equal(ci.path, intent.seal.workflowPath);
  assert.equal(ci.event, 'pull_request');
  assert.equal(ci.conclusion, 'success');
  assert.ok(ci.pull_requests?.some(pr => pr.number === event.number && pr.head.sha === event.pull_request.head.sha), 'Stale candidate CI');
  assert.equal(testedTree, mergeTree, 'Tested PR tree differs from final merge; fresh validation required');
}
export function selectRequiredRuns(runs, paths, headSHA, prNumber) {
  assert.ok(Array.isArray(paths) && paths.length >= 1 && new Set(paths).size === paths.length, 'Required workflow set is missing');
  return paths.map(path => {
    assert.match(path, /^\.github\/workflows\/[a-z0-9-]+\.ya?ml$/);
    const selected = runs.filter(run => run.path === path && run.head_sha === headSHA && run.event === 'pull_request' && run.pull_requests?.some(pr => pr.number === prNumber && pr.head.sha === headSHA)).sort((a, b) => b.id - a.id)[0];
    assert.ok(selected, `Required final-head workflow is absent: ${path}`);
    return selected;
  });
}
export function validateArchiveProvenance(receipt, event, testedCommit, testedTree, headTree, mergeTree) {
  assert.equal(receipt.sourceCommit, event.pull_request.head.sha, 'Receipt must identify the reviewed PR head');
  assert.match(testedCommit, /^[a-f0-9]{40}$/);
  assert.equal(receipt.testedCommit, testedCommit, 'Receipt must identify the actual artifact producer commit');
  assert.equal(receipt.testedTree, testedTree, 'Receipt tree differs from GitHub commit metadata');
  assert.equal(testedTree, headTree, 'Synthetic test merge differs from reviewed PR head');
  assert.equal(testedTree, mergeTree, 'Final merged tree was not tested; fresh final-tree validation is required');
}
export async function immutablePublish(record, api, action = 'reuse') {
  assert.ok(['publish', 'reuse'].includes(action));
  const existing = await api.lookup(record);
  if (existing !== null) {
    assert.equal(existing, record.sha256, `Immutable version checksum mismatch: ${record.name}`);
    if (action === 'reuse') {
      assert.equal(record.reuseEvidence?.archiveSha256, existing, 'Existing versions require explicit reuse evidence');
      assert.equal(record.reuseEvidence?.compatible, true);
      await api.verifyReuse(record);
    } else { assert.equal(typeof api.verifySealedCandidate, 'function', 'Resuming a new version requires sealed candidate validation'); await api.verifySealedCandidate(record); }
    return;
  }
  assert.equal(action, 'publish', `Reviewed reuse version is absent: ${record.name}`);
  await api.publish(record);
  await confirmRegistryChecksum(record, api.lookup, api.sleep);
}
export async function confirmRegistryChecksum(record, lookup, sleep = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds))) {
  for (let attempt = 0; attempt < 6; attempt++) {
    const checksum = await lookup(record);
    if (checksum !== null) { assert.equal(checksum, record.sha256, 'Published immutable bytes differ'); return; }
    if (attempt < 5) await sleep(3000);
  }
  throw new Error('Published bytes unavailable; stop and inspect registry propagation before resuming');
}
export function validateFinalRelease(existingTagCommit, existingRelease, mergeCommit, intentHash) {
  if (existingTagCommit !== null) assert.equal(existingTagCommit, mergeCommit, 'Existing tag points to different source');
  if (existingRelease !== null) {
    assert.equal(existingTagCommit, mergeCommit, 'Release has no verified tag');
    assert.equal(existingRelease.draft, false); assert.equal(existingRelease.prerelease, false);
    assert.ok(existingRelease.body?.includes(`<!-- farm-release-intent:${intentHash} -->`), 'Existing Release has different provenance');
    return 'already-complete';
  }
  return 'create';
}
export async function verifyRegistryArchive(response, expectedHash, expectedBytes) {
  assert.ok(response?.body && Number.isSafeInteger(expectedBytes) && expectedBytes > 0 && expectedBytes <= 100 * 1024 ** 2, 'Invalid registry archive bound');
  const reader = response.body.getReader(), hash = crypto.createHash('sha256'); let bytes = 0;
  try {
    for (;;) {
      const chunk = await reader.read(); if (chunk.done) break;
      bytes += chunk.value.length;
      if (bytes > expectedBytes) { await reader.cancel(); throw new Error('Registry archive exceeds sealed byte count'); }
      hash.update(chunk.value);
    }
  } finally { reader.releaseLock(); }
  assert.equal(bytes, expectedBytes, 'Registry archive is truncated');
  assert.equal(hash.digest('hex'), expectedHash, 'Public registry archive differs from sealed bytes');
}
export async function orchestrate(api) {
  await api.prepare();
  await api.sourceGate();
  await api.rust();
  await api.publicationGate();
  await api.npm();
  await api.registryVerify();
  await api.release();
}
const run = (command, args, options = {}) => execFileSync(command, args, { cwd: root, stdio: 'inherit', ...options });
const read = file => JSON.parse(fs.readFileSync(file));
const safeFile = (directory, file) => {
  assert.match(file, /^[a-zA-Z0-9][a-zA-Z0-9._-]{0,179}$/, 'Unsafe evidence filename');
  assert.equal(path.basename(file), file, 'Evidence file must be a basename');
  return path.join(directory, file);
};
async function main() {
  const stage = process.argv[2] ?? 'prepare';
  assert.ok(['prepare', 'rust', 'npm', 'verify', 'release'].includes(stage), 'Unknown release stage');
  const intentFile = path.join(root, 'docs/releases/v2-release-intent.json');
  const bytes = fs.readFileSync(intentFile);
  const intent = JSON.parse(bytes);
  const event = read(process.env.GITHUB_EVENT_PATH);
  assert.equal(process.env.GITHUB_EVENT_NAME, 'pull_request');
  validateTrigger(event, intent, Buffer.from(bytes.toString('utf8').replaceAll('\r\n', '\n')), process.env.FARM_RELEASE_INTENT_SHA256);
  const repository = process.env.GITHUB_REPOSITORY;
  assert.match(repository, /^[\w.-]+\/[\w.-]+$/);
  assert.equal(event.pull_request.head.repo.full_name, repository, 'Release candidates must come from this repository');
  const directory = path.join(root, '.release-candidate');
  const ghJson = args => JSON.parse(execFileSync('gh', args, { cwd: root, encoding: 'utf8' }));
  const request = async (url, options = {}) => {
    const response = await fetch(url, { ...options, signal: AbortSignal.timeout(30000), headers: { 'User-Agent': 'farm-v2-release', Accept: 'application/json', ...options.headers } });
    if (response.status === 404 && !options.method) return null;
    assert.ok(response.ok, `Registry request failed: ${response.status}`);
    return response;
  };
  async function inspectFinalRelease() {
    const refs = ghJson(['api', `repos/${repository}/git/matching-refs/tags/${intent.tag}`]).filter(row => row.ref === `refs/tags/${intent.tag}`);
    assert.ok(refs.length <= 1);
    let object = refs[0]?.object;
    for (let depth = 0; object?.type === 'tag' && depth < 5; depth++) object = ghJson(['api', `repos/${repository}/git/tags/${object.sha}`]).object;
    assert.ok(!object || object.type === 'commit', 'Unsupported release tag object');
    const response = await request(`https://api.github.com/repos/${repository}/releases/tags/${intent.tag}`, { headers: { Authorization: `Bearer ${process.env.GH_TOKEN}` } });
    return { tagExists: Boolean(object), state: validateFinalRelease(object?.sha ?? null, response ? await response.json() : null, event.pull_request.merge_commit_sha, digest(bytes)) };
  }
  let receipt, sourceValidated = false;
  const gate = publication => {
    const report = JSON.parse(execFileSync(process.execPath, ['scripts/v2-release-preflight.mjs', ...(publication ? ['--publication'] : []), '--candidate-seal', safeFile(directory, intent.candidateSeal), '--sdk-proof', safeFile(directory, intent.sdkProof)], { cwd: root, encoding: 'utf8' }));
    for (const key of ['candidateArchivesVerified', 'externalSdkVerified', 'stableVersionStateReady']) assert.equal(report[key], true, `${key} gate incomplete`);
    if (publication) assert.equal(report.readyToPublish, true);
    sourceValidated = true;
  };
  const api = {
    async prepare() {
      const plan = read(path.join(root, 'docs/releases/v2-candidate.json'));
      assert.equal(digest(JSON.stringify(plan)), intent.candidatePlanSha256);
      assert.equal(intent.tag, `v${plan.npm.primary.find(pkg => pkg.name === '@farmfe/core').version}`, 'Release tag differs from selected stable Core');
      const runs = ghJson(['api', `repos/${repository}/actions/runs?event=pull_request&head_sha=${event.pull_request.head.sha}&per_page=100`]).workflow_runs;
      for (const selected of selectRequiredRuns(runs, intent.requiredWorkflows, event.pull_request.head.sha, event.number)) {
        run('gh', ['run', 'watch', String(selected.id), '--repo', repository, '--exit-status']);
        const completed = ghJson(['api', `repos/${repository}/actions/runs/${selected.id}`]);
        assert.equal(completed.repository.full_name, repository); assert.equal(completed.head_sha, event.pull_request.head.sha);
        assert.equal(completed.path, selected.path); assert.equal(completed.status, 'completed'); assert.equal(completed.conclusion, 'success');
      }
      const candidates = runs.filter(row => row.path === intent.seal.workflowPath && row.pull_requests?.some(pr => pr.number === event.number && pr.head.sha === event.pull_request.head.sha));
      assert.ok(candidates.length, 'No final PR evidence run; producer must complete before publication');
      const selected = candidates.sort((a, b) => b.id - a.id)[0];
      const headSha = selected.head_sha;
      assert.equal(headSha, event.pull_request.head.sha);
      // Wait on this exact existing run. GITHUB_TOKEN is never expected to trigger another workflow.
      run('gh', ['run', 'watch', String(selected.id), '--repo', repository, '--exit-status']);
      const ci = ghJson(['api', `repos/${repository}/actions/runs/${selected.id}`]);
      assert.equal(ci.repository.full_name, repository);
      assert.equal(ci.head_sha, headSha, 'Stale candidate CI');
      assert.equal(ci.conclusion, 'success');
      const jobPages = ghJson(['api', '--paginate', '--slurp', `repos/${repository}/actions/runs/${selected.id}/jobs?filter=latest&per_page=100`]);
      validateRequiredCiJobs(jobPages.flatMap(page => page.jobs));
      // Full tested-tree equality, including scripts/docs/version inputs; head-only CI is insufficient.
      // workflow_run.head_sha identifies the PR head, not necessarily the checkout
      // used to build artifacts. Resolve the producer from the exact seal artifact.
      const pages = ghJson(['api', '--paginate', '--slurp', `repos/${repository}/actions/runs/${selected.id}/artifacts?per_page=100`]);
      const seals = pages.flatMap(page => page.artifacts).filter(row => /^v2-release-seal-[a-f0-9]{40}$/.test(row.name) && !row.expired);
      assert.equal(seals.length, 1, 'Exactly one complete final candidate seal is required');
      const producerSha = seals[0].name.slice('v2-release-seal-'.length);
      const tested = ghJson(['api', `repos/${repository}/git/commits/${producerSha}`]);
      const head = ghJson(['api', `repos/${repository}/git/commits/${headSha}`]);
      const merged = ghJson(['api', `repos/${repository}/git/commits/${event.pull_request.merge_commit_sha}`]);
      validateCandidateRun(ci, intent, event, tested.tree.sha, merged.tree.sha);
      run('git', ['fetch', 'origin', producerSha]);
      run('gh', ['run', 'download', String(selected.id), '--repo', repository, '--name', `v2-release-seal-${producerSha}`, '--dir', directory]);
      receipt = read(safeFile(directory, intent.candidateSeal));
      assert.equal(read(path.join(directory, 'materialization.json')).registry, 'registry', 'Release requires portable contained SDK storage');
      validateArchiveProvenance(receipt, event, producerSha, tested.tree.sha, head.tree.sha, merged.tree.sha);
      assert.equal(receipt.planSha256, intent.candidatePlanSha256);
      gate(false);
      await inspectFinalRelease(); // Reject a known tag/Release collision before publishing anything.
      // Discover every immutable collision before the first Rust upload, not
      // after partially publishing Rust and then finding an npm conflict.
      for (const record of receipt.records) {
        const response = await request(`https://registry.npmjs.org/${encodeURIComponent(record.name)}/${encodeURIComponent(record.version)}`);
        verifyExistingNpm(record, fs.readFileSync(safeFile(directory, record.file)), response ? await response.json() : null);
      }
      assert.equal(digest(fs.readFileSync(path.join(root, intent.registryVerification.script), 'utf8').replaceAll('\r\n', '\n')), intent.registryVerification.sha256, 'Registry consumer verifier changed');
      assert.deepEqual((receipt.rust ?? []).map(row => ({ name: row.name, version: row.version })), intent.rust.map(({ name, version }) => ({ name, version })));
      assert.equal(intent.rust.length, plan.cargo.length); assert.equal(new Set(intent.rust.map(row => row.name)).size, plan.cargo.length);
      assert.deepEqual(new Set(intent.rust.map(row => `${row.name}@${row.version}`)), new Set(plan.cargo.map(row => `${row.name}@${row.version}`)));
      const sdk = read(safeFile(directory, intent.sdkProof));
      const complete = new Set();
      for (const record of receipt.rust) {
        assert.equal(sdk.packages.find(pkg => pkg.name === record.name && pkg.version === record.version)?.sha256, record.sha256, 'Rust upload differs from independently tested SDK archive');
        assert.equal(digest(fs.readFileSync(safeFile(directory, record.file))), record.sha256);
        assert.ok(fs.statSync(safeFile(directory, record.file)).size < 100 * 1024 * 1024, 'Rust archive exceeds upload bound');
        const metadata = fs.readFileSync(safeFile(directory, record.metadata));
        assert.ok(metadata.length < 1024 * 1024, 'Rust upload metadata too large');
        assert.equal(digest(metadata), record.metadataSha256);
        const upload = JSON.parse(metadata);
        assert.equal(upload.name, record.name); assert.equal(upload.vers, record.version);
        assert.ok(Array.isArray(upload.deps) && typeof upload.features === 'object', 'Official crates.io upload metadata required');
        for (const dep of upload.deps) if (dep.kind !== 'dev' && intent.rust.some(row => row.name === dep.name)) assert.ok(complete.has(dep.name), 'Rust metadata dependency order invalid');
        for (const dep of record.dependencies ?? []) assert.ok(complete.has(dep), 'Rust dependency order invalid');
        const registry = await request(`https://crates.io/api/v1/crates/${encodeURIComponent(record.name)}/${encodeURIComponent(record.version)}`);
        if (registry) {
          const checksum = (await registry.json()).version.checksum;
          const approved = intent.rust.find(pkg => pkg.name === record.name);
          assert.equal(checksum, record.sha256, 'Immutable Rust version checksum mismatch before publication');
          if (approved.action === 'reuse') {
            assert.equal(approved.registryChecksum, checksum, 'Existing Rust version checksum requires review');
            assert.equal(record.reuseEvidence?.archiveSha256, checksum);
            assert.equal(record.reuseEvidence?.compatible, true);
            assert.match(approved.reuseEvidenceSha256 ?? '', /^[a-f0-9]{64}$/);
            assert.equal(record.reuseEvidence.sha256, approved.reuseEvidenceSha256);
            assert.equal(record.reuseEvidence.file, approved.reuseEvidenceFile);
            assert.equal(digest(JSON.stringify(read(path.join(root, record.reuseEvidence.file)))), approved.reuseEvidenceSha256);
          } else assert.equal(approved.action, 'publish');
        } else assert.equal(intent.rust.find(pkg => pkg.name === record.name).action, 'publish', 'Reviewed reuse version disappeared');
        complete.add(record.name);
      }
    },
    sourceGate: () => gate(false),
    async rust() {
      assert.ok(process.env.CARGO_REGISTRY_TOKEN, 'Existing CARGO_REGISTRY_TOKEN secret is required');
      const completed = new Set();
      for (const record of receipt.rust) {
        assert.match(record.sha256, /^[a-f0-9]{64}$/);
        const archive = fs.readFileSync(safeFile(directory, record.file));
        assert.ok(archive.length < 100 * 1024 * 1024, 'Rust archive exceeds upload bound');
        assert.equal(digest(archive), record.sha256);
        const metadata = fs.readFileSync(safeFile(directory, record.metadata));
        assert.equal(digest(metadata), record.metadataSha256);
        const upload = JSON.parse(metadata);
        assert.equal(upload.name, record.name); assert.equal(upload.vers, record.version);
        for (const dependency of record.dependencies ?? []) assert.ok(completed.has(dependency), 'Rust dependency order is not sealed');
        await immutablePublish(record, {
          async lookup(row) {
            const response = await request(`https://crates.io/api/v1/crates/${encodeURIComponent(row.name)}/${encodeURIComponent(row.version)}`);
            return response ? (await response.json()).version.checksum : null;
          },
          async verifyReuse(row) {
            const evidence = row.reuseEvidence;
            const approved = intent.rust.find(pkg => pkg.name === row.name);
            assert.equal(approved.registryChecksum, row.sha256, 'Existing Rust version lacks reviewed immutable checksum');
            assert.equal(approved.reuseEvidenceSha256, evidence.sha256, 'Rust compatibility proof is not reviewed in release intent');
            assert.equal(evidence.file, approved.reuseEvidenceFile);
            assert.equal(digest(JSON.stringify(read(path.join(root, evidence.file)))), evidence.sha256);
          },
          async verifySealedCandidate(row) {
            assert.equal(intent.rust.find(pkg => pkg.name === row.name)?.action, 'publish');
            assert.ok(receipt.rust.some(pkg => pkg.name === row.name && pkg.sha256 === row.sha256));
            assert.equal(sourceValidated, true, 'Candidate SDK/archive gate has not passed');
          },
          async publish() {
            const size = value => { const buffer = Buffer.alloc(4); buffer.writeUInt32LE(value); return buffer; };
            await request('https://crates.io/api/v1/crates/new', { method: 'PUT', headers: { Authorization: process.env.CARGO_REGISTRY_TOKEN, 'Content-Type': 'application/octet-stream' }, body: Buffer.concat([size(metadata.length), metadata, size(archive.length), archive]) });
          }
        }, intent.rust.find(pkg => pkg.name === record.name).action);
        completed.add(record.name);
      }
    },
    publicationGate: () => gate(true),
    npm() {
      assert.ok(process.env.NPM_TOKEN, 'Existing NPM_TOKEN secret is required');
      run(process.execPath, ['scripts/release.mjs'], { env: { ...process.env, NODE_AUTH_TOKEN: process.env.NPM_TOKEN, FARM_RELEASE_CANDIDATE_DIRECTORY: directory, FARM_RELEASE_SDK_PROOF: safeFile(directory, intent.sdkProof) } });
    },
    async registryVerify({ consumer = true } = {}) {
      for (const record of receipt.records) {
        const response = await request(`https://registry.npmjs.org/${encodeURIComponent(record.name)}/${encodeURIComponent(record.version)}`);
        assert.ok(response, 'Published npm version missing');
        const metadata = await response.json();
        const url = new URL(metadata.dist.tarball);
        assert.equal(url.protocol, 'https:'); assert.equal(url.hostname, 'registry.npmjs.org');
        const archive = await request(url);
        await verifyRegistryArchive(archive, record.sha256, fs.statSync(safeFile(directory, record.file)).size);
      }
      if (consumer) run(process.execPath, [intent.registryVerification.script], { env: { ...process.env, FARM_RELEASE_CANDIDATE_DIRECTORY: directory } });
    },
    async release() {
      // Existing tags/releases need inspection; never silently move or overwrite them.
      const state = await inspectFinalRelease();
      if (state.state === 'already-complete') return;
      if (!state.tagExists) run('gh', ['api', '--method', 'POST', `repos/${repository}/git/refs`, '-f', `ref=refs/tags/${intent.tag}`, '-f', `sha=${event.pull_request.merge_commit_sha}`]);
      await inspectFinalRelease();
      run('gh', ['release', 'create', intent.tag, '--repo', repository, '--verify-tag', '--title', `Farm ${intent.tag}`, '--notes', `<!-- farm-release-intent:${digest(bytes)} -->\nStable V2 from PR #${event.number}. Sealed source ${receipt.sourceCommit}. Registry archives and independent consumer verified.`]);
    }
  };
  await api.prepare();
  if (stage === 'prepare') return;
  if (stage === 'rust') { await api.rust(); return; }
  await api.publicationGate();
  if (stage === 'npm') { await api.npm(); return; }
  await api.registryVerify({ consumer: stage === 'verify' });
  if (stage === 'verify') {
    const resultDirectory = path.join(directory, 'public-consumer', `attempt-${process.env.GITHUB_RUN_ATTEMPT}`);
    const result = read(path.join(resultDirectory, 'result.json')); assert.equal(result.status, 'passed');
    fs.mkdirSync('.release-consumer-proof');
    fs.writeFileSync('.release-consumer-proof/receipt.json', JSON.stringify({ schemaVersion: 1, status: 'passed', sourceCommit: receipt.sourceCommit,
      mergeCommit: event.pull_request.merge_commit_sha, intentSha256: digest(bytes), runId: Number(process.env.GITHUB_RUN_ID), runAttempt: Number(process.env.GITHUB_RUN_ATTEMPT),
      npmLockSha256: digest(fs.readFileSync(path.join(resultDirectory, 'package-lock.json'))), rustLockSha256: digest(fs.readFileSync(path.join(resultDirectory, 'Cargo.lock'))), sdkLogSha256: digest(fs.readFileSync(path.join(resultDirectory, 'sdk.log'))) }, null, 2));
    return;
  }
  // The read-only consumer job is a separate machine with no production tokens.
  // Accept only its bounded data artifact from this exact trusted workflow run.
  const pages = ghJson(['api', '--paginate', '--slurp', `repos/${repository}/actions/runs/${process.env.GITHUB_RUN_ID}/artifacts?per_page=100`]);
  const prefix = `v2-registry-consumer-${event.pull_request.merge_commit_sha}-${process.env.GITHUB_RUN_ID}-attempt-`;
  const proofs = pages.flatMap(page => page.artifacts).filter(row => row.name.startsWith(prefix) && /^[1-9][0-9]*$/.test(row.name.slice(prefix.length)) && Number(row.name.slice(prefix.length)) <= Number(process.env.GITHUB_RUN_ATTEMPT) && !row.expired).sort((a, b) => Number(b.name.slice(prefix.length)) - Number(a.name.slice(prefix.length)));
  assert.ok(proofs.length && proofs[0].size_in_bytes <= 16384, 'Read-only public consumer proof is missing');
  run('gh', ['run', 'download', process.env.GITHUB_RUN_ID, '--repo', repository, '--name', proofs[0].name, '--dir', '.release-consumer-proof']);
  const proofFile = '.release-consumer-proof/receipt.json'; assert.ok(fs.statSync(proofFile).size <= 8192);
  validatePublicConsumerProof(read(proofFile), { sourceCommit: receipt.sourceCommit, mergeCommit: event.pull_request.merge_commit_sha, intentSha256: digest(bytes), runId: Number(process.env.GITHUB_RUN_ID), runAttempt: Number(process.env.GITHUB_RUN_ATTEMPT) });
  await api.release();
}
export function validatePublicConsumerProof(proof, expected) {
  assert.equal(proof.schemaVersion, 1); assert.equal(proof.status, 'passed');
  for (const key of ['sourceCommit', 'mergeCommit', 'intentSha256', 'runId']) assert.equal(proof[key], expected[key], `Consumer proof ${key} differs`);
  assert.ok(Number.isSafeInteger(proof.runAttempt) && proof.runAttempt > 0 && proof.runAttempt <= expected.runAttempt, 'Consumer proof attempt is invalid');
  for (const key of ['npmLockSha256', 'rustLockSha256', 'sdkLogSha256']) assert.match(proof[key], /^[a-f0-9]{64}$/);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
