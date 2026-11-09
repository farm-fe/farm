import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {validationDispatches} from './prepare-v2-release-intent.mjs';
const read=file=>fs.readFileSync(new URL('../'+file,import.meta.url),'utf8');

test('ordinary PR CI tests current candidates without a publication reference',()=>{
  const ci=read('.github/workflows/ci.yaml');
  assert.match(ci,/  pull_request:/);
  assert.doesNotMatch(ci,/release_pr|release_base/);
  assert.doesNotMatch(ci,/release-pr-validation|release-candidate-seal:/);
  assert.doesNotMatch(ci,/profile: release|FARM_BUILD_PROFILE: release/);
  assert.match(ci,/uses: \.\/.github\/workflows\/v2-candidate-seal.yaml/);
  assert.equal(fs.existsSync(new URL('../.github/workflows/release-pr-validation.yaml',import.meta.url)),false);
  assert.equal(fs.existsSync(new URL('./release-pr-validation.mjs',import.meta.url)),false);
  const dispatches=validationDispatches(2330,'a'.repeat(40));
  assert.deepEqual(dispatches.map(row=>row.workflow),['ci.yaml','rust-test.yaml','lint.yaml','code-spell-check.yaml']);
});

test('fixed-main release builds and freezes bytes then verifies registry read-back without tests',()=>{
  const release=read('.github/workflows/incremental-release.yaml');
  assert.doesNotMatch(release,/  pull_request:|pr-validation-scope:|pr-release-reference:|release-pr-validation/);
  assert.match(release,/branches: \[main\]/);assert.match(release,/default: false/);
  assert.ok(release.includes("github.ref == 'refs/heads/main' && (github.event_name == 'push' || github.event_name == 'workflow_dispatch')"));
  assert.ok(release.includes("assert os.environ['GITHUB_WORKFLOW_SHA'] == os.environ['GITHUB_SHA']"));
  for(const stage of ['  producer:','  publisher:','  public-proof:','  finalize:'])assert.ok(release.includes(stage),stage);
  assert.doesNotMatch(release,/run-sdk-consumer|seal-v2-rust-packages|CARGO_REGISTRY_TOKEN/);
  assert.match(release,/seal-npm/);
  assert.doesNotMatch(release,/release-candidate-consumers|release-public-consumers|verify-v2-npm-consumer|verify-published-v2-consumers/);
  assert.match(release,/incremental-release-record.py registry/);
  assert.ok(read('.github/workflows/v2-candidate-seal.yaml').includes('scripts/verify-v2-npm-consumer.mjs'));
  assert.ok(release.includes("github.ref == 'refs/heads/main' && needs.source.outputs.version_ready == 'true'"));
  assert.ok(release.includes("github.event_name == 'push' || (github.event_name == 'workflow_dispatch' && inputs.publish == true)"));
  assert.ok(release.includes("needs.sign-frozen-release.result == 'success'"));
  assert.match(release,/node scripts\/automatic-release-source\.mjs/);
  assert.match(release,/group: farm-independent-release\n  cancel-in-progress: false/);
  assert.equal((read('.github/workflows/release-native-build.yaml').match(/profile: release/g)||[]).length,2);
});

test('Docs remains a CI quality lane using formal recipes, not a release reference',()=>{
  const docs=read('.github/workflows/farm-docs.yaml');
  assert.match(docs,/FARM_BUILD_PROFILE: ci/);assert.match(docs,/scripts\/docs-native-build.mjs/);assert.match(docs,/scripts\/docs-native-products.py consume/);
  assert.doesNotMatch(docs,/run build:rs\n|release-profile reference/);
  assert.match(docs,/windows-latest/);assert.match(docs,/ubuntu-24.04/);
});
