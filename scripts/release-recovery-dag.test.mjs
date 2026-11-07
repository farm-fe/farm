import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {parse} from 'yaml';
const text=fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml',import.meta.url),'utf8');
const jobs=parse(text).jobs;
const identity=jobs['candidate-identity'];
function selected(attempt,current){
 const folder=fs.mkdtempSync(path.join(os.tmpdir(),'release-candidate-attempt-'));
 const output=path.join(folder,'output');
 try{
  execFileSync(process.env.FARM_PERFORMANCE_PYTHON??'python3',['-c',identity.steps[0].run],{env:{PATH:process.env.PATH,CANDIDATE_ATTEMPT:String(attempt),GITHUB_RUN_ATTEMPT:String(current),GITHUB_OUTPUT:output}});
  return fs.readFileSync(output,'utf8').trim();
 }finally{fs.rmSync(folder,{recursive:true,force:true});}
}
test('actual YAML selector preserves fresh, carried and frozen producer attempts',()=>{
 assert.equal(identity.steps[0].env.CANDIDATE_ATTEMPT,"${{ needs.source.outputs.reuse == 'true' && needs.source.outputs.producer_attempt || needs.producer.outputs.candidate_attempt }}");
 for(const [producer,consumer] of [[1,1],[2,2],[2,3],[1,3]])assert.equal(selected(producer,consumer),`attempt=${producer}`);
 assert.throws(()=>selected(4,3));assert.throws(()=>selected('',3));
 assert.equal(jobs.producer.outputs.candidate_attempt,'${{ github.run_attempt }}');
 assert.equal(jobs.plan.outputs.plan_attempt,'${{ github.run_attempt }}');
});
test('carried plan and newly frozen candidate have independent attempt names',()=>{
 const plan=jobs.producer.steps.find(step=>step.uses==='actions/download-artifact@v4'&&step.with.path==='release-work');
 assert.equal(plan.with.name,'independent-release-plan-${{ github.run_id }}-${{ needs.plan.outputs.plan_attempt }}');
 const store=new Set(['independent-release-plan-50-1','farm-release-candidates-source-2']);
 assert.ok(store.has(plan.with.name.replace('${{ github.run_id }}','50').replace('${{ needs.plan.outputs.plan_attempt }}','1')));
 for(const name of ['candidate-consumers','publisher','public-consumers','public-proof','finalize']){
  assert.ok(jobs[name].needs.includes('candidate-identity'));
  if(jobs[name].uses){
   assert.equal(jobs[name].with.candidate_attempt,'${{ needs.candidate-identity.outputs.attempt }}');
   const child=parse(fs.readFileSync(new URL('../'+jobs[name].uses.replace('./',''),import.meta.url),'utf8'));
   for(const job of Object.values(child.jobs)){
    const download=job.steps.find(step=>step.uses==='actions/download-artifact@v4');
    assert.equal(download.with.name,'farm-release-candidates-${{ github.sha }}-${{ inputs.candidate_attempt }}');
    assert.ok(store.has(download.with.name.replace('${{ github.sha }}','source').replace('${{ inputs.candidate_attempt }}','2')));
   }
  }else{
   const download=jobs[name].steps.find(step=>step.uses==='actions/download-artifact@v4');
   assert.equal(download.with.name,'farm-release-candidates-${{ github.sha }}-${{ needs.candidate-identity.outputs.attempt }}');
   assert.ok(store.has(download.with.name.replace('${{ github.sha }}','source').replace('${{ needs.candidate-identity.outputs.attempt }}','2')));
  }
 }
});
test('recovery wake-up never checks out triggering code or receives publication tokens',()=>{
 const recovery=fs.readFileSync(new URL('../.github/workflows/release-evidence-recovery.yaml',import.meta.url),'utf8');const doc=parse(recovery);
 assert.deepEqual(doc.on.workflow_run.types,['completed']);assert.deepEqual(doc.on.workflow_run.branches,['main']);
 assert.equal(doc.jobs.recover.steps[0].with.ref,'${{ github.workflow_sha }}');assert.equal(doc.jobs.recover.steps[0].with['persist-credentials'],false);
 assert.equal(doc.jobs.recover.permissions.actions,'write');assert.equal(doc.jobs.recover.permissions.contents,'read');
 assert.ok(!/NPM_TOKEN|CARGO_REGISTRY_TOKEN|pull_request_target/.test(recovery));
 assert.equal(doc.concurrency['cancel-in-progress'],false);
 const script=fs.readFileSync(new URL('./release-evidence-recovery.py',import.meta.url),'utf8');assert.ok(script.includes('/rerun-failed-jobs'));assert.ok(!script.includes('/dispatches'));
});
test('all release needs remain acyclic and mandatory candidate checks cannot be skipped',()=>{
 const done=new Set(),active=new Set();function visit(name){assert.ok(jobs[name]);assert.ok(!active.has(name));if(done.has(name))return;active.add(name);for(const need of jobs[name].needs??[])visit(need);active.delete(name);done.add(name);}
 for(const name of Object.keys(jobs))visit(name);
 for(const name of ['candidate-consumers','publisher'])assert.ok(jobs[name].if.includes("needs.candidate-identity.result == 'success'"));
 assert.ok(identity.if.includes("needs.source.outputs.reuse == 'true'"));
});
