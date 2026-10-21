import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {test} from 'node:test';
import {preparePlatformPackages} from '../packages/plugin-tools/src/prepare.ts';

test('standalone platform packages retain embedded resource notices without copying unrelated files', async () => {
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'farm-docs-platform-'));
  try {
    const platform=path.join(root,'npm','win32-x64-msvc');
    await fs.mkdir(platform,{recursive:true});
    await fs.mkdir(path.join(root,'licenses','nested'),{recursive:true});
    await fs.writeFile(path.join(root,'package.json'),JSON.stringify({name:'@farmfe/plugin-fixture',version:'0.0.1',files:['scripts']}));
    await fs.writeFile(path.join(platform,'package.json'),JSON.stringify({name:'@farmfe/plugin-fixture-win32-x64-msvc',version:'0.0.1',files:['index.farm']}));
    await fs.writeFile(path.join(root,'THIRD_PARTY_NOTICES.md'),'Original resource notices\n');
    await fs.writeFile(path.join(root,'licenses','nested','resource.txt'),'Original resource license\n');
    await fs.writeFile(path.join(root,'private-evidence.json'),'must not ship');
    await preparePlatformPackages(root,true);
    assert.equal(await fs.readFile(path.join(platform,'THIRD_PARTY_NOTICES.md'),'utf8'),'Original resource notices\n');
    assert.equal(await fs.readFile(path.join(platform,'licenses','nested','resource.txt'),'utf8'),'Original resource license\n');
    const manifest=JSON.parse(await fs.readFile(path.join(platform,'package.json'),'utf8'));
    assert.deepEqual(manifest.files,['index.farm','THIRD_PARTY_NOTICES.md','licenses']);
    const parent=JSON.parse(await fs.readFile(path.join(root,'package.json'),'utf8'));
    assert.deepEqual(parent.files,['scripts','THIRD_PARTY_NOTICES.md','licenses']);
    await assert.rejects(fs.access(path.join(platform,'private-evidence.json')),{code:'ENOENT'});
  } finally {
    assert.equal(path.dirname(root),os.tmpdir());
    assert.ok(path.basename(root).startsWith('farm-docs-platform-'));
    await fs.rm(root,{recursive:true,force:true});
  }
});

test('platform preparation carries the parent license into standalone platform packages', async () => {
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'farm-docs-platform-'));
  try {
    const platform=path.join(root,'npm','win32-x64-msvc');
    await fs.mkdir(platform,{recursive:true});
    await fs.writeFile(path.join(root,'package.json'),JSON.stringify({name:'@farmfe/plugin-fixture',version:'0.0.1'}));
    await fs.writeFile(path.join(platform,'package.json'),JSON.stringify({name:'@farmfe/plugin-fixture-win32-x64-msvc',version:'0.0.1',files:['index.farm']}));
    const license='MIT License\n\nCopyright fixture\n';
    await fs.writeFile(path.join(root,'LICENSE'),license);
    await preparePlatformPackages(root,false);
    await assert.rejects(fs.access(path.join(platform,'LICENSE')),{code:'ENOENT'});
    await preparePlatformPackages(root,true);
    assert.equal(await fs.readFile(path.join(platform,'LICENSE'),'utf8'),license);
    const manifest=JSON.parse(await fs.readFile(path.join(platform,'package.json'),'utf8'));
    assert.deepEqual(manifest.files,['index.farm','LICENSE']);
  } finally {
    assert.equal(path.dirname(root),os.tmpdir());
    assert.ok(path.basename(root).startsWith('farm-docs-platform-'));
    await fs.rm(root,{recursive:true,force:true});
  }
});

test('platform preparation writes exact optional versions and Linux libc without publishing', async () => {
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'farm-docs-platform-'));
  try {
    const parent={name:'@farmfe/plugin-fixture',version:'0.0.1',optionalDependencies:{unrelated:'1.0.0'}};
    await fs.writeFile(path.join(root,'package.json'),JSON.stringify(parent));
    for(const abi of ['linux-x64-gnu','linux-arm64-musl','win32-x64-msvc']) {
      await fs.mkdir(path.join(root,'npm',abi),{recursive:true});
      await fs.writeFile(path.join(root,'npm',abi,'package.json'),JSON.stringify({name:`${parent.name}-${abi}`,version:'old',main:'index.farm'}));
    }
    const before=await fs.readFile(path.join(root,'package.json'),'utf8');
    const preview=await preparePlatformPackages(root,false);
    assert.equal(preview.published,false);
    assert.equal(await fs.readFile(path.join(root,'package.json'),'utf8'),before);
    const applied=await preparePlatformPackages(root,true);
    assert.equal(applied.published,false);
    assert.equal(applied.optionalDependencies.unrelated,'1.0.0');
    for(const [abi,libc] of [['linux-x64-gnu',['glibc']],['linux-arm64-musl',['musl']],['win32-x64-msvc',undefined]]) {
      const manifest=JSON.parse(await fs.readFile(path.join(root,'npm',abi,'package.json'),'utf8'));
      assert.equal(manifest.version,parent.version);
      assert.equal(applied.optionalDependencies[manifest.name],parent.version);
      assert.deepEqual(manifest.libc,libc);
    }
  } finally {
    assert.equal(path.dirname(root),os.tmpdir());
    assert.ok(path.basename(root).startsWith('farm-docs-platform-'));
    await fs.rm(root,{recursive:true,force:true});
  }
});
