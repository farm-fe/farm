import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import test from 'node:test';
import {validateDocsExtraction} from './farm-docs-validate-extraction.mjs';
import {startDocsLocalRegistry} from './farm-docs-local-registry.mjs';

const python=process.env.FARM_DOCS_TEST_PYTHON??'python';
const extract=fileURLToPath(new URL('./farm-docs-extract-package.py',import.meta.url));
const execute=promisify(execFile);
async function fixture(t){
  const root=await fs.mkdtemp(path.join(os.tmpdir(),'farm-docs-chain-'));
  t.after(()=>fs.rm(root,{recursive:true,force:true}));
  const repository=path.join(root,'repository');await fs.mkdir(repository);
  const original={name:'@farmfe/docs-chain-fixture',version:'0.0.1',type:'module',license:'MIT',exports:{'.':'./dist/index.js'}};
  const archive=path.join(root,'fixture.tgz');
  const code="import io,json,sys,tarfile\nfiles={'package.json':sys.argv[2],'dist/index.js':\"throw Error('must not execute');\",'README.md':'Fixture','LICENSE':'Fixture license','client.html':'<div>fixture</div>'}\nwith tarfile.open(sys.argv[1],'w:gz') as out:\n for name,text in files.items():\n  body=text.encode(); item=tarfile.TarInfo('package/'+name); item.size=len(body); out.addfile(item,io.BytesIO(body))\n";
  await execute(python,['-c',code,archive,JSON.stringify(original)],{windowsHide:true,timeout:20000});
  const bytes=await fs.readFile(archive),sha256=createHash('sha256').update(bytes).digest('hex');
  const destination=path.join(root,'extracted'),receiptFile=path.join(root,'receipt.json');
  await execute(python,[extract,'--archive',archive,'--sha256',sha256,'--destination',destination,'--repository',repository,'--receipt',receiptFile],{windowsHide:true,timeout:20000});
  const receipt=JSON.parse(await fs.readFile(receiptFile,'utf8'));
  return {archive,sha256,destination,receipt,options:{archive,sha256,original,versions:new Map(),requiredFiles:['client.html']}};
}

test('real fixture archive flows through extraction, directory contract and registry bytes',async t=>{
  const f=await fixture(t),pack=await validateDocsExtraction(f.receipt,f.options);
  assert.deepEqual(pack.manifest,f.options.original);
  const registry=await startDocsLocalRegistry([pack]);t.after(()=>registry.close());
  const metadata=await (await fetch(registry.url+'/@farmfe%2fdocs-chain-fixture')).json();
  const bytes=Buffer.from(await (await fetch(metadata.versions['0.0.1'].dist.tarball)).arrayBuffer());
  assert.equal(createHash('sha256').update(bytes).digest('hex'),f.sha256);
  assert.equal(pack.directoryReceipt.files.length,5);
});

test('rejects code or manifest drift after extraction instead of trusting old receipts',async t=>{
  const f=await fixture(t);
  await fs.writeFile(path.join(f.destination,'dist/index.js'),'changed but still a valid public entry');
  await assert.rejects(validateDocsExtraction(f.receipt,f.options),/extracted.*(files|digest)|receipt.*files/i);
  const g=await fixture(t);
  await fs.writeFile(path.join(g.destination,'package.json'),JSON.stringify({...g.options.original,description:'changed after extraction'}));
  await assert.rejects(validateDocsExtraction(g.receipt,g.options),/extracted.*(files|manifest)|receipt.*(files|manifest)/i);
});

test('rejects failed receipts, another pack digest and changed source archive bytes',async t=>{
  const f=await fixture(t);
  await assert.rejects(validateDocsExtraction({...f.receipt,success:false},f.options),/success/i);
  await assert.rejects(validateDocsExtraction(f.receipt,{...f.options,sha256:'0'.repeat(64)}),/archive.*digest/i);
  await fs.writeFile(f.archive,'changed archive');
  await assert.rejects(validateDocsExtraction(f.receipt,f.options),/archive.*digest/i);
});
