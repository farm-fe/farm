import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {spawn} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';

const exact = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
const lifecycle = ['preinstall','install','postinstall','prepare'];
const otherPlatform = /-(darwin-(x64|arm64)|linux-(x64|arm64)-(gnu|musl)|win32-(x64|arm64|ia32)-msvc)$/;
function fail(code, message) { throw new Error(`[${code}] ${message}`); }
function version(value) {
  if (typeof value === 'string' && /\d+\.\d+\.\d+-/.test(value)) fail('PRERELEASE', 'Prerelease versions/selectors are not allowed');
  if (typeof value !== 'string' || !exact.test(value)) fail('EXACT_VERSION', 'Expected stable x.y.z');
}
function selector(value) {
  if (typeof value !== 'string' || /^(workspace:|file:|link:|portal:|patch:|npm:|git(?:\+|:)|github:|gitlab:|bitbucket:|ssh:|https?:|\.|\/|~\/|[A-Za-z]:)/i.test(value) || /\.git(?:#|$)/.test(value)) fail('FORBIDDEN_SOURCE', 'Non-registry or unsupported dependency selector');
  if (/\d+\.\d+\.\d+-/.test(value)) fail('PRERELEASE', 'Prerelease versions/selectors are not allowed');
  if (!/^[0-9xXv*~^<>=| .-]+$/.test(value)) fail('FORBIDDEN_SOURCE', 'Non-registry or unsupported dependency selector');
}
function inside(base, file) { const relative=path.relative(base,file); return relative==='' || (!relative.startsWith('..'+path.sep) && relative!=='..' && !path.isAbsolute(relative)); }
function safeFile(name) { if (typeof name!=='string'||!name||name.includes('\\')||path.isAbsolute(name)||path.win32.isAbsolute(name)||name.split('/').includes('..')) fail('UNSAFE_PAYLOAD_PATH',String(name)); }
function sha256(data) { return crypto.createHash('sha256').update(data).digest('hex'); }
export function validateReleaseSet(release, plan) {
  if (!release) fail('RELEASE_SET_REQUIRED','Provide an approved publisher release-set; inventory/template is not one');
  if(release.schemaVersion!==1 || release.status!=='approved' || !release.releaseId || release.farmMajor!==2 || !/^[a-f0-9]{40}$/.test(release.baselineCommit??'') || !release.nativeAbi) fail('RELEASE_SET_NOT_APPROVED','Missing v2 cohort approval, baseline or ABI identity');
  if(release.registry!=='https://registry.npmjs.org') fail('REGISTRY_NOT_APPROVED','This draft only audits the official public npm registry');
  if(!plan || !Array.isArray(plan.packages)||plan.packages.length!==40) fail('PLAN_REQUIRED','Audited 40-package plan required; re-audit closure changes');
  if(!Array.isArray(release.packages)) fail('RELEASE_SET_INCOMPLETE','Missing packages');
  const entries=new Map();
  for(const p of release.packages){
    if(entries.has(p.name)) fail('RELEASE_SET_DUPLICATE',p.name); version(p.version);
    const sri=p.integrity?.match(/^sha512-([A-Za-z0-9+/]+={0,2})$/);
    if(!sri||Buffer.from(sri[1],'base64').length!==64||Buffer.from(sri[1],'base64').toString('base64')!==sri[1]) fail('RELEASE_INTEGRITY',p.name);
    if(!p.files || !p.files['package.json'] || !Object.keys(p.files).length) fail('PAYLOAD_MANIFEST_REQUIRED',p.name);
    for(const [file,hash]of Object.entries(p.files)){safeFile(file);if(!/^[a-f0-9]{64}$/.test(hash))fail('PAYLOAD_MANIFEST_REQUIRED',`${p.name}/${file}`);}
    entries.set(p.name,p);
  }
  const expected=new Set(plan.packages.map(p=>p.name));
  if(entries.size!==expected.size||[...expected].some(name=>!entries.has(name))||[...entries.keys()].some(name=>!expected.has(name))) fail('RELEASE_SET_INCOMPLETE','Release-set must cover exactly audited names, including both native platforms');
  for(const p of plan.packages){
    const chosen=entries.get(p.name).version.split('.').map(Number),floor=p.candidateVersion.split('.').map(Number);
    const difference=chosen[0]-floor[0]||chosen[1]-floor[1]||chosen[2]-floor[2];
    if(difference<0)fail('VERSION_DOWNGRADE',`${p.name}: ${entries.get(p.name).version} predates audited ${p.candidateVersion}; re-audit rather than substitute old packages`);
  }
  for(const p of plan.packages)if(p.role==='native-platform'){
    const entry=entries.get(p.name);
    if(entry.nativeAbi!==release.nativeAbi || !entry.files[p.artifactPath])fail('NATIVE_MANIFEST_REQUIRED',p.name);
    if(entry.version!==entries.get(p.owner)?.version)fail('COHORT_MISMATCH',`${p.name} must match its wrapper version`);
  }
  for(const name of ['@farmfe/core','@farmfe/cli'])if(!entries.get(name).version.startsWith('2.'))fail('FARM_MAJOR_MISMATCH',name);
  return entries;
}
function validateConsumer(root, entries, plan) {
  const real=fs.realpathSync(root);let parent=real;
  for(;;){
    if(fs.existsSync(path.join(parent,'pnpm-workspace.yaml')))fail('WORKSPACE_FORBIDDEN',parent);
    const manifest=path.join(parent,'package.json');
    if(fs.existsSync(manifest)&&JSON.parse(fs.readFileSync(manifest)).workspaces)fail('WORKSPACE_FORBIDDEN',parent);
    const next=path.dirname(parent);if(next===parent)break;parent=next;
  }
  if(fs.existsSync(path.join(real,'Cargo.toml'))||fs.existsSync(path.join(real,'rust-plugins'))||fs.existsSync(path.join(real,'packages/core')))fail('SOURCE_TREE_FORBIDDEN',real);
  const pkg=JSON.parse(fs.readFileSync(path.join(real,'package.json')));
  for(const value of [pkg.overrides,pkg.resolutions,pkg.pnpm?.overrides,pkg.pnpm?.patchedDependencies,pkg.pnpm?.packageExtensions])if(value&&Object.keys(value).length)fail('CONSUMER_OVERRIDES_FORBIDDEN','Independent consumer may not redirect sealed dependencies');
  for(const section of ['dependencies','devDependencies','optionalDependencies','peerDependencies'])for(const [name,spec]of Object.entries(pkg[section]??{})){
    selector(spec);if(name.startsWith('@farmfe/')){if(!entries.has(name))fail('CLOSURE_DRIFT',name);if(spec!==entries.get(name).version)fail('COHORT_UNPINNED',name);}
  }
  const direct={...pkg.dependencies,...pkg.devDependencies};
  for(const name of plan.directFarmPackages)if(direct[name]!==entries.get(name)?.version)fail('DIRECT_PIN_REQUIRED',name);
  return real;
}
async function registryMetadata(name, version, registry) {
  let response;try{response=await fetch(`${registry}/${encodeURIComponent(name)}/${encodeURIComponent(version)}`,{signal:AbortSignal.timeout(20000)});}catch{fail('REGISTRY_UNAVAILABLE',`${name}@${version}: metadata request failed`);}
  if(!response.ok)fail(response.status===404?'FORMAL_PACKAGE_MISSING':'REGISTRY_UNAVAILABLE',`${name}@${version}: registry HTTP ${response.status}`);
  try{return await response.json();}catch{fail('REGISTRY_UNAVAILABLE',`${name}@${version}: invalid metadata`);}
}
function validateMetadata(meta, entry, entries, release) {
  if(meta.name!==entry.name||meta.version!==entry.version)fail('FORMAL_PACKAGE_MISSING',`${entry.name}@${entry.version}`);
  version(meta.version);if(meta.dist?.integrity!==entry.integrity)fail('REGISTRY_INTEGRITY',entry.name);
  for(const section of ['dependencies','optionalDependencies','peerDependencies'])for(const[name,spec]of Object.entries(meta[section]??{})){
    selector(spec);if(!name.startsWith('@farmfe/'))continue;
    if(!entries.has(name)){if(section==='optionalDependencies'&&otherPlatform.test(name))continue;fail('CLOSURE_DRIFT',`${entry.name} -> ${name}`);}
    if(spec!==entries.get(name).version)fail('COHORT_MISMATCH',`${entry.name} -> ${name}: ${spec} vs ${entries.get(name).version}`);
  }
  for(const key of lifecycle){const script=meta.scripts?.[key];if(!script)continue;
    if(/cargo|rustup|rustc|napi\s+build|farm-plugin-tools\s+build|workspace|--filter/i.test(script))fail('SOURCE_BUILD_FORBIDDEN',`${entry.name} ${key}`);
    if(entry.installScripts?.[key]!==script)fail('INSTALL_SCRIPT_NOT_APPROVED',`${entry.name} ${key}`);
  }
}
function detectTarget() {
  if(process.platform==='win32'&&process.arch==='x64')return 'win32-x64-msvc';
  if(process.platform==='linux'&&process.arch==='x64'&&process.report?.getReport().header.glibcVersionRuntime)return 'linux-x64-gnu';
  fail('PLATFORM_NOT_APPROVED',`${process.platform}-${process.arch}; no source fallback`);
}
function findPackage(name, contexts, nodeModules) {
  for(const context of contexts)for(const specifier of [name+'/package.json',name]){
    let entry;try{entry=context.resolve(specifier);}catch{continue;}
    const real=fs.realpathSync(entry);if(!inside(nodeModules,real))fail('WORKSPACE_ESCAPE',`${name}: ${real}`);
    let dir=path.dirname(real);
    for(;;){const file=path.join(dir,'package.json');if(fs.existsSync(file)){const pkg=JSON.parse(fs.readFileSync(file));if(pkg.name===name)return{directory:dir,pkg};}
      const next=path.dirname(dir);if(next===dir||!inside(nodeModules,next))break;dir=next;
    }
  }
  return null;
}
function shippedFiles(directory, current=directory) {
  const files=[];
  for(const item of fs.readdirSync(current,{withFileTypes:true})){
    if(item.name==='node_modules')continue;
    const file=path.join(current,item.name),real=fs.realpathSync(file);
    if(!inside(directory,real))fail('WORKSPACE_ESCAPE',file);
    if(fs.statSync(real).isDirectory()){
      if(item.isSymbolicLink())fail('UNSEALED_PAYLOAD','Linked payload directory: '+file);
      files.push(...shippedFiles(directory,file));
    }else files.push(path.relative(directory,file).split(path.sep).join('/'));
  }
  return files;
}
function installedPackages(root, entries, plan, target) {
  const nm=fs.realpathSync(path.join(root,'node_modules'));if(!inside(root,nm))fail('WORKSPACE_ESCAPE',nm);
  const required=plan.packages.filter(p=>p.role!=='native-platform'||p.requiredOn===target);
  const contexts=[createRequire(path.join(root,'package.json'))],found=new Map();let pending=[...required];
  while(pending.length){const next=[];let progress=false;
    for(const p of pending){const resolved=findPackage(p.name,contexts,nm);if(!resolved){next.push(p);continue;}
      const entry=entries.get(p.name);if(resolved.pkg.version!==entry.version)fail('INSTALLED_VERSION_MISMATCH',p.name);
      for(const[file,expected]of Object.entries(entry.files)){
        const payload=path.join(resolved.directory,file);let real,data;
        try{real=fs.realpathSync(payload);data=fs.readFileSync(real);}catch{fail(p.role==='native-platform'&&file===p.artifactPath?'MISSING_NATIVE':'PUBLISHED_FILE_MISSING',`${p.name}/${file}; no build fallback`);}
        if(!inside(nm,real)||!inside(resolved.directory,real))fail('WORKSPACE_ESCAPE',real);
        if(sha256(data)!==expected)fail('PAYLOAD_INTEGRITY',`${p.name}/${file}`);
        if(p.role==='native-platform'&&file===p.artifactPath){const prefix=p.requiredOn.startsWith('win32')?Buffer.from('MZ'):Buffer.from([0x7f,0x45,0x4c,0x46]);if(!data.subarray(0,prefix.length).equals(prefix))fail('NATIVE_FORMAT',p.name);}
      }
      const unexpected=shippedFiles(resolved.directory).filter(file=>!Object.hasOwn(entry.files,file));
      if(unexpected.length)fail('UNSEALED_PAYLOAD',`${p.name}: ${unexpected.join(', ')}`);
      found.set(p.name,resolved);contexts.push(createRequire(path.join(resolved.directory,'package.json')));progress=true;
    }
    if(!progress){const p=next[0];fail(p.role==='native-platform'?'MISSING_NATIVE':'PUBLISHED_PACKAGE_MISSING',`${p.name}; only registry precompiled artifacts allowed`);}pending=next;
  }
  return found;
}
function execute(command){return new Promise((resolve,reject)=>{const child=spawn(command.executable,command.args,{cwd:command.cwd,env:command.env,stdio:'inherit',windowsHide:true});child.once('error',reject);child.once('exit',(code)=>resolve(code));});}
export async function runConsumer(options, hooks={}) {
  const {releaseSet:release,plan}=options,entries=validateReleaseSet(release,plan);
  const env={...(options.environment??process.env)};
  for(const key of ['NAPI_RS_NATIVE_LIBRARY_PATH','NAPI_RS_FORCE_WASI','NODE_PATH'])if(env[key])fail('BINDING_OVERRIDE_FORBIDDEN',key);
  const nodeOptions=(env.NODE_OPTIONS??'').trim();
  if(nodeOptions&&nodeOptions.split(/\s+/).some(option=>!/^--max[-_]old[-_]space[-_]size=\d+$/.test(option)&&!['--no-warnings','--trace-warnings','--enable-source-maps'].includes(option)))fail('BINDING_OVERRIDE_FORBIDDEN','NODE_OPTIONS outside memory/diagnostic allowlist');
  const root=validateConsumer(options.root,entries,plan),phase=options.phase??'build';
  if(!['preflight','build'].includes(phase))fail('UNKNOWN_PHASE',phase);
  const get=hooks.getMetadata??registryMetadata;
  for(const entry of entries.values())validateMetadata(await get(entry.name,entry.version,release.registry),entry,entries,release);
  if(phase==='preflight')return{phase,releaseId:release.releaseId,baselineCommit:release.baselineCommit,nativeAbi:release.nativeAbi,packages:entries.size,commandsRun:0};
  const lock=path.join(root,'pnpm-lock.yaml');if(!fs.existsSync(lock))fail('LOCK_REQUIRED','Generate independent lock normally after preflight, then use frozen install');
  if(/workspace:|\blink:|\bfile:|git\+|git:\/\/|github:/i.test(fs.readFileSync(lock,'utf8')))fail('FORBIDDEN_SOURCE','Consumer lock contains non-registry source');
  const target=options.target??detectTarget();if(!plan.targetPlatforms.includes(target))fail('PLATFORM_NOT_APPROVED',target);
  const installed=installedPackages(root,entries,plan,target),cli=installed.get('@farmfe/cli');
  const bin=typeof cli.pkg.bin==='string'?cli.pkg.bin:cli.pkg.bin?.farm;if(!bin)fail('CLI_BIN_MISSING','@farmfe/cli');
  const normalized=bin.replace(/^\.\//,'');safeFile(normalized);if(!entries.get('@farmfe/cli').files[normalized])fail('CLI_BIN_NOT_SEALED',normalized);
  const file=fs.realpathSync(path.join(cli.directory,normalized));if(!inside(cli.directory,file))fail('WORKSPACE_ESCAPE',file);
  const run=hooks.execute??execute;
  for(const args of [[file,'--help'],[file,'build','--config','farm.config.ts']]){const code=await run({executable:process.execPath,args,cwd:root,env});if(code!==0)fail('CLI_FAILED',`exit ${code}; no retry with Cargo/rustup/workspace`);}
  return{phase,releaseId:release.releaseId,baselineCommit:release.baselineCommit,nativeAbi:release.nativeAbi,target,packages:installed.size,packageVersions:[...installed].map(([name,p])=>({name,version:p.pkg.version})),lockSha256:sha256(fs.readFileSync(lock)),releaseSetObjectSha256:sha256(JSON.stringify(release)),commandsRun:2};
}
async function main(){
  const args=process.argv.slice(2),phase=args.shift()??'build',flags={};
  while(args.length){const key=args.shift();if(!['--root','--release-set','--plan'].includes(key)||!args.length)fail('ARGUMENTS','Use preflight|build --root DIR --release-set FILE [--plan FILE]');flags[key]=args.shift();}
  if(!flags['--release-set'])fail('RELEASE_SET_REQUIRED','No default inventory/local version fallback');
  let release;try{release=JSON.parse(fs.readFileSync(flags['--release-set'],'utf8'));}catch{fail('RELEASE_SET_REQUIRED','Cannot read approved release-set');}
  const plan=JSON.parse(fs.readFileSync(flags['--plan']??fileURLToPath(new URL('../docs/farm-docs/registry-release-plan.draft.json',import.meta.url)),'utf8'));
  if(Number(process.versions.node.split('.')[0])!==22)fail('NODE_VERSION','Node22 required');
  console.log(JSON.stringify(await runConsumer({root:flags['--root']??process.cwd(),releaseSet:release,plan,phase}),null,2));
}
if(process.argv[1]&&import.meta.url===pathToFileURL(path.resolve(process.argv[1])).href)main().catch(error=>{console.error(error.message);process.exitCode=1;});
