import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { ignoreReleaseBuild, deploymentHandoff } from './farm-docs-deploy-order.mjs';
const source = 'a'.repeat(40);
const env = { VERCEL_ENV: 'production', VERCEL_GIT_COMMIT_REF: 'main', VERCEL_GIT_COMMIT_SHA: source };
const pkg = { name: '@farmfe/docs', version: '0.0.2' };
const releasePins = new Map([[pkg.name, pkg.version]]);
const plan = { selected: new Map([[pkg.name, pkg]]), manifests: new Map([[pkg.name, pkg]]) };
const meta = { ...pkg, dist: { integrity: 'sha512-' + Buffer.alloc(64).toString('base64'), tarball: 'https://registry.npmjs.org/docs.tgz' } };
const missing = async () => { throw new Error('[RELEASE_NOT_PUBLISHED] @farmfe/docs@0.0.2: HTTP 404'); };
test('only exact missing main production pins cancel the Git build', async () => {
  assert.equal(await ignoreReleaseBuild({releasePins,env, plan, getMetadata: missing}), true);
  assert.equal(await ignoreReleaseBuild({releasePins,env, plan, getMetadata: async () => meta}), false);
  for (const patch of [{VERCEL_ENV:'preview'},{VERCEL_ENV:'development'},{VERCEL_GIT_COMMIT_REF:'feature'},{VERCEL_GIT_COMMIT_SHA:''}]) {
    let called = false;
    assert.equal(await ignoreReleaseBuild({releasePins,env:{...env,...patch},plan,getMetadata:async()=>{called=true;return meta;}}),false);
    assert.equal(called,false);
  }
});
test('authentication, network and invalid registry metadata are not hidden as waiting',async()=>{
  for(const message of ['[REGISTRY_UNAVAILABLE] HTTP 401','[REGISTRY_UNAVAILABLE] HTTP 403','[REGISTRY_UNAVAILABLE] HTTP 500','timeout'])
    assert.equal(await ignoreReleaseBuild({releasePins,env,plan,getMetadata:async()=>{throw new Error(message);}}),false);
  await assert.rejects(ignoreReleaseBuild({releasePins,env,plan,getMetadata:async()=>({...meta,version:'0.0.1'})}));
  const two={...plan,selected:new Map([...plan.selected,['@farmfe/core',{name:'@farmfe/core',version:'2.0.1'}]])};
  assert.equal(await ignoreReleaseBuild({releasePins,env,plan:two,getMetadata:async name=>{if(name===pkg.name)return missing();throw new Error('[REGISTRY_UNAVAILABLE] HTTP 403');}}),false);
});
const release={GITHUB_REPOSITORY:'farm-fe/farm',GITHUB_EVENT_NAME:'push',GITHUB_REF:'refs/heads/main',GITHUB_SHA:source,GITHUB_WORKFLOW_SHA:source,FARM_RELEASE_FINALIZED:'success'};
test('handoff needs exact trusted finalized source and declines stale releases',()=>{
  const value=deploymentHandoff(release,source);
  assert.equal(value.status,'manual-exact-source-redeploy');assert.equal(value.source,source);
  assert.equal(deploymentHandoff(release,'b'.repeat(40)).status,'superseded');
  for(const patch of [{GITHUB_REPOSITORY:'fork/farm'},{GITHUB_EVENT_NAME:'pull_request'},{GITHUB_REF:'refs/heads/other'},{GITHUB_WORKFLOW_SHA:'b'.repeat(40)},{FARM_RELEASE_FINALIZED:'failure'},{FARM_RELEASE_FINALIZED:'skipped'}])assert.throws(()=>deploymentHandoff({...release,...patch},source));
});
test('no optional credentials means a safe manual path, never a deployment request',()=>{
  const script=fs.readFileSync(new URL('./farm-docs-deploy-order.mjs',import.meta.url),'utf8');
  assert.doesNotMatch(script,/VERCEL_TOKEN|deployHook|method:\s*['"]POST|api\.vercel\.com/);
  const workflow=fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml',import.meta.url),'utf8');
  const job=workflow.split('  docs-deployment-handoff:')[1].split('\n# Publication never')[0];
  assert.match(job,/needs: \[finalize\]/);assert.match(job,/needs.finalize.result == 'success'/);
  assert.match(job,/continue-on-error: true/);assert.match(job,/ref: \$\{\{ github.sha \}\}/);
  assert.doesNotMatch(job,/secrets\.|contents: write|id-token: write|workflow_dispatch/);
});

test('missing versions outside the committed candidate never suppress a build',async()=>{
  assert.equal(await ignoreReleaseBuild({env,plan,releasePins:new Map(),getMetadata:missing}),false);
});
test('a fast 404 cannot hide corrupt metadata on another closure member',async()=>{
  const two={...plan,selected:new Map([...plan.selected,['@farmfe/core',{name:'@farmfe/core',version:'2.0.1'}]])};
  await assert.rejects(ignoreReleaseBuild({env,plan:two,releasePins,getMetadata:async name=>name===pkg.name?missing():({name:'@farmfe/core',version:'2.0.1',dist:{}})}));
});
