import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import YAML from 'yaml';
const read = file => YAML.parse(fs.readFileSync(new URL('../.github/workflows/' + file, import.meta.url), 'utf8'));
const ci = read('ci.yaml');
const fields = ['artifact_id', 'artifact_digest', 'run_attempt'];

test('all current E2E bindings come from explicit needs outputs', () => {
  const job = ci.jobs['impact-summary'];
  const env = job.steps.find(step => step.env?.CI_E2E_UPLOADS).env;
  const inventory = JSON.parse(env.CI_E2E_UPLOADS);
  assert.equal(Object.keys(inventory).length, 25);
  for (const [cell, row] of Object.entries(inventory)) {
    assert.deepEqual(Object.keys(row).sort(), [...fields].sort(), cell);
    for (const expression of Object.values(row)) {
      const match = expression.match(/^\$\{\{ needs\.([\w-]+)\.outputs\.([\w-]+) \}\}$/);
      assert.ok(match, cell);
      const [, producer, field] = match;
      assert.ok(job.needs.includes(producer), cell);
      const caller = ci.jobs[producer];
      assert.equal(caller.strategy, undefined, producer);
      const outputs = caller.uses ? read(caller.uses.split('/').at(-1)).on.workflow_call.outputs : caller.outputs;
      assert.ok(Object.hasOwn(outputs, field), cell + ':' + field);
    }
  }
});

test('secondary native output names are unique and every caller selects one ABI', () => {
  const workflow = read('ci-secondary-native.yaml');
  assert.equal(Object.keys(workflow.jobs).length, 6);
  const targets = new Set();
  for (const [id, job] of Object.entries(workflow.jobs)) {
    assert.equal(job.strategy, undefined);
    const defaults = read('rust-build-release.yaml').jobs.build.strategy.matrix.settings;
    const literal = defaults.match(/inputs.platform == 'all' && '([^']+)'/)[1];
    const selected = JSON.parse(literal).filter(row => row.abi === job.with.platform);
    assert.equal(selected.length, 1);
    assert.equal(id, selected[0].abi.replaceAll('-', '_'));
    assert.equal(job.with.platform, selected[0].abi);
    targets.add(selected[0].abi);
    for (const field of fields)
      assert.equal(workflow.on.workflow_call.outputs[id + '_' + field].value, '${{ jobs.' + id + '.outputs.' + field + ' }}');
  }
  assert.equal(targets.size, 6);
});

test('platform safety uses three fixed callers and preserves all-success gate', () => {
  const gate = ci.jobs['platform-contract-tests'];
  assert.equal(gate.strategy, undefined);
  for (const [key, os] of [['linux', 'ubuntu-24.04'], ['macos', 'macos-latest'], ['windows', 'windows-latest']]) {
    const id = 'platform-contract-' + key, job = ci.jobs[id];
    assert.ok(gate.needs.includes(id));
    assert.equal(job.with.os, os);
    assert.equal(job.uses, './.github/workflows/ci-platform-safety.yaml');
    assert.equal(job.strategy, undefined);
  }
  assert.match(gate.steps[0].run, /all\(v\['result'\]=='success'/);
});

test('native and Docs output chain ends at fixed receipt upload action', () => {
  const primary = read('ci-primary-platform.yaml');
  for (const [prefix, job, file] of [['native', 'native', 'rust-build-user.yaml'], ['docs', 'docs', 'farm-docs.yaml']]) {
    for (const field of fields)
      assert.equal(primary.on.workflow_call.outputs[prefix + '_shadow_' + field].value, '${{ jobs.' + job + '.outputs.' + field + ' }}');
    const child = read(file), key = prefix === 'native' ? 'build' : 'docs';
    for (const field of fields) assert.ok(child.on.workflow_call.outputs[field].value.includes('jobs.' + key + '.outputs.'));
    const producer = child.jobs[key];
    assert.equal(producer.steps.find(step => step.id === 'shadow-upload').uses, './.github/actions/ci-impact-cell');
    for (const field of fields) assert.ok(producer.outputs[field].includes('steps.shadow-upload.outputs.'));
  }
  assert.equal(primary.jobs.native.with.platform, '${{ inputs.abi }}');
  assert.match(primary.jobs.docs.with.platforms, /^\$\{\{ format\('\[\{\{/);
});

test('full suite consumers do not inherit historical selection or test success', () => {
  const type = ci.jobs['type-check'];
  assert.equal(type.steps.some(step => ['selection-upload', 'selection-binding'].includes(step.id)), false);
  assert.equal(type.outputs?.suite_selection_upload, undefined);
  let callers = 0;
  for (const job of Object.values(ci.jobs)) {
    if (job.uses === './.github/workflows/ci-primary-platform.yaml') {
      assert.equal(job.with.suite_selection_upload, undefined);
      callers++;
    }
    for (const step of job.steps ?? [])
      assert.equal(step.env?.FARM_SUITE_SELECTION_UPLOAD, undefined);
  }
  assert.equal(callers, 3);
  const primary = read('ci-primary-platform.yaml');
  assert.equal(primary.on.workflow_call.inputs.suite_selection_upload, undefined);
  for (const job of Object.values(primary.jobs)) for (const step of job.steps ?? [])
    assert.equal(step.env?.FARM_SUITE_SELECTION_UPLOAD, undefined);
});

test('native audit receives nine singleton product uploads independently of cell receipts', () => {
  const assembly = ci.jobs['check-native-candidate-assembly'];
  const step = assembly.steps.find(row => row.run?.includes('--audit-current-run'));
  const references = [...step.env.CI_NATIVE_UPLOADS.matchAll(/\$\{\{ needs\.([\w-]+)\.outputs\.([\w-]+) \}\}/g)];
  assert.equal(references.length, 9);
  assert.equal(new Set(references.map(row => row[1] + ':' + row[2])).size, 9);
  for (const [, job, output] of references) {
    assert.ok(assembly.needs.includes(job));
    const caller = ci.jobs[job];
    assert.equal(caller.strategy, undefined);
    assert.ok(read(caller.uses.split('/').at(-1)).on.workflow_call.outputs[output]);
  }
  for (const file of ['rust-build-user.yaml', 'rust-build-release.yaml']) {
    const workflow = read(file), build = workflow.jobs.build;
    assert.equal(build.steps.find(row => row.id === 'product-upload').uses, 'actions/upload-artifact@v4');
    assert.equal(build.outputs.product_upload, '${{ steps.product-binding.outputs.binding }}');
    const env = build.steps.find(row => row.id === 'product-binding').env;
    assert.equal(env.PRODUCT_ID, '${{ steps.product-upload.outputs.artifact-id }}');
    assert.equal(env.PRODUCT_DIGEST, '${{ steps.product-upload.outputs.artifact-digest }}');
  }
});

test('Docs native bytes receive only the singleton producer upload binding', () => {
  const native = read('rust-build-user.yaml');
  const primary = read('ci-primary-platform.yaml');
  const docs = read('farm-docs.yaml');
  assert.equal(native.on.workflow_call.outputs.docs_upload.value, '${{ jobs.build.outputs.docs_upload }}');
  assert.equal(native.jobs.build.outputs.docs_upload, '${{ steps.docs-binding.outputs.binding }}');
  const upload = native.jobs.build.steps.find(step => step.id === 'docs-upload');
  assert.equal(upload.name, 'Upload same-run Docs native products');
  assert.equal(upload.uses, 'actions/upload-artifact@v4');
  const binding = native.jobs.build.steps.find(step => step.id === 'docs-binding');
  assert.equal(binding.env.DOCS_ARTIFACT_ID, '${{ steps.docs-upload.outputs.artifact-id }}');
  assert.equal(binding.env.DOCS_ARTIFACT_DIGEST, '${{ steps.docs-upload.outputs.artifact-digest }}');
  assert.match(binding.run, /os\.environ\['GITHUB_RUN_ATTEMPT'\]/);
  assert.equal(primary.jobs.docs.with.native_upload, '${{ needs.native.outputs.docs_upload }}');
  assert.ok(primary.jobs.docs.needs.includes('native'));
  assert.equal(docs.on.workflow_call.inputs.native_upload.type, 'string');
  const consumers = docs.jobs.docs.steps.filter(step => step.env?.FARM_DOCS_NATIVE_UPLOAD);
  assert.equal(consumers.length, 1);
  assert.equal(consumers[0].env.FARM_DOCS_NATIVE_UPLOAD, '${{ inputs.native_upload }}');
  assert.match(consumers[0].run, /docs-native-products\.py consume/);
  assert.equal(docs.jobs.docs.env?.FARM_DOCS_NATIVE_UPLOAD, undefined);
});
