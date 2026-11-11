// Production workflow contracts; no hosted execution or signing is simulated here.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {execFileSync} from 'node:child_process';
import YAML from 'yaml';
const read = name => fs.readFileSync(name, 'utf8');
const workflow = name => YAML.parse(read(`.github/workflows/${name}.yaml`));
const needs = job => typeof job.needs === 'string' ? [job.needs] : job.needs || [];

test('same-source release callers match their exact checkout templates without revision substitution', () => {
  for (const scope of ['frozen', 'registry', 'native', 'cargo']) {
    const root = `scripts/release-provenance-templates/${scope}`;
    const index = JSON.parse(read(`${root}/index.json`));
    for (const name of index) {
      const template = read(`${root}/${name}`);
      assert.ok(!template.includes('__REVIEWED_SIGNER_REVISION__'));
      assert.equal(read(name), template, name);
    }
  }
  assert.equal(workflow('release-proof-call').jobs.sign.uses,
    './.github/workflows/release-provenance-signer.yaml');
  assert.equal(fs.existsSync('scripts/release-provenance-trust.json'), false);
});

test('the relative signer checks out its exact job source and exposes both authenticated identities', () => {
  const steps = workflow('release-provenance-signer').jobs.sign.steps;
  const checkout = steps.find(step => step.uses?.startsWith('actions/checkout@'));
  assert.equal(checkout.with.ref, '${{ job.workflow_sha }}');
  assert.equal(checkout.with.repository, 'farm-fe/farm');
  assert.equal(checkout.with['persist-credentials'], false);
  const prepare = steps.find(step => step.run === 'python3 -B scripts/release-provenance.py');
  assert.equal(prepare.env.SIGNER_WORKFLOW_SHA, '${{ job.workflow_sha }}');
  assert.equal(prepare.env.SIGNER_WORKFLOW_REF, '${{ job.workflow_ref }}');
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
  for (const group of ['primary','extra']) {
    const job=release.jobs['release-native-'+group];
    assert.equal(job.strategy.matrix, '${{ fromJSON(needs.plan.outputs.native_'+group+'_matrix) }}');
    assert.equal(release.jobs.plan.outputs['native_'+group+'_matrix'], '${{ steps.release-plan.outputs.native_'+group+'_matrix }}');
    assert.ok(job.if.includes("needs.plan.outputs.native_"+group+"_required == 'true'"));
    assert.equal(job.with.abi,'${{ matrix.abi }}');
    assert.equal(job.with.native_owners,'${{ matrix.native_owners }}');
    assert.equal(job.strategy['fail-fast'], false);
  }
});

test('real matrix outputs contain only exact selected owner-platform cells and omit empty groups', () => {
  const run = result => JSON.parse(execFileSync(process.env.FARM_PERFORMANCE_PYTHON??'python3', ['-B','-c',
    'import importlib.util,json,sys\ns=importlib.util.spec_from_file_location("entry","scripts/incremental-release-entry.py");m=importlib.util.module_from_spec(s);s.loader.exec_module(m)\nprint(json.dumps(m.native_matrices(json.load(sys.stdin))))'],
    {input:JSON.stringify(result),encoding:'utf8',stdio:['pipe','pipe','pipe']}));
  const cell=(owner,platform)=>({owner,platform,package:'npm:'+owner+'-'+platform});
  const result=run({nativeBuildCells:[cell('@farmfe/core','linux-x64-musl'),cell('create-farm','linux-x64-musl')],
    preparationNativeCells:[cell('@farmfe/core','linux-x64-gnu'),cell('@farmfe/core','linux-x64-gnu')]});
  assert.deepEqual(result,{native_primary:{include:[{abi:'linux-x64-gnu',native_owners:'["@farmfe/core"]'}]},
    native_extra:{include:[{abi:'linux-x64-musl',native_owners:'["@farmfe/core","create-farm"]'}]}});
  for(const matrix of Object.values(result))for(const row of matrix.include)assert.ok(JSON.parse(row.native_owners).length>0);
  assert.deepEqual(run({nativeBuildCells:[],preparationNativeCells:[]}),{native_primary:{include:[]},native_extra:{include:[]}});
  assert.throws(()=>run({nativeBuildCells:[cell('@farmfe/core','unknown-platform')]}));
});

test('public proof consumes immutable publisher outcomes without extending signer authority', () => {
  const jobs=workflow('incremental-release').jobs;
  const upload=jobs.publisher.steps.find(step=>step.id==='results-upload');
  assert.equal(upload.if,'always()');
  for(const [output,field] of [['results_id','artifact-id'],['results_digest','artifact-digest']])
    assert.equal(jobs.publisher.outputs[output],'${{ steps.results-upload.outputs.'+field+' }}');
  const download=jobs['public-proof'].steps.find(step=>step.with?.path==='publication-results');
  assert.equal(download.with['artifact-ids'],'${{ needs.publisher.outputs.results_id }}');
  assert.equal(download.with.name,undefined);
  const record=jobs['public-proof'].steps.find(step=>step.run?.includes('incremental-release-record.py registry'));
  assert.equal(record.env.EXPECTED_PUBLISHER_RESULT_ARTIFACT_ID,'${{ needs.publisher.outputs.results_id }}');
  assert.equal(record.env.EXPECTED_PUBLISHER_RESULT_ARTIFACT_DIGEST,'${{ needs.publisher.outputs.results_digest }}');
  assert.ok(record.run.includes('--publisher-results publication-results/results.json'));
  for(const name of ['sign-frozen-release','sign-public-acceptance'])assert.ok(jobs[name].if.includes("github.event_name == 'push'"));
});

// Local model of the final YAML conditions; actual Actions parsing is separate.
// Feature dispatch is allowed only as an all-skipped, non-publishing probe.
test('feature dispatch with publish false and empty producer skips every release job', () => {
  const doc = workflow('incremental-release');
  assert.deepEqual(doc.concurrency, { group: 'farm-independent-release', 'cancel-in-progress': false, queue: 'max' });
  const results = {};
  function evaluate(job) {
    const dependencies = needs(job);
    if (!/always\(\)|cancelled\(\)|success\(\)|failure\(\)/.test(job.if || '') &&
        dependencies.some(name => results[name] !== 'success')) return false;
    const expression = (job.if || 'true').replace(/^\$\{\{\s*|\s*\}\}$/g, '')
      .replace(/!cancelled\(\)|always\(\)/g, 'true')
      .replace(/needs\.([\w-]+)\.result/g, (_, name) => JSON.stringify(results[name] || 'skipped'))
      .replace(/needs\.[\w-]+\.outputs\.[\w-]+/g, '""')
      .replace(/github\.ref/g, JSON.stringify('refs/heads/codex/fix-release-signer-relative-20261110'))
      .replace(/github\.event_name/g, JSON.stringify('workflow_dispatch'))
      .replace(/github\.repository/g, JSON.stringify('farm-fe/farm'))
      .replace(/inputs\.publish/g, 'false').replace(/inputs\.producer_run/g, '""');
    assert.match(expression, /^[\s\w'"!&|=()./-]+$/);
    return Function(`return (${expression})`)();
  }
  function visit(name) {
    if (name in results) return;
    for (const dependency of needs(doc.jobs[name])) visit(dependency);
    results[name] = evaluate(doc.jobs[name]) ? 'would-run' : 'skipped';
    assert.equal(results[name], 'skipped', name);
  }
  for (const name of Object.keys(doc.jobs)) visit(name);
  assert.equal(Object.keys(results).length, Object.keys(doc.jobs).length);
});
