import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createDeploymentLock } from './farm-docs-refresh-deployment-lock.mjs';
import test from 'node:test';
import { lockedDeploymentPlan, preflight } from './farm-docs-vercel-build.mjs';
const read = file => JSON.parse(fs.readFileSync(new URL('../'+file, import.meta.url)));
const website = read('website/package.json');
const lock = read('website/farm-release-lock.json');
const clone = value => JSON.parse(JSON.stringify(value));
const metadata = new Map(lock.packages.map(pkg=>[pkg.name,pkg]));
test('deployment pins remain independent of newer workspace versions on all verified targets', async()=>{
  for (const target of ['linux-x64-gnu','darwin-arm64','win32-x64-msvc']) {
    const plan = lockedDeploymentPlan(website,lock,target);
    assert.equal(plan.selected.get('@farmfe/docs').version,'0.0.1');
    assert.equal(plan.selected.get('@farmfe/core').version,'2.0.0');
    assert.equal(plan.selected.size,29);
    for(const pkg of plan.selected.values()) if(pkg.os) assert.ok(pkg.name.endsWith('-'+target));
    const verified = await preflight(plan, async name=>clone(metadata.get(name)));
    assert.equal(verified.length,29);
    assert.doesNotMatch(JSON.stringify(plan.manifest),/workspace:|latest|overrides|patches/);
  }
});
test('changed website dependency intent requires a separately reviewed lock refresh',()=>{
  for(const change of [w=>w.dependencies.react='19.0.0',w=>w.dependencies['@farmfe/docs']='0.0.2',w=>delete w.devDependencies['@farmfe/plugin-sass'],w=>w.dependencies['@farmfe/new']='workspace:*']) {
    const next=clone(website);change(next);
    assert.throws(()=>lockedDeploymentPlan(next,lock,'linux-x64-gnu'),/dependencies changed/);
  }
  const pinned=clone(website);pinned.dependencies['@farmfe/docs']='0.0.2';
  const stale=clone(lock);stale.websiteDependencies.dependencies['@farmfe/docs']='0.0.2';
  assert.throws(()=>lockedDeploymentPlan(pinned,stale,'linux-x64-gnu'),/selector disagrees/);
  const next=clone(website);next.scripts.build='unrelated local script';
  assert.doesNotThrow(()=>lockedDeploymentPlan(next,lock,'linux-x64-gnu'));
});
test('missing native, duplicate lock entries and unverified targets fail closed',()=>{
  assert.throws(()=>lockedDeploymentPlan(website,lock,'linux-arm64-musl'),/Target not verified/);
  const duplicate=clone(lock);duplicate.packages.push(duplicate.packages[0]);
  assert.throws(()=>lockedDeploymentPlan(website,duplicate,'linux-x64-gnu'),/Duplicate/);
  const missing=clone(lock);missing.packages=missing.packages.filter(p=>p.name!=='@farmfe/core-linux-x64-gnu');
  assert.throws(()=>lockedDeploymentPlan(website,missing,'linux-x64-gnu'),/UNKNOWN_FARM_DEPENDENCY/);
});
test('registry mutation, native metadata, install scripts and cohort changes stay forbidden',async()=>{
  const plan=lockedDeploymentPlan(website,lock,'linux-x64-gnu');
  for(const mutate of [p=>p.dist.integrity='sha512-'+Buffer.alloc(64).toString('base64'),p=>p.dist.tarball='https://elsewhere.invalid/x.tgz',p=>p.scripts={install:'false'},p=>p.dependencies['@farmfe/core']='2.0.1',p=>p.version='0.0.2']) {
    await assert.rejects(preflight(plan,async name=>{const p=clone(metadata.get(name));if(name==='@farmfe/docs')mutate(p);return p;}));
  }
  await assert.rejects(preflight(plan,async name=>{const p=clone(metadata.get(name));if(name==='@farmfe/core-linux-x64-gnu')p.cpu=['arm64'];return p;}),/platform metadata/);
  for(const status of [404,401,403,500]) await assert.rejects(preflight(plan,async()=>{throw Error('HTTP '+status);}),new RegExp('HTTP '+status));
});
test('no deployment controller or publication workflow changes remain',()=>{
  const config=read('website/vercel.json');
  assert.deepEqual(config.git,{deploymentEnabled:{main:true}});
  assert.equal(config.ignoreCommand,undefined);
  const workflow=fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml',import.meta.url),'utf8');
  assert.doesNotMatch(workflow,/docs-deployment-handoff|VERCEL_TOKEN|docs-production/);
  assert.equal(website.dependencies['@farmfe/docs'],'workspace:*');
});


test('standalone refresh is deterministic and rejects absent or changed release members',async()=>{
  const next=await createDeploymentLock(website,metadata,lock.sourceSHA,async name=>clone(metadata.get(name)));
  assert.deepEqual(next,lock);
  await assert.rejects(createDeploymentLock(website,metadata,lock.sourceSHA,async()=>{throw Error('HTTP 404');}),/404/);
  await assert.rejects(createDeploymentLock(website,metadata,lock.sourceSHA,async name=>{const pkg=clone(metadata.get(name));if(name==='@farmfe/docs')pkg.dependencies['@farmfe/core']='2.0.1';return pkg;}));
});
