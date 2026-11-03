import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {shouldBuild,planDigest} from './formal-native-scope.mjs';
import {runFormalNativeBuild} from './formal-native-build.mjs';
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
function fixture(t) {
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'farm-formal-scope-'));t.after(()=>fs.rmSync(root,{recursive:true,force:true}));
 const abi='darwin-arm64',sourceSHA='a'.repeat(40),sourceTree='b'.repeat(40),owner='@farmfe/core';
 const env={FARM_BUILD_PROFILE:'ci',FARM_BUILD_ABI:abi,GITHUB_REPOSITORY:'farm/farm',GITHUB_REPOSITORY_ID:'1',GITHUB_RUN_ID:'2',GITHUB_RUN_ATTEMPT:'3',GITHUB_JOB:'build',GITHUB_WORKFLOW_REF:'farm/farm/.github/workflows/ci.yaml@refs/heads/main'};
 const product=Buffer.from('cffaedfe0c00000100000000','hex');
 const record={owner,file:'packages/core/binding/farm.darwin-arm64.node',bytes:product.length,sha256:sha(product),recipe:{target:'aarch64-apple-darwin'},contentKey:'c'.repeat(64)};
 const context={rustc:'rustc pinned',cargo:'cargo pinned',linker:{version:'Apple clang'},formalFlagsHash:'d'.repeat(64),platform:{sdk:'27.2',host:{node:'v22.20.0'}}};
 const runtime={...context,abi,target:record.recipe.target,platformSdk:context.platform.sdk,runtime:context.platform.host,errors:[],unboundBuildVariables:[]};
 const resolution={schemaVersion:2,acceptanceInherited:false,sourceSHA,sourceTree,abi,consumer:{...env,sourceSHA,sourceTree},graphHash:'e'.repeat(64),planHash:'f'.repeat(64),products:{[owner]:record},context,reused:{[owner]:{record}}};
 function write(file,bytes){fs.mkdirSync(path.dirname(path.join(root,file)),{recursive:true});fs.writeFileSync(path.join(root,file),bytes);}
 write(record.file,product);write(`.ci-native-proof/${abi}/runtime.json`,JSON.stringify(runtime));
 const save=()=>{resolution.planHash=planDigest({consumer:resolution.consumer,graphHash:resolution.graphHash,products:resolution.products});const bytes=Buffer.from(JSON.stringify(resolution));write(`.formal-native/${abi}/resolution.json`,bytes);env.FARM_FORMAL_RESOLUTION_HASH=sha(bytes);};save();
 return {root,env,owner,record,resolution,runtime,write,save,git:(...args)=>args.at(-1)==='HEAD'?sourceSHA:sourceTree};
}
test('verified current materialization skips only its exact owner',t=>{const f=fixture(t);assert.equal(shouldBuild(f.owner,f),false);assert.equal(shouldBuild('other',f),true);});
for(const key of ['GITHUB_RUN_ATTEMPT','GITHUB_RUN_ID','GITHUB_REPOSITORY_ID','GITHUB_JOB'])test(`old consumer ${key} cannot skip current build`,t=>{const f=fixture(t);f.env[key]='different';assert.throws(()=>shouldBuild(f.owner,f));});
test('changed measured environment prevents skip',t=>{const f=fixture(t);f.runtime.rustc='other compiler';f.write('.ci-native-proof/darwin-arm64/runtime.json',JSON.stringify(f.runtime));assert.throws(()=>shouldBuild(f.owner,f),/environment/);});
test('same-length corrupted byte prevents skip',t=>{const f=fixture(t);f.write(f.record.file,Buffer.alloc(f.record.bytes));assert.throws(()=>shouldBuild(f.owner,f));});
test('redirected payload ancestor prevents skip',t=>{const f=fixture(t);const binding=path.join(f.root,'packages/core/binding');fs.renameSync(binding,binding+'-saved');fs.symlinkSync(binding+'-saved',binding,'dir');assert.throws(()=>shouldBuild(f.owner,f),/redirect/);});
test('release never consumes PR resolution',t=>{const f=fixture(t);f.env.FARM_BUILD_PROFILE='release';assert.equal(shouldBuild(f.owner,f),true);});
test('executor performs actual environment preparation, resolution, then same formal build',t=>{
 const f=fixture(t),calls=[];
 runFormalNativeBuild({root:f.root,env:f.env,run:(command,args,options)=>{calls.push({command,args,env:options.env});return {status:0};}});
 assert.equal(calls.length,3);assert.equal(calls[0].env.FARM_NATIVE_PREPARE_ONLY,'true');assert.ok(calls[1].args.includes('resolve'));assert.equal(calls[2].env.FARM_FORMAL_RESOLUTION_HASH,f.env.FARM_FORMAL_RESOLUTION_HASH);
});
test('release executor has one source-build invocation and no resolver',t=>{
 const f=fixture(t),calls=[];f.env.FARM_BUILD_PROFILE='release';
 runFormalNativeBuild({root:f.root,env:f.env,run:(command,args)=>{calls.push(args);return {status:0};}});
 assert.deepEqual(calls,[['scripts/ci-build-rust-artifacts.sh']]);
});

test('bound resolution rejects an independently altered plan hash',t=>{const f=fixture(t);f.resolution.planHash='0'.repeat(64);const bytes=JSON.stringify(f.resolution);f.write('.formal-native/darwin-arm64/resolution.json',bytes);f.env.FARM_FORMAL_RESOLUTION_HASH=sha(bytes);assert.throws(()=>shouldBuild(f.owner,f),/plan binding/);});

for (const abi of ['linux-x64-gnu', 'linux-x64-musl']) {
 for (const profile of ['ci', 'release']) {
  test(`formal ${profile} ${abi} routes every build through reviewed Docker wrapper`, t => {
   const f=fixture(t),calls=[];
   const env={...f.env,FARM_BUILD_ABI:abi,FARM_BUILD_PROFILE:profile,FARM_BUILD_IMAGE:'reviewed-image',FARM_BUILD_TARGET:'reviewed-target',FARM_BUILD_ZIG:'false'};
   f.write(`.formal-native/${abi}/resolution.json`,JSON.stringify({reused:{}}));
   runFormalNativeBuild({root:f.root,env,run:(command,args,options)=>{calls.push({command,args,options});return {status:0};}});
   const builds=calls.filter(call=>!call.args.includes('resolve'));
   assert.equal(builds.length,profile==='ci'?2:1);
   for(const call of builds) {
    assert.equal(call.command,process.execPath);
    assert.deepEqual(call.args,['scripts/v2-docker-cargo-cache.mjs']);
    assert.equal(call.options.cwd,f.root);
    for(const key of ['FARM_BUILD_ABI','FARM_BUILD_PROFILE','FARM_BUILD_IMAGE','FARM_BUILD_TARGET','FARM_BUILD_ZIG'])assert.equal(call.options.env[key],env[key]);
   }
   if(profile==='ci') {
    assert.equal(calls[0].options.env.FARM_NATIVE_PREPARE_ONLY,'true');
    assert.ok(calls[1].args.includes('resolve'));
    assert.match(calls[2].options.env.FARM_FORMAL_RESOLUTION_HASH,/^[a-f0-9]{64}$/);
   } else assert.equal(calls.length,1);
  });
 }
}
