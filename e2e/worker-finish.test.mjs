import assert from 'node:assert/strict';
import test from 'node:test';
import {finishWorker,normalizeWorkerFailure} from './worker-finish.mjs';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {setLogFile,logger,flushLogFiles} from './utils.mjs';
test('terminal completion waits for log flush and IPC callback before clean exit',async()=>{
 const events=[];
 await finishWorker({flushLogs:async()=>events.push('flushed'),send:(message,callback)=>{events.push(message.type);queueMicrotask(()=>{events.push('sent');callback();});},
  disconnect:()=>events.push('disconnected'),exit:code=>events.push(code)});
 assert.deepEqual(events,['flushed','done','sent','disconnected',0]);
});
test('fatal completion never sends done or exits successfully',async()=>{
 const events=[];
 await finishWorker({flushLogs:async()=>{},send:(message,callback)=>{events.push(message.type);callback();},disconnect:()=>{},exit:code=>events.push(code)},new Error('cleanup failed'));
 assert.deepEqual(events,['fatal',1]);
});
test('failed log flush cannot produce a successful terminal message',async()=>{
 let sent=false,exited=false;
 await assert.rejects(finishWorker({flushLogs:async()=>{throw Error('flush failed');},send:()=>{sent=true;},disconnect:()=>{},exit:()=>{exited=true;}}),/flush failed/);
 assert.equal(sent,false);assert.equal(exited,false);
});
test('failed terminal delivery cannot produce a successful exit',async()=>{
 let exited=false;
 await assert.rejects(finishWorker({flushLogs:async()=>{},send:(message,callback)=>callback(Error('IPC failed')),disconnect:()=>{},exit:()=>{exited=true;}}),/IPC failed/);
 assert.equal(exited,false);
});
test('actual file log stream is finished before terminal completion is sent',async()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'farm-worker-flush-')),file=path.join(directory,'worker.log');
 setLogFile(file);logger('worker-flush-contract');
 await finishWorker({flushLogs:flushLogFiles,send:(message,callback)=>{
  assert.match(fs.readFileSync(file,'utf8'),/worker-flush-contract/);callback();
 },disconnect:()=>{},exit:code=>assert.equal(code,0)});
});
test('every explicitly rejected value remains fatal and retains its original cause',async()=>{
 for(const value of [undefined,null,0,'',false]){
  const events=[];
  await finishWorker({flushLogs:async()=>{},send:(message,callback)=>{events.push(message.type);callback();},disconnect:()=>{},exit:code=>events.push(code)},value);
  assert.deepEqual(events,['fatal',1]);assert.equal(normalizeWorkerFailure(value).cause,value);
 }
});
