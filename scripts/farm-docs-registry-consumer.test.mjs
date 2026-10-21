import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { test } from 'node:test';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { validateReleaseSet, runConsumer } from './farm-docs-registry-consumer.mjs';
const plan = JSON.parse(fs.readFileSync(new URL('../docs/farm-docs/registry-release-plan.draft.json', import.meta.url)));
const sha = data => crypto.createHash('sha256').update(data).digest('hex');
const integrity = 'sha512-' + Buffer.alloc(64, 7).toString('base64');
function fixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-registry-entry-'));
  t.after(() => { assert.ok(path.resolve(root).startsWith(path.resolve(os.tmpdir()) + path.sep)); fs.rmSync(root, {recursive:true}); });
  const release = {schemaVersion:1,status:'approved',releaseId:'SYNTHETIC_UNIT_FIXTURE_NOT_PUBLISHED',farmMajor:2,baselineCommit:'a'.repeat(40),nativeAbi:'unit-fixture-abi',registry:'https://registry.npmjs.org',packages:[]};
  const rootPackage = {private:true,dependencies:{}};
  const metadata = new Map();
  for (const p of plan.packages) {
    const dir = path.join(root,'node_modules',p.name); fs.mkdirSync(dir,{recursive:true});
    const pkg = {name:p.name,version:p.candidateVersion,main:'index.js'};
    if (p.name==='@farmfe/cli') pkg.bin={farm:'bin/farm.mjs'};
    const files = {'package.json':JSON.stringify(pkg),'index.js':'// synthetic unit fixture\n'};
    if (pkg.bin) files['bin/farm.mjs']='// synthetic CLI: never executed in tests\n';
    if (p.role==='native-platform') files[p.artifactPath]=Buffer.from(p.requiredOn.startsWith('win32')?'MZsynthetic-unit-fixture':'\x7fELFsynthetic-unit-fixture');
    const hashes={}; for(const [name,data] of Object.entries(files)){fs.mkdirSync(path.dirname(path.join(dir,name)),{recursive:true});fs.writeFileSync(path.join(dir,name),data);hashes[name]=sha(data);}
    release.packages.push({name:p.name,version:p.candidateVersion,integrity,files:hashes,nativeAbi:p.role==='native-platform'?release.nativeAbi:undefined,installScripts:{}});
    metadata.set(p.name,{name:p.name,version:p.candidateVersion,dist:{integrity},dependencies:{},optionalDependencies:{},peerDependencies:{}});
    if(plan.directFarmPackages.includes(p.name)) rootPackage.dependencies[p.name]=p.candidateVersion;
  }
  fs.writeFileSync(path.join(root,'package.json'),JSON.stringify(rootPackage));
  fs.writeFileSync(path.join(root,'pnpm-lock.yaml'),"lockfileVersion: '9.0'\n");
  const calls=[];
  const options={root,releaseSet:release,plan,target:'win32-x64-msvc',phase:'build'};
  const hooks={getMetadata:async name=>{const p=metadata.get(name);if(!p)throw new Error('[FORMAL_PACKAGE_MISSING] '+name);return p;},execute:async command=>{calls.push(command);return 0;}};
  return {root,release,metadata,calls,options,hooks,rootPackage};
}
test('missing or draft release-set fails before commands',async t=>{
  assert.throws(()=>validateReleaseSet(null,plan),/RELEASE_SET_REQUIRED/);
  const f=fixture(t);f.release.status='draft';await assert.rejects(runConsumer(f.options,f.hooks),/RELEASE_SET_NOT_APPROVED/);assert.equal(f.calls.length,0);
});
test('release-set must cover all 40 names and formal exact versions',t=>{
  const f=fixture(t);f.release.packages.pop();assert.throws(()=>validateReleaseSet(f.release,plan),/RELEASE_SET_INCOMPLETE/);
  f.release.packages=fixture(t).release.packages;f.release.packages[0].version='2.0.0-beta.1';assert.throws(()=>validateReleaseSet(f.release,plan),/PRERELEASE|EXACT_VERSION/);
});
test('workspace/file/link/git/URL/prerelease selectors fail closed',async t=>{
  const f=fixture(t);
  for(const value of ['workspace:*','file:../core','link:../core','git+https://example.org/core.git','github:org/core','org/core#main','git@github.com:org/core.git','https://example.org/core.tgz','2.0.0-beta.1']){
    f.rootPackage.dependencies['@farmfe/core']=value;fs.writeFileSync(path.join(f.root,'package.json'),JSON.stringify(f.rootPackage));
    await assert.rejects(runConsumer(f.options,f.hooks),/FORBIDDEN_SOURCE|PRERELEASE/);
  }assert.equal(f.calls.length,0);
});
test('missing formal registry metadata and integrity mismatch never run CLI',async t=>{
  const f=fixture(t);f.metadata.delete('@farmfe/core');await assert.rejects(runConsumer(f.options,f.hooks),/FORMAL_PACKAGE_MISSING/);assert.equal(f.calls.length,0);
  const g=fixture(t);g.metadata.get('@farmfe/core').dist.integrity='sha512-wrong';await assert.rejects(runConsumer(g.options,g.hooks),/REGISTRY_INTEGRITY/);assert.equal(g.calls.length,0);
});
test('old HMR 2.0.0 peer runtime 0.2.0 cannot masquerade as v2 cohort',async t=>{
  const f=fixture(t);f.metadata.get('@farmfe/runtime-plugin-hmr').peerDependencies={'@farmfe/runtime':'0.2.0'};
  await assert.rejects(runConsumer(f.options,f.hooks),/COHORT_MISMATCH/);assert.equal(f.calls.length,0);
});
test('registry install source-build scripts are rejected',async t=>{
  const f=fixture(t);f.metadata.get('@farmfe/core').scripts={postinstall:'cargo build'};
  await assert.rejects(runConsumer(f.options,f.hooks),/SOURCE_BUILD_FORBIDDEN/);assert.equal(f.calls.length,0);
});
test('new Farm dependency outside frozen plan requires re-audit',async t=>{
  const f=fixture(t);f.metadata.get('@farmfe/docs').dependencies={'@farmfe/plugin-mdx':'2.0.0'};
  await assert.rejects(runConsumer(f.options,f.hooks),/CLOSURE_DRIFT/);assert.equal(f.calls.length,0);
});
test('missing native and changed bytes fail without compilation fallback',async t=>{
  const f=fixture(t);const item=plan.packages.find(p=>p.name==='@farmfe/core-win32-x64-msvc');fs.unlinkSync(path.join(f.root,'node_modules',item.name,item.artifactPath));
  await assert.rejects(runConsumer(f.options,f.hooks),/MISSING_NATIVE/);assert.equal(f.calls.length,0);
  const g=fixture(t);fs.appendFileSync(path.join(g.root,'node_modules',item.name,item.artifactPath),'changed');
  await assert.rejects(runConsumer(g.options,g.hooks),/PAYLOAD_INTEGRITY/);assert.equal(g.calls.length,0);
});
test('preflight never installs or runs commands; valid build only calls local Node CLI',async t=>{
  const f=fixture(t);await runConsumer({...f.options,phase:'preflight'},f.hooks);assert.equal(f.calls.length,0);
  await runConsumer(f.options,f.hooks);assert.equal(f.calls.length,2);assert.deepEqual(f.calls.map(c=>c.args.slice(1)),[['--help'],['build','--config','farm.config.ts']]);
  assert.ok(f.calls.every(c=>c.executable===process.execPath&&c.args[0].startsWith(f.root)&&c.cwd===f.root));
});
test('CLI failure is propagated, never retried with source tools',async t=>{
  const f=fixture(t);f.hooks.execute=async command=>{f.calls.push(command);return 19;};
  await assert.rejects(runConsumer(f.options,f.hooks),/CLI_FAILED/);assert.equal(f.calls.length,1);
});


test('Linux GNU ELF payloads pass, unsupported target fails',async t=>{
  const f=fixture(t);await runConsumer({...f.options,target:'linux-x64-gnu'},f.hooks);assert.equal(f.calls.length,2);
  const g=fixture(t);await assert.rejects(runConsumer({...g.options,target:'linux-x64-musl'},g.hooks),/PLATFORM_NOT_APPROVED/);assert.equal(g.calls.length,0);
});
test('pnpm-style transitive dependency stays inside consumer',async t=>{
  const f=fixture(t);const original=path.join(f.root,'node_modules/@farmfe/runtime'),stored=path.join(f.root,'node_modules/.isolated-store/runtime');
  assert.ok(path.resolve(original).startsWith(f.root+path.sep));assert.ok(path.resolve(stored).startsWith(f.root+path.sep));fs.mkdirSync(path.dirname(stored),{recursive:true});fs.renameSync(original,stored);
  const link=path.join(f.root,'node_modules/@farmfe/core/node_modules/@farmfe/runtime');fs.mkdirSync(path.dirname(link),{recursive:true});fs.symlinkSync(stored,link,'junction');
  await runConsumer(f.options,f.hooks);assert.equal(f.calls.length,2);
});
test('a local source package outside consumer cannot satisfy resolution',async t=>{
  const f=fixture(t),g=fixture(t);const original=path.join(f.root,'node_modules/@farmfe/runtime');
  assert.ok(path.resolve(original).startsWith(f.root+path.sep));fs.rmSync(original,{recursive:true});fs.symlinkSync(path.join(g.root,'node_modules/@farmfe/runtime'),original,'junction');
  await assert.rejects(runConsumer(f.options,f.hooks),/WORKSPACE_ESCAPE/);assert.equal(f.calls.length,0);
});
test('root workspace marker and install overrides are rejected',async t=>{
  const f=fixture(t);fs.writeFileSync(path.join(f.root,'pnpm-workspace.yaml'),'packages: []');await assert.rejects(runConsumer(f.options,f.hooks),/WORKSPACE_FORBIDDEN/);assert.equal(f.calls.length,0);
  const g=fixture(t);g.rootPackage.pnpm={overrides:{'@farmfe/core':'file:../core'}};fs.writeFileSync(path.join(g.root,'package.json'),JSON.stringify(g.rootPackage));await assert.rejects(runConsumer(g.options,g.hooks),/CONSUMER_OVERRIDES_FORBIDDEN/);assert.equal(g.calls.length,0);
});
test('native overrides and injected loaders cannot replace precompiled inputs',async t=>{
  const f=fixture(t);
  for(const env of [{NAPI_RS_NATIVE_LIBRARY_PATH:'C:/local/farm.node'},{NAPI_RS_FORCE_WASI:'1'},{NODE_PATH:'/outside'},{NODE_OPTIONS:'--import ./hook.mjs'},{NODE_OPTIONS:'-r./hook.cjs'},{NODE_OPTIONS:'"--require" ./hook.cjs'}]){
    await assert.rejects(runConsumer({...f.options,environment:env},f.hooks),/BINDING_OVERRIDE_FORBIDDEN/);
  }assert.equal(f.calls.length,0);
});


test('a numerically older package cannot be substituted as the new v2 ecosystem',t=>{
  const f=fixture(t);f.release.packages.find(p=>p.name==='@farmfe/runtime-plugin-hmr').version='2.0.0';
  assert.throws(()=>validateReleaseSet(f.release,plan),/VERSION_DOWNGRADE/);assert.equal(f.calls.length,0);
});


test('actual entry without release-set exits clearly before any build',()=>{
  const result=spawnSync(process.execPath,[fileURLToPath(new URL('./farm-docs-registry-consumer.mjs',import.meta.url)),'build'],{encoding:'utf8',windowsHide:true});
  assert.equal(result.status,1);assert.match(result.stderr,/RELEASE_SET_REQUIRED/);assert.equal(result.stdout,'');
});


test('an unsealed local binding cannot shadow the published native package',async t=>{
  const f=fixture(t),dir=path.join(f.root,'node_modules/@farmfe/core/binding');fs.mkdirSync(dir,{recursive:true});fs.writeFileSync(path.join(dir,'farm.win32-x64-msvc.node'),'MZold-local-binary');
  await assert.rejects(runConsumer(f.options,f.hooks),/UNSEALED_PAYLOAD/);assert.equal(f.calls.length,0);
});


test('forbidden source errors do not echo credentials from rejected URLs',async t=>{
  const f=fixture(t);f.rootPackage.dependencies['@farmfe/core']='git+https://user:unit-secret@invalid.example/core.git';fs.writeFileSync(path.join(f.root,'package.json'),JSON.stringify(f.rootPackage));
  await assert.rejects(runConsumer(f.options,f.hooks),error=>error.message.includes('FORBIDDEN_SOURCE')&&!error.message.includes('unit-secret'));assert.equal(f.calls.length,0);
});
