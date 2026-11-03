import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {nativePlatformMatches} from './release-archive.mjs';

const digest = body => crypto.createHash('sha256').update(body).digest('hex');
export function planDigest(value) {
  const sort = value => Array.isArray(value) ? value.map(sort) : value && typeof value === 'object' ? Object.fromEntries(Object.keys(value).sort().map(key => [key, sort(value[key])])) : value;
  return digest(JSON.stringify(sort(value)).replace(/[\u007f-\uffff]/g, character => '\\u' + character.charCodeAt(0).toString(16).padStart(4, '0')));
}
export function normalFile(root, relative) {
  assert.ok(typeof relative === 'string' && relative && !relative.includes('\\') && !relative.includes(':'));
  assert.equal(path.posix.normalize(relative), relative); assert.ok(!path.isAbsolute(relative));
  let file = fs.realpathSync(root);
  for (const part of relative.split('/')) {
    assert.ok(part && part !== '.' && part !== '..'); file = path.join(file, part);
    assert.ok(!fs.lstatSync(file).isSymbolicLink(), 'Native path redirects');
    assert.equal(fs.realpathSync(file), file, 'Native junction redirects');
  }
  assert.ok(fs.statSync(file).isFile()); return file;
}

export function shouldBuild(owner, {root=process.cwd(),env=process.env,git=(...args)=>execFileSync('git',args,{cwd:root,encoding:'utf8'}).trim()}={}) {
  if(env.FARM_BUILD_PROFILE!=='ci'||!env.FARM_FORMAL_RESOLUTION_HASH)return true;
  const abi=env.FARM_BUILD_ABI;
  assert.match(abi,/^(?:linux|darwin|win32)-[a-z0-9-]+$/);
  const bytes=fs.readFileSync(normalFile(root,`.formal-native/${abi}/resolution.json`));
  assert.equal(digest(bytes),env.FARM_FORMAL_RESOLUTION_HASH,'Resolver output changed');
  const resolution=JSON.parse(bytes);
  assert.equal(resolution.schemaVersion,2);assert.equal(resolution.acceptanceInherited,false);
  // Missing evidence is a build miss. A purported accepted product is held to
  // the full identity/path/environment contract before any build is omitted.
  const reused=resolution.reused?.[owner]; if(!reused)return true;
  const sourceSHA=git('rev-parse','HEAD'),sourceTree=git('rev-parse','HEAD^{tree}');
  assert.equal(resolution.sourceSHA,sourceSHA);assert.equal(resolution.sourceTree,sourceTree);assert.equal(resolution.abi,abi);
  for(const key of ['GITHUB_REPOSITORY','GITHUB_REPOSITORY_ID','GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT','GITHUB_JOB','GITHUB_WORKFLOW_REF']) {
    assert.ok(env[key]);assert.equal(resolution.consumer[key],env[key],'Consumer identity/attempt differs');
  }
  assert.equal(resolution.consumer.sourceSHA,sourceSHA);assert.equal(resolution.consumer.sourceTree,sourceTree);
  assert.match(resolution.graphHash,/^[a-f0-9]{64}$/);assert.match(resolution.planHash,/^[a-f0-9]{64}$/);
  assert.equal(resolution.planHash,planDigest({consumer:resolution.consumer,graphHash:resolution.graphHash,products:resolution.products}),'Materialization plan binding differs');
  const expected=resolution.products[owner];assert.ok(expected);assert.equal(expected.owner,owner);
  for(const [key,value] of Object.entries(expected))assert.deepEqual(reused.record[key],value,'Resolved product differs from bound plan');
  const runtime=JSON.parse(fs.readFileSync(normalFile(root,`.ci-native-proof/${abi}/runtime.json`)));
  assert.deepEqual(runtime.errors,[]);assert.deepEqual(runtime.unboundBuildVariables,[]);
  for(const key of ['rustc','cargo','linker','formalFlagsHash'])assert.deepEqual(runtime[key],resolution.context[key],'Actual build environment changed after resolution');
  assert.equal(runtime.abi,abi);assert.equal(runtime.target,expected.recipe.target);
  if(resolution.context.platform.sdk) {
    assert.equal(runtime.platformSdk,resolution.context.platform.sdk);
    assert.deepEqual(runtime.runtime,resolution.context.platform.host);
  }
  const product=fs.readFileSync(normalFile(root,expected.file));
  assert.equal(product.length,reused.record.bytes);assert.equal(digest(product),reused.record.sha256);
  assert.ok(nativePlatformMatches(product,abi),'Native payload architecture differs');
  return false;
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const owner=process.argv[2];assert.ok(owner);
  if(owner==='--plugins') {
    const selected=(process.argv[3]??'').split(' ').filter(Boolean).filter(directory=>{
      assert.match(directory,/^[a-z0-9-]+$/);
      return shouldBuild(JSON.parse(fs.readFileSync(`rust-plugins/${directory}/package.json`)).name);
    }); console.log(selected.join(' '));
  } else console.log(String(shouldBuild(owner)));
}
