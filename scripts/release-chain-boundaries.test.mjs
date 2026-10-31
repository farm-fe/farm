import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {validationDispatches} from './prepare-v2-release-intent.mjs';
const read=file=>fs.readFileSync(new URL('../'+file,import.meta.url),'utf8');

test('ordinary PR CI never calls a release reference or special candidate chain',()=>{
  const ci=read('.github/workflows/ci.yaml');
  assert.match(ci,/  pull_request:/);
  assert.doesNotMatch(ci,/release_pr|release_base/);
  assert.doesNotMatch(ci,/release-pr-validation|v2-candidate-seal.yaml|release-candidate-seal:/);
  assert.doesNotMatch(ci,/profile: release|FARM_BUILD_PROFILE: release/);
  assert.equal(fs.existsSync(new URL('../.github/workflows/release-pr-validation.yaml',import.meta.url)),false);
  assert.equal(fs.existsSync(new URL('./release-pr-validation.mjs',import.meta.url)),false);
  const dispatches=validationDispatches(2330,'a'.repeat(40));
  assert.deepEqual(dispatches.map(row=>row.workflow),['ci.yaml','rust-test.yaml','lint.yaml','code-spell-check.yaml']);
});

test('one fixed-main Release contains candidate SDK and consumer validation before explicit publication',()=>{
  const release=read('.github/workflows/incremental-release.yaml');
  assert.doesNotMatch(release,/  pull_request:|pr-validation-scope:|pr-release-reference:|release-pr-validation/);
  assert.match(release,/branches: \[main\]/);assert.match(release,/default: false/);
  assert.ok(release.includes("github.ref == 'refs/heads/main' && (github.event_name == 'push' || github.event_name == 'workflow_dispatch')"));
  assert.ok(release.includes("assert os.environ['GITHUB_WORKFLOW_SHA'] == os.environ['GITHUB_SHA']"));
  for(const stage of ['  producer:','  candidate-consumers:','  publisher:','  public-consumers:','  public-proof:','  finalize:'])assert.ok(release.includes(stage),stage);
  assert.ok(release.includes('scripts/run-sdk-consumer.mjs'));
  assert.ok(release.includes('scripts/verify-v2-npm-consumer.mjs'));
  assert.ok(release.includes('scripts/verify-published-v2-consumers.mjs'));
  assert.ok(release.includes("github.ref == 'refs/heads/main' && inputs.publish == true"));
  assert.match(release,/group: farm-independent-release\n  cancel-in-progress: false/);
  assert.equal((release.match(/profile: release/g)||[]).length,2);
});

test('Docs remains a CI quality lane using CI recipes, not a release reference',()=>{
  const docs=read('.github/workflows/farm-docs.yaml');
  assert.match(docs,/FARM_BUILD_PROFILE: ci/);assert.match(docs,/run build:rs:ci/);assert.match(docs,/--profile ci/);
  assert.doesNotMatch(docs,/run build:rs\n|release-profile reference/);
  assert.match(docs,/os: \[windows-latest, ubuntu-24.04\]/);
});
