import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import test from 'node:test';
import {verifyDocsPackageDirectory} from './farm-docs-package-contract.mjs';

async function fixture(t) {
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'farm-docs-package-contract-'));
  t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const manifest={name:'@farmfe/docs',version:'0.0.1',type:'module',license:'MIT',exports:{'.':'./dist/index.js'},dependencies:{'@farmfe/core':'2.0.0'}};
  const original={...manifest,dependencies:{'@farmfe/core':'workspace:*'}};
  const put=async(name,body)=>{const file=path.join(root,name);await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,body);};
  for(const [name,body] of Object.entries({'package.json':JSON.stringify(manifest),'dist/index.js':'export {};','README.md':'Docs','LICENSE':'MIT text','schema/docs.schema.json':'{}','client.html':'<div id="root"></div>'}))await put(name,body);
  return {root,manifest,original,put,options:{original,versions:new Map([['@farmfe/core','2.0.0']]),requiredFiles:['schema/docs.schema.json','client.html']}};
}

test('records actual public package resources and hashes without loading code',async t=>{
  const f=await fixture(t);
  await f.put('dist/index.js','throw new Error("must never execute");');
  const receipt=await verifyDocsPackageDirectory(f.root,f.options);
  assert.equal(receipt.name,'@farmfe/docs');
  assert.equal(receipt.files.find(x=>x.path==='client.html').sha256,createHash('sha256').update('<div id="root"></div>').digest('hex'));
  assert.equal(receipt.files.length,6);
});

test('rejects missing public entries, required resources, license and unconverted workspace references',async t=>{
  for(const missing of ['dist/index.js','schema/docs.schema.json','LICENSE']){
    const f=await fixture(t);await fs.unlink(path.join(f.root,missing));
    await assert.rejects(verifyDocsPackageDirectory(f.root,f.options),/missing|unpublished/i);
  }
  const f=await fixture(t);await f.put('package.json',JSON.stringify(f.original));
  await assert.rejects(verifyDocsPackageDirectory(f.root,f.options),/workspace/i);
});

test('rejects leaked build/private directories and symlink escape',async t=>{
  for(const name of ['docs/farm-docs/evidence/private.json','node_modules/x/index.js','target/debug/payload','.git/config']){
    const f=await fixture(t);await f.put(name,'private');
    await assert.rejects(verifyDocsPackageDirectory(f.root,f.options),/private|forbidden/i);
  }
  const f=await fixture(t),outside=await fs.mkdtemp(path.join(os.tmpdir(),'farm-docs-package-outside-'));
  t.after(()=>fs.rm(outside,{recursive:true,force:true}));
  await fs.symlink(outside,path.join(f.root,'escaped'),process.platform==='win32'?'junction':'dir');
  await assert.rejects(verifyDocsPackageDirectory(f.root,f.options),/symlink|junction/i);
});

test('platform payload must match formal native digest and can omit README',async t=>{
  const f=await fixture(t);await fs.unlink(path.join(f.root,'README.md'));await f.put('index.farm','native fixture');
  const options={...f.options,platform:true,native:{file:'index.farm',sha256:createHash('sha256').update('native fixture').digest('hex')}};
  assert.equal((await verifyDocsPackageDirectory(f.root,options)).native.sha256,options.native.sha256);
  await f.put('index.farm','other bytes');
  await assert.rejects(verifyDocsPackageDirectory(f.root,options),/native.*digest/i);
  await assert.rejects(verifyDocsPackageDirectory(f.root,{...options,native:undefined}),/native/i);
});

test('rejects dropped or changed exact platform dependencies in the packed manifest',async t=>{
  for(const packedOptional of [{},{'@farmfe/plugin-docs-win32-x64-msvc':'0.0.2'}]){
    const f=await fixture(t);
    f.original.optionalDependencies={'@farmfe/plugin-docs-win32-x64-msvc':'0.0.1'};
    await f.put('package.json',JSON.stringify({...f.manifest,optionalDependencies:packedOptional}));
    await assert.rejects(verifyDocsPackageDirectory(f.root,f.options),/dependenc/i);
  }
});

test('rejects removed public subpaths even when remaining packed entries exist',async t=>{
  const f=await fixture(t);
  f.original.exports={...f.original.exports,'./client':'./dist/client.js'};
  // The packed manifest and files both omit the promised client subpath.
  await assert.rejects(verifyDocsPackageDirectory(f.root,f.options),/public.*entr/i);
});

test('rejects repository paths in text including escaped Windows paths and source maps',async t=>{
  for(const leak of ['C:/work/farm/packages/docs/src/index.ts','c:\\work\\farm\\packages\\docs','C:\\\\work\\\\farm\\\\packages\\\\docs','file:///C:/work/farm/packages/docs']) {
    const f=await fixture(t);
    await f.put('dist/index.js.map',JSON.stringify({sources:[leak]}));
    await assert.rejects(verifyDocsPackageDirectory(f.root,{...f.options,forbiddenRoots:['C:\\work\\farm']}),/repository path.*index.js.map/i);
  }
});

test('path scan preserves portable source maps, public URLs and opaque native payloads',async t=>{
  const f=await fixture(t);
  await f.put('dist/index.js.map',JSON.stringify({sources:['../src/index.ts'],sourcesContent:['https://farmfe.org/docs']}));
  await f.put('index.farm',Buffer.concat([Buffer.from([0]),Buffer.from('C:/work/farm/private')]));
  const receipt=await verifyDocsPackageDirectory(f.root,{...f.options,forbiddenRoots:['C:/work/farm']});
  assert.equal(receipt.files.length,8);
});

test('repository roots reject drive-relative inputs and match UNC paths case-insensitively',async t=>{
  const f=await fixture(t);
  await f.put('dist/index.js','// source: \\\\SERVER\\Share\\Farm\\src\\index.ts');
  await assert.rejects(verifyDocsPackageDirectory(f.root,{...f.options,forbiddenRoots:['\\\\server\\share\\farm']}),/repository path/i);
  for(const roots of [[],['C:work'],['/'],['C:/']])await assert.rejects(verifyDocsPackageDirectory(f.root,{...f.options,forbiddenRoots:roots}),/root/i);
});
