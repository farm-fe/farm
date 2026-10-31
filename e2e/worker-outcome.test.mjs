import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { runWorker } from '../scripts/test-e2e.mjs';

function fixture(body) {
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'farm-e2e-worker-outcome-'));
  const file=path.join(directory,'worker.mjs');
  fs.writeFileSync(file, `process.on('message',()=>{${body}});`);
  return file;
}
const passed="{type:'results',example:'before',results:[{fullName:'before',passed:true,skipped:false,duration:1}]}";
test('unhandled crash after a passing result fails missing cases and worker lifecycle',async()=>{
  const active=new Set(),file=fixture(`process.send(${passed},()=>{Promise.reject(new Error('EXPECTED_WORKER_CRASH'));});`);
  const results=await runWorker(['before','tree-shake-antd','after'],0,file,'unused',active);
  assert.equal(results.get('before')[0].passed,true);
  assert.equal(results.get('tree-shake-antd')[0].passed,false);
  assert.equal(results.get('after')[0].passed,false);
  assert.equal(results.get('__worker_1__')[0].passed,false);
  assert.equal(active.size,0);
});
test('fatal IPC error remains failed even if all example results arrived',async()=>{
  const file=fixture(`process.send(${passed},()=>{process.send({type:'fatal',error:'EXPECTED_FATAL'});});`);
  const results=await runWorker(['before'],0,file,'unused',new Set());
  assert.match(results.get('__worker_1__')[0].error.message,/EXPECTED_FATAL/);
});
test('zero exit with done but missing results is still a failed run',async()=>{
  const file=fixture(`process.send(${passed},()=>{process.send({type:'done'});});`);
  const results=await runWorker(['before','missing'],0,file,'unused',new Set());
  assert.equal(results.get('missing')[0].passed,false);
});
test('fully reported clean worker preserves passing results',async()=>{
  const file=fixture(`process.send(${passed},()=>{process.send({type:'done'});});`);
  const results=await runWorker(['before'],0,file,'unused',new Set());
  assert.equal(results.size,1);
  assert.equal(results.get('before')[0].passed,true);
});
