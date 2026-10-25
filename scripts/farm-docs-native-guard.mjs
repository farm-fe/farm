import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {isMainThread,threadId} from 'node:worker_threads';

// QA controls are private; Core intentionally exposes FARM_ and VITE_ values.
// Remove inherited controls from older drivers before any config compilation.
const legacyPrivateSuffixes=['CACHE_ATTEMPT','CACHE_RECOVERY_ATTEMPT','CANCEL_ATTEMPT','DISCOVERY_ATTEMPT','EVIDENCE_PHASE','KEEP_DIAGNOSTICS','METADATA_ATTEMPT','NATIVE_BASELINE','NATIVE_GUARD_DIR','OUTPUT_SEAL_ATTEMPT','PROBE_DIR','SEAL_ATTEMPT','SEO_ATTEMPT','SNAPSHOT_LIFECYCLE_ATTEMPT','TRANSACTION_ATTEMPT','WORKER_ATTEMPT'];
for(const suffix of legacyPrivateSuffixes)delete process.env['FARM'+'_DOCS_'+suffix];

// This preload only verifies test inputs; it never loads or replaces a native binary.
const baselinePath=process.env.CODEX_DOCS_NATIVE_BASELINE;
// A worker shares its owner's PID. Its forced termination need not run exit
// callbacks; the owning main thread observes before/after the entire process.
if(baselinePath&&isMainThread){
 const baselineText=fs.readFileSync(baselinePath,'utf8');
 const baseline=JSON.parse(baselineText);
 if(!Array.isArray(baseline.binaries)||!baseline.binaries.length)throw Error('[DOCS_NATIVE_DRIFT] empty native baseline');
 const record={pid:process.pid,threadId,scope:'process',baselinePath,baselineSha256:crypto.createHash('sha256').update(baselineText).digest('hex').toUpperCase(),checks:[]};
 const directory=process.env.CODEX_DOCS_NATIVE_GUARD_DIR;
 const recordName=`native-guard-${process.pid}-${crypto.randomUUID()}.json`;
 const save=()=>{if(directory){fs.mkdirSync(directory,{recursive:true});fs.writeFileSync(path.join(directory,recordName),JSON.stringify(record,null,2));}};
 const check=phase=>{
  const observed=baseline.binaries.map(item=>{
   let sha256,error;
   try{sha256=crypto.createHash('sha256').update(fs.readFileSync(item.path)).digest('hex').toUpperCase();}catch(value){error=value.code??String(value);}
   return {name:item.name,path:item.path,expected:item.sha256,sha256,error};
  });
  const changes=observed.filter(item=>item.error||item.sha256!==item.expected).map(item=>item.name);
  try{if(fs.readFileSync(baselinePath,'utf8')!==baselineText)changes.push('baseline-file');}catch{changes.push('baseline-file');}
  record.checks.push({phase,at:new Date().toISOString(),observed,changes});save();
  return changes;
 };
 const initial=check('before');
 if(initial.length)throw Error(`[DOCS_NATIVE_DRIFT] before execution: ${initial.join(', ')}`);
 process.once('exit',()=>{
  const changes=check('after');
  if(changes.length){process.stderr.write(`[DOCS_NATIVE_DRIFT] after execution: ${changes.join(', ')}\n`);process.exitCode=1;}
 });
}
