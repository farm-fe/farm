import assert from 'node:assert/strict';
import test from 'node:test';
import {planDocsTargetClosure,selectDocsRegistryPacks} from './farm-docs-target-closure.mjs';

function inventory(){
  const main=name=>({name,version:'0.0.1',kind:'workspace'});
  const platform=(name,os,cpu,libc)=>({name,version:'0.0.1',kind:'platform',os:[os],cpu:[cpu],...(libc?{libc:[libc]}:{})});
  const edge=(from,to,field='dependencies',source='repository-manifest')=>({from,to,field,source,range:'0.0.1'});
  return {roots:['@farmfe/docs'],local:[main('@farmfe/docs'),main('@farmfe/theme'),main('@farmfe/native'),platform('@farmfe/native-win32','win32','x64'),platform('@farmfe/native-linux-gnu','linux','x64','glibc'),platform('@farmfe/native-linux-musl','linux','x64','musl')],edges:[
    edge('@farmfe/docs','@farmfe/theme'),edge('@farmfe/theme','@farmfe/docs','peerDependencies'),edge('@farmfe/docs','@farmfe/native'),
    ...['win32','linux-gnu','linux-musl'].map(suffix=>edge('@farmfe/native','@farmfe/native-'+suffix,'optionalDependencies')),
    edge('@farmfe/docs','react','peerDependencies','registry-or-unresolved')
  ]};
}
const windows={platform:'win32',arch:'x64'};

test('walks dependency and peer cycles, selecting only target native platform',()=>{
  const plan=planDocsTargetClosure(inventory(),windows);
  assert.deepEqual(plan.required.map(x=>x.name),['@farmfe/docs','@farmfe/native','@farmfe/native-win32','@farmfe/theme']);
  assert.deepEqual(plan.skipped.map(x=>x.name).sort(),['@farmfe/native-linux-gnu','@farmfe/native-linux-musl']);
  assert.equal(plan.external[0].to,'react');
});

test('requires explicit Linux libc and rejects unsupported or ambiguous native selections',()=>{
  assert.throws(()=>planDocsTargetClosure(inventory(),{platform:'linux',arch:'x64'}),/libc/i);
  const plan=planDocsTargetClosure(inventory(),{platform:'linux',arch:'x64',libc:'musl'});
  assert.ok(plan.required.some(x=>x.name==='@farmfe/native-linux-musl'));
  assert.ok(!plan.required.some(x=>x.name==='@farmfe/native-linux-gnu'));
  assert.throws(()=>planDocsTargetClosure(inventory(),{platform:'win32',arch:'arm64'}),/compatible.*platform/i);
  const duplicate=inventory();duplicate.local.push({...duplicate.local.find(x=>x.name==='@farmfe/native-win32'),name:'@farmfe/native-win32-second'});duplicate.edges.push({from:'@farmfe/native',to:'@farmfe/native-win32-second',field:'optionalDependencies',source:'repository-manifest',range:'0.0.1'});
  assert.throws(()=>planDocsTargetClosure(duplicate,windows),/ambiguous/i);
});

test('rejects unresolved Farm dependencies and incomplete platform metadata',()=>{
  const missing=inventory();missing.edges.push({from:'@farmfe/docs',to:'@farmfe/missing',field:'dependencies',source:'registry-or-unresolved',range:'1.0.0'});
  assert.throws(()=>planDocsTargetClosure(missing,windows),/unresolved.*Farm/i);
  const invalid=inventory();delete invalid.local.find(x=>x.name==='@farmfe/native-win32').cpu;
  assert.throws(()=>planDocsTargetClosure(invalid,windows),/platform.*metadata/i);
});

test('registry selection requires every planned pack and exact versions without unplanned substitutes',()=>{
  const plan=planDocsTargetClosure(inventory(),windows),packs=plan.required.map(x=>({manifest:{name:x.name,version:x.version,os:x.os,cpu:x.cpu,libc:x.libc},archive:x.name+'.tgz',sha256:'fixture-shape-only'}));
  assert.equal(selectDocsRegistryPacks(plan,packs).length,4);
  assert.throws(()=>selectDocsRegistryPacks(plan,packs.slice(0,-1)),/missing.*pack/i);
  assert.throws(()=>selectDocsRegistryPacks(plan,[...packs,packs[0]]),/duplicate/i);
  assert.throws(()=>selectDocsRegistryPacks(plan,packs.map((x,i)=>i===0?{...x,manifest:{...x.manifest,version:'9.0.0'}}:x)),/version/i);
  assert.throws(()=>selectDocsRegistryPacks(plan,[...packs,{manifest:{name:'@farmfe/other',version:'1.0.0'}}]),/unplanned/i);
  assert.throws(()=>selectDocsRegistryPacks(plan,packs.map(pack=>pack.manifest.name==='@farmfe/native-win32'?{...pack,manifest:{...pack.manifest,cpu:['arm64']}}:pack)),/platform.*metadata/i);
});
