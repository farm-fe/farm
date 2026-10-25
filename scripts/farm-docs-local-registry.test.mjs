import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import test from 'node:test';
import {startDocsLocalRegistry} from './farm-docs-local-registry.mjs';

async function fixture(t){
  const dir=await fs.mkdtemp(path.join(os.tmpdir(),'farm-docs-registry-'));
  t.after(()=>fs.rm(dir,{recursive:true,force:true}));
  const archive=path.join(dir,'fixture.tgz'),bytes=Buffer.from('HTTP fixture bytes; not an actual npm archive');
  await fs.writeFile(archive,bytes);
  const manifest={name:'@farmfe/docs',version:'0.0.1',dependencies:{'@farmfe/core':'2.0.0'}};
  const pack={manifest,archive,sha256:createHash('sha256').update(bytes).digest('hex')};
  return {archive,bytes,pack};
}

test('serves exact current metadata and integrity-bound bytes over loopback only',async t=>{
  const f=await fixture(t),registry=await startDocsLocalRegistry([f.pack]);t.after(()=>registry.close());
  assert.equal(new URL(registry.url).hostname,'127.0.0.1');
  for(const encoded of ['@farmfe%2fdocs','%40farmfe%2Fdocs','@farmfe/docs']){
    const response=await fetch(registry.url+'/'+encoded);assert.equal(response.status,200);
    const doc=await response.json();assert.deepEqual(Object.keys(doc.versions),['0.0.1']);
    assert.equal(doc['dist-tags'].latest,'0.0.1');
    const dist=doc.versions['0.0.1'].dist;
    assert.equal(dist.integrity,'sha512-'+createHash('sha512').update(f.bytes).digest('base64'));
    assert.equal(new URL(dist.tarball).origin,registry.url);
    const bytes=Buffer.from(await (await fetch(dist.tarball)).arrayBuffer());assert.deepEqual(bytes,f.bytes);
    const head=await fetch(dist.tarball,{method:'HEAD'});assert.equal(head.status,200);assert.equal((await head.arrayBuffer()).byteLength,0);
  }
  assert.equal(registry.packs[0].sha256,f.pack.sha256);
  assert.ok(registry.requests.length>=9);
});

test('does not proxy missing names, alternate versions, traversal or publishing requests',async t=>{
  const f=await fixture(t),registry=await startDocsLocalRegistry([f.pack]);t.after(()=>registry.close());
  for(const route of ['/@farmfe%2fmissing','/@farmfe%2fdocs/9.0.0','/tarballs/../../package.json','/%E0%A4%A'])assert.equal((await fetch(registry.url+route)).status,404);
  for(const method of ['PUT','POST','DELETE'])assert.equal((await fetch(registry.url+'/@farmfe%2fdocs',{method,body:'{}'})).status,405);
});

test('rejects unbound bytes, unresolved workspace refs and duplicate package versions before listening',async t=>{
  const f=await fixture(t);
  await assert.rejects(startDocsLocalRegistry([{...f.pack,sha256:'0'.repeat(64)}]),/digest/i);
  await assert.rejects(startDocsLocalRegistry([{...f.pack,manifest:{...f.pack.manifest,dependencies:{'@farmfe/core':'workspace:*'}}}]),/workspace/i);
  await assert.rejects(startDocsLocalRegistry([f.pack,f.pack]),/duplicate/i);
});

test('refuses archive drift and close releases the listener',async t=>{
  const f=await fixture(t),registry=await startDocsLocalRegistry([f.pack]);t.after(()=>registry.close());
  const doc=await (await fetch(registry.url+'/@farmfe%2fdocs')).json(),url=doc.versions['0.0.1'].dist.tarball;
  await fs.writeFile(f.archive,'changed bytes');
  assert.equal((await fetch(url)).status,409);
  await registry.close();
  await assert.rejects(fetch(registry.url+'/@farmfe%2fdocs'));
});
