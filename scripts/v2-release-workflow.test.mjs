import assert from 'node:assert/strict';
import fs from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import { evaluateNodeEngines } from './v2-node-engines.mjs';
import {createReleaseIntent, independentPublicationPolicy, independentReleaseWorkflow, serializeReleaseIntent, validationDispatches} from './prepare-v2-release-intent.mjs';
const workflow = fs.readFileSync(new URL('../.github/workflows/release.yaml', import.meta.url), 'utf8');
test('all stable publication jobs use the supported pinned Node version', () => {
  const versions = [...workflow.matchAll(/node-version:\s*([0-9.]+)/g)].map(match => match[1]);
  assert.equal(versions.length, 2);
  assert.deepEqual([...new Set(versions)], ['22.20.0']);
  const tools = JSON.parse(fs.readFileSync(new URL('../packages/plugin-tools/package.json', import.meta.url)));
  const rows = evaluateNodeEngines([{ name: tools.name, engine: tools.engines.node, optional: false }], ['20.18.0', ...versions]);
  assert.equal(rows[0].satisfiesInstalledRanges, false);
  assert.ok(rows.slice(1).every(row => row.satisfiesInstalledRanges));
});
test('ordinary main changes use Changesets version PRs without publication credentials', () => {
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /github.ref == 'refs\/heads\/main'/);
  assert.match(workflow, /^  workflow_run:/m);
  assert.match(workflow, /workflows: \[Release-plz\]/);
  assert.match(workflow, /workflow_run.conclusion == 'success'/);
  assert.match(workflow, /uses: changesets\/action@v1/);
  assert.match(workflow, /FARM_CARGO_PROPOSAL:/);
  assert.ok(workflow.indexOf('prepare-cargo-version-proposal.mjs --seed') < workflow.indexOf('uses: changesets/action@v1'), 'Cargo-only input must be visible before Changesets decides whether to version');
  assert.match(workflow, /title: 'chore: release packages'/);
  assert.doesNotMatch(workflow, /NPM_TOKEN|CARGO_REGISTRY_TOKEN|run: pnpm run release/);
  const stable = fs.readFileSync(new URL('../.github/workflows/v2-stable-release.yaml', import.meta.url), 'utf8');
  assert.match(stable, /workflow_dispatch:/);
  assert.doesNotMatch(stable, /pull_request:|secrets\.|v2-auto-release|contents: write/);
  const independent = fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml', import.meta.url), 'utf8').replaceAll('\r\n', '\n');
  assert.match(independent, /publish:[\s\S]*?type: boolean\n        default: false/);
  assert.match(independent, /github.ref == 'refs\/heads\/main' && inputs.publish == true/);
  assert.match(independent, /profile: release/);
  assert.doesNotMatch(independent, /v2-auto-release\.mjs|uses: .*ci\.yaml/);
  assert.doesNotMatch(workflow, /uses: \.\/\.github\/workflows\/rust-build/);
  const release = fs.readFileSync(new URL('./release.mjs', import.meta.url), 'utf8');
  assert.match(release, /assertStableReleaseState\(root\)/);
  assert.match(release, /v2-release-preflight\.mjs/);
  assert.doesNotMatch(release, /changeset publish|buildJsPlugins|npm config set/);
});
test('Changesets filesystem and parsing errors cannot select publication', () => {
  assert.match(workflow, /id: changesets/);
  assert.match(workflow, /run: node scripts\/prepare-cargo-version-proposal\.mjs --inspect/);
  assert.match(workflow, /changeset_state == 'pending'/);
  assert.doesNotMatch(workflow, /changeset_state == 'empty'/);
  assert.doesNotMatch(workflow, /if node .*check-changeset-status/);
  assert.match(workflow, /group: farm-changesets-version-/);
  assert.doesNotMatch(workflow, /group: farm-stable-release/);
  const cargo = fs.readFileSync(new URL('../.github/workflows/release-plz.yaml', import.meta.url), 'utf8');
  assert.match(cargo, /group: farm-cargo-version-/);
  assert.doesNotMatch(cargo, /group: farm-stable-release/);
  assert.doesNotMatch(cargo, /CARGO_REGISTRY_TOKEN/);
  assert.match(cargo, /if: steps.release-plz.outcome == 'success'/);
  assert.match(cargo, /FARM_CARGO_PRS_CREATED: \$\{\{ steps.release-plz.outputs.prs_created \}\}/);
  assert.doesNotMatch(cargo, /outputs.pr != ''/);
  assert.match(cargo, /uses: actions\/upload-artifact@v4/);
  assert.match(cargo, /path: .release-cargo-producer\/receipt.json/);
  assert.match(cargo, /--index-production/);
  const stable = fs.readFileSync(new URL('../.github/workflows/v2-stable-release.yaml', import.meta.url), 'utf8');
  assert.doesNotMatch(stable, /contents: write|secrets\./);
  const independent = fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml', import.meta.url), 'utf8').replaceAll('\r\n', '\n');
  assert.match(independent, /group: farm-independent-release\n  cancel-in-progress: false/);
});


test('reviewed version intents select independent main release without authorizing publication', () => {
  const policy = independentPublicationPolicy();
  assert.equal(policy.workflowPath, independentReleaseWorkflow);
  assert.equal(policy.profile, 'release'); assert.equal(policy.chain, 'release');
  assert.equal(policy.publishDefault, false); assert.equal(policy.explicitAuthorizationRequired, true);
  assert.equal(policy.ciBuildBytesAccepted, false);
  const intent = {publication: policy, seal: {workflowPath: independentReleaseWorkflow}};
  assert.deepEqual(JSON.parse(serializeReleaseIntent(intent)), intent);
  assert.throws(() => serializeReleaseIntent({...intent, publication: {...policy, publishDefault: true}}), /cannot enable/);
  assert.throws(() => serializeReleaseIntent({...intent, seal: {workflowPath: '.github/workflows/ci.yaml'}}), /independent release/);
});

test('version preparation dispatches independent read-only planning and never edits a publication workflow', () => {
  const dispatches = validationDispatches(2330, 'a'.repeat(40));
  assert.equal(dispatches.length, 5);
  assert.deepEqual(dispatches.at(-1), {workflow: 'incremental-release.yaml', inputs: ['publish=false']});
  assert.ok(dispatches.slice(0, 4).every(row => row.inputs.includes('release_pr=2330') && row.inputs.includes('release_base=' + 'a'.repeat(40))));
  const source = fs.readFileSync(new URL('./prepare-v2-release-intent.mjs', import.meta.url), 'utf8');
  assert.doesNotMatch(source, /v2-stable-release\.yaml|FARM_RELEASE_INTENT_SHA256|--no-verify|publish=true/);
  assert.match(source, /serializeReleaseIntent\(intent\)/);
});


test('generated intent binds the independent release producer and preserves reviewed version data', () => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const plan = {npm: {primary: [{name: '@farmfe/core', version: '2.0.0'}], payloads: []},
    cargo: [{name: 'farmfe_core', version: '2.2.0'}], counts: {primary: 1, payloads: 0, total: 1}};
  const metadata = {packages: [{name: 'farmfe_core', version: '2.2.0', dependencies: []}]};
  const intent = createReleaseIntent(root, plan, metadata, 2330);
  assert.equal(intent.pullRequest, 2330); assert.equal(intent.tag, 'v2.0.0');
  assert.equal(intent.seal.workflowPath, independentReleaseWorkflow);
  assert.deepEqual(intent.publication, independentPublicationPolicy());
  assert.deepEqual(intent.requiredWorkflows, ['.github/workflows/ci.yaml', '.github/workflows/rust-test.yaml', independentReleaseWorkflow]);
  assert.equal(intent.performanceRequired, true);
  assert.ok(intent.requiredCiJobs.includes('Independent release candidate consumer (windows-latest)'));
  assert.equal(JSON.parse(serializeReleaseIntent(intent)).publication.publishDefault, false);
});


test('independent workflow consumes actual release delta APIs and supports a native-free producer', () => {
  const read = name => fs.readFileSync(new URL('../' + name, import.meta.url), 'utf8');
  const release = read('.github/workflows/incremental-release.yaml');
  for (const file of ['rust-build-user.yaml', 'rust-build-release.yaml']) {
    const reusable = read('.github/workflows/' + file);
    assert.match(reusable, /native_owners:/);
    assert.match(reusable, /inputs.native_owners/);
  }
  assert.equal((release.match(/native_owners: \$\{\{ needs.plan.outputs.native_owners \}\}/g) || []).length, 2);
  assert.ok(release.includes("needs.plan.outputs.native_required == 'false' && needs.release-native-primary.result == 'skipped' && needs.release-native-extra.result == 'skipped'"));
  assert.ok(release.includes('prepare release-work/release-plan.json release-native release-work/native-assembly.json release-work/candidate-plan.json'));
  assert.ok(release.includes('pack release-work/native-assembly.json release-work/artifact-index.json release-work/npm release-work/candidate-plan.json'));
  assert.equal((release.match(/FARM_RELEASE_CANDIDATE_PLAN: release-work\/candidate-plan.json/g) || []).length, 3);
  assert.ok(release.includes("plan['planHash'] == os.environ['PLAN_HASH']"));
  assert.ok(release.includes('Upload independent release candidate acceptance'));
  assert.ok(release.includes('path: candidate-results/result.json'));
});
