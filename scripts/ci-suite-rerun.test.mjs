import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import test from 'node:test';
import fs from 'node:fs';
import {validateCollectedSuites, receiptIdentity} from './ci-suite-execution.mjs';
import {makeSuiteExecutionPlan} from './ci-suite-inventory.mjs';

function fixture() {
  const raw={schemaVersion:1,protocol:'farm-suite-discovery-v1',fileEnumerationComplete:true,selectionReady:false,fullOnlyReasons:[],sourceIdentity:{commit:'a'.repeat(40),tree:'b'.repeat(40)},items:[{id:'ts:packages/a/a.spec.ts',kind:'ts',requiresActualCases:true},{id:'examples-browser:examples/react',kind:'examples-browser',requiresActualCases:true}]};
  const inventory={...raw,inventoryHash:crypto.createHash('sha256').update(JSON.stringify(raw)).digest('hex')};
  const env={GITHUB_ACTIONS:'true',GITHUB_RUN_ID:'10',GITHUB_RUN_ATTEMPT:'3',GITHUB_REPOSITORY_ID:'5',GITHUB_SHA:'a'.repeat(40),GITHUB_JOB:'performance-summary',GITHUB_WORKFLOW_REF:'farm-fe/farm/.github/workflows/ci.yaml@refs/pull/2330/merge',GITHUB_EVENT_NAME:'pull_request'};
  const rows=['linux-x64-gnu','darwin-arm64','win32-x64-msvc'].flatMap(abi=>['ts','examples-browser'].map(kind=>{
    const plan=makeSuiteExecutionPlan(inventory,kind),original={...env,GITHUB_RUN_ATTEMPT:'1',FARM_SUITE_ABI:abi,GITHUB_JOB:kind==='ts'?'ts-test':'examples-test'};
    return {artifactName:`farm-ci-suite-${kind==='ts'?'ts':'browser'}-${env.GITHUB_SHA}-${abi}-1`,data:{...receiptIdentity(inventory,original),schemaVersion:1,receiptKind:'actual-suite-execution-not-byte-reuse',kind,inventoryHash:inventory.inventoryHash,planHash:plan.planHash,selectedIds:plan.ids,executedIds:plan.ids,caseCounts:plan.ids.map(id=>({id,executedCases:1})),executedCases:1,failed:0,skipped:0}};
  }));
  const resolution={schemaVersion:1,sourceSHA:env.GITHUB_SHA,sourceTree:inventory.sourceIdentity.tree,runId:10,aggregationAttempt:3,selectionAttempt:null,producers:Object.fromEntries(rows.map((row,i)=>[row.artifactName,{runAttempt:1,jobId:200+i,artifactId:100+i,artifactDigest:'sha256:'+'d'.repeat(64),profile:'ci',workflowPath:'.github/workflows/ci.yaml'}]))};
  return {inventory,env,rows,resolution};
}

test('attempt3 gate consumes original six attempt1 names and records original producer without inheriting acceptance',()=>{
  const f=fixture(),coverage=validateCollectedSuites(f.inventory,f.rows,f.env,f.resolution);
  assert.equal(coverage.run.runAttempt,3);
  assert.ok(coverage.cells.every(row=>row.run.runAttempt===1 && row.artifactName.endsWith('-1') && row.producerBinding.runAttempt===1 && row.acceptanceInherited===false));
  assert.throws(()=>validateCollectedSuites(f.inventory,f.rows,f.env),/runAttempt/);
});

test('renamed receipt, changed plan/source/profile, unknown skipped cases and future producer reject',()=>{
  for(const mutate of [f=>{f.rows[0].artifactName=f.rows[0].artifactName.replace(/-1$/,'-3');},f=>{f.rows[0].data.planHash='0'.repeat(64);},f=>{f.rows[0].data.sourceTree='0'.repeat(40);},f=>{f.resolution.producers[f.rows[0].artifactName].profile='release';},f=>{f.rows[0].data.skipped=1;},f=>{f.resolution.producers[f.rows[0].artifactName].runAttempt=4;}]){
    const f=fixture();mutate(f);assert.throws(()=>validateCollectedSuites(f.inventory,f.rows,f.env,f.resolution));
  }
});

test('both CI consumers resolve original receipts before count verification; no current-attempt wildcard remains',()=>{
  const yaml=fs.readFileSync(new URL('../.github/workflows/ci.yaml',import.meta.url),'utf8');
  assert.equal((yaml.match(/scripts\/ci-suite-attempts.py --output/g)||[]).length,2);
  assert.ok(!yaml.includes('pattern: farm-ci-suite-*-${{ github.sha }}-*-${{ github.run_attempt }}'));
  assert.ok(yaml.includes('python scripts/ci-suite-attempts.test.py'));
});


test('all six native restore sites have complete checkout history before source-reader parent proof',()=>{
  let restores=0;
  for(const file of ['ci.yaml','ci-primary-platform.yaml']){
    const yaml=fs.readFileSync(new URL('../.github/workflows/'+file,import.meta.url),'utf8').split('\njobs:\n')[1];
    const jobs=yaml.split(/\n(?=  [a-z][a-z0-9-]*:)/);
    for(const block of jobs){
      const matches=[...block.matchAll(/scripts\/ci-leaf-native.py restore/g)];
      for(const found of matches){
        restores++;
        const before=block.slice(0,found.index),checkout=before.lastIndexOf('      - uses: actions/checkout@');
        assert.ok(checkout>=0,'Restore has no checkout');
        const step=before.slice(checkout).split('\n      - ')[0];assert.match(step,/fetch-depth: 0/);
      }
    }
  }
  assert.equal(restores,6);
  assert.ok(fs.readFileSync(new URL('../.github/workflows/ci.yaml',import.meta.url),'utf8').includes('python scripts/ci-restore-depth.test.py'));
});
