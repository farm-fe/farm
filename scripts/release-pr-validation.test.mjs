import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {validationIdentity, releaseValidationRequired, verifyValidationMarker, verifyNativeValidation} from './release-pr-validation.mjs';

const sha = 'a'.repeat(40), tree = 'b'.repeat(40);
const env = {GITHUB_ACTIONS: 'true', GITHUB_EVENT_NAME: 'pull_request', GITHUB_REPOSITORY: 'farm-fe/farm', GITHUB_REF: 'refs/pull/2330/merge',
  GITHUB_WORKFLOW_REF: 'farm-fe/farm/.github/workflows/incremental-release.yaml@refs/pull/2330/merge', GITHUB_SHA: sha,
  FARM_BUILD_PROFILE: 'release', GITHUB_RUN_ID: '10', GITHUB_RUN_ATTEMPT: '2'};
const event = {number: 2330, pull_request: {head: {repo: {full_name: 'farm-fe/farm'}}, base: {repo: {full_name: 'farm-fe/farm'}, ref: 'main'}}};
const identity = () => validationIdentity(env, event, {sha, tree});
test('PR validation cannot authorize publication or successful release baselines', () => {
  const result = identity();
  assert.equal(result.chain, 'release-validation'); assert.equal(result.namespace, 'farm-release-pr-validation-v1');
  assert.equal(result.publicationAuthorized, false); assert.equal(result.baselineEligible, false);
  for (const [key, value] of [['GITHUB_REF', 'refs/heads/main'], ['GITHUB_EVENT_NAME', 'pull_request_target'], ['FARM_BUILD_PROFILE', 'ci'],
    ['GITHUB_WORKFLOW_REF', env.GITHUB_WORKFLOW_REF + '-extra'], ['NPM_TOKEN', 'not-a-real-token'], ['CARGO_REGISTRY_TOKEN', 'not-a-real-token']]) {
    assert.throws(() => validationIdentity({...env, [key]: value}, event, {sha, tree}));
  }
  const fork = structuredClone(event); fork.pull_request.head.repo.full_name = 'elsewhere/farm';
  assert.throws(() => validationIdentity(env, fork, {sha, tree}));
  assert.throws(() => validationIdentity(env, event, {sha: 'c'.repeat(40), tree}));
});
test('release recipes and product inputs require a reference, ordinary documentation does not', () => {
  for (const file of ['.github/workflows/incremental-release.yaml', '.github/workflows/release-pr-validation.yaml',
    'scripts/release-pr-validation.test.mjs', 'scripts/ci-build-rust-artifacts.sh', 'scripts/assemble-v2-native-artifacts.mjs',
    'scripts/v2-npm-cli.mjs', 'scripts/v2-pnpm-cli.mjs', 'scripts/performance-report.mjs', '.github/actions/cargo-source-cache/action.yml', 'packages/core/src/index.ts',
    'rust-plugins/react/src/lib.rs', 'Cargo.lock', '.cargo/config.toml', 'docs/releases/v2-candidate.json']) {
    assert.equal(releaseValidationRequired([file]), true, file);
  }
  assert.equal(releaseValidationRequired(['docs/experiences/guide.md']), false);
  assert.equal(releaseValidationRequired(['README.md']), true);
  assert.equal(releaseValidationRequired(['unknown-build-input.config']), true);
  assert.throws(() => releaseValidationRequired(['../elsewhere']));
});
test('validation marker binds current source, tree, run, attempt and sealed receipt', () => {
  const receipt = {testedCommit: sha, testedTree: tree, status: 'complete-seal', published: false};
  const marker = {...identity(), receiptSha256: crypto.createHash('sha256').update(JSON.stringify(receipt)).digest('hex')};
  verifyValidationMarker(marker, identity(), receipt);
  for (const [key, value] of [['chain', 'release'], ['attempt', 1], ['sourceTree', 'c'.repeat(40)], ['baselineEligible', true]]) {
    assert.throws(() => verifyValidationMarker({...marker, [key]: value}, identity(), receipt));
  }
  assert.throws(() => verifyValidationMarker(marker, identity(), {...receipt, published: true}));
});
test('all nine native targets require current-run release-profile byte matches', () => {
  const records = Array.from({length: 9}, (_, i) => ({platform: `target-${i}`, name: `package-${i}`, bytes: 4, sha256: 'd'.repeat(64)}));
  const proofs = records.map(row => ({abi: row.platform, profile: 'release', originBuildSHA: sha, runId: '10', runAttempt: '2',
    reuseAuthorized: false, records: [{name: row.name, bytes: row.bytes, sha256: row.sha256}]}));
  verifyNativeValidation(records, proofs, identity());
  for (const [field, value] of [['profile', 'ci'], ['runId', '11'], ['runAttempt', '1'], ['originBuildSHA', 'c'.repeat(40)], ['reuseAuthorized', true]]) {
    const changed = structuredClone(proofs); changed[0][field] = value; assert.throws(() => verifyNativeValidation(records, changed, identity()));
  }
  assert.throws(() => verifyNativeValidation(records.slice(1), proofs.slice(1), identity()));
  const changed = structuredClone(proofs); changed[0].records[0].sha256 = 'e'.repeat(64);
  assert.throws(() => verifyNativeValidation(records, changed, identity()));
});
test('PR release workflow uses only its own release builds and isolated candidate consumers', () => {
  const read = file => fs.readFileSync(new URL(`../.github/workflows/${file}`, import.meta.url), 'utf8');
  const caller = read('incremental-release.yaml'), workflow = read('release-pr-validation.yaml');
  assert.ok(caller.includes("github.event_name == 'pull_request' && github.event.pull_request.head.repo.full_name == github.repository"));
  assert.ok(caller.includes('uses: ./.github/workflows/release-pr-validation.yaml'));
  const scope = caller.split('  pr-validation-scope:')[1].split('  pr-release-reference:')[0];
  assert.match(scope, /needs: \[source\]/);
  assert.doesNotMatch(scope, /needs: \[source, plan\]|versionRequired|publicationBlocked/);
  assert.ok(caller.includes("assert plan['publicationBlocked'] is False and plan['versionRequired'] == []"));
  const publisher = caller.split('  publisher:')[1].split('  public-consumers:')[0];
  assert.ok(publisher.includes("github.ref == 'refs/heads/main' && inputs.publish == true"));

  assert.equal(workflow.match(/profile: release/g)?.length, 2);
  assert.ok(workflow.includes('needs: [native-primary, native-extra]'));
  assert.ok(workflow.includes('node scripts/release-pr-validation.mjs pack'));
  assert.ok(workflow.includes('node scripts/release-pr-validation.mjs seal release-candidate'));
  assert.ok(workflow.includes('node scripts/release-pr-validation.mjs verify release-candidate'));
  assert.ok(workflow.includes('node scripts/run-sdk-consumer.mjs'));
  assert.ok(workflow.includes('node scripts/verify-v2-npm-consumer.mjs'));
  assert.ok(workflow.includes('os: [ubuntu-24.04, macos-latest, windows-latest]'));
  assert.doesNotMatch(workflow, /secrets:|secrets\.|contents: write|packages: write|id-token|producer_run|run-id:|incremental-release-publish/);
  assert.doesNotMatch(workflow, /name: farm-release-candidates-|name: farm-release-seal-/);
});
