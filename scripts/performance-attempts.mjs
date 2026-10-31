import {observeAttempts,actualJob as selectActualJob} from './ci-attempt-producers.mjs';
export {attemptJobs} from './ci-attempt-producers.mjs';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { readSafeZip, runArtifacts, nativePlugins } from './performance-cohorts.mjs';

export const measurementPlatforms = [
  ['linux-x64-gnu', 'linux', 'x64', 'ubuntu-24.04', 'Primary Linux / native / Build and Upload User Artifacts - linux-x64-gnu'],
  ['darwin-arm64', 'darwin', 'arm64', 'macos-latest', 'Primary macOS / native / Build and Upload User Artifacts - darwin-arm64'],
  ['win32-x64-msvc', 'win32', 'x64', 'windows-latest', 'Primary Windows / native / Build and Upload User Artifacts - win32-x64-msvc'],
];
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const integer = value => Number.isSafeInteger(value) && value > 0;
const time = value => { const result = Date.parse(value); assert.ok(Number.isFinite(result), 'Invalid producer time'); return result; };

export async function measurementResolver({ api, repository, summary, producerSHA, testedTree }) {
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/);
  assert.match(producerSHA, /^[a-f0-9]{40}$/); assert.match(testedTree, /^[a-f0-9]{40}$/);
  assert.ok(integer(summary.runId) && integer(summary.runAttempt) && summary.runAttempt <= 32, 'Attempt exceeds verification bound');
  const prefix = `/repos/${repository}`;
  const run = await api(`${prefix}/actions/runs/${summary.runId}`);
  assert.equal(run.repository.id, summary.repositoryId); assert.equal(run.repository.full_name, repository);
  assert.equal(run.path, '.github/workflows/ci.yaml'); assert.equal(run.name, 'E2E Tests');
  assert.equal(run.event, summary.event); assert.equal(run.head_sha, summary.headSHA);
  assert.equal(run.run_attempt, summary.runAttempt, 'Aggregation attempt moved');
  const commit = await api(`${prefix}/git/commits/${producerSHA}`);
  assert.equal(commit.sha, producerSHA); assert.equal(commit.tree.sha, testedTree);
  const artifacts = await runArtifacts(api, prefix, summary.runId);
  const attempts = await observeAttempts(api, prefix, run);
  const actualJob = name => selectActualJob(attempts, name);
  async function archive(name, execution) {
    const selected = artifacts.filter(row => row.name === name && !row.expired);
    assert.equal(selected.length, 1, 'Exact producer archive missing or ambiguous');
    const artifact = await api(`${prefix}/actions/artifacts/${selected[0].id}`);
    assert.equal(artifact.name, name); assert.equal(artifact.workflow_run.id, run.id);
    assert.equal(artifact.workflow_run.repository_id, summary.repositoryId);
    assert.equal(artifact.workflow_run.head_sha, summary.headSHA); assert.equal(artifact.expired, false);
    assert.ok(execution.job.status === 'completed', 'Producer is not terminal');
    assert.ok(time(execution.job.started_at) <= time(artifact.created_at) && time(artifact.created_at) <= time(execution.job.completed_at), 'Artifact outside actual producer execution');
    assert.match(artifact.digest, /^sha256:[a-f0-9]{64}$/);
    assert.ok(integer(artifact.size_in_bytes) && artifact.size_in_bytes <= 16 * 1024 * 1024, 'Producer archive exceeds bound');
    const bytes = await api(`${prefix}/actions/artifacts/${artifact.id}/zip`, true);
    assert.equal(bytes.length, artifact.size_in_bytes); assert.equal('sha256:' + hash(bytes), artifact.digest, 'Producer archive digest differs');
    return { artifact, files: readSafeZip(bytes) };
  }
  return async ([abi, os, arch, runner, nativeJob]) => {
    const execution = actualJob(`Commerce Performance (${runner})`);
    const measured = await archive(`performance-${producerSHA}-${abi}-attempt-${execution.attempt}`, execution);
    const bytes = measured.files.get('summary.json'); assert.ok(bytes && bytes.length <= 2 * 1024 * 1024, 'Raw summary missing or oversized');
    const raw = JSON.parse(new TextDecoder('utf-8', {fatal:true}).decode(bytes));
    assert.equal(raw.protocol, summary.protocol); assert.equal(raw.metadata?.revision, producerSHA);
    assert.equal(raw.metadata.baseSHA, summary.baseSHA); assert.equal(raw.metadata.os, os); assert.equal(raw.metadata.arch, arch);
    assert.match(raw.metadata.harnessSha256, /^[a-f0-9]{64}$/);
    assert.equal(raw.metadata.artifactProducerSHA, producerSHA);
    if (raw.status === 'complete') assert.equal(execution.job.conclusion, 'success', 'Complete measurement producer failed');
    const reused = raw.metadata.nativeReuse != null;
    assert.ok(!reused || abi === 'linux-x64-gnu', 'Unsupported reuse platform');
    const native = actualJob(reused ? 'Type Check & Changesets' : nativeJob); assert.equal(native.job.conclusion, 'success');
    const proof = await archive(reused ? `ci-consumer-native-proof-${run.id}-${native.attempt}` : `native-shadow-${producerSHA}-${abi}-${native.attempt}`, native);
    const productBytes = proof.files.get(reused ? 'consumer.json' : 'products.json'); assert.ok(productBytes && productBytes.length <= 2 * 1024 * 1024, 'Exact native products missing');
    const products = JSON.parse(new TextDecoder('utf-8', {fatal:true}).decode(productBytes));
    assert.equal(products.profile, 'ci', 'Release bytes cannot authorize CI measurement');
    assert.equal(products.abi, abi);
    if (reused) {
      const claim = raw.metadata.nativeReuse;
      assert.equal(products.kind, 'ci-consumer-materialization-not-a-new-build'); assert.equal(products.chain, 'ci');
      assert.equal(products.consumerSHA, producerSHA); assert.equal(products.consumerTree, testedTree); assert.equal(products.acceptanceInherited, false);
      assert.equal(claim.chain, 'ci'); assert.equal(claim.consumerSHA, producerSHA); assert.equal(claim.acceptanceInherited, false);
      assert.equal(products.originBuildSHA, claim.originBuildSHA); assert.equal(String(products.originArtifactId), String(claim.originArtifactId));
      assert.equal(String(products.attempt), String(native.attempt)); assert.equal(products.originContext?.profile, 'ci');
    } else {
      assert.equal(products.originBuildSHA, producerSHA); assert.equal(String(products.runAttempt), String(native.attempt));
      assert.equal(products.repository, repository);
    }
    assert.equal(String(products.runId), String(run.id)); assert.ok(Array.isArray(products.records));
    const nativeRows = new Map(products.records.map(row => [row.owner, row]));
    assert.equal(nativeRows.size, products.records.length, 'Duplicate native owner');
    assert.ok(raw.native?.artifacts && Object.keys(raw.native.artifacts).length > 0, 'Measured native bytes absent');
    const requiredOwners = ['@farmfe/core', ...nativePlugins.map(name => '@farmfe/plugin-' + name)].sort();
    assert.deepEqual(Object.keys(raw.native.artifacts).sort(), requiredOwners, 'Incomplete measured native cohort');
    for (const [owner, record] of Object.entries(raw.native.artifacts)) {
      assert.equal(nativeRows.get(owner)?.sha256, record.sha256, 'Measured native bytes differ from CI producer');
      assert.equal(nativeRows.get(owner)?.bytes, record.bytes);
    }
    const source = {abi, runId:run.id, runAttempt:execution.attempt, jobId:execution.job.id,
      artifactId:measured.artifact.id, artifactDigest:measured.artifact.digest, rawSummarySha256:hash(bytes),
      producerSHA, testedTree, profile:'ci', workflowPath:run.path,
      nativeKind:reused ? 'materialization' : 'build', nativeAttempt:native.attempt, nativeJobId:native.job.id, nativeArtifactId:proof.artifact.id,
      nativeArtifactDigest:proof.artifact.digest, harnessSha256:raw.metadata.harnessSha256};
    return {raw, source};
  };
}
