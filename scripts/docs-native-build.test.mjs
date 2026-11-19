import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import YAML from 'yaml';
import { spawnSync } from 'node:child_process';
import {buildDocsNative,plugins} from './docs-native-build.mjs';
const hash=body=>crypto.createHash('sha256').update(body).digest('hex');
function fixture(missing=[]) {
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'docs-native-'));
 const plan={schemaVersion:1,sourceSHA:'a'.repeat(40),sourceTree:'b'.repeat(40),abi:'darwin-arm64',missing,reused:{}};
 for(const owner of ['@farmfe/core',...plugins.map(name=>'@farmfe/plugin-'+name)]) {
  if(missing.includes(owner))continue;
  const file=owner==='@farmfe/core'?'packages/core/binding/farm.darwin-arm64.node':`rust-plugins/${owner.slice('@farmfe/plugin-'.length)}/npm/darwin-arm64/index.farm`;
  fs.mkdirSync(path.dirname(path.join(root,file)),{recursive:true});fs.writeFileSync(path.join(root,file),owner);
  plan.reused[owner]={file,sha256:hash(owner)};
 }
 const bytes=JSON.stringify(plan);fs.writeFileSync(path.join(root,'plan.json'),bytes);
 const calls=[];return {root,plan,calls,options:{root,platform:'darwin',arch:'arm64',env:{FARM_DOCS_PLAN_HASH:hash(bytes)},git:(...args)=>args.at(-1)==='HEAD'?'a'.repeat(40):'b'.repeat(40),run:(...args)=>{calls.push(args);return {status:0};}}};
}
for(const missing of [[],['@farmfe/plugin-docs'],['@farmfe/core',...plugins.map(name=>'@farmfe/plugin-'+name)]])test(`build exactly ${missing.length} missing owners`,()=>{
 const f=fixture(missing);try {
  const result=buildDocsNative('plan.json',f.options);assert.deepEqual(result.built,missing);assert.equal(f.calls.length,missing.length?1:0);
  if(missing.length) {assert.deepEqual(f.calls[0].slice(0,2),['bash',['scripts/ci-build-rust-artifacts.sh']]);assert.deepEqual(JSON.parse(f.calls[0][2].env.FARM_RELEASE_NATIVE_OWNERS),missing);assert.equal(f.calls[0][2].env.FARM_FORMAL_RESOLUTION_HASH,'');assert.equal(f.calls[0][2].env.FARM_BUILD_PROFILE,'release');}
 }finally{fs.rmSync(f.root,{recursive:true,force:true});}
});
for(const kind of ['plan','bytes','source','platform','build-failure'])test(`reject ${kind} drift`,()=>{
 const f=fixture(['@farmfe/plugin-docs']);try{
  if(kind==='plan')fs.appendFileSync(path.join(f.root,'plan.json'),' ');
  if(kind==='bytes')fs.appendFileSync(path.join(f.root,f.plan.reused['@farmfe/core'].file),'changed');
  if(kind==='source')f.options.git=()=> 'c'.repeat(40);
  if(kind==='platform')f.options.platform='win32';
  if(kind==='build-failure')f.options.run=()=>({status:1});
  assert.throws(()=>buildDocsNative('plan.json',f.options));
 }finally{fs.rmSync(f.root,{recursive:true,force:true});}
});
const workflow=name=>YAML.parse(fs.readFileSync(new URL(`../.github/workflows/${name}.yaml`,import.meta.url),'utf8'));
test('only Linux and Windows call Docs with their own verified native producer',()=>{
 const ci=workflow('ci'),primary=workflow('ci-primary-platform');
 assert.equal(primary.jobs.docs,undefined);
 assert.equal(primary.on.workflow_call.outputs.native_docs_upload.value,'${{ jobs.native.outputs.docs_upload }}');
 const calls=Object.entries(ci.jobs).filter(([,job])=>job.uses==='./.github/workflows/farm-docs.yaml');
 assert.deepEqual(calls.map(([id])=>id),['docs-linux','docs-windows']);
 for(const [platform,label,os,abi] of [['linux','Linux','ubuntu-24.04','linux-x64-gnu'],['windows','Windows','windows-latest','win32-x64-msvc']]) {
  const docs=ci.jobs['docs-'+platform];
  assert.equal(docs.name,'Primary '+label+' / Docs CI quality checks');
  assert.deepEqual(docs.needs,['primary-'+platform]);
  assert.equal(docs.if,"always() && needs.primary-"+platform+".result == 'success'");
  assert.deepEqual(docs.permissions,{contents:'read',actions:'read'});
  assert.equal(docs.with.consume_native,true);
  assert.deepEqual(JSON.parse(docs.with.platforms),[{os,abi}]);
  assert.equal(docs.with.native_upload,'${{ needs.primary-'+platform+'.outputs.native_docs_upload }}');
 }
 assert.equal(ci.jobs['docs-acceptance'].name,'Docs CI quality checks');
 assert.deepEqual(ci.jobs['docs-acceptance'].needs,['version-classification','docs-linux','docs-windows']);
 const gate=ci.jobs['docs-acceptance'].steps[0];
 assert.equal(gate.env.LINUX_RESULT,'${{ needs.docs-linux.result }}');
 assert.equal(gate.env.WINDOWS_RESULT,'${{ needs.docs-windows.result }}');
 for(const status of ['success','failure','cancelled','skipped','']) for(const platform of ['LINUX','WINDOWS']) {
  const result=spawnSync('bash',['-c',gate.run],{env:{...process.env,LINUX_RESULT:'success',WINDOWS_RESULT:'success',[platform+'_RESULT']:status}});
  assert.equal(result.status===0,status==='success');
 }
 const visit=(id,stack=[])=>{assert.ok(!stack.includes(id),'dependency cycle '+id);for(const dep of ci.jobs[id].needs??[])visit(dep,[...stack,id]);};
 for(const id of Object.keys(ci.jobs)) visit(id);
 const quality=workflow('farm-docs');assert.equal(quality.permissions.actions,'read');
 for(const name of ['Build public JS packages','Check native content, blog planning and development invalidation','Check real MDX and snippets highlighted SSR equivalence','Build the real website with its independent Farm configuration','Exercise public theme, runtime, and full blog build/dev behavior']) assert.ok(quality.jobs.docs.steps.some(step=>step.name===name),name);
 const producer=workflow('rust-build-user').jobs.build.steps;
 assert.ok(producer.some(step=>step.name==='Seal same-run Docs native products'&&!step['continue-on-error']));
 assert.ok(producer.some(step=>step.name==='Upload same-run Docs native products'&&step.with['if-no-files-found']==='error'));
});
