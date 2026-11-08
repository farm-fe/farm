import assert from 'node:assert/strict';
import fs from 'node:fs';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import test from 'node:test';
import YAML from 'yaml';
const root = path.resolve(import.meta.dirname, '..');
const marker = '__REVIEWED_SIGNER_REVISION__';
const trust = JSON.parse(fs.readFileSync(path.join(root, 'scripts/ci-provenance-trust.json')));
assert.deepEqual(Object.keys(trust), ['schemaVersion', 'signerSHA']);
assert.equal(trust.schemaVersion, 1);
assert.match(trust.signerSHA, /^[a-f0-9]{40}$/);

for (const [scope, caller, count] of [['ci', 'ci.yaml', 8], ['unit', 'rust-test.yaml', 1]]) {
  test(`${scope} A/B template authenticates exact caller and fixed needs outputs`, () => {
    const pinned = file => execFileSync('git', ['show', `${trust.signerSHA}:scripts/ci-provenance-templates/${scope}/${file}`], { cwd: root, encoding: 'utf8' });
    const files = JSON.parse(pinned('index.json'));
    assert.equal(new Set(files).size, files.length);
    const callerPath = `.github/workflows/${caller}`;
    assert.ok(files.includes(callerPath));
    for (const file of files) {
      const bytes = pinned(file);
      const current = fs.readFileSync(path.join(root, file), 'utf8');
      if (file === callerPath) {
        assert.equal(bytes.replace(marker, trust.signerSHA), current);
        assert.equal(bytes.split(marker).length, 2);
      } else assert.equal(bytes, current, file);
    }
    const source = pinned(callerPath);
    const workflow = YAML.parse(source.replace(marker, 'a'.repeat(40)));
    const job = workflow.jobs['sign-ci-bindings'];
    assert.equal(job.uses, `farm-fe/farm/.github/workflows/ci-provenance-signer.yaml@${'a'.repeat(40)}`);
    assert.equal(job.with.scope, scope);
    assert.equal(job.strategy, undefined);
    assert.match(job.if, /github\.event_name == 'push'/);
    assert.match(job.if, /github\.ref == 'refs\/heads\/main'/);
    assert.deepEqual(job.permissions, { contents: 'read', actions: 'read', 'id-token': 'write', attestations: 'write' });
    const outputs = JSON.parse(job.with.upload_bindings);
    assert.equal(Object.keys(outputs).length, count);
    for (const [cell, fields] of Object.entries(outputs)) {
      assert.deepEqual(Object.keys(fields), ['artifact_id', 'artifact_digest', 'run_attempt']);
      for (const expression of Object.values(fields)) {
        const match = expression.match(/^\$\{\{ needs\.([\w-]+)\.outputs\.([\w-]+) \}\}$/);
        assert.ok(match, cell);
        const [, id, field] = match;
        assert.ok(job.needs.includes(id), cell);
        const producer = workflow.jobs[id];
        assert.equal(producer.strategy, undefined);
        const declared = producer.uses
          ? YAML.parse(fs.readFileSync(path.join(root, producer.uses), 'utf8')).on.workflow_call.outputs
          : producer.outputs;
        assert.ok(Object.hasOwn(declared, field), `${cell}:${field}`);
      }
    }
    assert.ok(scope === 'ci' ? job.needs.includes('incremental-execution-gate') : job.needs.includes('cargo-test-windows'));
    assert.ok(!job.needs.includes('impact-summary'));
    assert.ok(!job.needs.includes('changesets-check'));
  });
}
