import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import crypto from 'node:crypto';
import {receiptIdentity,validateCollectedSuites} from './ci-suite-execution.mjs';
import {makeSuiteExecutionPlan} from './ci-suite-inventory.mjs';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';
import {fileURLToPath} from 'node:url';
import {readGitSuiteFiles, discoverSuiteInventory, suitePolicy} from './ci-suite-inventory.mjs';
import {executionPlan} from './ci-suite-selection.mjs';
const workflow=YAML.parse(fs.readFileSync(new URL('../.github/workflows/ci.yaml',import.meta.url),'utf8'));
const gate=workflow.jobs['incremental-execution-gate'];
const code=gate.steps[0].run;
const full=()=>Object.fromEntries(gate.needs.map(key=>[key,{result:'success',outputs:{mode:'full'}}]));
const run=rows=>spawnSync('python3',['-c',code],{encoding:'utf8',env:{...process.env,RESULTS:JSON.stringify(rows)}});
test('actual always gate accepts every completed planned platform',()=>{assert.equal(gate.if,'always()');assert.equal(run(full()).status,0);});
for(const state of ['failure','cancelled','skipped'])for(const key of gate.needs)test(`actual gate rejects ${key} ${state}`,()=>{const rows=full();rows[key].result=state;assert.notEqual(run(rows).status,0);});
function version(){const rows=full();for(const [key,row]of Object.entries(rows))if(!['version-classification','type-check'].includes(key))row.result='skipped';rows['version-classification'].outputs={mode:'verified-version-only',proof_hash:'a'.repeat(64)};return rows;}
test('verified version PR has explicit lightweight success and planned skips',()=>assert.equal(run(version()).status,0));
for(const mutation of ['missing','extra','proof','type-failed','unexpected-heavy'])test(`version gate rejects ${mutation}`,()=>{const rows=version();if(mutation==='missing')delete rows['primary-linux'];if(mutation==='extra')rows.unexpected={result:'skipped'};if(mutation==='proof')rows['version-classification'].outputs.proof_hash='z'.repeat(64);if(mutation==='type-failed')rows['type-check'].result='failure';if(mutation==='unexpected-heavy')rows['primary-linux'].result='success';assert.notEqual(run(rows).status,0);});
test('each primary platform advances its own native and suites without other platform dependencies',()=>{for(const key of ['primary-linux','primary-macos','primary-windows'])assert.ok(!workflow.jobs[key].needs.some(name=>name.startsWith('primary-')));const primary=YAML.parse(fs.readFileSync(new URL('../.github/workflows/ci-primary-platform.yaml',import.meta.url),'utf8'));assert.ok(Object.values(primary.jobs).some(job=>job.uses?.includes('rust-build-user')));assert.ok(Object.values(primary.jobs).filter(job=>job.uses?.includes('test')).every(job=>!JSON.stringify(job.needs).includes('primary-linux')));});

test('type and Changesets policy remain independent actual checks',()=>{
  const types=workflow.jobs['type-check'],changesets=workflow.jobs['changesets-check'];
  assert.equal(types.name,'Type Check');assert.equal(changesets.name,'Changesets Check');
  assert.deepEqual(changesets.needs,['version-classification']);
  assert.ok(!JSON.stringify(types).includes('check-pr-changesets.mjs'));
  assert.ok(changesets.steps.some(step=>step.run==='node scripts/check-pr-changesets.mjs'));
  assert.ok(changesets.steps.some(step=>step.run==='pnpm install --frozen-lockfile --ignore-scripts'));
  assert.ok(!types['continue-on-error']&&!changesets['continue-on-error']);
  assert.ok(!types.steps.find(step=>step.name==='Type Check With Tsc')['continue-on-error']);
  assert.ok(!changesets.steps.find(step=>step.run==='node scripts/check-pr-changesets.mjs')['continue-on-error']);
  assert.ok(!gate.needs.includes('changesets-check'),'Policy must not suppress actual suite evidence validation');
});
for(const [label,typeResult,changesetResult,expected]of [
  ['both successful','success','success',3],
  ['actual type error','failure','success',0],
  ['approved consumed Changesets policy failure','success','failure',3],
  ['ordinary missing Changeset policy failure','success','failure',3]
])test(`actual workflow schedules product plans for ${label}`,()=>{
  const rows={...full(),'release-contract-tests':{result:'success'},'platform-contract-tests':{result:'success'}};
  rows['type-check'].result=typeResult;rows['changesets-check']={result:changesetResult};
  const scheduled=['primary-linux','primary-macos','primary-windows'].filter(key=>{
    const job=workflow.jobs[key];
    assert.equal(job.if,"needs.version-classification.outputs.mode != 'verified-version-only'");
    assert.ok(job.needs.includes('type-check'));assert.ok(!job.needs.includes('changesets-check'));
    assert.ok(!job.needs.includes('type-changesets-compatibility'));
    return job.needs.every(dependency=>rows[dependency].result==='success');
  });
  assert.equal(scheduled.length,expected);
  const snapshot=readGitSuiteFiles(fileURLToPath(new URL('../',import.meta.url)));
  const inventory=discoverSuiteInventory(snapshot.files,suitePolicy,snapshot.sourceIdentity,snapshot.exampleDirectories);
  for(const kind of ['ts','examples-browser']){
    const plan=executionPlan(inventory,kind,undefined,{});
    assert.ok(plan.ids.length>0);assert.equal(plan.mode,'all');
    assert.equal(scheduled.length*plan.ids.length,expected*plan.ids.length);
  }
  if(changesetResult==='failure')assert.equal(run(full()).status,0,'Independent policy failure must not suppress suite collection');
});

test('every real product summary is independent of policy while evidence validation remains mandatory',()=>{
  for(const key of ['incremental-execution-gate','examples-test','ts-test','docs-acceptance','execution-baseline']) {
    const seen=new Set();const visit=name=>{if(seen.has(name))return;seen.add(name);for(const dependency of workflow.jobs[name].needs??[])visit(dependency);};visit(key);
    assert.ok(!seen.has('changesets-check'),key);assert.ok(!seen.has('type-changesets-compatibility'),key);
  }
  for(const name of ['Resolve original successful suite attempts without rebuilding','Validate actual selected suite files and case counts on every platform']) {
    const step=gate.steps.find(step=>step.name===name);assert.ok(step);assert.equal(step.if,"needs.version-classification.outputs.mode != 'verified-version-only'");assert.ok(!step['continue-on-error']);
  }
});
for(const scenario of ['policy-failure','suite-failure','missing-evidence','missing-file','wrong-case-count','type-failure'])test(`actual product gate and collection propagate ${scenario} correctly without legacy contexts`,()=>{
  const raw={schemaVersion:1,protocol:'farm-suite-discovery-v1',fileEnumerationComplete:true,selectionReady:false,fullOnlyReasons:[],sourceIdentity:{commit:'a'.repeat(40),tree:'b'.repeat(40)},
    items:[{id:'ts:packages/a/a.spec.ts',kind:'ts',requiresActualCases:true},{id:'examples-browser:examples/react',kind:'examples-browser',requiresActualCases:true}]};
  const inventory={...raw,inventoryHash:crypto.createHash('sha256').update(JSON.stringify(raw)).digest('hex')};
  const env={GITHUB_ACTIONS:'true',GITHUB_RUN_ID:'1',GITHUB_RUN_ATTEMPT:'2',GITHUB_REPOSITORY_ID:'3',GITHUB_SHA:'a'.repeat(40),GITHUB_JOB:'incremental-execution-gate',
    GITHUB_WORKFLOW_REF:'farm-fe/farm/.github/workflows/ci.yaml@refs/pull/2330/merge',GITHUB_EVENT_NAME:'pull_request'};
  const receipts=['linux-x64-gnu','darwin-arm64','win32-x64-msvc'].flatMap(abi=>['ts','examples-browser'].map(kind=>{
    const plan=makeSuiteExecutionPlan(inventory,kind);
    return {artifactName:`farm-ci-suite-${kind==='ts'?'ts':'browser'}-${env.GITHUB_SHA}-${abi}-2`,data:{...receiptIdentity(inventory,{...env,FARM_SUITE_ABI:abi,GITHUB_JOB:kind==='ts'?'ts-test':'examples-test'}),
      schemaVersion:1,receiptKind:'actual-suite-execution-not-byte-reuse',kind,inventoryHash:inventory.inventoryHash,planHash:plan.planHash,selectedIds:plan.ids,executedIds:plan.ids,
      executedCases:1,caseCounts:plan.ids.map(id=>({id,executedCases:1})),failed:0,skipped:0}};
  }));
  const rows=full();if(scenario==='type-failure')rows['type-check'].result='failure';
  if(scenario==='suite-failure')receipts[0].data.failed=1;
  if(scenario==='missing-evidence')receipts.pop();
  if(scenario==='missing-file')receipts[0].data.executedIds=[];
  if(scenario==='wrong-case-count')receipts[0].data.executedCases=2;
  let collected=false,accepted=false;
  if(run(rows).status===0){collected=true;try{accepted=validateCollectedSuites(inventory,receipts,env).cells.length===6;}catch{accepted=false;}}
  assert.equal(collected,scenario!=='type-failure');assert.equal(accepted,scenario==='policy-failure');
});

test('obsolete compatibility jobs are absent and all workflow dependencies still resolve',()=>{
  const unit=YAML.parse(fs.readFileSync(new URL('../.github/workflows/rust-test.yaml',import.meta.url),'utf8'));
  for(const key of ['type-changesets-compatibility','required-examples-compatibility','required-typescript-compatibility'])assert.ok(!workflow.jobs[key]);
  assert.ok(!unit.jobs['required-linux-test-compatibility']);
  for(const value of [workflow,unit])for(const job of Object.values(value.jobs)) {
    const needs=typeof job.needs==='string'?[job.needs]:job.needs??[];
    for(const dependency of needs)assert.ok(value.jobs[dependency],dependency);
  }
  for(const key of ['type-check','changesets-check','primary-linux','primary-macos','primary-windows','call-rust-build-release','check-native-candidate-assembly','examples-test','ts-test','docs-acceptance','execution-baseline','incremental-execution-gate'])assert.ok(workflow.jobs[key],key);
  assert.ok(unit.jobs['cargo-check'].steps.some(step=>step.run?.includes('cargo check')));
  assert.ok(unit.jobs['cargo-test'].steps.some(step=>step.run?.includes('cargo test')));
});
