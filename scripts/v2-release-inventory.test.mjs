import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { npmInventory, cargoInventory, nativeSourceDigest, cargoPackageInputsDigest, changesetsPublication, validationEvent } from './v2-release-inventory.mjs';
import fs from 'node:fs';
import path from 'node:path';
import { root } from './v2-release-inventory.mjs';
import { prCandidatePlan } from './prepare-pr-candidate-plan.mjs';

// The committed inventory describes a reviewed release, not every later PR.
// Exercise the existing PR-only generator against this checkout without changing
// that release's source fingerprints or granting publication authority.
const candidatePath = path.join(root, 'docs/releases/v2-candidate.json');
const historicalBytes = fs.readFileSync(candidatePath, 'utf8');
let checkoutPlan;
function currentCandidate() {
  if (!checkoutPlan) {
    const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
    const commit = git('rev-parse', 'HEAD');
    const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--locked', '--offline', '--no-deps', '--format-version', '1'], { cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 ** 2 }));
    checkoutPlan = prCandidatePlan({ base: JSON.parse(historicalBytes), npm: npmInventory(), cargo: cargoInventory(metadata),
      checkout: { testedCommit: commit, sourceCommit: commit, workflowSHA: commit, testedTree: git('rev-parse', 'HEAD^{tree}'), eventName: 'workflow_dispatch', parents: [] } });
  }
  return structuredClone(checkoutPlan);
}
function runPreflight(candidate, args = []) {
  const source = `import fs from 'node:fs';
    const read = fs.readFileSync;
    fs.readFileSync = (file, ...args) => String(file) === ${JSON.stringify(candidatePath)}
      ? ${JSON.stringify(JSON.stringify(candidate))} : read(file, ...args);
    process.argv.push(...${JSON.stringify(args)});
    await import(${JSON.stringify(new URL('./v2-release-preflight.mjs', import.meta.url).href)});`;
  const env = { ...process.env };
  delete env.FARM_RELEASE_CANDIDATE_PLAN;
  delete env.FARM_PR_CANDIDATE_PLAN;
  return spawnSync(process.execPath, ['--input-type=module', '--eval', source], { encoding: 'utf8', env });
}
after(() => assert.equal(fs.readFileSync(candidatePath, 'utf8'), historicalBytes, 'Tests must not rewrite historical release evidence'));


test('candidate scope includes SDK generator and approved Vuetify patch', () => {
  const inventory = npmInventory();
  const plan = currentCandidate();
  assert.deepEqual(inventory, plan.npm);
  const historical = JSON.parse(historicalBytes);
  if (historical.preparation?.mode !== 'changesets') {
    assert.equal(historical.npm.primary.length, 51);
    assert.equal(historical.npm.payloads.length, 246);
    assert.equal(historical.npm.primary.find(pkg => pkg.name === 'create-farm-plugin').version, '2.0.0');
    assert.equal(historical.npm.primary.find(pkg => pkg.name === '@farmfe/js-plugin-vuetify').version, '0.0.2');
  }
  assert.ok(!inventory.primary.some(pkg => pkg.name === '@farmfe/js-plugin-copy'));
  for (const payload of inventory.payloads) {
    assert.equal(payload.version, inventory.primary.find(pkg => pkg.name === payload.owner).version);
    const platform = JSON.parse(fs.readFileSync(path.join(root, payload.manifestPath)));
    const owner = inventory.primary.find(pkg => pkg.name === payload.owner);
    const parent = JSON.parse(fs.readFileSync(path.join(root, owner.manifestPath)));
    assert.equal(platform.version, owner.version, `Platform version drift: ${payload.name}`);
    assert.equal(parent.optionalDependencies[payload.name], owner.version, `Optional dependency drift: ${payload.name}`);
  }
});
const source = 'a'.repeat(64);
const baseline = () => ({ npm: { primary: [
  { name: '@farmfe/core', version: '2.0.0', sourceSha256: source },
  { name: '@farmfe/js-plugin-example', version: '1.0.0', sourceSha256: source }
], payloads: [{ name: '@farmfe/core-linux-x64-gnu', owner: '@farmfe/core', version: '2.0.0' }] }, cargo: [{ name: 'farmfe_core', version: '2.2.0', sourceSha256: source }] });
test('JS-only Changesets delta leaves Core, Rust and native versions immutable', () => {
  const old = baseline(), family = structuredClone(old);
  family.npm.primary[1].version = '1.0.1';
  family.npm.primary[1].sourceSha256 = 'b'.repeat(64);
  const delta = changesetsPublication(family, old, [{ name: '@farmfe/js-plugin-example', newVersion: '1.0.1' }]);
  assert.equal(delta.counts.total, 1); assert.equal(delta.cargo.length, 0); assert.equal(delta.npm.payloads.length, 0);
  assert.match(delta.releaseIdentity, /^packages-[a-f0-9]{16}$/);
});
test('Core/native versions and source changes without version bumps fail closed', () => {
  const old = baseline(), family = structuredClone(old);
  family.npm.primary[0].version = '2.0.1';
  const releases = [{ name: '@farmfe/core', newVersion: '2.0.1' }];
  assert.throws(() => changesetsPublication(family, old, releases), /owner\/payload version drift/);
  family.npm.payloads[0].version = '2.0.1';
  assert.equal(changesetsPublication(family, old, releases).releaseIdentity, 'v2.0.1');
  family.cargo[0].sourceSha256 = 'b'.repeat(64);
  assert.throws(() => changesetsPublication(family, old, releases), /Rust source changed without a version bump/);
  family.cargo[0].version = '2.2.1';
  assert.equal(changesetsPublication(family, old, releases).cargo.length, 1);
  assert.throws(() => changesetsPublication(family, old, []), /official Changesets/);
});
test('source consistency cannot be mistaken for publication approval', () => {
  const plan = currentCandidate();
  const consistency = runPreflight(plan);
  assert.equal(consistency.status, 0, consistency.stderr);
  assert.equal(JSON.parse(consistency.stdout).readyToPublish, false);
  const publication = runPreflight(plan, ['--publication']);
  assert.equal(publication.status, 1);
  assert.match(publication.stderr, /PR candidate inventory never authorizes publication/);
});


test('generic CI dispatch binds an ordinary same-repository PR head and exact base',()=>{
  const repository='farm-fe/farm',sha='a'.repeat(40),base='b'.repeat(40);
  const environment={GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_REPOSITORY:repository,GITHUB_SHA:sha,GITHUB_REF:'refs/heads/topic',GITHUB_WORKFLOW_REF:repository+'/.github/workflows/ci.yaml@refs/heads/topic'};
  const event={inputs:{pr_number:'2330',base_sha:base}},pr={number:2330,state:'open',head:{sha,ref:'topic',repo:{full_name:repository}},base:{sha:base,ref:'main',repo:{full_name:repository}}};
  const options={environment,event,readPullRequest:()=>pr};
  assert.equal(validationEvent(options).pull_request,pr);
  for(const mutate of [value=>{value.event.inputs.base_sha='c'.repeat(40);},value=>{value.environment.GITHUB_SHA='c'.repeat(40);},value=>{value.environment.GITHUB_REF='refs/heads/other';},value=>{value.environment.GITHUB_WORKFLOW_REF=repository+'/.github/workflows/incremental-release.yaml@refs/heads/topic';}]){
    const copy={environment:{...environment},event:structuredClone(event),readPullRequest:()=>pr};mutate(copy);assert.throws(()=>validationEvent(copy));
  }
  const fork=structuredClone(pr);fork.head.repo.full_name='fork/farm';assert.throws(()=>validationEvent({...options,readPullRequest:()=>fork}));
});

test('candidate Cargo fingerprints match the current source bytes', () => {
  const plan = currentCandidate();
  for (const pkg of plan.cargo) {
    assert.equal(nativeSourceDigest(path.dirname(path.join(root, pkg.manifestPath))), pkg.sourceSha256, `Cargo candidate source drifted: ${pkg.name}`);
    assert.equal(cargoPackageInputsDigest(path.dirname(path.join(root, pkg.manifestPath))), pkg.packageInputsSha256, `Cargo candidate package inputs drifted: ${pkg.name}`);
  }
});

test('preflight rejects stale or missing Cargo source fingerprints before reporting consistency', () => {
  for (const missing of [false, true]) {
    const candidate = currentCandidate();
    if (missing) delete candidate.cargo[0].sourceSha256;
    else candidate.cargo[0].sourceSha256 = '0'.repeat(64);
    const result = runPreflight(candidate);
    assert.equal(result.status, 1, result.stdout);
    assert.match(result.stderr, /Cargo candidate source drifted/);
    assert.doesNotMatch(result.stdout, /"sourceInventory": "consistent"/);
  }
});

test('preflight still rejects stale reviewed release evidence instead of silently refreshing it', () => {
  const stale = currentCandidate();
  stale.preparation = { mode: 'changesets' };
  stale.npm.primary[0].sourceSha256 = '0'.repeat(64);
  const result = runPreflight(stale);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Candidate manifests drifted/);
});

test('npm-only release retains Cargo source evidence without publishing unchanged Cargo versions', () => {
  const old = baseline(), family = structuredClone(old);
  family.publicationScope = 'npm';
  family.npm.primary[0].version = '2.0.1';
  family.npm.payloads[0].version = '2.0.1';
  family.cargo[0].sourceSha256 = 'b'.repeat(64);
  const releases = [{ name: '@farmfe/core', newVersion: '2.0.1' }];
  const before = structuredClone(family);
  const delta = changesetsPublication(family, old, releases);
  assert.equal(delta.cargo.length, 0);
  assert.equal(delta.npm.primary[0].version, '2.0.1');
  assert.deepEqual(family, before, 'Never replace new source evidence with published Cargo hashes');
  const versionedCargo = structuredClone(family);
  versionedCargo.cargo[0].version = '2.2.1';
  assert.equal(changesetsPublication(versionedCargo, old, releases).cargo.length, 0, 'npm-only never selects Cargo uploads, even if a Cargo version also changed');
  delete family.publicationScope;
  assert.throws(() => changesetsPublication(family, old, releases), /Rust source changed without a version bump/);
});

test('npm-only Cargo source changes still require a new Core family and valid evidence', () => {
  const old = baseline(), family = structuredClone(old);
  family.publicationScope = 'npm';
  family.cargo[0].sourceSha256 = 'b'.repeat(64);
  family.npm.primary[1].version = '1.0.1';
  assert.throws(() => changesetsPublication(family, old, [{ name: '@farmfe/js-plugin-example', newVersion: '1.0.1' }]), /new Core npm\/native family/);
  family.npm.primary[0].version = '2.0.1';
  family.npm.payloads[0].version = '2.0.1';
  delete family.cargo[0].sourceSha256;
  assert.throws(() => changesetsPublication(family, old, [{ name: '@farmfe/js-plugin-example', newVersion: '1.0.1' }, { name: '@farmfe/core', newVersion: '2.0.1' }]), /Missing Rust source evidence/);
});
