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
  for (const scope of ['pending', 'frozen', 'public', 'cargo']) {
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
  for (const name of ['incremental-release', 'cargo-source-release', 'release-candidate-consumers', 'release-public-consumers']) {
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
  assert.match(jobs['sign-pending-evidence'].if, /always\(\)/);
  assert.match(jobs['sign-pending-evidence'].if, /needs\.main-evidence\.result == 'failure'/);
  assert.match(jobs['sign-pending-evidence'].if, /pending_id != ''/);
  assert.deepEqual(needs(jobs['sign-public-acceptance']), ['public-proof', 'candidate-consumers', 'public-consumers']);
});

test('candidate and public receipts use explicit platform jobs, never matrix last-writer outputs', () => {
  for (const kind of ['candidate', 'public']) {
    const doc = workflow(`release-${kind}-consumers`);
    assert.deepEqual(Object.keys(doc.jobs), ['linux', 'macos', 'windows']);
    assert.equal(Object.keys(doc.on.workflow_call.outputs).length, 9);
    for (const platform of ['linux', 'macos', 'windows']) {
      assert.equal(doc.jobs[platform].strategy, undefined);
      for (const field of ['artifact_id', 'artifact_digest', 'run_attempt']) {
        assert.equal(doc.on.workflow_call.outputs[`${platform}_${field}`].value,
          `\${{ jobs.${platform}.outputs.${field} }}`);
      }
    }
  }
});
