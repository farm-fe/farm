import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import test from 'node:test';
import {verifyInstalledDocsPackages} from './farm-docs-consumer-isolation.mjs';

async function fixture(t) {
  const base=await fs.mkdtemp(path.join(os.tmpdir(),'farm-docs-isolation-'));
  t.after(()=>fs.rm(base,{recursive:true,force:true}));
  const consumer=path.join(base,'consumer'),repository=path.join(base,'repository');
  await fs.mkdir(consumer);await fs.mkdir(repository);
  await fs.writeFile(path.join(consumer,'package.json'),'{"type":"module"}');
  async function pkg(parent,version='0.0.1'){
    const dir=path.join(parent,'node_modules','@farmfe','fixture');await fs.mkdir(dir,{recursive:true});
    await fs.writeFile(path.join(dir,'package.json'),JSON.stringify({name:'@farmfe/fixture',version,exports:{'.':'./entry.js'}}));
    await fs.writeFile(path.join(dir,'entry.js'),'throw new Error("do not load product code");');
    return dir;
  }
  return {base,consumer,repository,pkg,options:{repository,packages:[{name:'@farmfe/fixture',version:'0.0.1'}],env:{}}};
}

test('records resolved package paths and bytes without executing package code',async t=>{
  const f=await fixture(t);const dir=await f.pkg(f.consumer);
  const report=await verifyInstalledDocsPackages(f.consumer,f.options);
  assert.equal(report.packages.length,1);
  assert.equal(report.packages[0].entry,await fs.realpath(path.join(dir,'entry.js')));
  assert.equal(report.packages[0].version,'0.0.1');
  assert.match(report.packages[0].entrySha256,/^[a-f0-9]{64}$/);
});

test('rejects version mismatch and native byte drift',async t=>{
  const f=await fixture(t),dir=await f.pkg(f.consumer,'0.0.2');
  await assert.rejects(verifyInstalledDocsPackages(f.consumer,f.options),/version/i);
  await f.pkg(f.consumer);
  const nativeSha256=createHash('sha256').update('formal native').digest('hex');
  const options={...f.options,packages:[{name:'@farmfe/fixture',version:'0.0.1',nativeSha256}]};
  await assert.rejects(verifyInstalledDocsPackages(f.consumer,options),/native.*digest/i);
  await fs.writeFile(path.join(dir,'entry.js'),'formal native');
  assert.equal((await verifyInstalledDocsPackages(f.consumer,options)).packages[0].entrySha256,nativeSha256);
});

test('rejects parent node_modules fallback and external junction targets',async t=>{
  const f=await fixture(t);await f.pkg(f.base);await fs.mkdir(path.join(f.consumer,'node_modules'));
  await assert.rejects(verifyInstalledDocsPackages(f.consumer,f.options),/outside.*consumer|escape/i);
  const f2=await fixture(t);
  const external=await f2.pkg(f2.repository),link=path.join(f2.consumer,'node_modules','@farmfe','fixture');
  await fs.mkdir(path.dirname(link),{recursive:true});
  await fs.symlink(external,link,process.platform==='win32'?'junction':'dir');
  await assert.rejects(verifyInstalledDocsPackages(f2.consumer,f2.options),/outside.*consumer|escape/i);
});

test('rejects consumer inside repository, external node_modules and NODE_PATH',async t=>{
  const f=await fixture(t);await f.pkg(f.consumer);
  await assert.rejects(verifyInstalledDocsPackages(f.consumer,{...f.options,repository:f.base}),/repository/i);
  await assert.rejects(verifyInstalledDocsPackages(f.consumer,{...f.options,env:{NODE_PATH:f.repository}}),/NODE_PATH/);
  const other=path.join(f.base,'other');await fs.mkdir(other);await fs.writeFile(path.join(other,'package.json'),'{}');
  await fs.symlink(path.join(f.consumer,'node_modules'),path.join(other,'node_modules'),process.platform==='win32'?'junction':'dir');
  await assert.rejects(verifyInstalledDocsPackages(other,f.options),/node_modules.*outside|escape/i);
});

test('accepts a pnpm-style package symlink when its real target stays inside the consumer',async t=>{
  const f=await fixture(t);
  const nested=path.join(f.consumer,'node_modules','.pnpm','fixture@0.0.1');
  const actual=await f.pkg(nested),link=path.join(f.consumer,'node_modules','@farmfe','fixture');
  await fs.mkdir(path.dirname(link),{recursive:true});
  await fs.symlink(actual,link,process.platform==='win32'?'junction':'dir');
  assert.equal((await verifyInstalledDocsPackages(f.consumer,f.options)).packages[0].entry,await fs.realpath(path.join(actual,'entry.js')));
});

test('resolves a declared transitive package from its verified parent without requiring root hoisting',async t=>{
  const f=await fixture(t),parent=await f.pkg(f.consumer);
  const manifest=JSON.parse(await fs.readFile(path.join(parent,'package.json'),'utf8'));
  manifest.dependencies={'@farmfe/child':'0.0.1'};
  await fs.writeFile(path.join(parent,'package.json'),JSON.stringify(manifest));
  const child=path.join(parent,'node_modules/@farmfe/child');await fs.mkdir(child,{recursive:true});
  await fs.writeFile(path.join(child,'package.json'),JSON.stringify({name:'@farmfe/child',version:'0.0.1',main:'entry.js'}));
  await fs.writeFile(path.join(child,'entry.js'),'throw Error("never execute");');
  const packages=[...f.options.packages,{name:'@farmfe/child',version:'0.0.1',from:'@farmfe/fixture'}];
  const report=await verifyInstalledDocsPackages(f.consumer,{...f.options,packages});
  assert.equal(report.packages[1].entry,await fs.realpath(path.join(child,'entry.js')));
  await assert.rejects(verifyInstalledDocsPackages(f.consumer,{...f.options,packages:[packages[1]]}),/verified parent/i);
  delete manifest.dependencies;await fs.writeFile(path.join(parent,'package.json'),JSON.stringify(manifest));
  await assert.rejects(verifyInstalledDocsPackages(f.consumer,{...f.options,packages}),/declare/i);
});
