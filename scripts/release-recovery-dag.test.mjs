import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import {parse} from 'yaml';
// cspell:words finalizer prepublish postpublish
const text=fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml',import.meta.url),'utf8');
const jobs=parse(text).jobs;
const identity=jobs['candidate-identity'];
function selected(attempt,current){
 const folder=fs.mkdtempSync(path.join(os.tmpdir(),'release-candidate-attempt-'));
 const output=path.join(folder,'output');
 try{
  execFileSync(process.env.FARM_PERFORMANCE_PYTHON??'python3',['-c',identity.steps[0].run],{env:{PATH:process.env.PATH,CANDIDATE_ATTEMPT:String(attempt),GITHUB_RUN_ATTEMPT:String(current),GITHUB_OUTPUT:output}});
  return fs.readFileSync(output,'utf8').trim();
 }finally{fs.rmSync(folder,{recursive:true,force:true});}
}
test('actual YAML selector preserves fresh, carried and frozen producer attempts',()=>{
 assert.equal(identity.steps[0].env.CANDIDATE_ATTEMPT,"${{ needs.source.outputs.reuse == 'true' && needs.source.outputs.producer_attempt || needs.producer.outputs.candidate_attempt }}");
 for(const [producer,consumer] of [[1,1],[2,2],[2,3],[1,3]])assert.equal(selected(producer,consumer),`attempt=${producer}`);
 assert.throws(()=>selected(4,3));assert.throws(()=>selected('',3));
 assert.equal(jobs.producer.outputs.candidate_attempt,'${{ github.run_attempt }}');
 assert.equal(jobs.plan.outputs.plan_attempt,'${{ github.run_attempt }}');
});
test('carried plan and newly frozen candidate have independent attempt names',()=>{
 const plan=jobs.producer.steps.find(step=>step.uses==='actions/download-artifact@v4'&&step.with.path==='release-work');
 assert.equal(plan.with.name,'independent-release-plan-${{ github.run_id }}-${{ needs.plan.outputs.plan_attempt }}');
 const store=new Set(['independent-release-plan-50-1','farm-release-candidates-source-2']);
 assert.ok(store.has(plan.with.name.replace('${{ github.run_id }}','50').replace('${{ needs.plan.outputs.plan_attempt }}','1')));
 for(const name of ['publisher','public-proof','finalize']){
  assert.ok(jobs[name].needs.includes('candidate-identity'));
  if(jobs[name].uses){
   assert.equal(jobs[name].with.candidate_attempt,'${{ needs.candidate-identity.outputs.attempt }}');
   const child=parse(fs.readFileSync(new URL('../'+jobs[name].uses.replace('./',''),import.meta.url),'utf8'));
   for(const job of Object.values(child.jobs)){
    const download=job.steps.find(step=>step.uses==='actions/download-artifact@v4');
    assert.equal(download.with.name,'farm-release-candidates-${{ github.sha }}-${{ inputs.candidate_attempt }}');
    assert.ok(store.has(download.with.name.replace('${{ github.sha }}','source').replace('${{ inputs.candidate_attempt }}','2')));
   }
  }else{
   const download=jobs[name].steps.find(step=>step.uses==='actions/download-artifact@v4');
   assert.equal(download.with.name,'farm-release-candidates-${{ github.sha }}-${{ needs.candidate-identity.outputs.attempt }}');
   assert.ok(store.has(download.with.name.replace('${{ github.sha }}','source').replace('${{ needs.candidate-identity.outputs.attempt }}','2')));
  }
 }
});
test('main-test wait and its wake-up controller are absent; retries retain the original frozen producer',()=>{
 assert.equal(jobs['main-evidence'],undefined);
 assert.equal(jobs['sign-pending-evidence'],undefined);
 for(const file of ['.github/workflows/release-evidence-recovery.yaml','scripts/release-evidence-recovery.py'])
  assert.equal(fs.existsSync(new URL('../'+file,import.meta.url)),false);
 assert.doesNotMatch(text,/workflow_run:|functional_evidence|incremental-release-functional/);
 assert.ok(text.includes('module.original_attempt(module.actions_transport()'));
 assert.ok(text.includes('incremental-release-entry.py'));
 assert.doesNotMatch(text,/incremental-release-publish\.py/);
 assert.ok(text.includes('release-npm-best-effort.py'));
});

test('actual YAML selects exact version presence and fails partial publication without durable state',()=>{
 const plan=jobs.plan.steps.find(step=>step.id==='release-plan');
 assert.equal(plan.run,'python -B scripts/incremental-release-entry.py plan --version-presence --source "$SOURCE_SHA" --output release-plan.json');
 const publish=jobs.publisher.steps.find(step=>step.id==='publish-packages');
 assert.equal(publish.run,'python -B scripts/release-npm-best-effort.py --candidate release-candidate --output publication-results/results.json');
 assert.equal(publish.env.FARM_EXPLICIT_PUBLICATION,'true');
 assert.deepEqual(jobs.publisher.permissions,{contents:'read',actions:'read','id-token':'write'});
 assert.equal(jobs.publisher['continue-on-error'],undefined);
 assert.equal(publish['continue-on-error'],undefined);
 const outcomes=jobs.publisher.steps.find(step=>step.uses==='actions/upload-artifact@v4');
 assert.equal(outcomes.if,'always()');
 assert.equal(outcomes.with.path,'publication-results/results.json');
 assert.equal(outcomes.with.name,'npm-publication-results-${{ github.run_id }}-${{ github.run_attempt }}');
 assert.equal(outcomes.with['if-no-files-found'],'error');
 const workflow=parse(text);
 assert.equal(workflow.concurrency.group,'farm-independent-release');
 assert.equal(workflow.concurrency['cancel-in-progress'],false);
 const publisher=fs.readFileSync(new URL('./release-npm-best-effort.py',import.meta.url),'utf8');
 assert.doesNotMatch(publisher,/GitRefStore\(|GitHubStateAPI\(|state\.Publication\(|frozen_authorization\(|\.recover\(|\.handoff\(/);
 assert.match(publisher,/return 0 if report\['complete'\] and report\['success'\] else 1/);
 const entry=fs.readFileSync(new URL('./incremental-release-entry.py',import.meta.url),'utf8');
 const presence=entry.split('if args.version_presence:')[1].split('elif args.full_source:')[0];
 assert.ok(presence.includes("selector.collect(source, 'npm'"));
 assert.ok(presence.includes('source_plan(source, registry_snapshot=snapshot)'));
 assert.doesNotMatch(presence,/require_idle_publication|published_baseline|GitRefStore|state\.Publication|stateBaseline|baseline=/);
 assert.doesNotMatch(plan.run,/--baseline|--state|--full-source/);
 assert.ok(jobs.producer.if.includes("needs.plan.outputs.transaction_required == 'true'"));
 assert.ok(jobs.publisher.if.includes("needs.plan.outputs.transaction_required == 'true'"));
 assert.ok(jobs['no-publication'].if.includes("needs.plan.outputs.transaction_required != 'true'"));
 const fallback=jobs['release-report'];assert.ok(fallback.if.includes('always()'));
 assert.ok(fallback.needs.includes('producer')&&fallback.needs.includes('publisher'));
 assert.equal(fallback.steps.find(step=>step.uses==='actions/upload-artifact@v4').if,'always()');
});

test('publisher reruns execute protected main and cannot bypass its attempt guard through workflow skipping or legacy recovery',()=>{
 const publisher=jobs.publisher;
 const command=publisher.steps.find(step=>step.id==='publish-packages');
 assert.equal(command.run,'python -B scripts/release-npm-best-effort.py --candidate release-candidate --output publication-results/results.json');
 assert.equal(command.if,undefined);
 assert.equal(command['continue-on-error'],undefined);
 assert.equal(publisher['continue-on-error'],undefined);
 assert.doesNotMatch(publisher.if,/run_attempt|runAttempt|GITHUB_RUN_ATTEMPT/);
 for(const step of publisher.steps)if(step.run)assert.doesNotMatch(step.run,/incremental-release-publish\.py|--recover\b|--handoff\b|--finalize\b/);
});

test('publisher and finalizer retain distinct original-byte and complete public-proof gates',()=>{
 const publisher=fs.readFileSync(new URL('./release-npm-best-effort.py',import.meta.url),'utf8');
 const publish=publisher.slice(publisher.indexOf('def main():'));
 const authorization=publish.indexOf('authorize(manifest, directory, os.environ)');
 const writer=publish.indexOf('with publisher.RegistryWriter(');
 assert.ok(authorization>=0&&writer>authorization);
 assert.match(publisher,/SourceProofVerifier\(transport, current\)\.verify\(manifest, directory \/ 'archives', phase='prepublish'\)/);
 assert.match(publisher,/proof\.get\('sourceAndSealVerified'\) is True/);
 assert.match(publisher,/entry\.strict_archive\(directory, row\)/);
 assert.match(publisher,/publisher\.npm_tarball\.validate\(row, body\)/);
 assert.match(publisher,/registry\.lookup\(row\)/);
 assert.match(publisher,/actual is None or actual == row\['archiveSha256'\]/);
 assert.match(publisher,/entry\.evidence\.successful_job\(jobs, entry\.PUBLISHER\)/);
 assert.match(publisher,/\(SourceProofVerifier if manifest\.get\('selectionPolicy'\) == 'registry-exact-version-v2' else publisher\.RecoveryVerifier\)\(transport, current\)\.verify\(manifest, directory \/ 'archives', phase='postpublish'\)/);
 assert.match(publisher,/return entry\.ReleaseVerifier\.verify\(self, manifest, archive_directory, phase='postpublish', current_run=self\.current\)/);
 assert.match(publisher,/return super\(\)\.verify\(manifest, archive_directory, phase=phase, current_run=current_run\)/);
 assert.match(publisher,/proof\.get\('verified'\) is True/);
 assert.match(publisher,/proof = authorize\(manifest, directory, environment, finalize=True\)/);
 assert.match(publisher,/publisher\.confirm_release\(publisher\.GitHubWriter\(environment, 'finalize'\), manifest, proof\)/);
 const finalize=jobs.finalize.steps.find(step=>step.run?.includes(' --finalize '));
 assert.equal(finalize.run,'python -B scripts/release-npm-best-effort.py --finalize --candidate release-candidate');
 assert.equal(finalize.env.FARM_EXPLICIT_PUBLICATION,'true');
 assert.deepEqual(jobs.finalize.permissions,{contents:'write',actions:'read'});
 for(const dependency of ['publisher','public-proof','sign-public-acceptance','candidate-identity']){
  assert.ok(jobs.finalize.needs.includes(dependency));
  assert.ok(jobs.finalize.if.includes(`needs.${dependency}.result == 'success'`));
 }
});

test('all release needs remain acyclic and mandatory candidate checks cannot be skipped',()=>{
 const done=new Set(),active=new Set();function visit(name){assert.ok(jobs[name]);assert.ok(!active.has(name));if(done.has(name))return;active.add(name);for(const need of jobs[name].needs??[])visit(need);active.delete(name);done.add(name);}
 for(const name of Object.keys(jobs))visit(name);
 for(const name of ['publisher'])assert.ok(jobs[name].if.includes("needs.candidate-identity.result == 'success'"));
 assert.ok(identity.if.includes("needs.source.outputs.reuse == 'true'"));
});

function condition(job, { results = {}, cancelled = false, event = 'push' } = {}) {
 const expression = job.if.replace(/^\$\{\{\s*|\s*\}\}$/g, '')
   .replace(/!cancelled\(\)/g, String(!cancelled))
   .replace(/needs\.([\w-]+)\.result/g, (_, name) => JSON.stringify(results[name] ?? 'success'))
   .replace(/needs\.producer\.outputs\.seal_id/g, JSON.stringify('90'))
   .replace(/github\.repository/g, JSON.stringify('farm-fe/farm'))
   .replace(/github\.event_name/g, JSON.stringify(event))
   .replace(/github\.ref/g, JSON.stringify('refs/heads/main'));
 // Only the fixed, reviewed boolean grammar is evaluated by this local model.
 assert.match(expression, /^[\s\w'"!&|=()./-]+$/);
 return Function(`return (${expression})`)();
}
test('actual public/recovery conditions admit carried ancestry but reject direct failure and cancellation', () => {
 for (const name of ['public-proof', 'finalize', 'sign-public-acceptance', 'sign-frozen-release']) {
  const job = jobs[name];
  assert.match(job.if, /!cancelled\(\)/);
  const carried = { plan: 'skipped', quality: 'skipped' };
  if (!job.needs.includes('producer')) carried.producer = 'skipped';
  assert.equal(condition(job, { results: carried }), true, name);
  assert.equal(condition(job, { cancelled: true }), false, name);
  for (const dependency of job.needs) for (const result of ['failure', 'cancelled', 'skipped']) {
   assert.equal(condition(job, { results: { ...carried, [dependency]: result } }), false, `${name}/${dependency}/${result}`);
  }
  if (name.startsWith('sign-')) assert.equal(condition(job, { event: 'workflow_dispatch' }), false);
 }
});
test('nested workflow concurrency isolates distinct caller runs while retaining platform identity', () => {
 for (const file of ['farm-docs', 'rust-build-user', 'rust-build-release']) {
  const workflow = parse(fs.readFileSync(new URL(`../.github/workflows/${file}.yaml`, import.meta.url), 'utf8'));
  const groups = [workflow.concurrency, ...Object.values(workflow.jobs).map(job => job.concurrency)].filter(Boolean);
  assert.ok(groups.length);
  for (const { group } of groups) {
   assert.ok(group.includes('github.run_id'), file);
   const render = (run, platform) => group.replace(/\$\{\{\s*github.run_id\s*\}\}/g, String(run)).replace(/\$\{\{[^}]*platform[^}]*\}\}/g, platform);
   assert.notEqual(render(10, 'linux'), render(11, 'linux'));
   if (group.includes('platform')) assert.notEqual(render(10, 'linux'), render(10, 'windows'));
  }
 }
});

test('empty observation and failed native cells still reach the source seal while failed publication cannot finalize',()=>{
 const evaluate=(name,values={})=>{
  const base={
   'github.ref':'refs/heads/main','github.repository':'farm-fe/farm','github.event_name':'push',
   'inputs.publish':false,'inputs.producer_run':'',
   'needs.source.result':'success','needs.plan.result':'success','needs.producer.result':'success',
   'needs.publisher.result':'success','needs.candidate-identity.result':'success','needs.sign-frozen-release.result':'success',
   'needs.source.outputs.reuse':'false','needs.source.outputs.version_ready':'true',
   'needs.plan.outputs.transaction_required':'true','needs.plan.outputs.publish_required':'false','needs.plan.outputs.build_required':'false',
   'needs.public-proof.result':'success','needs.sign-public-acceptance.result':'success'};
  const expression=jobs[name].if.replace(/^\$\{\{\s*|\s*\}\}$/g,'')
   .replace(/!cancelled\(\)|always\(\)/g,'true')
   .replace(/(?:needs|github|inputs)\.[\w.-]+/g,key=>{assert.ok(Object.hasOwn({...base,...values},key),'Unbound DAG condition: '+key);return JSON.stringify({...base,...values}[key]);});
  assert.match(expression,/^[\s\w'"!&|=()./-]+$/);return Function(`return (${expression})`)();
 };
 assert.equal(evaluate('producer'),true);
 assert.equal(evaluate('publisher'),true);
 assert.equal(evaluate('producer',{'needs.release-native-primary.result':'failure','needs.release-native-extra.result':'success','needs.plan.outputs.build_required':'true'}),true);
 assert.equal(evaluate('publisher',{'needs.sign-frozen-release.result':'failure'}),false);
 assert.equal(evaluate('publisher',{'github.event_name':'workflow_dispatch'}),false);
 assert.equal(evaluate('release-report',{'needs.producer.result':'failure','needs.publisher.result':'skipped'}),true);
 assert.equal(evaluate('finalize',{'needs.publisher.result':'failure'}),false);
 assert.equal(evaluate('finalize',{'needs.public-proof.result':'failure'}),false);
 assert.equal(evaluate('finalize',{'needs.sign-public-acceptance.result':'failure'}),false);
});

const sourceFamily=JSON.parse(fs.readFileSync(new URL('../docs/releases/v2-candidate.json',import.meta.url),'utf8')).npm;
const expectedFamily=[...sourceFamily.primary,...sourceFamily.payloads].map(({name,version})=>({name,version}));
function fallbackInline(jobName,outcome,existing){
 const step=jobs[jobName].steps.find(step=>step.shell==='python'&&step.run.includes("Path('publication-results/results.json')"));
 assert.ok(step,jobName);
 assert.equal(step.if,'always()');
 const control=jobName==='publisher'?'PACKAGE_EXECUTION':'PUBLISHER_RESULT';
 const folder=fs.mkdtempSync(path.join(os.tmpdir(),'release-missing-report-'));
 const output=path.join(folder,'publication-results/results.json');
 const summary=path.join(folder,'summary');
 try{
  fs.mkdirSync(path.join(folder,'docs/releases'),{recursive:true});
  fs.writeFileSync(path.join(folder,'docs/releases/v2-candidate.json'),JSON.stringify({npm:sourceFamily}));
  if(existing!==undefined){fs.mkdirSync(path.dirname(output),{recursive:true});fs.writeFileSync(output,existing);}
  const env={PATH:process.env.PATH,GITHUB_SHA:'a'.repeat(40),GITHUB_RUN_ID:'991',GITHUB_RUN_ATTEMPT:'2',GITHUB_STEP_SUMMARY:summary};
  for(const key of Object.keys(step.env))env[key]='skipped';
  if(outcome===undefined)delete env[control];else env[control]=outcome;
  execFileSync(process.env.FARM_PERFORMANCE_PYTHON??'python3',['-c',step.run],{cwd:folder,env});
  const raw=fs.readFileSync(output,'utf8');
  return {raw,report:JSON.parse(raw),summary:fs.existsSync(summary)?fs.readFileSync(summary,'utf8'):''};
 }finally{fs.rmSync(folder,{recursive:true,force:true});}
}
for(const jobName of ['publisher','release-report']){
 for(const outcome of ['failure','cancelled','success','','skipped',undefined]){
  test(`missing results in ${jobName} preserve full-family uncertainty for ${outcome===undefined?'absent':outcome||'empty'} execution outcome`,()=>{
   const {report,summary}=fallbackInline(jobName,outcome);
   const skipped=outcome==='skipped';
   assert.deepEqual(report.packages.map(({name,version})=>({name,version})),expectedFamily);
   for(const row of report.packages){
    assert.equal(row.status,skipped?'not-attempted':'unknown');
    assert.equal(row.uploadAttempted,skipped?false:null);
    assert.equal(row.outcomeUncertain,!skipped);
   }
   assert.equal(report.stage,skipped?(jobName==='publisher'?'package-execution-skipped':'publisher-skipped'):'publication-outcome-unknown');
   assert.equal(report.stages[jobName==='publisher'?'PACKAGE_EXECUTION':'PUBLISHER_RESULT'],outcome??'');
   for(const field of ['complete','success','baselineAdvanced','releaseCreatedByThisStage','inventoryVerified'])assert.equal(report[field],false);
   assert.match(summary,/stage outcomes:/);
   if(!skipped){assert.match(summary,/upload outcomes are unknown/);assert.doesNotMatch(summary,/uploads were not started|execution was skipped/);}
  });
 }
 test(`${jobName} fallback never overwrites existing package outcomes`,()=>{
  const existing=JSON.stringify({kind:'existing-result',success:false,packages:[{name:'example',version:'1.0.0',status:'published',uploadAttempted:true}]},null,3)+'\n';
  for(const outcome of ['failure','cancelled','success','','skipped',undefined])assert.equal(fallbackInline(jobName,outcome,existing).raw,existing);
 });
}
