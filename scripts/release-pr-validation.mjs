import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import crypto from 'node:crypto';
import {resolvePnpmCli} from './v2-pnpm-cli.mjs';
const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

export function validationIdentity(env, event, checkout) {
  assert.equal(env.GITHUB_ACTIONS, 'true');
  assert.equal(env.GITHUB_EVENT_NAME, 'pull_request');
  assert.equal(event.pull_request?.head.repo.full_name, env.GITHUB_REPOSITORY);
  assert.equal(event.pull_request?.base.repo.full_name, env.GITHUB_REPOSITORY);
  assert.equal(event.pull_request.base.ref, 'main');
  assert.ok(Number.isSafeInteger(event.number) && event.number > 0);
  assert.equal(env.GITHUB_REF, `refs/pull/${event.number}/merge`);
  assert.equal(env.GITHUB_WORKFLOW_REF, `${env.GITHUB_REPOSITORY}/.github/workflows/incremental-release.yaml@${env.GITHUB_REF}`);
  assert.match(env.GITHUB_SHA, /^[a-f0-9]{40}$/);
  assert.equal(checkout.sha, env.GITHUB_SHA);
  assert.match(checkout.tree, /^[a-f0-9]{40}$/);
  assert.equal(env.FARM_BUILD_PROFILE, 'release');
  for (const name of ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT']) assert.match(env[name] ?? '', /^[1-9][0-9]*$/);
  for (const name of ['NPM_TOKEN', 'NODE_AUTH_TOKEN', 'CARGO_REGISTRY_TOKEN']) assert.ok(!env[name], 'Publication credentials are forbidden in PR validation');
  return {schemaVersion: 1, chain: 'release-validation', namespace: 'farm-release-pr-validation-v1', profile: 'release',
    sourceSHA: checkout.sha, sourceTree: checkout.tree, runId: Number(env.GITHUB_RUN_ID), attempt: Number(env.GITHUB_RUN_ATTEMPT),
    publicationAuthorized: false, baselineEligible: false};
}

export function releaseValidationRequired(paths) {
  assert.ok(Array.isArray(paths) && paths.every(p => typeof p === 'string' && p && !p.startsWith('/') && !p.split('/').includes('..')));
  // Only reviewed documentation-only inputs can omit this reference. Unknown
  // inputs, all scripts and every shared action conservatively require it.
  return paths.some(p => !/^docs\/experiences\/.+\.md$/.test(p));
}

export function verifyValidationMarker(marker, identity, receipt) {
  for (const [key, value] of Object.entries(identity)) assert.deepEqual(marker[key], value, `Validation identity differs: ${key}`);
  assert.equal(marker.receiptSha256, sha256(Buffer.from(JSON.stringify(receipt))));
  assert.equal(receipt.testedCommit, identity.sourceSHA); assert.equal(receipt.testedTree, identity.sourceTree);
  assert.equal(receipt.status, 'complete-seal'); assert.equal(receipt.published, false);
}

export function verifyNativeValidation(records, proofs, identity) {
  const abis = [...new Set(records.map(row => row.platform))].sort();
  assert.equal(abis.length, 9, 'Complete nine-target release reference required');
  assert.equal(proofs.length, abis.length);
  for (const abi of abis) {
    const matching = proofs.filter(row => row.abi === abi); assert.equal(matching.length, 1);
    const proof = matching[0];
    assert.equal(proof.profile, 'release'); assert.equal(proof.originBuildSHA, identity.sourceSHA);
    assert.equal(Number(proof.runId), identity.runId); assert.equal(Number(proof.runAttempt), identity.attempt);
    assert.equal(proof.reuseAuthorized, false);
    const actual = records.filter(row => row.platform === abi).map(row => ({name: row.name, bytes: row.bytes, sha256: row.sha256}));
    const declared = proof.records.map(row => ({name: row.name, bytes: row.bytes, sha256: row.sha256}));
    const order = rows => rows.sort((a, b) => a.name.localeCompare(b.name));
    assert.deepEqual(order(actual), order(declared), 'Native release-profile output bytes differ');
  }
}

async function main() {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const git = (...args) => execFileSync('git', args, {cwd: root, encoding: 'utf8', timeout: 30000}).trim();
  const env = process.env, event = JSON.parse(fs.readFileSync(env.GITHUB_EVENT_PATH, 'utf8'));
  const identity = validationIdentity(env, event, {sha: git('rev-parse', 'HEAD'), tree: git('rev-parse', 'HEAD^{tree}')});
  const [stage, ...args] = process.argv.slice(2);
  if (stage === 'scope') {
    const base = event.pull_request.base.sha, head = event.pull_request.head.sha;
    for (const sha of [base, head]) assert.match(sha, /^[a-f0-9]{40}$/);
    assert.equal(git('rev-parse', 'HEAD^1'), base); assert.equal(git('rev-parse', 'HEAD^2'), head);
    const paths = execFileSync('git', ['diff', '--name-only', '-z', '--no-renames', `${base}...${head}`], {cwd: root, timeout: 30000}).toString('utf8').split('\0').filter(Boolean);
    fs.appendFileSync(env.GITHUB_OUTPUT, `required=${releaseValidationRequired(paths)}\n`);
    return;
  }
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json'), 'utf8'));
  if (stage === 'pack') {
    const {packNpmCandidates} = await import('./pack-v2-npm-candidates.mjs');
    const [assemblyFile, indexFile, destination] = args; assert.ok(assemblyFile && indexFile && destination);
    const pages = JSON.parse(fs.readFileSync(indexFile, 'utf8'));
    const artifactIndex = Array.isArray(pages) ? pages.flatMap(page => page.artifacts) : pages.artifacts;
    assert.ok(artifactIndex.every(row => row.workflow_run?.id === identity.runId), 'Cross-run native bytes forbidden');
    const assembly = JSON.parse(fs.readFileSync(assemblyFile, 'utf8'));
    const proofs = [...new Set(assembly.records.map(row => row.platform))].map(abi => JSON.parse(fs.readFileSync(
      path.join(root, '.release-native-proof', `native-shadow-${identity.sourceSHA}-${abi}-${identity.attempt}`, 'products.json'), 'utf8')));
    verifyNativeValidation(assembly.records, proofs, identity);
    return packNpmCandidates({root, plan, assembly, artifactIndex,
      sourceCommit: identity.sourceSHA, testedCommit: identity.sourceSHA, testedTree: identity.sourceTree,
      runId: identity.runId, output: path.resolve(destination), pnpm: resolvePnpmCli()});
  }
  const [directory] = args; assert.ok(directory);
  const receipt = JSON.parse(fs.readFileSync(path.join(directory, 'receipt.json'), 'utf8'));
  const markerFile = path.join(directory, 'pr-validation.json');
  if (stage === 'seal') {
    assert.ok(!fs.existsSync(path.join(directory, 'manifest.json')), 'PR validation cannot contain a publication manifest');
    const marker = {...identity, receiptSha256: sha256(Buffer.from(JSON.stringify(receipt)))};
    verifyValidationMarker(marker, identity, receipt);
    fs.writeFileSync(markerFile, JSON.stringify(marker, null, 2) + '\n', {flag: 'wx'});
  } else if (stage === 'verify') verifyValidationMarker(JSON.parse(fs.readFileSync(markerFile, 'utf8')), identity, receipt);
  else throw Error('Unknown PR release validation stage');
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
