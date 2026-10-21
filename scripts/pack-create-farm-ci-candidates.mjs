import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
const [rootArg, outArg, pnpmArg, abi, ...templates] = process.argv.slice(2);
const root=path.resolve(rootArg), out=path.resolve(outArg), pnpm=path.resolve(pnpmArg);
if (!/^(linux-x64-gnu|darwin-arm64|win32-x64-msvc)$/.test(abi) || !templates.length) throw Error('Explicit supported CI ABI and templates required');
if(!abi.startsWith(process.platform+'-'+process.arch+'-') && abi!==process.platform+'-'+process.arch)throw Error('ABI must match actual CI host');
if(fs.existsSync(out))throw Error('Preserve existing pack evidence');
fs.mkdirSync(out,{recursive:true});
const hash=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const head=()=>execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim();
const state={head:head(),started:new Date().toISOString(),abi,roots:[],platforms:[],published:false,state:'running'};
const status=path.join(out,'producer.json'),save=()=>fs.writeFileSync(status,JSON.stringify(state,null,2));
const workspace=new Map();
for(const group of ['packages','rust-plugins','js-plugins'])for(const directory of fs.readdirSync(path.join(root,group))){
  const dir=path.join(root,group,directory),file=path.join(dir,'package.json');if(!fs.existsSync(file))continue;
  const manifest=JSON.parse(fs.readFileSync(file));if(workspace.has(manifest.name))throw Error('Duplicate package');workspace.set(manifest.name,{dir,manifest});
}
const selected=new Set();
function add(name){if(selected.has(name))return;const row=workspace.get(name);if(!row||row.manifest.private)throw Error('Missing public candidate '+name);selected.add(name);
  for(const section of ['dependencies','optionalDependencies','peerDependencies'])for(const dep of Object.keys(row.manifest[section]??{}))if(workspace.has(dep))add(dep);
}
for(const template of templates){
  if(!/^[a-z0-9-]+$/.test(template))throw Error('Invalid template');
  const m=JSON.parse(fs.readFileSync(path.join(root,'crates/create-farm-rs/templates',template,'package.json')));
  for(const section of ['dependencies','devDependencies'])for(const [name,range] of Object.entries(m[section]??{}))if(name.startsWith('@farmfe/')){
    add(name);if(range!=='^2.0.0'||!/^2\.\d+\.\d+$/.test(workspace.get(name).manifest.version))throw Error('Review candidate version/template constraint '+name+'@'+range);
  }
}
function command(args,cwd,log){const fd=fs.openSync(log,'wx');let result;try{result=spawnSync(process.execPath,args,{cwd,windowsHide:true,stdio:['ignore',fd,fd],timeout:120000});}finally{fs.closeSync(fd);}if(result.error)throw result.error;if(result.status!==0)throw Error('Pack failed: '+log);}
function pack(dir,name){const destination=path.join(out,'packs',name.replace(/[@/]/g,'_'));fs.mkdirSync(destination,{recursive:true});command([pnpm,'pack','--pack-destination',destination],dir,path.join(destination,'pack.log'));const files=fs.readdirSync(destination).filter(f=>f.endsWith('.tgz'));if(files.length!==1)throw Error('Expected one normal pack');const archive=path.join(destination,files[0]),manifest=JSON.parse(execFileSync('tar',['-xOf',archive,'package/package.json'],{encoding:'utf8'}));if(manifest.name!==name)throw Error('Wrong packed package');for(const section of ['dependencies','devDependencies','optionalDependencies','peerDependencies'])for(const [dep,spec] of Object.entries(manifest[section]??{}))if(spec.startsWith('workspace:'))throw Error('Unconverted workspace reference '+dep);return{name,version:manifest.version,archive,sha256:hash(archive),manifest};}
try{
  for(const name of selected){const row=workspace.get(name),manifestFile=path.join(row.dir,'package.json');
    // npm/pnpm pack never calls prepublishOnly. Reject any additional pack hook pending review.
    for(const hook of ['prepack','postpack','prepare'])if(row.manifest.scripts?.[hook])throw Error('Review candidate pack hook: '+name+'/'+hook);
    const packed=pack(row.dir,name);if(packed.version!==row.manifest.version)throw Error('Version drift');packed.sourceManifestSha256=hash(manifestFile);state.roots.push(packed);save();
    const platformDir=path.join(row.dir,'npm',abi);
    if(!fs.existsSync(platformDir))continue;
    const platform=JSON.parse(fs.readFileSync(path.join(platformDir,'package.json')));
    if(platform.name!==name+'-'+abi||platform.version!==packed.version||packed.manifest.optionalDependencies?.[platform.name]!==platform.version)throw Error('Native package cohort mismatch');
    if(typeof platform.main!=='string'||platform.main.includes('..')||path.isAbsolute(platform.main))throw Error('Unsafe native entry');
    const native=name==='@farmfe/core'?path.join(row.dir,'binding',platform.main):path.join(platformDir,platform.main);
    if(!fs.existsSync(native))throw Error('Missing current job native artifact: '+native);
    const stage=path.join(out,'platform-stage',platform.name.replace(/[@/]/g,'_'));fs.mkdirSync(stage,{recursive:true});
    // Only reviewed platform metadata, license and this job's native output are staged.
    let license=fs.existsSync(path.join(platformDir,'LICENSE'))?path.join(platformDir,'LICENSE'):path.join(row.dir,'LICENSE');
    if(!fs.existsSync(license)){if(platform.license!=='MIT'||row.manifest.license!=='MIT')throw Error('Missing package license');license=path.join(root,'LICENSE');}
    fs.copyFileSync(license,path.join(stage,'LICENSE'));fs.copyFileSync(native,path.join(stage,platform.main));
    function copyNotice(from,to){const stat=fs.lstatSync(from);if(stat.isSymbolicLink())throw Error('Notice symlink unsupported');if(stat.isDirectory()){fs.mkdirSync(to,{recursive:true});for(const entry of fs.readdirSync(from))copyNotice(path.join(from,entry),path.join(to,entry));}else if(stat.isFile())fs.copyFileSync(from,to);else throw Error('Unsupported notice entry');}
    for(const item of ['THIRD_PARTY_NOTICES.md','licenses'])if(fs.existsSync(path.join(row.dir,item))){copyNotice(path.join(row.dir,item),path.join(stage,item));platform.files=[...new Set([...(platform.files??[]),item])];}
    fs.writeFileSync(path.join(stage,'package.json'),JSON.stringify(platform,null,2)+'\n');
    const p=pack(stage,platform.name);p.nativeSource=native;p.nativeSha256=hash(native);state.platforms.push(p);save();
  }
  if(head()!==state.head)throw Error('Source head changed');state.state='packed';
}catch(error){state.state='failed';state.error=String(error.stack??error);process.exitCode=1;}
state.ended=new Date().toISOString();save();
