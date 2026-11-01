import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';
const workflow=YAML.parse(fs.readFileSync(new URL('../.github/workflows/ci.yaml',import.meta.url),'utf8'));
const gate=workflow.jobs['incremental-execution-gate'];
const code=gate.steps[0].run;
const full=()=>Object.fromEntries(gate.needs.map(key=>[key,{result:'success',outputs:{mode:'full'}}]));
const run=rows=>spawnSync('python3',['-c',code],{encoding:'utf8',env:{...process.env,RESULTS:JSON.stringify(rows)}});
test('actual always gate accepts every completed planned platform',()=>{assert.equal(gate.if,'always()');assert.equal(run(full()).status,0);});
for(const state of ['failure','cancelled','skipped'])for(const key of gate.needs)test(`actual gate rejects ${key} ${state}`,()=>{const rows=full();rows[key].result=state;assert.notEqual(run(rows).status,0);});
function version(){const rows=full();for(const [key,row]of Object.entries(rows))if(!['version-classification','type-check'].includes(key))row.result='skipped';rows['version-classification'].outputs={mode:'verified-version-only',proof_hash:'a'.repeat(64)};return rows;}
test('verified version PR has explicit lightweight success and planned skips',()=>assert.equal(run(version()).status,0));
for(const mutation of ['missing','extra','proof','type-failed','unexpected-heavy'])test(`version gate rejects ${mutation}`,()=>{const rows=version();if(mutation==='missing')delete rows['primary-linux'];if(mutation==='extra')rows.unexpected={result:'skipped'};if(mutation==='proof')rows['version-classification'].outputs.proof_hash='z'.repeat(64);if(mutation==='type-failed')rows['type-check'].result='failure';if(mutation==='unexpected-heavy')rows['primary-linux'].result='success';assert.notEqual(run(rows).status,0);});
test('each primary platform advances its own native and suites without other platform dependencies',()=>{for(const key of ['primary-linux','primary-macos','primary-windows'])assert.ok(!workflow.jobs[key].needs.some(name=>name.startsWith('primary-')));const primary=YAML.parse(fs.readFileSync(new URL('../.github/workflows/ci-primary-platform.yaml',import.meta.url),'utf8'));assert.ok(Object.values(primary.jobs).some(job=>job.uses?.includes('rust-build-user')));assert.ok(Object.values(primary.jobs).filter(job=>job.uses?.includes('test')).every(job=>!JSON.stringify(job.needs).includes('primary-linux')));});
