import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import YAML from 'yaml';
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
test('Docs starts after its own native producer without a cross-platform or Examples barrier',()=>{
 const primary=workflow('ci-primary-platform'),docs=primary.jobs.docs;
 assert.equal(docs.uses,'./.github/workflows/farm-docs.yaml');assert.deepEqual(docs.needs,['native','native-disposition']);
 assert.match(docs.if,/needs.native-disposition.result == 'success'/);assert.match(docs.if,/inputs.abi != 'darwin-arm64'/);
 assert.equal(docs.with.consume_native,true);assert.match(docs.with.platforms,/inputs.os, inputs.abi/);
 assert.deepEqual(workflow('ci').jobs['docs-acceptance'].needs,['version-classification','primary-linux','primary-windows']);
 const quality=workflow('farm-docs');assert.equal(quality.permissions.actions,'read');
 for(const name of ['Build public JS packages','Check native content, blog planning and development invalidation','Check real MDX and snippets highlighted SSR equivalence','Build the real website with its independent Farm configuration','Exercise public theme, runtime, and full blog build/dev behavior']) assert.ok(quality.jobs.docs.steps.some(step=>step.name===name),name);
 const producer=workflow('rust-build-user').jobs.build.steps;
 assert.ok(producer.some(step=>step.name==='Seal same-run Docs native products'&&!step['continue-on-error']));
 assert.ok(producer.some(step=>step.name==='Upload same-run Docs native products'&&step.with['if-no-files-found']==='error'));
});
