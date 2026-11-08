import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { npmInventory, nativeSourceDigest, changesetsPublication, validationEvent } from './v2-release-inventory.mjs';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { root } from './v2-release-inventory.mjs';

test('candidate scope includes SDK generator and approved Vuetify patch', () => {
  const inventory = npmInventory();
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  assert.deepEqual(inventory, plan.npm);
  if (plan.preparation?.mode !== 'changesets') {
    assert.equal(inventory.primary.length, 51);
    assert.equal(inventory.payloads.length, 246);
    assert.equal(inventory.primary.find(pkg => pkg.name === 'create-farm-plugin').version, '2.0.0');
    assert.equal(inventory.primary.find(pkg => pkg.name === '@farmfe/js-plugin-vuetify').version, '0.0.2');
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
  const run = (...args) => spawnSync(process.execPath, [fileURLToPath(new URL('./v2-release-preflight.mjs', import.meta.url)), ...args], { encoding: 'utf8' });
  assert.equal(run().status, 0);
  const publication = run('--publication');
  assert.equal(publication.status, 1);
  assert.equal(JSON.parse(publication.stdout).readyToPublish, false);
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
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  for (const pkg of plan.cargo) {
    assert.equal(nativeSourceDigest(path.dirname(path.join(root, pkg.manifestPath))), pkg.sourceSha256, `Cargo candidate source drifted: ${pkg.name}`);
  }
});

test('preflight rejects stale or missing Cargo source fingerprints before reporting consistency', () => {
  const candidatePath = path.join(root, 'docs/releases/v2-candidate.json');
  const preflightUrl = new URL('./v2-release-preflight.mjs', import.meta.url).href;
  for (const missing of [false, true]) {
    // Alter only the child process fixture, never the checked-in candidate.
    const source = `import fs from 'node:fs';
      const read = fs.readFileSync;
      fs.readFileSync = (file, ...args) => {
        const bytes = read(file, ...args);
        if (String(file) !== ${JSON.stringify(candidatePath)}) return bytes;
        const candidate = JSON.parse(bytes);
        if (${missing}) delete candidate.cargo[0].sourceSha256;
        else candidate.cargo[0].sourceSha256 = '0'.repeat(64);
        return JSON.stringify(candidate);
      };
      await import(${JSON.stringify(preflightUrl)});`;
    const result = spawnSync(process.execPath, ['--input-type=module', '--eval', source], { encoding: 'utf8' });
    assert.equal(result.status, 1, result.stdout);
    assert.match(result.stderr, /Cargo candidate source drifted/);
    assert.doesNotMatch(result.stdout, /"sourceInventory": "consistent"/);
  }
});
