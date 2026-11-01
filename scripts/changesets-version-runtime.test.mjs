import test from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {resolveVersionEvidence,resolveMergedVersion} from './changesets-version-runtime.mjs';
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
function fixture(){
 const base='b'.repeat(40),head='a'.repeat(40),tree='c'.repeat(40),main='d'.repeat(40);
 const before={'packages/a/package.json':JSON.stringify({name:'@farm/a',version:'1.0.0'}),'.changeset/one.md':'---\n"@farm/a": patch\n---\nFix\n'},after={'packages/a/package.json':JSON.stringify({name:'@farm/a',version:'1.0.1'})};
 const receipt={kind:'changesets-version-preparation',repositoryId:1,pr:5,baseSHA:base,headSHA:head,headTree:tree,runId:10,attempt:2,consumed:{'.changeset/one.md':hash(before['.changeset/one.md'])}};
 const bytes=Buffer.from(JSON.stringify(receipt)+'\n'),archive=Buffer.from('API archive fixture');
 const repo={id:1,default_branch:'main'},pr={number:5,merged_at:'2026-11-01T10:01:00Z',merge_commit_sha:main,user:{id:41898282,type:'Bot',login:'github-actions[bot]'},base:{sha:base,ref:'main',repo:{id:1}},head:{sha:head,repo:{id:1}}};
 const run={id:10,run_attempt:2,head_sha:base,head_branch:'main',path:'.github/workflows/release.yaml',event:'workflow_run',status:'completed',conclusion:'success',repository:{id:1},head_repository:{id:1}};
 const artifact={id:20,workflow_run:{id:10},expired:false,size_in_bytes:archive.length,digest:'sha256:'+hash(archive),created_at:'2026-11-01T10:00:01Z'};
 const job={run_id:10,name:'Prepare reviewed version PR',status:'completed',conclusion:'success',steps:[{name:'Upload version-only provenance',conclusion:'success',started_at:'2026-11-01T10:00:00Z',completed_at:'2026-11-01T10:00:02Z'}]};
 const calls=[];
 const readApi=route=>{
  calls.push(route);
  if(route==='repos/farm/farm')return repo;
  if(route==='repos/farm/farm/pulls/5')return pr;
  if(route.includes('/commits/'))return [pr];
  if(route.includes('/workflows/'))return {workflow_runs:[{id:10,run_attempt:2}]};
  if(route.endsWith('/actions/runs/10'))return run;
  if(route.includes('/artifacts?'))return {artifacts:[artifact]};
  if(route.endsWith('/artifacts/20'))return artifact;
  if(route.includes('/jobs?'))return {jobs:[job]};
  throw Error('Unexpected API request '+route);
 };
 const environment={GITHUB_REPOSITORY:'farm/farm',GITHUB_SHA:head,VERSION_PR:'5'};
 const options={environment,event:{pull_request:structuredClone(pr)},readApi,runGit:args=>args[0]==='fetch'?'':tree,
  readSnapshots:()=>({before,after,modesBefore:Object.fromEntries(Object.keys(before).map(k=>[k,'100644'])),modesAfter:Object.fromEntries(Object.keys(after).map(k=>[k,'100644']))}),
  download:()=>archive,extractReceipt:()=>bytes,wait:async()=>{}};
 return {options,pr,run,artifact,job,calls,tree,main,environment};
}
test('API replay grants exact trusted version PR and can repeat without writes',async()=>{
 const f=fixture();for(let i=0;i<2;i++)assert.equal((await resolveVersionEvidence(f.options)).mode,'verified-version-only');
 assert.ok(f.calls.every(route=>!route.includes('dispatch')&&!route.includes('approval')));
});
test('latest failed producer does not fall back to old successful evidence',async()=>{const f=fixture();f.run.conclusion='failure';const r=await resolveVersionEvidence(f.options);assert.equal(r.mode,'full');assert.ok(!f.calls.some(route=>route.includes('/artifacts')));});
test('expired provenance retains ordinary CI',async()=>{const f=fixture();f.artifact.expired=true;assert.equal((await resolveVersionEvidence(f.options)).mode,'full');});
test('in-flight producer must become complete before allowing a fastlane',async()=>{
 const f=fixture();f.run.status='in_progress';let waits=0;f.options.wait=async()=>{waits++;f.run.status='completed';};
 assert.equal((await resolveVersionEvidence(f.options)).mode,'verified-version-only');assert.equal(waits,1);
});
test('moved head is rejected before proof download',async()=>{const f=fixture();f.pr.head.sha='e'.repeat(40);await assert.rejects(resolveVersionEvidence(f.options),/moved/);assert.ok(!f.calls.some(route=>route.includes('/artifacts')));});
test('unclassified source mixed into version PR uses ordinary CI',async()=>{const f=fixture(),read=f.options.readSnapshots;f.options.readSnapshots=()=>{const data=read();data.after['packages/a/src/index.ts']='source';data.modesAfter['packages/a/src/index.ts']='100644';return data;};assert.equal((await resolveVersionEvidence(f.options)).mode,'full');});
test('release accepts only exact merged version tree and keeps publish separate',async()=>{
 const f=fixture();const result=await resolveMergedVersion({environment:{...f.environment,GITHUB_REF:'refs/heads/main',GITHUB_SHA:f.main},readApi:f.options.readApi,
 runGit:args=>args.at(-1)==='HEAD'?f.main:f.tree,resolve:options=>resolveVersionEvidence({...f.options,...options,runGit:f.options.runGit})});
 assert.equal(result.releaseReady,true);assert.equal(result.sourceSHA,f.main);assert.equal(result.publish,undefined);
});
test('ordinary main commit does not release merely because there are no Changesets',async()=>{
 const f=fixture();f.pr.merge_commit_sha='f'.repeat(40);const result=await resolveMergedVersion({environment:{...f.environment,GITHUB_REF:'refs/heads/main',GITHUB_SHA:f.main},readApi:f.options.readApi,runGit:()=>f.main});assert.equal(result.releaseReady,false);
});
test('merged tree drift blocks release even with version PR provenance',async()=>{
 const f=fixture();await assert.rejects(resolveMergedVersion({environment:{...f.environment,GITHUB_REF:'refs/heads/main',GITHUB_SHA:f.main},readApi:f.options.readApi,
 runGit:args=>args.at(-1)==='HEAD'?f.main:'f'.repeat(40),resolve:async()=>({mode:'verified-version-only',receiptHash:'a'.repeat(64),headTree:f.tree})}),/tree differs/);
});
