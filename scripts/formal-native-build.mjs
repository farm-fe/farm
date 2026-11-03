// Prepare and measure the actual compiler environment, resolve trusted main
// products on the host, then execute the same formal recipe for every miss.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
export function runFormalNativeBuild({env=process.env,run=spawnSync,root=process.cwd()}={}) {
 const abi=env.FARM_BUILD_ABI;assert.match(abi,/^(?:linux|darwin|win32)-[a-z0-9-]+$/);
 const build=(extra={})=>{
  const command=env.FARM_BUILD_IMAGE?process.execPath:'bash';
  const args=env.FARM_BUILD_IMAGE?['scripts/v2-docker-cargo-cache.mjs']:['scripts/ci-build-rust-artifacts.sh'];
  const result=run(command,args,{cwd:root,env:{...env,FARM_NATIVE_PROOF_MODE:'shadow',...extra},stdio:'inherit',timeout:130*60*1000});
  assert.ok(!result.error&&result.status===0,'Formal native recipe failed');
 };
 let resolutionHash;
 if(env.FARM_BUILD_PROFILE==='ci') {
  build({FARM_NATIVE_PREPARE_ONLY:'true'});
  const output=`.formal-native/${abi}/resolution.json`;
  const result=run(env.FARM_RELEASE_PYTHON??'python',['-B','scripts/formal-native-products.py','resolve','--abi',abi,'--output',output],
   {cwd:root,env,stdio:'inherit',timeout:20*60*1000});
  assert.ok(!result.error&&result.status===0,'Formal resolver failed to produce a build/miss decision');
  resolutionHash=crypto.createHash('sha256').update(fs.readFileSync(path.join(root,output))).digest('hex');
 }
 build(resolutionHash?{FARM_FORMAL_RESOLUTION_HASH:resolutionHash}:{});
 if(resolutionHash&&env.FARM_BUILD_IMAGE) {
  const resolution=JSON.parse(fs.readFileSync(path.join(root,`.formal-native/${abi}/resolution.json`)));
  if(Object.keys(resolution.reused).length) {
   const actual=JSON.parse(fs.readFileSync(path.join(root,`.ci-native-cache/${abi}.json`)));
   assert.equal(actual.containerImageId,resolution.context.platform.image,'Container image moved between resolution and build');
  }
 }
 return {resolutionHash};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url))runFormalNativeBuild();
