// cspell:words pathlib unstarted
import test from 'node:test';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { measurementResolver, measurementPlatforms } from './performance-attempts.mjs';
import { nativePlugins } from './performance-cohorts.mjs';

const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
function zip(name, value) {
  const body = Buffer.from(JSON.stringify(value)), filename = Buffer.from(name);
  let crc=0xffffffff;for(const value of body){crc^=value;for(let bit=0;bit<8;bit++)crc=(crc>>>1)^((crc&1)?0xedb88320:0);}crc=(crc^0xffffffff)>>>0;
  const local = Buffer.alloc(30); local.writeUInt32LE(crc,14);local.writeUInt32LE(body.length,18);local.writeUInt32LE(body.length,22);local.writeUInt32LE(0x04034b50); local.writeUInt16LE(filename.length, 26);
  const directory = Buffer.alloc(46); directory.writeUInt32LE(crc,16);directory.writeUInt32LE(0x02014b50); directory.writeUInt32LE(body.length, 20); directory.writeUInt32LE(body.length, 24); directory.writeUInt16LE(filename.length, 28);
  const end = Buffer.alloc(22); end.writeUInt32LE(0x06054b50); end.writeUInt16LE(1, 8); end.writeUInt16LE(1, 10); end.writeUInt32LE(directory.length+filename.length,12);end.writeUInt32LE(local.length + filename.length + body.length, 16);
  return Buffer.concat([local, filename, body, directory, filename, end]);
}
function fixture() {
  const producerSHA = 'a'.repeat(40), testedTree = 'b'.repeat(40), head = 'c'.repeat(40), base = 'd'.repeat(40);
  const summary = {runId:10, runAttempt:2, repositoryId:5, headSHA:head, baseSHA:base, event:'pull_request', protocol:'commerce-business-2048-farm-supervised-v2'};
  const prefix = '/repos/farm-fe/farm';
  const run = {id:10, run_attempt:2, repository:{id:5, full_name:'farm-fe/farm'}, head_sha:head, event:'pull_request', path:'.github/workflows/ci.yaml', name:'E2E Tests'};
  const start = attempt => `2026-10-31T0${attempt}:00:00Z`;
  const end = attempt => `2026-10-31T0${attempt}:20:00Z`;
  const jobs = {1:[], 2:[]}, bodies = new Map(), artifacts = [], raw = new Map();
  function add(name, value, job, attempt) {
    const id = 100 + artifacts.length, bytes = zip(name.startsWith('performance-') ? 'summary.json' : 'products.json', value);
    bodies.set(id, bytes); artifacts.push({id, name, expired:false, created_at:`2026-10-31T0${attempt}:10:00Z`, size_in_bytes:bytes.length,
      digest:'sha256:' + hash(bytes), workflow_run:{id:10,repository_id:5,head_sha:head}});
    jobs[attempt].push(job); return id;
  }
  for (const [index, [abi, os, arch, runner, nativeJob]] of measurementPlatforms.entries()) {
    const attempt = index === 0 ? 2 : 1;
    const native = Object.fromEntries(['@farmfe/core', ...nativePlugins.map(name=>'@farmfe/plugin-'+name)].map(owner=>[owner,{sha256:'e'.repeat(64),bytes:10}]));
    const report = {status:'complete', protocol:summary.protocol, metadata:{revision:producerSHA, artifactProducerSHA:producerSHA, baseSHA:base,os,arch,harnessSha256:'f'.repeat(64)},native:{artifacts:native}};
    const job = {id:200+index,run_id:10,name:`Commerce Performance (${runner})`,status:'completed',conclusion:'success',started_at:start(attempt),completed_at:end(attempt)};
    add(`performance-${producerSHA}-${abi}-attempt-${attempt}`, report, job, attempt); raw.set(abi, report);
    const products = {abi,profile:'ci',originBuildSHA:producerSHA,runId:'10',runAttempt:'1',repository:'farm-fe/farm',records:Object.entries(native).map(([owner,row])=>({owner,...row}))};
    add(`native-shadow-${producerSHA}-${abi}-1`, products,{id:300+index,run_id:10,name:nativeJob,status:'completed',conclusion:'success',started_at:start(1),completed_at:end(1)},1);
  }
  // A rerun's successful carried jobs must retain attempt 1 provenance.
  jobs[2].push(...jobs[1].map(job=>({...job})));
  const routes = [];
  const api = async (route, binary) => {
    routes.push(route);
    if(route === `${prefix}/actions/runs/10`) return run;
    if(route === `${prefix}/git/commits/${producerSHA}`) return {sha:producerSHA,tree:{sha:testedTree}};
    if(route.includes('/artifacts?')) return {artifacts,total_count:artifacts.length};
    const attempt = /\/attempts\/(\d+)(.*)/.exec(route);
    if(attempt) return attempt[2].includes('/jobs?') ? {jobs:jobs[attempt[1]]} : {...run,run_attempt:Number(attempt[1]),run_started_at:run.run_started_at ?? start(Number(attempt[1]))};
    const artifact = /\/artifacts\/(\d+)(\/zip)?$/.exec(route);
    if(artifact) return binary ? bodies.get(Number(artifact[1])) : artifacts.find(row=>row.id===Number(artifact[1]));
    throw Error('Unexpected proof request');
  };
  return {api,repository:'farm-fe/farm',summary,producerSHA,testedTree,run,jobs,artifacts,bodies,raw,routes};
}

test('same-run Linux retry and earlier Mac/Windows measurements retain exact original sources', async()=>{
  const f=fixture(), resolve=await measurementResolver(f);
  const values=await Promise.all(measurementPlatforms.map(resolve));
  assert.deepEqual(values.map(row=>row.source.runAttempt),[2,1,1]);
  assert.deepEqual(values.map(row=>row.source.nativeAttempt),[1,1,1]);
  assert.equal(values[1].source.jobId,201); assert.equal(values[1].source.nativeJobId,301);
  assert.ok(values.every(row=>row.source.profile==='ci' && row.source.testedTree===f.testedTree));
});

test('latest failed execution without an archive cannot fall back to an older success',async()=>{
  const f=fixture();f.jobs[2].push({id:900,run_id:10,name:'Commerce Performance (macos-latest)',status:'completed',conclusion:'failure',started_at:'2026-10-31T02:01:00Z',completed_at:'2026-10-31T02:03:00Z'});
  // Remove the carried view; two same-name entries themselves fail closed.
  f.jobs[2]=f.jobs[2].filter(job=>job.id!==201);
  const resolve=await measurementResolver(f);await assert.rejects(()=>resolve(measurementPlatforms[1]),/Latest actual measurement producer/);
});

test('wrong tree, repository, workflow and moved attempt fail before archive reads',async()=>{
  for(const mutate of [f=>{f.testedTree='9'.repeat(40);},f=>{f.run.repository.id=7;},f=>{f.run.path='.github/workflows/incremental-release.yaml';},f=>{f.run.run_attempt=3;}]){
    const f=fixture();mutate(f);await assert.rejects(()=>measurementResolver(f));assert.ok(!f.routes.some(route=>route.endsWith('/zip')));
  }
});

test('digest, producer upload interval, source identity, native profile and byte mismatches reject',async()=>{
  for(const mutate of [f=>{f.artifacts[0].digest='sha256:'+'0'.repeat(64);},f=>{f.artifacts[0].created_at='2026-10-31T03:00:00Z';},f=>{f.artifacts[0].workflow_run.head_sha='0'.repeat(40);},f=>{
    const a=f.artifacts[1],value=JSON.parse(f.bodies.get(a.id).subarray(30+'products.json'.length, -22-46-'products.json'.length));
    value.profile='release';const bytes=zip('products.json',value);f.bodies.set(a.id,bytes);a.digest='sha256:'+hash(bytes);a.size_in_bytes=bytes.length;
  }]){
    const f=fixture();mutate(f);const resolve=await measurementResolver(f);await assert.rejects(()=>resolve(measurementPlatforms[0]));
  }
});

test('missing proof, complete report from failed producer, and incomplete native cohort reject',async()=>{
  for(const mutate of [f=>{f.artifacts.splice(1,1);},f=>{f.jobs[2][0].conclusion='failure';},f=>{
    const value=f.raw.get('linux-x64-gnu');delete value.native.artifacts['@farmfe/plugin-react'];const a=f.artifacts[0],bytes=zip('summary.json',value);f.bodies.set(a.id,bytes);a.digest='sha256:'+hash(bytes);a.size_in_bytes=bytes.length;
  }]){const f=fixture();mutate(f);const resolve=await measurementResolver(f);await assert.rejects(()=>resolve(measurementPlatforms[0]));}
});


test('carried execution detection does not assume run_started_at changes on rerun',async()=>{
 const f=fixture();f.run.run_started_at='2026-10-31T01:00:00Z';const resolve=await measurementResolver(f);
 assert.deepEqual((await Promise.all(measurementPlatforms.map(resolve))).map(row=>row.source.runAttempt),[2,1,1]);
});


test('unstarted skipped cancelled queued and in-progress cells cannot hide behind old success',async()=>{
  for(const [status,conclusion] of [['completed','skipped'],['completed','cancelled'],['queued',null],['in_progress',null]]) {
    const f=fixture(),job=f.jobs[2].find(row=>row.name==='Commerce Performance (macos-latest)');
    Object.assign(job,{status,conclusion,started_at:null,completed_at:null});
    const resolve=await measurementResolver(f);await assert.rejects(()=>resolve(measurementPlatforms[1]),/Latest actual measurement producer/);
  }
});
test('same timestamps with a different job ID do not prove a carried measurement',async()=>{
  const f=fixture();f.jobs[2].find(row=>row.name==='Commerce Performance (macos-latest)').id=999;
  const resolve=await measurementResolver(f);await assert.rejects(()=>resolve(measurementPlatforms[1]),/Unproven carried/);
});
import {compactReport} from './aggregate-performance-reports.mjs';
import {validateSummary, report as publishReport} from './performance-pr-reporter.mjs';
import {spawnSync} from 'node:child_process';
import fs from 'node:fs';

test('same immutable build and reused-native ZIPs pass aggregate schema3 PR publisher and independent Python release verifier',async()=>{
 for(const reused of [false,true]) {
  const f=fixture();
  if(reused) {
    const raw=f.raw.get('linux-x64-gnu');
    raw.metadata.nativeReuse={chain:'ci',originBuildSHA:'9'.repeat(40),originArtifactId:'901',consumerSHA:f.producerSHA,acceptanceInherited:false};
    const receipt={kind:'ci-consumer-materialization-not-a-new-build',chain:'ci',profile:'ci',abi:'linux-x64-gnu',consumerSHA:f.producerSHA,consumerTree:f.testedTree,runId:10,attempt:1,originBuildSHA:'9'.repeat(40),originArtifactId:901,originContext:{profile:'ci'},acceptanceInherited:false,records:Object.entries(raw.native.artifacts).map(([owner,row])=>({owner,...row}))};
    const artifact=f.artifacts.find(row=>row.name===`native-shadow-${f.producerSHA}-linux-x64-gnu-1`),bytes=zip('consumer.json',receipt);
    artifact.name='ci-consumer-native-proof-10-1';artifact.digest='sha256:'+hash(bytes);artifact.size_in_bytes=bytes.length;f.bodies.set(artifact.id,bytes);
    for(const jobs of Object.values(f.jobs))for(const job of jobs)if(job.id===300)job.name='Type Check & Changesets';
  }
  Object.assign(f.run,{status:'completed',conclusion:'success',head_repository:{id:5},pull_requests:[{number:2330,head:{sha:f.summary.headSHA}}]});
  for(const [abi,value] of f.raw){
    Object.assign(value.metadata,{node:'v22.20.0',pnpm:'9.4.0',runnerImage:'contract runner',fixtureSha256:'1'.repeat(64),lockSha256:'2'.repeat(64)});
    value.cells=['cold','warm'].map(cacheMode=>({cacheMode,samples:3,validSamples:3,medianMs:100,iqrMs:1}));value.comparison={mode:'current-only',baseCells:[]};
    const artifact=f.artifacts.find(row=>row.name.startsWith(`performance-${f.producerSHA}-${abi}-`)),bytes=zip('summary.json',value);
    f.bodies.set(artifact.id,bytes);Object.assign(artifact,{size_in_bytes:bytes.length,digest:'sha256:'+hash(bytes)});
  }
  const resolve=await measurementResolver(f),values=await Promise.all(measurementPlatforms.map(resolve));
  const summary={schemaVersion:3,...f.summary,prNumber:2330,reports:values.map(row=>compactReport(row.raw)),measurementSources:values.map(row=>row.source)};
  validateSummary(JSON.stringify(summary));assert.deepEqual(summary.measurementSources.map(row=>row.runAttempt),[2,1,1]);
  f.jobs[2].push({id:998,run_id:10,name:'Aggregate informational performance report',status:'completed',conclusion:'success',started_at:'2026-10-31T02:00:00Z',completed_at:'2026-10-31T02:20:00Z'});
  f.jobs[3]=f.jobs[2].map(job=>({...job}));f.run.run_attempt=3;
  const summaryBytes=zip('summary.json',summary),writes=[];
  const summaryArtifact={id:999,name:'performance-pr-summary-2',expired:false,size_in_bytes:summaryBytes.length,digest:'sha256:'+hash(summaryBytes),workflow_run:{id:10},created_at:'2026-10-31T02:10:00Z'};
  const pr={number:2330,state:'open',head:{sha:f.summary.headSHA,repo:{id:5}},base:{sha:f.summary.baseSHA,repo:{id:5}}};
  await publishReport({token:'mock',repository:f.repository,event:{workflow_run:{id:10},repository:{id:5}},fetcher:async(url,options={})=>{
    if(options.method){writes.push(options);return Response.json({id:1});}
    if(url.includes('/attempts/')) return Response.json(await f.api(new URL(url).pathname+new URL(url).search));
    if(url.endsWith('/zip'))return new Response(summaryBytes);
    if(url.includes('/artifacts?'))return Response.json({artifacts:[summaryArtifact]});
    if(url.includes('/comments?'))return Response.json([]);
    if(url.includes('/pulls/'))return Response.json(pr);
    return Response.json(f.run);
  }});
  assert.equal(writes.length,1);assert.match(JSON.parse(writes[0].body).body,/Measurement attempt 1/);
  const payload={run:f.run,sourceSHA:f.producerSHA,sourceTree:f.testedTree,baseSHA:f.summary.baseSHA,jobs:f.jobs,artifacts:[...f.artifacts,summaryArtifact],bodies:Object.fromEntries([...f.bodies.entries(),[999,summaryBytes]].map(([id,body])=>[id,body.toString('base64')]))};
  const python=`import base64, importlib.util, json, sys\nfrom pathlib import Path\np=json.load(sys.stdin)\nspec=importlib.util.spec_from_file_location('joint_functional',Path('scripts/incremental-release-functional.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)\nclass API:\n def get(self,route):\n  if '/artifacts?' in route:return {'total_count':1,'artifacts':[p['artifacts'][-1]]}\n  if route.startswith('/actions/artifacts/'):return next(row for row in p['artifacts'] if row['id']==int(route.rsplit('/',1)[1]))\n  attempt=int(route.split('/attempts/')[1].split('/')[0])\n  return {'jobs':p['jobs'][str(attempt)]} if '/jobs?' in route else {**p['run'],'run_attempt':attempt}\n def archive(self,identity):return base64.b64decode(p['bodies'][str(identity)])\nsource={'sha':p['sourceSHA'],'tree':p['sourceTree'],'files':{'scripts/performance-cohorts.mjs':Path('scripts/performance-cohorts.mjs').read_bytes()}}\naggregate=[{'id':998,'run_id':10,'name':'Aggregate informational performance report','status':'completed','conclusion':'success','started_at':'2026-10-31T02:00:00Z','completed_at':'2026-10-31T02:20:00Z'}]\nm.verify_performance(API(),p['run'],aggregate,source,{'baseSHA':p['baseSHA']})\nprint('independent release verifier accepted original ZIPs')\n`;
  const checked=spawnSync(process.env.FARM_TEST_PYTHON??'python',['-B','-c',python],{input:JSON.stringify(payload),encoding:'utf8'});
  assert.equal(checked.status,0,checked.stderr);assert.match(checked.stdout,/accepted original ZIPs/);
 }
});


test('workflow_run publisher sparse checkout includes the real schema validator dependency',()=>{
  const yaml=fs.readFileSync(new URL('../.github/workflows/performance-report-comment.yaml',import.meta.url),'utf8');
  assert.ok(yaml.includes('scripts/performance-pr-reporter.mjs') && yaml.includes('scripts/performance-metrics.mjs'));
  assert.ok(yaml.includes('ref: ${{ github.workflow_sha }}'));
});
