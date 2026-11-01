// cspell:words infolist
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { classifyVersionDiff, verifyVersionProducer } from './changesets-version-only.mjs';
const root = fileURLToPath(new URL('../', import.meta.url));
const digest = value => crypto.createHash('sha256').update(value).digest('hex');
const git = args => execFileSync('git', args, {cwd:root,encoding:'utf8',timeout:60000,maxBuffer:32*1024**2});
const api = route => JSON.parse(execFileSync('gh',['api',route],{encoding:'utf8',timeout:30000,maxBuffer:4*1024**2}));

export function readVersionSnapshots(base, head) {
  for(const sha of [base,head])assert.match(sha,/^[a-f0-9]{40}$/);
  const changes=git(['diff','--name-only','--no-renames','-z',base,head]).split('\0').filter(Boolean);
  const snapshots=[base,head].map(sha=>{
    const rows=git(['ls-tree','-r','-z',sha]).split('\0').filter(Boolean).map(row=>{
      const [metadata,file]=row.split('\t'),[mode,type,oid]=metadata.split(' ');return {file,mode,type,oid};
    });
    const files={},modes={};
    for(const row of rows) {
      if(!changes.includes(row.file)&&!/^((packages|js-plugins|rust-plugins|crates|rust-ecosystem)\/[^/]+\/(?:npm\/[^/]+\/)?)?(package\.json|Cargo\.toml)$/.test(row.file))continue;
      modes[row.file]=row.mode;
      assert.equal(row.type,'blob');assert.ok(['100644','100755'].includes(row.mode));
      files[row.file]=git(['show',sha+':'+row.file]);
    }
    return {files,modes};
  });
  return {before:snapshots[0].files,after:snapshots[1].files,modesBefore:snapshots[0].modes,modesAfter:snapshots[1].modes};
}

export function writeVersionReceipt(pr, environment=process.env) {
  assert.equal(environment.GITHUB_WORKFLOW_REF,`${environment.GITHUB_REPOSITORY}/.github/workflows/release.yaml@refs/heads/main`);
  assert.equal(environment.GITHUB_JOB,'changeset-version');
  const head=git(['rev-parse','HEAD']).trim();assert.equal(head,pr.head.sha);
  const snapshots=readVersionSnapshots(pr.base.sha,head),consumed={},generated={};
  for(const [file,text] of Object.entries(snapshots.before))if(file.startsWith('.changeset/')&&file.endsWith('.md')&&!(file in snapshots.after))consumed[file]=digest(text);
  for(const [file,text] of Object.entries(snapshots.after))if(file.startsWith('docs/releases/')||file.startsWith('docs/release-evidence/'))generated[file]=digest(text);
  const receipt={kind:'changesets-version-preparation',repositoryId:Number(environment.GITHUB_REPOSITORY_ID),pr:pr.number,
    baseSHA:pr.base.sha,headSHA:head,headTree:git(['rev-parse','HEAD^{tree}']).trim(),runId:Number(environment.GITHUB_RUN_ID),attempt:Number(environment.GITHUB_RUN_ATTEMPT),consumed,generated};
  const classification=classifyVersionDiff({...snapshots,receipt});
  receipt.semanticMode=classification.mode;
  fs.mkdirSync(path.join(root,'version-provenance'),{recursive:true});
  fs.writeFileSync(path.join(root,'version-provenance/ledger.json'),JSON.stringify(receipt)+'\n',{flag:'wx'});
  return receipt;
}

export function extractVersionReceipt(archiveBytes) {
  return execFileSync(process.env.FARM_GRAPH_PYTHON??'python3',['-c',`import io,sys,zipfile,stat
b=sys.stdin.buffer.read()
with zipfile.ZipFile(io.BytesIO(b)) as z:
 r=z.infolist();assert len(r)==1 and r[0].filename=='ledger.json' and 0<r[0].file_size<262144
 assert stat.S_IFMT(r[0].external_attr>>16) in [0,stat.S_IFREG]
 sys.stdout.buffer.write(z.read('ledger.json'))`],{input:archiveBytes,timeout:10000,maxBuffer:262144});
}

export async function resolveVersionEvidence({environment=process.env,event=JSON.parse(fs.readFileSync(environment.GITHUB_EVENT_PATH)),readApi=api,runGit=git,readSnapshots=readVersionSnapshots,
  download=(route)=>execFileSync('gh',['api',route],{timeout:30000,maxBuffer:1024**2}),extractReceipt=extractVersionReceipt,
  wait=()=>new Promise(resolve=>setTimeout(resolve,5000))}={}) {
  const api=readApi,git=runGit;
  const repository=environment.GITHUB_REPOSITORY;
  assert.match(repository,/^[\w.-]+\/[\w.-]+$/);
  const number=Number(event.pull_request?.number??event.number??environment.VERSION_PR);
  if(!Number.isSafeInteger(number)||number<=0)return {mode:'full',reason:'Not a version PR event'};
  const repo=api('repos/'+repository),pr=api(`repos/${repository}/pulls/${number}`);
  if(pr.user.id!==41898282||pr.user.type!=='Bot'||pr.base.ref!=='main'||pr.head.repo.id!==repo.id)return {mode:'full',reason:'Not an authenticated same-repository version bot PR'};
  const eventHead=event.pull_request?.head.sha??environment.GITHUB_SHA;
  const requestedBase=event.inputs?.base_sha??event.inputs?.release_base;
  if(requestedBase)assert.equal(requestedBase,pr.base.sha);
  assert.equal(eventHead,pr.head.sha,'PR moved after event');
  const runs=api(`repos/${repository}/actions/workflows/release.yaml/runs?per_page=20`).workflow_runs;
  for(const summary of runs) {
    // Only completed success can grant a fastlane. A running producer is a miss;
    // validation dispatch waits for this exact producer to finish before trust.
    let run=api(`repos/${repository}/actions/runs/${summary.id}`);
    // Dispatch follows upload in the producer's last step. Wait only for this
    // matching in-flight run to finish; never trust its intermediate status.
    for(let poll=0;run.head_sha===pr.base.sha&&run.status!=='completed'&&poll<6;poll++) {
      await wait();
      run=api(`repos/${repository}/actions/runs/${summary.id}`);
    }
    if(run.status!=='completed'||run.conclusion!=='success') {
      if(run.head_sha===pr.base.sha)return {mode:'full',reason:'Latest version producer for this base has not succeeded'};
      continue;
    }
    const name=`version-provenance-${number}-${run.id}-${run.run_attempt}`;
    const rows=api(`repos/${repository}/actions/runs/${run.id}/artifacts?per_page=2&name=${encodeURIComponent(name)}`).artifacts;
    if(rows.length!==1||rows[0].expired)continue;
    const artifact=api(`repos/${repository}/actions/artifacts/${rows[0].id}`);
    assert.ok(artifact.size_in_bytes>0&&artifact.size_in_bytes<1024**2);
    const archiveBytes=download(`repos/${repository}/actions/artifacts/${artifact.id}/zip`);
    assert.equal(archiveBytes.length,artifact.size_in_bytes);
    const receiptBytes=extractReceipt(archiveBytes);
    const receipt=JSON.parse(receiptBytes);
    if(receipt.headSHA!==pr.head.sha||receipt.baseSHA!==pr.base.sha)continue;
    const jobs=api(`repos/${repository}/actions/runs/${run.id}/attempts/${run.run_attempt}/jobs?per_page=100`).jobs;
    const matching=jobs.filter(row=>row.name==='Prepare reviewed version PR');assert.equal(matching.length,1);
    const job={...matching[0],run_attempt:run.run_attempt};
    verifyVersionProducer({pr,repository:repo,run,job,artifact,receipt,receiptBytes,archiveBytes});
    const step=job.steps.find(row=>row.name==='Upload version-only provenance');
    assert.ok(step.started_at<=artifact.created_at&&artifact.created_at<=step.completed_at,'Upload attempt binding differs');
    git(['fetch','--no-tags','origin',pr.base.sha,pr.head.sha]);
    assert.equal(git(['rev-parse',pr.head.sha+'^{tree}']).trim(),receipt.headTree);
    return {...classifyVersionDiff({...readSnapshots(pr.base.sha,pr.head.sha),receipt}),receiptHash:digest(receiptBytes),headTree:receipt.headTree,baseSHA:receipt.baseSHA,headSHA:receipt.headSHA};
  }
  return {mode:'full',reason:'No complete successful producer evidence for this exact version PR'};
}

// Release entry is a reviewed version commit, not an empty Changesets directory.
// Read only: no branch creation, approval, workflow dispatch or registry writes.
export async function resolveMergedVersion({environment=process.env,readApi=api,runGit=git,resolve=resolveVersionEvidence}={}) {
  assert.equal(environment.GITHUB_REF,'refs/heads/main');
  const source=environment.GITHUB_SHA;assert.match(source,/^[a-f0-9]{40}$/);
  assert.equal(runGit(['rev-parse','HEAD']).trim(),source);
  const repository=environment.GITHUB_REPOSITORY;
  const prs=readApi(`repos/${repository}/commits/${source}/pulls?per_page=100`);
  const matches=prs.filter(pr=>pr.merged_at&&pr.merge_commit_sha===source&&pr.base.ref==='main'&&pr.user?.id===41898282);
  if(matches.length!==1)return {releaseReady:false,mode:'full',reason:'Fixed main source is not one authenticated merged Changesets version PR'};
  const pr=readApi(`repos/${repository}/pulls/${matches[0].number}`);
  const result=await resolve({environment:{...environment,GITHUB_SHA:pr.head.sha,VERSION_PR:String(pr.number)},event:{pull_request:pr},readApi,runGit});
  if(result.mode!=='verified-version-only'||!result.receiptHash)return {...result,releaseReady:false,reason:'Merged version semantic/provenance proof incomplete'};
  assert.equal(runGit(['rev-parse',source+'^{tree}']).trim(),result.headTree,'Main merge tree differs from verified version PR');
  return {...result,releaseReady:true,versionPR:pr.number,sourceSHA:source};
}

if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  let result;
  try {
    if(process.argv[2]==='produce')result=writeVersionReceipt(api(`repos/${process.env.GITHUB_REPOSITORY}/pulls/${process.env.VERSION_PR}`));
    else if(process.argv[2]==='--release-source')result=await resolveMergedVersion();
    else result=await resolveVersionEvidence();
  } catch(error) {
    if(process.argv[2]==='produce')throw error;
    result={mode:'full',reason:error.message};
  }
  if(process.env.GITHUB_OUTPUT)fs.appendFileSync(process.env.GITHUB_OUTPUT,`mode=${result.mode??'full'}\nproof_hash=${result.receiptHash??''}\nrelease_ready=${result.releaseReady===true}\n`);
  console.log(JSON.stringify(result));
}
