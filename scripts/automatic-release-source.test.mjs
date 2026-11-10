import assert from 'node:assert/strict';
import {test} from 'node:test';
import YAML from 'yaml';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {verifyPreparedMigration,resolveAutomaticRelease,versionFamily} from './automatic-release-source.mjs';
const anchor='640215ed91bf635b60c6931eb7137dd3d0c10ed3';
const consumption='70404dd9959aef7c4c6cee3be66eeec5e3c6ad74';
const hash=value=>crypto.createHash('sha256').update(value).digest('hex');
const repository={id:507542208,full_name:'farm-fe/farm',default_branch:'main'};
function fixture(t,{advance=false,squash=false}={}) {
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'farm-prepared-merge-'));
 t.after(()=>fs.rmSync(root,{recursive:true,force:true}));
 const git=args=>execFileSync('git',args,{cwd:root,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
 const write=(name,body)=>{fs.mkdirSync(path.dirname(path.join(root,name)),{recursive:true});fs.writeFileSync(path.join(root,name),body);};
 const commit=message=>{git(['add','.']);git(['commit','-m',message]);return git(['rev-parse','HEAD']);};
 git(['init','-b','main']);git(['config','user.name','Local Fixture']);git(['config','user.email','fixture@example.invalid']);
 const plan={npm:{primary:['core','plugin-tailwindcss','plugin-tools'].map(name=>({name:'@farmfe/'+name,version:'2.0.0',manifestPath:'packages/'+name+'/package.json'})),payloads:[{name:'@farmfe/core-darwin-arm64',version:'2.0.0',owner:'@farmfe/core',platform:'darwin-arm64',manifestPath:'packages/core/npm/darwin-arm64/package.json'}]},cargo:[{name:'farmfe_core',version:'2.2.0',manifestPath:'crates/core/Cargo.toml'}]};
 for(const row of [...plan.npm.primary,...plan.npm.payloads])write(row.manifestPath,JSON.stringify({name:row.name,version:row.version}));
 write('crates/core/Cargo.toml','[package]\nname = "farmfe_core"\nversion = "2.2.0"\n');
 write('docs/releases/v2-candidate.json',JSON.stringify(plan));
 const consumedFollowups=JSON.parse(fs.readFileSync(new URL('../docs/releases/prepared-release-2330.json',import.meta.url))).consumedFollowups;
 for(const row of consumedFollowups)write(row.path,fs.readFileSync(new URL('../'+row.archive,import.meta.url)));
 const base=commit('already prepared versions');
 git(['checkout','-b','feature']);
 const policy={schemaVersion:1,kind:'prepared-first-stable-on-merge',repository:repository.full_name,repositoryId:repository.id,pullRequest:2330,anchorCommit:anchor,changesetsConsumptionCommit:consumption,candidate:'docs/releases/v2-candidate.json',consumedFollowups,retainedCrates:[]};
 for(const row of consumedFollowups){write(row.archive,fs.readFileSync(path.join(root,row.path)));fs.unlinkSync(path.join(root,row.path));}
 write('docs/releases/prepared-release-2330.json',JSON.stringify(policy));write('implementation.txt','prepared automation\n');
 const head=commit('automation');git(['checkout','main']);
 if(advance){write('unrelated.txt','another merged PR\n');commit('concurrent unrelated merge');}
 const target=git(['rev-parse','HEAD']);
 if(squash){git(['merge','--squash','feature']);commit('squash');}else git(['merge','--no-ff','feature','-m','merge']);
 const source=git(['rev-parse','HEAD']);
 const pr={number:2330,merged:true,merged_at:'2026-11-01T00:00:00Z',merge_commit_sha:source,base:{ref:'main',sha:target,repo:repository},head:{sha:head,repo:repository}};
 const runGit=args=>{
  const translated=args.map(arg=>arg.replaceAll(anchor,base).replaceAll(consumption,base));
  const result=execFileSync('git',translated,{cwd:root,encoding:'utf8'});
  return args[0]==='merge-base'&&result.trim()===base?args[1]+'\n':result;
 };
 return {root,git,write,commit,pr,source,runGit,plan,policy};
}
for(const [label,options] of [['merge',{}],['concurrent merge',{advance:true}],['squash after concurrent main',{advance:true,squash:true}]]) {
 test(`prepared existing versions authenticate real Git ${label}`,t=>{
  const f=fixture(t,options);const proof=verifyPreparedMigration({...f,repository});
  assert.equal(proof.releaseReady,true);assert.equal(proof.sourceSHA,f.source);assert.match(proof.receiptHash,/^[a-f0-9]{64}$/);
  for(const row of f.policy.consumedFollowups){assert.equal(fs.existsSync(path.join(f.root,row.path)),false);assert.equal(hash(fs.readFileSync(path.join(f.root,row.archive))),row.sha256);}
 });
}
test('platform is part of version family identity',()=>{
 const plan={npm:{primary:[],payloads:[{name:'payload',version:'2.0.0',owner:'core',manifestPath:'x',platform:'darwin-arm64'}]},cargo:[]};
 const changed=structuredClone(plan);changed.npm.payloads[0].platform='linux-x64-gnu';assert.notDeepEqual(versionFamily(plan),versionFamily(changed));
});
test('fork, wrong merge, unmerged PR and altered version family are rejected',t=>{
 const f=fixture(t);
 for(const mutate of [pr=>pr.head.repo={...repository,id:9},pr=>pr.merge_commit_sha='a'.repeat(40),pr=>pr.merged=false]) {
  const pr=structuredClone(f.pr);mutate(pr);assert.throws(()=>verifyPreparedMigration({...f,pr,repository}));
 }
 const runGit=args=>args[0]==='show'&&args[1]===f.source+':docs/releases/v2-candidate.json'?JSON.stringify({...f.plan,cargo:[]}):f.runGit(args);
 assert.throws(()=>verifyPreparedMigration({...f,runGit,repository}),/version family changed/);
});
test('fixed main event rejects spoofed/forced events and unrelated associations',async t=>{
 const f=fixture(t);const environment={GITHUB_REF:'refs/heads/main',GITHUB_SHA:f.source,GITHUB_REPOSITORY:repository.full_name,GITHUB_REPOSITORY_ID:String(repository.id),GITHUB_EVENT_NAME:'push'};
 const event={ref:'refs/heads/main',after:f.source};
 const readApi=route=>route.endsWith('/pulls/2330')?f.pr:route.includes('/commits/')?[f.pr]:repository;
 const input={environment,event,readApi,runGit:f.runGit};
 assert.equal((await resolveAutomaticRelease(input)).releaseReady,true);
 assert.deepEqual(await resolveAutomaticRelease(input),await resolveAutomaticRelease(input),'rerun identity is deterministic');
 await assert.rejects(resolveAutomaticRelease({...input,event:{...event,forced:true}}));
 await assert.rejects(resolveAutomaticRelease({...input,event:{...event,after:'a'.repeat(40)}}));
 const none=await resolveAutomaticRelease({...input,readApi:route=>route.includes('/commits/')?[]:repository});assert.equal(none.releaseReady,false);
 const duplicate=await resolveAutomaticRelease({...input,readApi:route=>route.includes('/commits/')?[f.pr,f.pr]:repository});assert.equal(duplicate.releaseReady,false);
});

test('automatic publisher stays bound to main provenance, frozen identity and build success',()=>{
 const release=fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml',import.meta.url),'utf8').replaceAll('\r\n','\n');
 const publisher=release.split('  publisher:')[1].split('    runs-on:')[0];
 assert.ok(publisher.includes("github.ref == 'refs/heads/main' && needs.source.outputs.version_ready == 'true'"));
 assert.ok(publisher.includes("(github.event_name == 'push' || (github.event_name == 'workflow_dispatch' && inputs.publish == true))"));
 assert.ok(publisher.includes("inputs.producer_run == ''"));
 assert.ok(publisher.includes("needs.candidate-identity.result == 'success'"));
 assert.ok(publisher.includes("needs.sign-frozen-release.result == 'success'"));
 assert.ok(!publisher.includes("candidate-consumers"));
 assert.ok(publisher.includes("needs.producer.result == 'success' && needs.plan.outputs.publish_required == 'true'"));
 for(const name of ['ci.yaml','rust-test.yaml','rust-build.yaml','rust-build-user.yaml','rust-build-release.yaml','farm-docs.yaml']) {
  const source=fs.readFileSync(new URL('../.github/workflows/'+name,import.meta.url),'utf8');
  const group=source.split('\n').find(line=>line.trim().startsWith('group:'));
  assert.ok(group.includes("github.ref == 'refs/heads/main' && github.sha || github.ref"),name);
 }
});

test('unconsumed, tampered or reassigned followups cannot authorize the prepared merge',t=>{
 const f=fixture(t);const row=f.policy.consumedFollowups[0];
 for(const runGit of [
  args=>args[0]==='ls-tree'?row.path+'\n':f.runGit(args),
  args=>args[0]==='show'&&args[1]===f.source+':'+row.archive?'tampered':f.runGit(args),
  args=>{if(args[0]==='show'&&args[1]===f.source+':docs/releases/prepared-release-2330.json'){const policy=structuredClone(f.policy);policy.consumedFollowups[0].includedIn[0].version='2.0.1';return JSON.stringify(policy);}return f.runGit(args);}
 ])assert.throws(()=>verifyPreparedMigration({...f,runGit,repository}));
});
test('main consumption is authenticated without weakening ordinary PR Changesets checks',()=>{
 const source=fs.readFileSync(new URL('./check-pr-changesets.mjs',import.meta.url),'utf8');
 assert.ok(source.includes("GITHUB_EVENT_NAME === 'push' && process.env.GITHUB_REF === 'refs/heads/main'"));
 assert.ok(source.includes('resolveAutomaticRelease()'));
 assert.ok(source.includes("mainProof?.releaseReady === true"));
 assert.ok(source.includes("'status', `--since=${base}`"));
});

test('normal verified version merge also tolerates an unrelated intervening main commit',async t=>{
 const {resolveMergedVersion}=await import('./changesets-version-runtime.mjs');
 const f=fixture(t,{advance:true,squash:true});const pr={...f.pr,number:3000,user:{id:41898282}};
 const environment={GITHUB_REF:'refs/heads/main',GITHUB_SHA:f.source,GITHUB_REPOSITORY:repository.full_name};
 const result=await resolveMergedVersion({environment,runGit:f.runGit,readApi:route=>route.includes('/commits/')?[pr]:pr,resolve:async()=>({mode:'verified-version-only',receiptHash:'f'.repeat(64),headTree:f.git(['rev-parse',pr.head.sha+'^{tree}'])})});
 assert.equal(result.releaseReady,true);assert.equal(result.sourceSHA,f.source);
});

test('real Git extra merge edits cannot borrow the reviewed head proof',t=>{
 const f=fixture(t,{advance:true});f.write('unreviewed.txt','unexpected');const source=f.commit('unreviewed merge edit');
 assert.throws(()=>verifyPreparedMigration({...f,source,pr:{...f.pr,merge_commit_sha:source},repository}),/does not preserve/);
});

test('publication queues retain pending runs without cancelling active transactions',()=>{
 for(const name of ['incremental-release.yaml','cargo-source-release.yaml']) {
  const workflow=YAML.parse(fs.readFileSync(new URL('../.github/workflows/'+name,import.meta.url),'utf8'));
  assert.equal(workflow.concurrency.queue,'max',name);
  assert.equal(workflow.concurrency['cancel-in-progress'],false,name);
  assert.ok(workflow.concurrency.group.startsWith('farm-'));
 }
});
