import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {verifyPublicEntries,verifyWorkspaceConversion} from './v2-normal-pack-contract.mjs';

function member(name) {
  assert(typeof name==='string'&&name.length>0&&!path.posix.isAbsolute(name)&&!path.win32.isAbsolute(name)&&!name.includes('\\')&&!name.split('/').some(part=>['','.','..'].includes(part)),`Invalid package member: ${name}`);
  return name;
}

// The caller must prove this directory was extracted from the recorded tarball.
// No package code, lifecycle, build or native module is executed here.
export async function verifyDocsPackageDirectory(directory,{original,versions,requiredFiles=[],platform=false,native,forbiddenRoots=[fileURLToPath(new URL('../',import.meta.url))]}={}) {
  assert(original&&versions instanceof Map,'Original manifest and current closure versions are required');
  assert(Array.isArray(forbiddenRoots)&&forbiddenRoots.length>0,'At least one repository root is required');
  const roots=forbiddenRoots.map(root=>{
    assert(typeof root==='string'&&(path.posix.isAbsolute(root)||path.win32.isAbsolute(root)),'Repository roots must be absolute');
    const normalized=root.replace(/[\\/]+/g,'/').replace(/\/$/,'');
    assert(normalized.length>3,'Repository root must not be a filesystem root');
    const windows=/^[a-z]:/i.test(normalized)||/^[\\/]{2}[^\\/]/.test(root);
    return {value:windows?normalized.toLowerCase():normalized,windows};
  });
  assert(!(await fs.lstat(directory)).isSymbolicLink(),'Package root symlink/junction is forbidden');
  const root=await fs.realpath(directory),files=[];
  async function walk(current,prefix='') {
    for(const entry of (await fs.readdir(current,{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))) {
      const name=member(prefix+entry.name),file=path.join(current,entry.name),stat=await fs.lstat(file);
      assert(!stat.isSymbolicLink(),`Package symlink/junction is forbidden: ${name}`);
      assert(!name.split('/').some(part=>['node_modules','target','.git','evidence'].includes(part)),`Private/forbidden package member: ${name}`);
      const resolved=await fs.realpath(file),relative=path.relative(root,resolved);
      assert(relative&&relative!=='..'&&!relative.startsWith('..'+path.sep)&&!path.isAbsolute(relative),`Package member escapes root: ${name}`);
      if(stat.isDirectory())await walk(file,name+'/');
      else {
        assert(stat.isFile(),`Unsupported package member: ${name}`);
        const bytes=await fs.readFile(file);
        // Inspect UTF-8 text without importing package code. Native/binary
        // payloads retain their separate formal-digest contract.
        if(!bytes.includes(0)) {
          let text;
          try { text=new TextDecoder('utf-8',{fatal:true}).decode(bytes); } catch {}
          if(text!==undefined) {
            const normalized=text.replace(/[\\/]+/g,'/');
            for(const root of roots)assert(!(root.windows?normalized.toLowerCase():normalized).includes(root.value),`Repository path leaked into package text: ${name}`);
          }
        }
        files.push({path:name,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')});
      }
    }
  }
  await walk(root);
  const names=new Map(files.map(file=>[file.path,file]));
  function requireFile(name) {assert(names.has(member(name)),`Missing package resource: ${name}`);}
  requireFile('package.json');
  const manifest=JSON.parse(await fs.readFile(path.join(root,'package.json'),'utf8'));
  assert.equal(manifest.name,original.name,'Packed package name differs');
  assert.equal(manifest.version,original.version,'Packed package version differs');
  assert(manifest.license,'Missing package license declaration');
  assert(files.some(file=>/^LICENSE(?:\.[^/]+)?$/i.test(file.path)&&file.bytes>0),'Missing package LICENSE text');
  if(!platform)assert(files.some(file=>/^README(?:\.[^/]+)?$/i.test(file.path)&&file.bytes>0),'Missing package README');
  for(const field of ['dependencies','devDependencies','optionalDependencies','peerDependencies']) {
    assert.deepEqual(Object.keys(manifest[field]??{}).sort(),Object.keys(original[field]??{}).sort(),`${field}: packed dependency names differ`);
    for(const [name,range] of Object.entries(manifest[field]??{})) {
      assert(typeof range==='string'&&!/^(?:workspace|file|link|portal):/i.test(range)&&!path.posix.isAbsolute(range)&&!path.win32.isAbsolute(range),`${name}: unresolved workspace/local dependency`);
      if(!original[field][name].startsWith('workspace:'))assert.equal(range,original[field][name],`${name}: packed dependency range differs`);
    }
  }
  verifyWorkspaceConversion(original,manifest,versions);
  for(const field of ['main','module','types','bin','exports'])assert.deepEqual(manifest[field],original[field],`Public entries differ: ${field}`);
  verifyPublicEntries(manifest,files);
  for(const name of requiredFiles)requireFile(name);
  let nativeReceipt;
  if(platform||native) {
    assert(native&&/^[a-f0-9]{64}$/.test(native.sha256),'Expected formal native digest is required');
    requireFile(native.file);nativeReceipt=names.get(native.file);
    assert.equal(nativeReceipt.sha256,native.sha256,'Native payload digest differs from formal build');
  }
  return {name:manifest.name,version:manifest.version,files,native:nativeReceipt??null};
}
