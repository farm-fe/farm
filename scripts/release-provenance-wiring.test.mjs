// Production workflow contracts; no hosted execution or signing is simulated here.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { execFileSync } from 'node:child_process';
import test from 'node:test';
import YAML from 'yaml';
const read = name => fs.readFileSync(name, 'utf8');
const workflow = name => YAML.parse(read(`.github/workflows/${name}.yaml`));
const pin = JSON.parse(read('scripts/release-provenance-trust.json')).signerSHA;
const needs = job => typeof job.needs === 'string' ? [job.needs] : job.needs || [];

test('each production caller is byte-exact to its immutable reviewed signer template', () => {
  assert.match(pin, /^[a-f0-9]{40}$/);
  for (const scope of ['frozen', 'registry', 'native', 'cargo']) {
    const root = `scripts/release-provenance-templates/${scope}`;
    const index = JSON.parse(execFileSync('git', ['show', `${pin}:${root}/index.json`], { encoding: 'utf8' }));
    for (const name of index) {
      const template = execFileSync('git', ['show', `${pin}:${root}/${name}`], { encoding: 'utf8' });
      assert.equal(read(name), template.replace('__REVIEWED_SIGNER_REVISION__', pin), name);
    }
  }
  assert.equal(workflow('release-proof-call').jobs.sign.uses,
    `farm-fe/farm/.github/workflows/release-provenance-signer.yaml@${pin}`);
});

test('the immutable signer uses the same corrected verifier as release consumers', () => {
  // Cargo restore verifies its original proof inside the pinned signer checkout.
  // Updating only the consumer would leave that path on the rejected CLI flags.
  for (const file of ['ci-attested-bindings.py', 'ci-pinned-attestation-cli.py', 'release-provenance.py']) {
    assert.equal(execFileSync('git', ['show', `${pin}:scripts/${file}`], { encoding: 'utf8' }),
      read(`scripts/${file}`), file);
  }
});

test('nested signer permissions and events stay within the approved main-push scope', () => {
  for (const name of ['release-proof-call', 'release-provenance-signer']) {
    const doc = workflow(name);
    assert.deepEqual(Object.keys(doc.on), ['workflow_call']);
    assert.deepEqual(doc.permissions, {});
    assert.deepEqual(doc.jobs.sign.permissions, {
      contents: 'read', actions: 'read', 'id-token': 'write', attestations: 'write'
    });
    assert.match(doc.jobs.sign.if, /github\.repository == 'farm-fe\/farm'/);
    assert.match(doc.jobs.sign.if, /github\.event_name == 'push'/);
    assert.match(doc.jobs.sign.if, /github\.ref == 'refs\/heads\/main'/);
    assert.equal(doc.on.workflow_call.inputs.cargo_restored.type, 'boolean');
  }
  const cargo = workflow('cargo-source-release');
  assert.equal(cargo.jobs.source.outputs.frozen_restored, "${{ steps.archive-upload.outcome != 'success' }}");
  assert.equal(cargo.jobs['sign-frozen-cargo'].with.cargo_restored, "${{ needs.source.outputs.frozen_restored == 'true' }}");
});

test('real release DAGs have no dangling needs or cycles and no publication/signature cycle', () => {
  for (const name of ['incremental-release', 'cargo-source-release', 'release-native-build', 'v2-candidate-seal']) {
    const { jobs } = workflow(name);
    const visited = new Set();
    const active = new Set();
    function visit(id) {
      assert.ok(jobs[id], `${name}: missing ${id}`);
      assert.ok(!active.has(id), `${name}: cycle at ${id}`);
      if (visited.has(id)) return;
      active.add(id);
      for (const dependency of needs(jobs[id])) visit(dependency);
      active.delete(id); visited.add(id);
    }
    for (const id of Object.keys(jobs)) visit(id);
  }
  const jobs = workflow('incremental-release').jobs;
  assert.ok(needs(jobs.publisher).includes('sign-frozen-release'));
  assert.ok(!needs(jobs.publisher).includes('sign-public-acceptance'));
  assert.ok(needs(jobs.finalize).includes('sign-public-acceptance'));
  assert.equal(jobs['sign-pending-evidence'], undefined);
  assert.equal(jobs['main-evidence'], undefined);
  assert.deepEqual(needs(jobs['sign-public-acceptance']), ['public-proof']);
  assert.equal(jobs['sign-public-acceptance'].with.scope, 'registry');
  assert.equal(jobs['candidate-consumers'], undefined);
  assert.equal(jobs['public-consumers'], undefined);
});

test('each native signer binds one fixed ABI upload without a matrix output merge', () => {
  const doc = workflow('release-native-build');
  assert.deepEqual(Object.keys(doc.jobs), ['primary', 'extra', 'sign']);
  const sign = doc.jobs.sign;
  assert.equal(sign.strategy, undefined);
  assert.deepEqual(sign.needs, ['primary', 'extra']);
  assert.equal(sign.with.scope, 'native-${{ inputs.abi }}');
  assert.match(sign.if, /!cancelled\(\)/);
  assert.match(sign.if, /github\.event_name == 'push'/);
  assert.match(sign.if, /github\.ref == 'refs\/heads\/main'/);
  assert.match(sign.if, /needs\.primary\.result == 'success' && needs\.extra\.result == 'skipped'/);
  assert.match(sign.if, /needs\.extra\.result == 'success' && needs\.primary\.result == 'skipped'/);
  const cells = JSON.parse(sign.with.upload_bindings);
  assert.deepEqual(Object.keys(cells), ['native']);
  for (const [field, output] of [['artifact_id','catalog_id'],['artifact_digest','catalog_digest'],['run_attempt','catalog_attempt']]) {
    assert.equal(cells.native[field], `\${{ needs.primary.outputs.${output} || needs.extra.outputs.${output} }}`);
  }
  for (const [job, child] of [['primary','rust-build-user'], ['extra','rust-build-release']]) {
    assert.equal(doc.jobs[job].with.platform, '${{ inputs.abi }}');
    assert.equal(doc.jobs[job].with.profile, 'release');
    const outputs = workflow(child).on.workflow_call.outputs;
    for (const name of ['catalog_id','catalog_digest','catalog_attempt']) assert.ok(outputs[name]);
  }
  const release = workflow('incremental-release');
  const parents = Object.values(release.jobs).filter(job => job.uses === './.github/workflows/release-native-build.yaml');
  assert.equal(parents.length, 2);
  assert.deepEqual(parents.map(job => job.strategy.matrix.abi.length).sort(), [3,6]);
  for (const job of parents) assert.equal(job.strategy['fail-fast'], false);
});
