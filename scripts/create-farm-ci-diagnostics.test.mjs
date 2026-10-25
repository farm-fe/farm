import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import test from 'node:test';

const producer=fileURLToPath(new URL('./pack-create-farm-ci-candidates.mjs',import.meta.url));
test('failed pack surfaces child output and keeps failed inventory',()=>{
  const fixture=fs.mkdtempSync(path.join(os.tmpdir(),'farm-candidate-diagnostic-test-'));
  const write=(relative,value)=>{const file=path.join(fixture,relative);fs.mkdirSync(path.dirname(file),{recursive:true});fs.writeFileSync(file,value);};
  try {
    for(const dir of ['packages/core','rust-plugins','js-plugins'])fs.mkdirSync(path.join(fixture,dir),{recursive:true});
    write('packages/core/package.json',JSON.stringify({name:'@farmfe/core',version:'2.0.0'}));
    write('crates/create-farm-rs/templates/react/package.json',JSON.stringify({devDependencies:{'@farmfe/core':'^2.0.0'}}));
    write('fail.cjs',"console.error('EXPECTED_CHILD_FAILURE');process.exit(7);\n");
    for(const args of [['init','--quiet'],['-c','user.name=Farm CI Test','-c','user.email=ci-test@example.invalid','commit','--allow-empty','--quiet','-m','fixture']]) {
      const result=spawnSync('git',args,{cwd:fixture,encoding:'utf8',windowsHide:true});assert.equal(result.status,0,result.stderr);
    }
    const abi=process.platform==='linux'?'linux-x64-gnu':process.platform+'-'+process.arch+(process.platform==='win32'?'-msvc':'');
    const result=spawnSync(process.execPath,[producer,fixture,path.join(fixture,'evidence'),path.join(fixture,'fail.cjs'),abi,'react'],{encoding:'utf8',windowsHide:true});
    assert.equal(result.status,1);assert.match(result.stderr,/EXPECTED_CHILD_FAILURE/);assert.match(result.stderr,/exit 7/);
    const inventory=JSON.parse(fs.readFileSync(path.join(fixture,'evidence/producer.json')));assert.equal(inventory.state,'failed');assert.equal(inventory.published,false);
  } finally {
    assert.equal(path.dirname(path.resolve(fixture)),path.resolve(os.tmpdir()));
    fs.rmSync(fixture,{recursive:true,force:true});
  }
});
