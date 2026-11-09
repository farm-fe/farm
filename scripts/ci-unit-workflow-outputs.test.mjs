import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import YAML from 'yaml';
const read=name=>YAML.parse(fs.readFileSync(new URL(`../.github/workflows/${name}`,import.meta.url),'utf8'));
const unit=read('rust-test.yaml'), platform=read('rust-test-platform.yaml');
const fields=['artifact_id','artifact_digest','run_attempt'];
const settings=[['linux','ubuntu-24.04','linux-x64-gnu'],['macos','macos-latest','darwin-arm64'],['windows','windows-latest','win32-x64-msvc']];
test('Unit has three independent fixed callers, never matrix last-writer outputs',()=>{
 assert.ok(!unit.jobs['cargo-test']);
 const consumer=unit.jobs['impact-summary'];
 const env=consumer.steps.find(s=>s.run?.includes('ci-impact-summary.py')).env;
 const shapes={};
 for(const [key,os,abi] of settings){
  const id='cargo-test-'+key, caller=unit.jobs[id];
  assert.equal(caller.uses,'./.github/workflows/rust-test-platform.yaml');assert.deepEqual(caller.with,{os,abi});
  assert.ok(!caller.strategy);assert.ok(!platform.jobs['cargo-test'].strategy);
  assert.ok(consumer.needs.includes(id));assert.equal(caller.if,undefined);
  shapes[id]={artifact_id:String(100+settings.findIndex(s=>s[0]===key)),artifact_digest:key==='linux'?'a'.repeat(64):key==='macos'?'b'.repeat(64):'c'.repeat(64),run_attempt:'1'};
 }
 shapes['impact-plan']={artifact_id:'10',artifact_digest:'d'.repeat(64),run_attempt:'1'};
 shapes['cargo-check']={artifact_id:'11',artifact_digest:'e'.repeat(64),run_attempt:'2'};
 const actual=JSON.parse(env.CI_UNIT_UPLOADS.replace(/\$\{\{ toJSON\(needs\.([a-z-]+)\.outputs\) \}\}/g,(_,id)=>JSON.stringify(shapes[id])));
 assert.deepEqual(Object.keys(actual).sort(),['planner','cargo-check',...settings.map(s=>'rust-tests:ci-test:'+s[2])].sort());
 for(const [key,,abi] of settings)assert.deepEqual(actual['rust-tests:ci-test:'+abi],shapes['cargo-test-'+key]);
 assert.deepEqual(actual.planner,shapes['impact-plan']);assert.deepEqual(actual['cargo-check'],shapes['cargo-check']);
});
test('every Unit result comes from a fixed upload action and original attempt',()=>{
 assert.deepEqual(Object.keys(platform.on.workflow_call.outputs).sort(),fields.slice().sort());
 assert.deepEqual(Object.keys(platform.jobs['cargo-test'].outputs).sort(),fields.slice().sort());
 for(const field of fields)assert.equal(platform.on.workflow_call.outputs[field].value,`\${{ jobs.cargo-test.outputs.${field} }}`);
 for(const job of [platform.jobs['cargo-test'],unit.jobs['cargo-check']]){
  assert.equal(job.outputs.artifact_id,'${{ steps.receipt.outputs.artifact-id }}');
  assert.equal(job.outputs.artifact_digest,'${{ steps.receipt.outputs.artifact-digest }}');
  assert.equal(job.outputs.run_attempt,'${{ steps.receipt.outputs.run-attempt }}');
  assert.equal(job.steps.find(s=>s.id==='receipt').uses,'./.github/actions/ci-impact-cell');
 }
 const plan=unit.jobs['impact-plan'];assert.equal(plan.steps.find(s=>s.id==='plan-upload').uses,'actions/upload-artifact@v4');
 assert.equal(plan.outputs.artifact_id,'${{ steps.plan-upload.outputs.artifact-id }}');assert.equal(plan.outputs.artifact_digest,'${{ steps.plan-upload.outputs.artifact-digest }}');assert.equal(plan.outputs.run_attempt,'${{ steps.plan-origin.outputs.attempt }}');
 assert.match(plan.steps.find(s=>s.id==='plan-origin').run,/GITHUB_RUN_ATTEMPT/);
 assert.equal(platform.jobs['cargo-test'].name,'Test - ${{ inputs.os }}');
 assert.equal(platform.jobs['cargo-test'].steps.find(s=>s.id==='receipt').with.cell,'rust-tests:ci-test:${{ inputs.abi }}');
 assert.equal(platform.jobs['cargo-test'].steps.find(s=>s.id==='receipt').with['artifact-suffix'],'${{ inputs.os }}');
 assert.ok(!platform.permissions);assert.ok(!platform.jobs['cargo-test'].permissions);
});
