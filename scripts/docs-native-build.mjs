// Reuse decisions are bound to the authenticated resolver output. Every miss
// uses the shared formal recipe, scoped to the required Docs owners only.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {normalFile} from './formal-native-scope.mjs';
const hash = value => crypto.createHash('sha256').update(value).digest('hex');
export const plugins = 'replace-dirname react mdx ssr-css syntax-highlight sass tailwindcss docs docs-snapshot docs-seo docs-snippets'.split(' ');
const owners = ['@farmfe/core', ...plugins.map(name => `@farmfe/plugin-${name}`)];
const targets = {'linux-x64-gnu':'x86_64-unknown-linux-gnu', 'win32-x64-msvc':'x86_64-pc-windows-msvc', 'darwin-arm64':'aarch64-apple-darwin'};
export function buildDocsNative(file, {root=process.cwd(), env=process.env, platform=process.platform, arch=process.arch, run=spawnSync,
  git=(...args)=>execFileSync('git',args,{cwd:root,encoding:'utf8'}).trim()}={}) {
  const bytes = fs.readFileSync(normalFile(root,file));
  assert.equal(hash(bytes),env.FARM_DOCS_PLAN_HASH,'Docs resolution changed');
  const plan=JSON.parse(bytes);
  assert.equal(plan.schemaVersion,1); assert.ok(targets[plan.abi]);
  assert.ok(plan.abi.startsWith(`${platform}-${arch}-`) || plan.abi===`${platform}-${arch}`,'Docs consumer platform differs');
  assert.equal(plan.sourceSHA,git('rev-parse','HEAD'));assert.equal(plan.sourceTree,git('rev-parse','HEAD^{tree}'));
  assert.ok(Array.isArray(plan.missing));
  assert.deepEqual([...plan.missing,...Object.keys(plan.reused)].sort(),[...owners].sort(),'Docs plan must cover exactly the required owners');
  function verify() {
    for(const [owner,row] of Object.entries(plan.reused)) {
      const expected=owner==='@farmfe/core'?`packages/core/binding/farm.${plan.abi}.node`:`rust-plugins/${owner.slice('@farmfe/plugin-'.length)}/npm/${plan.abi}/index.farm`;
      assert.equal(row.file,expected);assert.equal(hash(fs.readFileSync(normalFile(root,row.file))),row.sha256,'Docs reused bytes changed');
    }
  }
  verify();
  if(plan.missing.length) {
    const docker=plan.abi==='linux-x64-gnu';
    const home=env.FARM_CARGO_HOME??env.CARGO_HOME??path.join(os.homedir(),'.cargo');
    if(docker)for(const member of ['git','registry'])fs.mkdirSync(path.join(home,member),{recursive:true});
    const result=run(docker?process.execPath:'bash',docker?['scripts/v2-docker-cargo-cache.mjs']:['scripts/ci-build-rust-artifacts.sh'],{cwd:root,env:{...env,FARM_BUILD_PROFILE:'release',FARM_BUILD_ABI:plan.abi,
      FARM_BUILD_TARGET:targets[plan.abi],FARM_CARGO_HOME:home,FARM_BUILD_IMAGE:docker?'ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-debian':'',FARM_BUILD_ZIG:'false',FARM_RELEASE_NATIVE_OWNERS:JSON.stringify(plan.missing),
      FARM_FORMAL_RESOLUTION_HASH:'',FARM_NATIVE_PROOF_MODE:'off',FARM_NATIVE_PREPARE_ONLY:'false'},stdio:'inherit',timeout:90*60*1000});
    assert.ok(!result.error&&result.status===0,'Docs missing native products failed to build');
  }
  verify(); return {built:plan.missing,reused:Object.keys(plan.reused)};
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) console.log(JSON.stringify(buildDocsNative(process.argv[2])));
