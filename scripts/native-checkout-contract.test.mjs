// cspell:words safecrlf
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const root=fileURLToPath(new URL('../',import.meta.url));
const header='#!/usr/bin/env sh\nset -eu';
function checkout(t,attributes) {
 const dir=fs.mkdtempSync(path.join(os.tmpdir(),'farm-shell-checkout-'));
 t.after(()=>fs.rmSync(dir,{recursive:true,force:true}));
 fs.mkdirSync(path.join(dir,'scripts'));
 fs.writeFileSync(path.join(dir,'.gitattributes'),attributes);
 const files=fs.readdirSync(path.join(root,'scripts')).filter(name=>name.endsWith('.sh'));
 for(const name of files)fs.writeFileSync(path.join(dir,'scripts',name),fs.readFileSync(path.join(root,'scripts',name),'utf8').replaceAll('\r\n','\n'));
 function git(...args) {
  const result=spawnSync('git',['-c','core.autocrlf=true','-c','core.safecrlf=false',...args],{cwd:dir,encoding:'utf8',timeout:10000});
  assert.equal(result.status,0,result.stderr);
 }
 git('init','-q');git('add','.gitattributes',...files.map(name=>'scripts/'+name));
 for(const name of files)fs.unlinkSync(path.join(dir,'scripts',name));
 git('checkout-index','--all','--force');
 return {dir,files};
}
test('actual shell recipes retain LF under Windows-style Git checkout',t=>{
 const {dir,files}=checkout(t,fs.readFileSync(path.join(root,'.gitattributes'),'utf8'));
 assert.ok(files.includes('ci-build-rust-artifacts.sh'));
 for(const name of files) {
  const source=fs.readFileSync(path.join(dir,'scripts',name),'utf8');
  assert.ok(!source.includes('\r'),`${name}: Git checkout must preserve shell LF bytes`);
  const parsed=spawnSync('sh',['-n',path.join(dir,'scripts',name)],{encoding:'utf8',timeout:10000});
  assert.equal(parsed.status,0,parsed.stderr);
 }
 assert.ok(fs.readFileSync(path.join(dir,'scripts/ci-build-rust-artifacts.sh'),'utf8').startsWith(header));
});
test('without LF attributes the real header and shell execution fail under automatic CRLF checkout',t=>{
 const {dir}=checkout(t,'');
 const file=path.join(dir,'scripts/ci-build-rust-artifacts.sh');
 const source=fs.readFileSync(file,'utf8');
 assert.ok(source.includes('\r\n'));
 assert.equal(source.startsWith(header),false);
 // Only execute the real two-line header: no compiler, network or product build.
 const result=spawnSync('sh',['-c',source.split('\n').slice(0,2).join('\n')+'\n'],{encoding:'utf8',timeout:10000});
 assert.notEqual(result.status,0);
});
test('standalone native container routing contract participates in the complete Node suite',()=>{
 const result=spawnSync(process.execPath,['scripts/test-native-container-node.cjs'],{cwd:root,encoding:'utf8',timeout:120000});
 assert.equal(result.status,0,result.stderr+result.stdout);
 assert.match(result.stdout,/"status":"passed"/);
});
