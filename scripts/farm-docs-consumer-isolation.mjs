import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';

function inside(parent,file) {
  const relative=path.relative(parent,file);
  return relative!==''&&relative!=='..'&&!relative.startsWith('..'+path.sep)&&!path.isAbsolute(relative);
}
const sha256=bytes=>createHash('sha256').update(bytes).digest('hex');

// Resolution only. Does not import packages, install anything or inspect ABI.
export async function verifyInstalledDocsPackages(directory,{repository,packages,env=process.env}={}) {
  assert(!process.env.NODE_PATH&&!env.NODE_PATH,'NODE_PATH must be empty for an isolated consumer');
  assert(typeof repository==='string','Repository path is required');
  assert(Array.isArray(packages)&&packages.length>0,'Explicit expected package list is required');
  const consumer=await fs.realpath(directory),source=await fs.realpath(repository);
  assert(consumer!==source&&!inside(source,consumer)&&!inside(consumer,source),'Consumer and repository directories must be separate');
  const modules=await fs.realpath(path.join(consumer,'node_modules'));
  assert(inside(consumer,modules),'node_modules resolves outside consumer');
  const require=createRequire(path.join(consumer,'package.json')),rows=[],verified=new Map();
  for(const expected of packages) {
    assert(typeof expected.name==='string'&&/^(?:@[a-z0-9._-]+\/)?[a-z0-9._-]+$/i.test(expected.name),'Invalid expected package name');
    assert(typeof expected.version==='string'&&expected.version.length>0,'Expected package version is required');
    const specifier=expected.specifier??expected.name;
    assert(specifier===expected.name||specifier.startsWith(expected.name+'/'),'Public specifier must belong to expected package');
    let resolver=require;
    if(expected.from!==undefined) {
      const parent=verified.get(expected.from);
      assert(parent,'Resolution requires a previously verified parent: '+expected.from);
      assert(['dependencies','optionalDependencies','peerDependencies'].some(field=>Object.hasOwn(parent.manifest[field]??{},expected.name)),
        'Resolution parent must declare the dependency: '+expected.name);
      resolver=createRequire(parent.file);
    }
    const entry=await fs.realpath(resolver.resolve(specifier));
    assert(inside(modules,entry),'Resolved entry is outside consumer node_modules: '+specifier);
    let current=path.dirname(entry),manifest,manifestBytes,manifestFile;
    while(inside(modules,current)) {
      const file=path.join(current,'package.json');
      try {
        const bytes=await fs.readFile(file),candidate=JSON.parse(bytes);
        if(candidate.name===expected.name){manifest=candidate;manifestBytes=bytes;manifestFile=await fs.realpath(file);break;}
      }catch(error){if(error.code!=='ENOENT')throw error;}
      current=path.dirname(current);
    }
    assert(manifest&&inside(modules,manifestFile),'Expected package manifest is missing or escapes consumer');
    assert.equal(manifest.version,expected.version,'Installed package version differs: '+expected.name);
    const bytes=await fs.readFile(entry),entrySha256=sha256(bytes);
    if(expected.nativeSha256!==undefined) {
      assert(typeof expected.nativeSha256==='string'&&/^[a-f0-9]{64}$/i.test(expected.nativeSha256),'Expected native digest must be SHA256');
      assert.equal(entrySha256,expected.nativeSha256.toLowerCase(),'Native payload digest differs: '+expected.name);
    }
    verified.set(manifest.name,{manifest,file:manifestFile});
    rows.push({name:manifest.name,version:manifest.version,specifier,from:expected.from??null,entry,entrySha256,manifest:manifestFile,manifestSha256:sha256(manifestBytes)});
  }
  return {consumer,nodeModules:modules,resolutionConditions:'Node createRequire.resolve (require/default); no module execution',packages:rows};
}
