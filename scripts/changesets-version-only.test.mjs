import test from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { classifyVersionDiff, verifyVersionProducer } from './changesets-version-only.mjs';
const sha = text => crypto.createHash('sha256').update(text).digest('hex');
function fixture() {
  const pkg = {name:'@farm/a', version:'1.0.0', scripts:{build:'farm build'}, dependencies:{external:'1.0.0'}};
  const before = {'packages/a/package.json':JSON.stringify(pkg), '.changeset/patch-a.md':'---\n"@farm/a": patch\n---\nFix A\n'};
  const after = {'packages/a/package.json':JSON.stringify({...pkg,version:'1.0.1'})};
  return {before,after,modesBefore:Object.fromEntries(Object.keys(before).map(k=>[k,'100644'])),modesAfter:Object.fromEntries(Object.keys(after).map(k=>[k,'100644'])),receipt:{headSHA:'a'.repeat(40),baseSHA:'b'.repeat(40),consumed:{'.changeset/patch-a.md':sha(before['.changeset/patch-a.md'])}}};
}
test('pure advancing version and exact consumed Changeset yields metadata-only disposition',()=>{
 const r=classifyVersionDiff(fixture());assert.equal(r.mode,'verified-version-only');assert.equal(r.acceptanceInherited,false);
});
for(const field of ['scripts','dependencies','engines','files'])test(`non-version manifest ${field} retains full CI`,()=>{
 const f=fixture(),pkg=JSON.parse(f.after['packages/a/package.json']);pkg[field]={injected:'2.0.0'};f.after['packages/a/package.json']=JSON.stringify(pkg);assert.equal(classifyVersionDiff(f).mode,'full');
});
for(const file of ['packages/a/src/index.ts','.github/workflows/ci.yaml','scripts/tool.mjs','rust-toolchain.toml'])test(`mixed ${file} retains full CI`,()=>{
 const f=fixture();f.after[file]='new source';f.modesAfter[file]='100644';assert.equal(classifyVersionDiff(f).mode,'full');
});
test('repeated or backwards versions cannot skip',()=>{
 for(const version of ['1.0.0','0.9.0']) {const f=fixture();f.after['packages/a/package.json']=JSON.stringify({...JSON.parse(f.before['packages/a/package.json']),version});assert.equal(classifyVersionDiff(f).mode,'full');}
});
test('pnpm external package graph and integrity cannot change',()=>{
 const f=fixture();f.before['pnpm-lock.yaml']='lockfileVersion: 9.0\npackages:\n  external@1.0.0:\n    resolution: {integrity: old}\n';f.after['pnpm-lock.yaml']=f.before['pnpm-lock.yaml'].replace('old','new');f.modesBefore['pnpm-lock.yaml']=f.modesAfter['pnpm-lock.yaml']='100644';assert.equal(classifyVersionDiff(f).mode,'full');
});
test('Cargo third-party version fields cannot be normalized away',()=>{
 const f=fixture();f.before['crates/a/Cargo.toml']='[package]\nname="a"\nversion="1.0.0"\n[dependencies]\nserde={version="1.0.0"}\n';f.after['crates/a/Cargo.toml']=f.before['crates/a/Cargo.toml'].replace('serde={version="1.0.0"}','serde={version="1.0.1"}');f.modesBefore['crates/a/Cargo.toml']=f.modesAfter['crates/a/Cargo.toml']='100644';assert.equal(classifyVersionDiff(f).mode,'full');
});
function evidence() {
 const receipt={kind:'changesets-version-preparation',repositoryId:1,pr:5,baseSHA:'b'.repeat(40),headSHA:'a'.repeat(40),headTree:'c'.repeat(40),runId:10,attempt:2};
 const bytes=Buffer.from(JSON.stringify(receipt)+'\n');return {receipt,receiptBytes:bytes,archiveBytes:bytes,repository:{id:1,default_branch:'main'},pr:{number:5,user:{id:41898282,login:'github-actions[bot]',type:'Bot'},base:{sha:receipt.baseSHA,ref:'main',repo:{id:1}},head:{sha:receipt.headSHA,repo:{id:1}}},run:{id:10,run_attempt:2,head_sha:receipt.baseSHA,repository:{id:1},head_repository:{id:1},head_branch:'main',path:'.github/workflows/release.yaml',event:'workflow_run',status:'completed',conclusion:'success'},job:{run_id:10,run_attempt:2,name:'Prepare reviewed version PR',status:'completed',conclusion:'success',steps:[{name:'Upload version-only provenance',conclusion:'success'}]},artifact:{workflow_run:{id:10},expired:false,digest:'sha256:'+sha(bytes)}};
}
test('trusted completed exact producer identity is independently required',()=>assert.equal(verifyVersionProducer(evidence()).pr,5));
for(const [label,mutate] of [
 ['spoof bot',e=>e.pr.user.id=123],['fork',e=>e.pr.head.repo.id=2],['wrong base',e=>e.pr.base.ref='other'],
 ['head moved',e=>e.pr.head.sha='d'.repeat(40)],['failed newest attempt',e=>e.run.conclusion='failure'],
 ['old attempt',e=>e.job.run_attempt=1],['wrong workflow',e=>e.run.path='.github/workflows/ci.yaml'],
 ['expired receipt',e=>e.artifact.expired=true],['missing evidence',e=>delete e.job.steps]
])test(`${label} cannot authorize fastlane`,()=>{const e=evidence();mutate(e);assert.throws(()=>verifyVersionProducer(e));});
