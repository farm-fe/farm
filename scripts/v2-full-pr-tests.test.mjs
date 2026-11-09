import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import YAML from 'yaml';
const read = name => fs.readFileSync(`.github/workflows/${name}.yaml`, 'utf8');
const ci = YAML.parse(read('ci'));
const unit = YAML.parse(read('rust-test'));
test('PR quality is full for ordinary, docs and version-only changes', () => {
  for (const [name, workflow] of [['ci', ci], ['rust-test', unit]]) {
    assert.ok(workflow.on.pull_request);
    assert.equal(workflow.on.push, undefined);
    for (const [id, job] of Object.entries(workflow.jobs)) {
      if (id === 'changesets-check') continue;
      assert.doesNotMatch(job.if ?? '', /verified-version-only/);
      for (const step of job.steps ?? []) assert.doesNotMatch(step.if ?? '', /verified-version-only/);
    }
    assert.doesNotMatch(read(name), /ci-suite-baseline.py|FARM_SUITE_SELECTION|suite_selection/);
  }
  assert.doesNotMatch(read('ci-primary-platform'), /suite_selection|selection-only|FARM_SUITE_SELECTION/);
});
test('actual execution gate rejects missing native/assembly and every failed test chain', () => {
  const gate = ci.jobs['incremental-execution-gate'];
  const source = gate.steps.find(step => step.env?.RESULTS)?.run;
  assert.ok(source);
  assert.doesNotMatch(source, /verified-version-only|SystemExit/);
  for (const name of ['primary-linux', 'primary-macos', 'primary-windows', 'call-rust-build-release', 'check-native-candidate-assembly', 'examples-test', 'ts-test']) assert.ok(gate.needs.includes(name));
  assert.match(source, /any\(row\['result'\]!='success'/);
});
