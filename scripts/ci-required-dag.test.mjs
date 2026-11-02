import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
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

const compatibility=workflow.jobs['type-changesets-compatibility'];
const compatibilityRun=rows=>spawnSync('python3',['-c',compatibility.steps[0].run],{encoding:'utf8',env:{...process.env,RESULTS:JSON.stringify(rows)}});
test('type and Changesets policy are independent jobs with the old strict required context preserved',()=>{
  const types=workflow.jobs['type-check'],changesets=workflow.jobs['changesets-check'];
  assert.equal(types.name,'Type Check');assert.equal(changesets.name,'Changesets Check');
  assert.equal(compatibility.name,'Type Check & Changesets');assert.equal(compatibility.if,'always()');
  assert.deepEqual(changesets.needs,['version-classification']);
  assert.ok(!JSON.stringify(types).includes('check-pr-changesets.mjs'));
  assert.ok(changesets.steps.some(step=>step.run==='node scripts/check-pr-changesets.mjs'));
  assert.ok(changesets.steps.some(step=>step.run==='pnpm install --frozen-lockfile --ignore-scripts'));
  assert.ok(!types['continue-on-error']&&!changesets['continue-on-error']);
  assert.ok(!types.steps.find(step=>step.name==='Type Check With Tsc')['continue-on-error']);
  assert.ok(!changesets.steps.find(step=>step.run==='node scripts/check-pr-changesets.mjs')['continue-on-error']);
  assert.ok(gate.needs.includes('changesets-check'),'Ordinary PR policy failure remains a required gate failure');
});
for(const [label,typeResult,changesetResult,expected]of [
  ['both successful','success','success',3],
  ['actual type error','failure','success',0],
  ['approved consumed Changesets policy failure','success','failure',3],
  ['ordinary missing Changeset policy failure','success','failure',3]
])test(`actual workflow schedules product plans for ${label}`,()=>{
  const rows={...full(),'release-contract-tests':{result:'success'},'performance-contract-tests':{result:'success'}};
  rows['type-check'].result=typeResult;rows['changesets-check'].result=changesetResult;
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
  const combined=compatibilityRun(Object.fromEntries(compatibility.needs.map(key=>[key,rows[key]])));
  assert.equal(combined.status===0,typeResult==='success'&&changesetResult==='success');
  if(changesetResult==='failure'){
    const outcomes=full();outcomes['changesets-check'].result='failure';
    assert.notEqual(run(outcomes).status,0,'Running product tests must not erase Changesets policy failure');
  }
});
test('old required context rejects incomplete or cancelled checks and accepts only proven metadata skips',()=>{
  for(const key of ['version-classification','type-check','changesets-check'])for(const result of ['failure','skipped','cancelled']){
    const rows=Object.fromEntries(compatibility.needs.map(name=>[name,{result:'success',outputs:{mode:'full'}}]));rows[key].result=result;
    assert.notEqual(compatibilityRun(rows).status,0);
  }
  const rows=Object.fromEntries(compatibility.needs.map(key=>[key,version()[key]]));
  assert.equal(compatibilityRun(rows).status,0);delete rows['version-classification'].outputs.proof_hash;
  assert.notEqual(compatibilityRun(rows).status,0);
});
