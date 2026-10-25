import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {test} from 'node:test';
const utility=new URL('../packages/cli/dist/utils.js',import.meta.url).href;
for(const [name,expression,expected] of [
 ['native error with an empty stack',"Object.assign(new Error('[DOCS_LINK_MISSING] missing API target'),{stack:''})",'[DOCS_LINK_MISSING] missing API target'],
 ['ordinary error with a stack',"new Error('ordinary build failure')",'Error: ordinary build failure'],
 ['non-Error rejection',"'non-Error build failure'",'non-Error build failure']
]){
 test(`CLI displays ${name} before exiting`,()=>{
  const result=spawnSync(process.execPath,['--input-type=module','-e',`import {handleAsyncOperationErrors} from ${JSON.stringify(utility)};await handleAsyncOperationErrors(Promise.reject(${expression}),'error during build');`],{encoding:'utf8',timeout:15000});
  assert.equal(result.error,undefined);
  assert.equal(result.status,1);
  assert.ok((result.stdout+result.stderr).includes(expected),result.stdout+result.stderr);
 });
}
