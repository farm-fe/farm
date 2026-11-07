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

test('one fixed-main Release contains npm candidate and consumer validation before explicit publication',()=>{
  const release=read('.github/workflows/incremental-release.yaml');
  assert.doesNotMatch(release,/  pull_request:|pr-validation-scope:|pr-release-reference:|release-pr-validation/);
  assert.match(release,/branches: \[main\]/);assert.match(release,/default: false/);
  assert.ok(release.includes("github.ref == 'refs/heads/main' && (github.event_name == 'push' || github.event_name == 'workflow_dispatch')"));
  assert.ok(release.includes("assert os.environ['GITHUB_WORKFLOW_SHA'] == os.environ['GITHUB_SHA']"));
  for(const stage of ['  producer:','  candidate-consumers:','  publisher:','  public-consumers:','  public-proof:','  finalize:'])assert.ok(release.includes(stage),stage);
  assert.doesNotMatch(release,/run-sdk-consumer|seal-v2-rust-packages|CARGO_REGISTRY_TOKEN/);
  assert.match(release,/seal-npm/);
  assert.ok(release.includes('uses: ./.github/workflows/release-candidate-consumers.yaml'));
  assert.ok(release.includes('uses: ./.github/workflows/release-public-consumers.yaml'));
  assert.ok(read('.github/workflows/release-candidate-consumers.yaml').includes('scripts/verify-v2-npm-consumer.mjs'));
  assert.ok(read('.github/workflows/release-public-consumers.yaml').includes('scripts/verify-published-v2-consumers.mjs'));
  assert.ok(release.includes("github.ref == 'refs/heads/main' && needs.source.outputs.version_ready == 'true'"));
  assert.ok(release.includes("github.event_name == 'push' || (github.event_name == 'workflow_dispatch' && inputs.publish == true)"));
  assert.ok(release.includes("needs.candidate-consumers.result == 'success'"));
  assert.match(release,/node scripts\/automatic-release-source\.mjs/);
  assert.match(release,/group: farm-independent-release\n  cancel-in-progress: false/);
  assert.equal((release.match(/profile: release/g)||[]).length,2);
});

test('Docs remains a CI quality lane using formal recipes, not a release reference',()=>{
  const docs=read('.github/workflows/farm-docs.yaml');
  assert.match(docs,/FARM_BUILD_PROFILE: ci/);assert.match(docs,/scripts\/docs-native-build.mjs/);assert.match(docs,/scripts\/docs-native-products.py consume/);
  assert.doesNotMatch(docs,/run build:rs\n|release-profile reference/);
  assert.match(docs,/windows-latest/);assert.match(docs,/ubuntu-24.04/);
});
