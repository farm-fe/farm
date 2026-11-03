import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
import test from 'node:test';
import {resolveMainRegistrySource} from './automatic-release-source.mjs';
const sha='a'.repeat(40),tree='b'.repeat(40);
function fixture(files=['.changeset/config.json','.changeset/README.md']) {
 const environment={GITHUB_REF:'refs/heads/main',GITHUB_SHA:sha,GITHUB_WORKFLOW_SHA:sha,GITHUB_EVENT_NAME:'push',GITHUB_REPOSITORY:'farm-fe/farm',GITHUB_REPOSITORY_ID:'507542208',GITHUB_WORKFLOW_REF:'farm-fe/farm/.github/workflows/incremental-release.yaml@refs/heads/main'};
 const event={ref:'refs/heads/main',after:sha,before:'c'.repeat(40)};
 const routes=[];
 const readApi=route=>{routes.push(route);assert.equal(route,'repos/farm-fe/farm');return {id:507542208,full_name:'farm-fe/farm',default_branch:'main'};};
 const runGit=args=>{if(args[0]==='ls-tree')return files.join('\n');assert.equal(args[0],'rev-parse');return args[1]==='HEAD'?sha:tree;};
 return {environment,event,readApi,runGit,routes};
}
test('fixed squash main requires no PR association or unreachable prepared anchor',()=>{const f=fixture();const result=resolveMainRegistrySource(f);assert.equal(result.releaseReady,true);assert.equal(result.sourceSHA,sha);assert.equal(f.routes.length,1);});
test('pending Changesets and prerelease state defer to version action',()=>{for(const file of ['.changeset/new.md','.changeset/pre.json'])assert.equal(resolveMainRegistrySource(fixture(['.changeset/config.json',file])).releaseReady,false);});
test('fork, PR, wrong control/source SHA, forced push and non-main are rejected',()=>{for(const field of ['GITHUB_REF','GITHUB_WORKFLOW_SHA','GITHUB_EVENT_NAME','GITHUB_REPOSITORY_ID','GITHUB_WORKFLOW_REF']){const f=fixture();f.environment[field]='wrong';assert.throws(()=>resolveMainRegistrySource(f));}const f=fixture();f.event.forced=true;assert.throws(()=>resolveMainRegistrySource(f));});
test('missing Changesets state and API failures do not authorize release',()=>{assert.throws(()=>resolveMainRegistrySource(fixture([])));const f=fixture();f.readApi=()=>{throw Error('registry unavailable');};assert.throws(()=>resolveMainRegistrySource(f));});

test('actual squash and consecutive main commits keep source binding without the deleted PR branch',()=>{
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'farm-registry-squash-'));
 try {
  const git=(...args)=>execFileSync('git',args,{cwd:root,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
  const commit=message=>git('-c','user.name=Local Fixture','-c','user.email=fixture@example.invalid','commit','-m',message);
  git('init','-b','main');fs.mkdirSync(path.join(root,'.changeset'));fs.writeFileSync(path.join(root,'.changeset/config.json'),'{}');git('add','.');commit('initial');
  const base=git('rev-parse','HEAD');git('checkout','-b','version-proposal');
  fs.writeFileSync(path.join(root,'package.json'),' {"name":"fixture","version":"2.0.0"}\n');git('add','.');commit('version proposal');
  const proposal=git('rev-parse','HEAD');git('checkout','main');git('merge','--squash','version-proposal');commit('squash versions');git('branch','-D','version-proposal');
  const merged=git('rev-parse','HEAD');assert.notEqual(merged,proposal);
  const f=fixture();f.runGit=args=>git(...args);f.environment.GITHUB_SHA=merged;f.environment.GITHUB_WORKFLOW_SHA=merged;f.event.after=merged;f.event.before=base;
  assert.equal(resolveMainRegistrySource(f).sourceTree,git('rev-parse','HEAD^{tree}'));
  fs.writeFileSync(path.join(root,'control.txt'),'new control commit');git('add','.');commit('control repair');
  const next=git('rev-parse','HEAD');assert.throws(()=>resolveMainRegistrySource(f));
  f.environment.GITHUB_SHA=next;f.environment.GITHUB_WORKFLOW_SHA=next;f.event.after=next;f.event.before=merged;
  const proof=resolveMainRegistrySource(f);assert.equal(proof.sourceSHA,next);assert.notEqual(proof.sourceSHA,merged);assert.equal(proof.releaseReady,true);
 } finally {fs.rmSync(root,{recursive:true,force:true});}
});
