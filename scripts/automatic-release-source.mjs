// Authenticate a prepared version merge; never use a title/branch as authority.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const root=fileURLToPath(new URL('../',import.meta.url));
const sha=value=>crypto.createHash('sha256').update(value).digest('hex');
const git=args=>execFileSync('git',args,{cwd:root,encoding:'utf8',timeout:60000,maxBuffer:32*1024**2});
const api=route=>JSON.parse(execFileSync('gh',['api',route],{encoding:'utf8',timeout:30000,maxBuffer:4*1024**2}));
export function versionFamily(plan) {
 const rows=[...plan.npm.primary.map(row=>['npm',row.name,row.version,row.manifestPath]),
  ...plan.npm.payloads.map(row=>['payload',row.name,row.version,row.manifestPath,row.owner,row.platform]),
  ...plan.cargo.map(row=>['cargo',row.name,row.version,row.manifestPath])];
 assert.equal(new Set(rows.map(row=>row[0]+':'+row[1])).size,rows.length,'Duplicate release identity');
 return rows.sort((a,b)=>JSON.stringify(a).localeCompare(JSON.stringify(b)));
}
export function verifyPreparedMigration({pr,repository,source,runGit=git}) {
 assert.equal(repository.full_name,'farm-fe/farm');assert.equal(repository.id,507542208);
 assert.equal(pr.number,2330);assert.ok(pr.merged_at&&pr.merged===true);
 assert.equal(pr.base.ref,'main');assert.equal(pr.base.repo.id,repository.id);assert.equal(pr.head.repo.id,repository.id);
 assert.equal(pr.merge_commit_sha,source,'Merge is not the fixed event source');
 const policy=JSON.parse(runGit(['show',source+':docs/releases/prepared-release-2330.json']));
 assert.equal(policy.schemaVersion,1);assert.equal(policy.kind,'prepared-first-stable-on-merge');
 assert.equal(policy.repositoryId,repository.id);assert.equal(policy.repository,repository.full_name);assert.equal(policy.pullRequest,pr.number);
 assert.equal(policy.anchorCommit,'640215ed91bf635b60c6931eb7137dd3d0c10ed3');
 assert.equal(policy.changesetsConsumptionCommit,'70404dd9959aef7c4c6cee3be66eeec5e3c6ad74');
 for(const value of [source,pr.head.sha,pr.base.sha])assert.match(value,/^[a-f0-9]{40}$/);
 assert.equal(runGit(['merge-base',policy.anchorCommit,pr.head.sha]).trim(),policy.anchorCommit,'Reviewed source does not descend from prepared anchor');
 assert.equal(runGit(['merge-base',policy.changesetsConsumptionCommit,policy.anchorCommit]).trim(),policy.changesetsConsumptionCommit);
 const sourceTree=runGit(['rev-parse',source+'^{tree}']).trim();
 const headTree=runGit(['rev-parse',pr.head.sha+'^{tree}']).trim();
 if(sourceTree!==headTree) {
  const parent=runGit(['rev-parse',source+'^1']).trim();
  const merged=runGit(['merge-tree','--write-tree',parent,pr.head.sha]).trim().split('\n')[0];
  assert.equal(sourceTree,merged,'Actual merge does not preserve the reviewed change on its target parent');
 }
 assert.equal(policy.candidate,'docs/releases/v2-candidate.json');
 const plan=JSON.parse(runGit(['show',source+':'+policy.candidate]));
 const anchor=JSON.parse(runGit(['show',policy.anchorCommit+':'+policy.candidate]));
 assert.deepEqual(versionFamily(plan),versionFamily(anchor),'Prepared version family changed since reviewed anchor');
 for(const row of [...plan.npm.primary,...plan.npm.payloads]) {
  const manifest=JSON.parse(runGit(['show',source+':'+row.manifestPath]));
  assert.equal(manifest.name,row.name);assert.equal(manifest.version,row.version);assert.notEqual(manifest.private,true);
  if(row.owner)assert.equal(row.version,plan.npm.primary.find(owner=>owner.name===row.owner)?.version,'Native family version mismatch');
 }
 for(const row of plan.cargo) {
  const manifest=runGit(['show',source+':'+row.manifestPath]);
  assert.equal(manifest.match(/^name\s*=\s*"([^"]+)"/m)?.[1],row.name);
  assert.equal(manifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1],row.version);
 }
 // The two post-version fixes are explicitly included in the unpublished
 // family. Preserve their exact original contents, and prove consumption.
 assert.deepEqual(policy.consumedFollowups.map(row=>row.path).sort(),['.changeset/css-cache-recovery.md','.changeset/native-cli-paths.md']);
 for(const row of policy.consumedFollowups) {
  assert.equal(row.archive,'docs/releases/consumed-changesets/2330/'+path.posix.basename(row.path));
  const original=runGit(['show',policy.anchorCommit+':'+row.path]);
  const archived=runGit(['show',source+':'+row.archive]);
  assert.equal(sha(original),row.sha256,'Consumed Changeset differs from reviewed anchor');
  assert.equal(sha(archived),row.sha256,'Consumed Changeset archive changed');
  assert.equal(runGit(['ls-tree','-r','--name-only',source,'--',row.path]).trim(),'','Approved Changeset is still pending');
  const packages=[...original.matchAll(/^"([^"\n]+)": patch$/gm)].map(match=>match[1]).sort();
  assert.ok(packages.length>0);assert.deepEqual(row.includedIn.map(item=>item.name).sort(),packages);
  for(const item of row.includedIn)assert.equal(plan.npm.primary.find(owner=>owner.name===item.name)?.version,item.version,'Consumed fix belongs to a different version');
 }
 const result={mode:'verified-prepared-migration',releaseReady:true,versionPR:pr.number,
  sourceSHA:source,sourceTree,headSHA:pr.head.sha,headTree,baseSHA:pr.base.sha,
  familyHash:sha(JSON.stringify(versionFamily(plan))),policyHash:sha(JSON.stringify(policy)),retainedCrates:policy.retainedCrates};
 return {...result,receiptHash:sha(JSON.stringify(result))};
}
export async function resolveAutomaticRelease({environment=process.env,event=JSON.parse(fs.readFileSync(environment.GITHUB_EVENT_PATH)),readApi=api,runGit=git,resolveNormal}={}) {
 assert.equal(environment.GITHUB_REF,'refs/heads/main');
 const source=environment.GITHUB_SHA;assert.match(source,/^[a-f0-9]{40}$/);
 assert.equal(runGit(['rev-parse','HEAD']).trim(),source);
 const repository=readApi('repos/'+environment.GITHUB_REPOSITORY);
 assert.equal(repository.id,Number(environment.GITHUB_REPOSITORY_ID));assert.equal(repository.default_branch,'main');
 if(environment.GITHUB_EVENT_NAME==='push')assert.ok(event.ref==='refs/heads/main'&&event.after===source&&!event.deleted&&!event.forced,'Invalid main push');
 else if(environment.GITHUB_EVENT_NAME==='pull_request')assert.ok(event.action==='closed'&&event.pull_request?.merged===true&&event.pull_request.merge_commit_sha===source,'Invalid merge event');
 else assert.equal(environment.GITHUB_EVENT_NAME,'workflow_dispatch');
 const associated=readApi(`repos/${environment.GITHUB_REPOSITORY}/commits/${source}/pulls?per_page=100`);
 assert.ok(associated.length<100,'Ambiguous bounded PR association');
 const matching=associated.filter(pr=>pr.merged_at&&pr.merge_commit_sha===source&&pr.base.ref==='main');
 if(matching.length!==1)return {releaseReady:false,mode:'full',reason:'Not one exact prepared-version merge'};
 const pr=readApi(`repos/${environment.GITHUB_REPOSITORY}/pulls/${matching[0].number}`);
 assert.equal(pr.merge_commit_sha,source);assert.equal(pr.head.repo.id,repository.id);assert.equal(pr.base.repo.id,repository.id);
 if(pr.number===2330)return verifyPreparedMigration({pr,repository,source,runGit});
 const normal=resolveNormal??(await import('./changesets-version-runtime.mjs')).resolveMergedVersion;
 return normal({environment,readApi,runGit});
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
 const result=await resolveAutomaticRelease();
 if(process.argv.includes('--cargo')) {
  assert.equal(result.releaseReady,true,'Cargo source is not an authenticated prepared release');
  fs.writeFileSync('cargo-version-proof.json',JSON.stringify(result)+'\n');
 }
 if(process.env.GITHUB_OUTPUT)fs.appendFileSync(process.env.GITHUB_OUTPUT,`release_ready=${result.releaseReady===true}\n`);
 console.log(JSON.stringify(result));
}
