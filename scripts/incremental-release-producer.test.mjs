import assert from 'node:assert/strict';
import {test} from 'node:test';
import {makePublicationManifest,releaseProducerIdentity,candidatePlanForSelection,releaseBuildDirectories,publicationManifestDigest,runReleaseJsBuilds,verifyReleaseSelection} from './incremental-release-producer.mjs';
import {sha256} from './release-archive.mjs';
import fs from 'node:fs';
import os from 'node:os';
import {execFileSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {rustRegistryReuse,sdkConsumerManifest} from './v2-rust-registry-reuse.mjs';

const sha='a'.repeat(40),tree='b'.repeat(40);
const environment={GITHUB_ACTIONS:'true',GITHUB_REF:'refs/heads/main',GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_REPOSITORY:'farm-fe/farm',
 GITHUB_WORKFLOW_REF:'farm-fe/farm/.github/workflows/incremental-release.yaml@refs/heads/main',GITHUB_SHA:sha,GITHUB_RUN_ID:'10',FARM_BUILD_PROFILE:'release'};
test('only actual release main source can produce release packages',()=>{
 assert.equal(releaseProducerIdentity(environment,{sha,tree}).runId,10);
 for(const [key,value] of [['GITHUB_EVENT_NAME','pull_request'],['FARM_BUILD_PROFILE','ci'],['GITHUB_REF','refs/pull/2330/merge']])
  assert.throws(()=>releaseProducerIdentity({...environment,[key]:value},{sha,tree}));
 assert.throws(()=>releaseProducerIdentity(environment,{sha:'c'.repeat(40),tree}));
});
function fixture(){
 const body=Buffer.from('sealed archive');
 const row={name:'@farmfe/core',version:'2.0.0',file:'core.tgz',sha256:sha256(body)};
 return {plan:{abi:'0.5.4',cargo:[],npm:{primary:[{name:'@farmfe/core',version:'2.0.0',manifestPath:'packages/core/package.json'}],payloads:[]}},selection:{chain:'release',profile:'release',sourceSHA:sha,sourceTree:tree,planHash:'d'.repeat(64),publish:['npm:@farmfe/core']},
 receipt:{status:'complete-seal',testedCommit:sha,testedTree:tree,rust:[],records:[row]},source:{sha,tree},runId:10,attempt:1,readArchive:()=>body};
}
test('manifest binds exact original bytes, selected inventory and Core tag',()=>{
 const result=makePublicationManifest(fixture());
 assert.equal(result.tag,'v2.0.0');assert.equal(result.packages.length,1);assert.equal(result.packages[0].archive,'core.tgz');
 assert.equal(result.profile,'release');assert.equal(result.producerRunId,10);
});
test('producer rejects incomplete or different-byte publication scope',()=>{
 const missing=fixture();missing.selection.publish=['npm:missing'];assert.throws(()=>makePublicationManifest(missing));
 const changed=fixture();changed.readArchive=()=>Buffer.from('rebuilt');assert.throws(()=>makePublicationManifest(changed));
 const incomplete=fixture();incomplete.receipt.status='incomplete';assert.throws(()=>makePublicationManifest(incomplete));
});
test('plugin-only batch uses independent release identity without Core tag reuse',()=>{
 const value=fixture();value.receipt.records[0].name='@farmfe/js-plugin-dts';value.selection.publish=['npm:@farmfe/js-plugin-dts'];
 value.plan.npm.primary[0].name='@farmfe/js-plugin-dts';
 assert.equal(makePublicationManifest(value).tag,'packages-'+value.selection.planHash.slice(0,16));
});
test('npm runtime and payload prerequisites publish before their consumers',()=>{
 const value=fixture(),original=value.receipt.records[0];
 value.receipt.records.push({...original,name:'@farmfe/core-native',file:'native.tgz'});
 value.plan.npm.payloads.push({name:'@farmfe/core-native',version:'2.0.0',owner:'@farmfe/core',manifestPath:'packages/core/npm/native/package.json'});
 value.selection.publish.push('npm:@farmfe/core-native');
 value.readPackage=row=>row.name==='@farmfe/core'?{optionalDependencies:{'@farmfe/core-native':'2.0.0'}}:{};
 assert.deepEqual(makePublicationManifest(value).packages.map(row=>row.name),['@farmfe/core-native','@farmfe/core']);
});

test('delta manifest keeps complete family and rejects missing version or retirement preparation',()=>{
 const value=fixture();
 value.plan.npm.primary.push({name:'unchanged',version:'1.0.0',manifestPath:'packages/unchanged/package.json'});
 value.receipt.records.push({name:'unchanged',version:'1.0.0',file:'unchanged.tgz',sha256:value.receipt.records[0].sha256});
 const result=makePublicationManifest(value);
 assert.equal(result.family.length,2);assert.equal(result.packages.length,1);assert.deepEqual(result.publish,['npm:@farmfe/core']);
 value.selection.versionRequired=['npm:unchanged'];assert.throws(()=>makePublicationManifest(value));
 value.selection.versionRequired=[];value.selection.removedPackages=['npm:retired'];assert.throws(()=>makePublicationManifest(value));
});

test('mixed delta cannot silently discard a different-byte same-version rebuild',()=>{
 const value=fixture(),body=value.readArchive();
 value.plan.npm.primary.push({name:'unchanged',version:'1.0.0',manifestPath:'packages/unchanged/package.json'});
 value.receipt.records.push({name:'unchanged',version:'1.0.0',file:'unchanged.tgz',sha256:sha256(body)});
 value.selection.build=['npm:@farmfe/core','npm:unchanged'];
 value.selection.baselinePackages={'npm:unchanged':{version:'1.0.0',archiveSha256:sha256(body)}};
 assert.equal(makePublicationManifest(value).packages.length,1);
 value.selection.baselinePackages['npm:unchanged'].archiveSha256=sha256(Buffer.from('original different archive'));
 assert.throws(()=>makePublicationManifest(value),/prepare a new version/);
 value.selection.baselinePackages={};assert.throws(()=>makePublicationManifest(value),/independently verified original/);
});

test('JS-only delta builds source prerequisites and seals only selected archives',()=>{
 const root=fileURLToPath(new URL('../',import.meta.url));
 const primary=['packages/core','js-plugins/dts','js-plugins/babel'].map(directory=>{
  const pkg=JSON.parse(fs.readFileSync(path.join(root,directory,'package.json')));return {name:pkg.name,version:pkg.version,manifestPath:directory+'/package.json'};
 });
 const family={abi:'0.5.4',cargo:[{name:'farmfe_core',version:'2.2.0',manifestPath:'crates/core/Cargo.toml'}],npm:{primary,payloads:[]}};
 const selection={publicationScope:'npm',chain:'release',profile:'release',sourceSHA:sha,sourceTree:tree,planHash:'d'.repeat(64),baselineBinding:'e'.repeat(64),
  build:['npm:@farmfe/js-plugin-babel'],publish:['npm:@farmfe/js-plugin-babel'],versionRequired:[],baselinePackages:{}};
 for(const row of primary)selection.baselinePackages['npm:'+row.name]={version:row.version,archiveSha256:'a'.repeat(64),integrity:'sha512-'+Buffer.alloc(64).toString('base64')};
 selection.baselinePackages['cargo:farmfe_core']={version:'2.2.0',archiveSha256:'b'.repeat(64)};
 const candidate=candidatePlanForSelection(family,selection);
 assert.deepEqual(candidate.publication.npm.primary.map(row=>row.name),['@farmfe/js-plugin-babel']);
 assert.deepEqual(candidate.publication.npm.payloads,[]);
 assert.deepEqual(candidate.validationDependencies.npm.map(row=>row.name),['@farmfe/core','@farmfe/js-plugin-dts']);
 assert.deepEqual(releaseBuildDirectories(family,selection,root),['js-plugins/babel']);
 assert.deepEqual(candidate.publication.cargo,[]);
 assert.equal(candidate.publication.reuseNpm,undefined);assert.equal(candidate.publication.reuseRust,undefined);
 assert.equal(candidate.preparation.mode,'incremental-npm-source-release');
 assert.deepEqual(candidate.validationDependencies.cargo,[]);
 delete selection.baselinePackages['npm:@farmfe/core'];assert.throws(()=>candidatePlanForSelection(family,selection));
});

test('portable candidate digest matches Python canonical manifest encoding including Unicode',()=>{
 assert.equal(publicationManifestDigest({z:'中文 😀',a:[1,null,true]}),'cf887cc07ffa09c406c81cfc551c3e45d27f111fbf431032bc82846b8f020428');
 assert.throws(()=>publicationManifestDigest({bad:1.5}));
 assert.equal(sdkConsumerManifest({cargo:[{name:'farmfe_core',version:'2.2.1'}]},'[dependencies]\nfarmfe_core = "=2.2.0"\n'), '[dependencies]\nfarmfe_core = "=2.2.1"\n');
 assert.throws(()=>sdkConsumerManifest({cargo:[]},'[patch.crates-io]\n'));
});


test('terminal selection verification rejects changed membership, foreign digest and source after builds', () => {
  const selection = {...fixture().selection, build:['npm:@farmfe/core'], versionRequired:[], publicationBlocked:false};
  selection.planHash = publicationManifestDigest(Object.fromEntries(Object.entries(selection).filter(([key]) => key !== 'planHash')));
  const expected = selection.planHash;
  assert.equal(verifyReleaseSelection(selection, {sha,tree}, expected), selection);
  const changed = {...selection, build:[]};
  assert.throws(() => verifyReleaseSelection(changed, {sha,tree}, expected), /digest differs/);
  changed.planHash = publicationManifestDigest(Object.fromEntries(Object.entries(changed).filter(([key]) => key !== 'planHash')));
  assert.throws(() => verifyReleaseSelection(changed, {sha,tree}, expected), /trusted job output/);
  assert.throws(() => verifyReleaseSelection(selection, {sha:'f'.repeat(40),tree}, expected));
  assert.throws(() => verifyReleaseSelection(selection, {sha,tree:'f'.repeat(40)}, expected));
  assert.throws(() => verifyReleaseSelection(selection, {sha,tree}, undefined), /Trusted release plan hash/);
});

test('recipe-only rebuilt bytes are compared without a publisher transaction or baseline advance', () => {
  const value = fixture(); value.verificationOnly = true;
  value.selection = {...value.selection, publish:[], build:['npm:@farmfe/core'], baselineBinding:'e'.repeat(64),
    baselinePackages:{'npm:@farmfe/core':{version:'2.0.0',archiveSha256:value.receipt.records[0].sha256}}};
  const report = makePublicationManifest(value);
  assert.equal(report.kind, 'release-rebuild-verification'); assert.equal(report.originalBytesEqual, true);
  assert.equal(report.publicationExecuted, false); assert.equal(report.baselineAdvanced, false);
  assert.equal(report.packages, undefined); assert.equal(report.tag, undefined);
  const altered = Buffer.from('new same-version bytes');
  value.readArchive = () => altered; value.receipt.records[0].sha256 = sha256(altered);
  assert.throws(() => makePublicationManifest(value), /prepare a new version/);
  value.selection.build=[]; assert.throws(() => makePublicationManifest(value), /nonempty build/);
});

for (const protocol of ['file:', 'link:', 'workspace:']) for (const transitive of [false, true]) test(`real executor follows resolved ${protocol} private alias, transitive=${transitive}`, async () => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'farm-private-build-')));
  const files = {'Cargo.toml':'[workspace]\nmembers=[]\n', 'pnpm-workspace.yaml':'packages:\n  - packages/*\n'};
  try {
    const family = {npm:{primary:[{name:'consumer',version:'1.0.0',manifestPath:'packages/consumer/package.json'}],payloads:[]},cargo:[]};
    files['packages/consumer/package.json'] = JSON.stringify({name:'consumer',version:'1.0.0',scripts:{build:'node build.mjs'},dependencies:{bridgeAlias:protocol+'../bridge'}});
    files['packages/bridge/package.json'] = JSON.stringify({name:'bridge',private:true,scripts:{build:'node build.mjs'}});
    if (transitive) {
      family.npm.primary.push({name:'dependency',version:'1.0.0',manifestPath:'packages/dependency/package.json'});
      files['packages/dependency/package.json'] = JSON.stringify({name:'dependency',version:'1.0.0',scripts:{build:'node build.mjs'},dependencies:{bridgeAlias:protocol+'../bridge'}});
      files['packages/consumer/package.json'] = JSON.stringify({name:'consumer',version:'1.0.0',scripts:{build:'node build.mjs'},dependencies:{dependencyAlias:protocol+'../dependency'}});
    }
    for (const [file, body] of Object.entries(files)) {fs.mkdirSync(path.dirname(path.join(root,file)),{recursive:true});fs.writeFileSync(path.join(root,file),body);}
    fs.writeFileSync(path.join(root,'packages/bridge/build.mjs'), "import fs from 'node:fs'; fs.writeFileSync('ready','private built');");
    fs.writeFileSync(path.join(root,'packages/consumer/build.mjs'), "import fs from 'node:fs'; if(fs.readFileSync('../bridge/ready','utf8')!=='private built') throw Error('missing prerequisite'); fs.writeFileSync('ready','consumer built');");
    if (transitive) {
      fs.writeFileSync(path.join(root,'packages/dependency/build.mjs'), "import fs from 'node:fs'; fs.readFileSync('../bridge/ready'); fs.writeFileSync('ready','dependency built');");
      fs.writeFileSync(path.join(root,'packages/consumer/build.mjs'), "import fs from 'node:fs'; fs.readFileSync('../dependency/ready'); fs.writeFileSync('ready','consumer built');");
    }
    const planner = fileURLToPath(new URL('./incremental-chain-plan.py', import.meta.url));
    const python = 'import importlib.util,json,sys\ns=importlib.util.spec_from_file_location("planner",sys.argv[1]);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)\ng,issues=m.repository_graph({k:v.encode() for k,v in json.load(sys.stdin).items()});assert not issues,issues\nprint(json.dumps({k:sorted(v["productDeps"]) for k,v in g.items()}))';
    const buildEdges = JSON.parse(execFileSync('python3',['-B','-c',python,planner],{input:JSON.stringify(files),encoding:'utf8'}));
    const selection = {build:['npm:consumer',...(transitive ? ['npm:dependency'] : [])],buildNodes:Object.keys(buildEdges),buildEdges};
    assert.deepEqual(buildEdges['npm:packages/consumer'], [transitive ? 'npm:packages/dependency' : 'npm:packages/bridge']);
    const calls = [], options = {root,pnpm:'pinned-fixture-cli',verifyManifest:file=>files[file], execute:async(executable,args,options)=>{
      calls.push(path.basename(options.cwd)); assert.deepEqual(args,['pinned-fixture-cli','run','build']);
      execFileSync(executable,['build.mjs'],{cwd:options.cwd,encoding:'utf8',timeout:10000});
    }};
    const expected = ['bridge',...(transitive ? ['dependency'] : []),'consumer'];
    assert.deepEqual(await runReleaseJsBuilds(family,selection,options),expected.map(name=>'packages/'+name));
    assert.deepEqual(calls,expected); assert.equal(fs.readFileSync(path.join(root,'packages/consumer/ready'),'utf8'),'consumer built');
    fs.writeFileSync(path.join(root,'packages/bridge/package.json'), files['packages/bridge/package.json']+' ');
    await assert.rejects(runReleaseJsBuilds(family,selection,options),/differs from committed source/);
    fs.writeFileSync(path.join(root,'packages/bridge/package.json'), files['packages/bridge/package.json']);
    selection.buildEdges['npm:packages/bridge']=['npm:packages/consumer'];
    assert.throws(()=>releaseBuildDirectories(family,selection,root),/Cyclic/);
  } finally {fs.rmSync(root,{recursive:true,force:true});}
});

test('workflow routes rebuilds separately and terminal steps bind the trusted plan hash', () => {
  const workflow = fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml',import.meta.url),'utf8');
  for(const name of ['Verify recipe-only rebuild against original release archive bytes','Bind actual publication delta to the original archives']) {
    const step=workflow.split('- name: '+name)[1].split('\n      - ')[0];
    assert.ok(step.includes('FARM_RELEASE_PLAN_HASH: ${{ needs.plan.outputs.plan_hash }}'));
  }
  assert.ok(workflow.includes("needs.plan.outputs.publish_required == 'false' && needs.plan.outputs.build_required == 'false'"));
  const publisher=workflow.split('  publisher:')[1].split('  public-proof:')[0];
  assert.ok(publisher.includes("needs.plan.outputs.publish_required == 'true'"));
  assert.ok(!workflow.includes('  candidate-consumers:'));
});

test('registry-selected publication builds without a successful baseline and retains public dependencies only for validation',()=>{
 const value=fixture();
 value.plan.npm.primary.push({name:'published',version:'1.0.0',manifestPath:'packages/published/package.json'});
 value.selection={...value.selection,publicationScope:'npm',build:['npm:@farmfe/core'],versionRequired:[],registryRetained:{'npm:published':{version:'1.0.0',archiveSha256:'c'.repeat(64),integrity:'sha512-'+Buffer.alloc(64).toString('base64')}}};
 const candidate=candidatePlanForSelection(value.plan,value.selection);
 assert.deepEqual(candidate.publication.npm.primary.map(row=>row.name),['@farmfe/core']);
 assert.deepEqual(candidate.validationDependencies.npm.map(row=>row.name),['published']);
 assert.equal(candidate.incremental.baselineBinding,null);
 value.selection.registrySnapshot={schemaVersion:1,sourceSHA:sha,sourceTree:tree,packages:{}};
 assert.deepEqual(makePublicationManifest(value).registrySnapshot,value.selection.registrySnapshot);
 delete value.selection.registryRetained['npm:published'].integrity;
 assert.throws(()=>candidatePlanForSelection(value.plan,value.selection));
});


test('actual full npm family orders hard dependencies without treating peer cycles as upload cycles', () => {
 const root=fileURLToPath(new URL('../',import.meta.url));
 const family=JSON.parse(fs.readFileSync(path.join(root,'docs/releases/v2-candidate.json')));
 const entries=[...family.npm.primary,...family.npm.payloads];
 const bodies=new Map(entries.map(row=>[row.name,Buffer.from(row.name+'@'+row.version)]));
 const rows=entries.map((row,index)=>({name:row.name,version:row.version,file:index+'.tgz',sha256:sha256(bodies.get(row.name))}));
 const value={...fixture(),plan:family,selection:{...fixture().selection,publicationScope:'npm',publish:rows.map(row=>'npm:'+row.name)},
  receipt:{...fixture().receipt,records:rows},readArchive:file=>bodies.get(rows.find(row=>row.file===file).name),
  readPackage:row=>JSON.parse(fs.readFileSync(path.join(root,entries.find(pkg=>pkg.name===row.name).manifestPath)))};
 const result=makePublicationManifest(value),names=result.packages.map(row=>row.name);
 assert.equal(result.packages.length,297);assert.equal(result.family.length,297);
 assert.ok(names.indexOf('@farmfe/docs-theme-default')<names.indexOf('@farmfe/docs'));
 for(const row of result.packages)assert.equal(row.archiveSha256,sha256(bodies.get(row.name)));
});

test('only peer upload cycles are allowed; hard and optional cycles still reject', () => {
 const value=fixture(),original=value.receipt.records[0];
 value.receipt.records.push({...original,name:'theme',file:'theme.tgz'});
 value.plan.npm.primary.push({name:'theme',version:'2.0.0',manifestPath:'packages/theme/package.json'});
 value.selection.publish.push('npm:theme');
 for(const field of ['dependencies','optionalDependencies']) {
  value.readPackage=row=>({[field]:{[row.name==='theme'?'@farmfe/core':'theme']:'2.0.0'}});
  assert.throws(()=>makePublicationManifest(value),/Cyclic npm publication dependencies/);
 }
 value.readPackage=row=>({[row.name==='theme'?'peerDependencies':'dependencies']:{[row.name==='theme'?'@farmfe/core':'theme']:'2.0.0'}});
 assert.deepEqual(makePublicationManifest(value).packages.map(row=>row.name),['theme','@farmfe/core']);
 value.readPackage=row=>({peerDependencies:{[row.name==='theme'?'@farmfe/core':'theme']:'2.0.0'}});
 assert.equal(makePublicationManifest(value).packages.length,2);
 value.readArchive=()=>Buffer.from('tampered');assert.throws(()=>makePublicationManifest(value));
});

function presenceFixture() {
 const value=fixture();
 value.selection={...value.selection,selectionPolicy:'registry-exact-version-v2',build:['npm:@farmfe/core'],
  publicationScope:'npm',baselineBinding:null,registryRetained:{},registryUnknown:{},nativeBuildCells:[],preparationNativeCells:[],registryBuildInputs:{},
  registrySnapshot:{schemaVersion:2,sourceSHA:sha,sourceTree:tree,packages:{'npm:@farmfe/core':{status:'missing',name:'@farmfe/core',version:'2.0.0'}}}};
 value.receipt.buildFailures={};return value;
}
test('version presence permits observation-only manifests but never legacy empty publication',()=>{
 const value=presenceFixture();value.selection.build=[];value.selection.publish=[];value.receipt.records=[];
 value.selection.registrySnapshot.packages['npm:@farmfe/core'].status='published';
 value.selection.registryRetained={'npm:@farmfe/core':value.selection.registrySnapshot.packages['npm:@farmfe/core']};
 const result=makePublicationManifest(value);
 assert.deepEqual(result.packages,[]);assert.deepEqual(result.publish,[]);assert.equal(result.baseline,null);
 assert.equal(result.selectionPolicy,'registry-exact-version-v2');assert.deepEqual(result.buildFailures,{});
 delete value.selection.selectionPolicy;assert.throws(()=>makePublicationManifest(value),/No publication delta/);
});
test('version presence preserves full missing scope and accounts for every failed build',()=>{
 const value=presenceFixture();
 const failed={name:'@farmfe/core-linux-x64-musl',version:'2.0.0',owner:'@farmfe/core',platform:'linux-x64-musl',manifestPath:'packages/core/npm/linux-x64-musl/package.json'};
 value.plan.npm.payloads.push(failed);value.selection.build.push('npm:'+failed.name);value.selection.publish.push('npm:'+failed.name);
 value.receipt.buildFailures['npm:'+failed.name]='native-build-or-artifact-unavailable';
 const result=makePublicationManifest(value);
 assert.equal(result.packages.length,1);assert.equal(result.family.length,2);assert.equal(result.publish.length,2);
 assert.deepEqual(result.buildFailures,value.receipt.buildFailures);
 delete value.receipt.buildFailures['npm:'+failed.name];assert.throws(()=>makePublicationManifest(value),/every unsealed/);
 value.receipt.buildFailures['npm:'+failed.name]='failure with unbounded raw output';assert.throws(()=>makePublicationManifest(value),/Invalid package/);
});
test('version presence cannot seal retained packages or silently publish blocked missing inputs',()=>{
 const value=presenceFixture();value.selection.buildBlocked={'npm:@farmfe/core':{reason:'registry-host-tool-unknown'}};
 assert.throws(()=>makePublicationManifest(value),/Blocked source input/);
 value.selection.buildBlocked={};value.selection.publish=[];value.selection.build=[];
 assert.throws(()=>makePublicationManifest(value),/Registry-present package was sealed/);
});
test('unknown registry versions are not copied into source candidate dependencies',()=>{
 const value=presenceFixture();value.plan.npm.primary.push({name:'uncertain',version:'1.0.0',manifestPath:'packages/uncertain/package.json'});
 value.selection.registryUnknown={'npm:uncertain':{status:'unknown',version:'1.0.0',reason:'registry-unavailable'}};
 const candidate=candidatePlanForSelection(value.plan,value.selection);
 assert.deepEqual(candidate.validationDependencies.npm,[]);
 assert.deepEqual(candidate.publication.npm.primary.map(row=>row.name),['@farmfe/core']);
 assert.equal(candidate.incremental.selectionPolicy,'registry-exact-version-v2');
});
test('native cells never expand one owner across published or unknown platforms',async()=>{
 const {presenceNativePayloads,assemblePresenceNative}=await import('./incremental-release-producer.mjs');
 const value=presenceFixture();
 value.plan.npm.payloads=['linux-x64-gnu','linux-x64-musl','darwin-arm64'].map(platform=>({name:'native-'+platform,version:'2.0.0',owner:'@farmfe/core',platform,manifestPath:'packages/core/npm/'+platform+'/package.json'}));
 for(const [index,pkg] of value.plan.npm.payloads.entries()) value.selection.registrySnapshot.packages['npm:'+pkg.name]={status:index===2?'published':'missing'};
 value.selection.nativeBuildCells=value.plan.npm.payloads.slice(0,2).map(row=>({owner:row.owner,platform:row.platform,package:'npm:'+row.name}));
 value.selection.preparationNativeOwners=['@farmfe/core'];
 assert.deepEqual(presenceNativePayloads(value.plan,value.selection).map(row=>row.platform),['linux-x64-gnu','linux-x64-musl']);
 const calls=[];
 const result=assemblePresenceNative(value.plan,value.selection,{root:'/unused',pool:'/unused',sourceSHA:sha,assemble:(_root,_pool,_sha,plan)=>{
   assert.equal(plan.npm.payloads.length,1);const pkg=plan.npm.payloads[0];calls.push(pkg.platform);
   if(pkg.platform==='linux-x64-musl')throw Error('failed cell');return [{name:pkg.name,platform:pkg.platform}];
 }});
 assert.deepEqual(calls,['linux-x64-gnu','linux-x64-musl']);assert.equal(result.records.length,1);
 assert.deepEqual(result.failures,{'npm:native-linux-x64-musl':'native-build-or-artifact-unavailable'});
 value.selection.nativeBuildCells.push({owner:'@farmfe/core',platform:'darwin-arm64',package:'npm:native-darwin-arm64'});
 assert.throws(()=>presenceNativePayloads(value.plan,value.selection),/cannot be rebuilt/);
});
test('failed JS prerequisite blocks its consumer while independent JS still builds',async()=>{
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'farm-presence-js-'));
 try {
  const files={},primary=['a','b','c'].map(name=>({name,version:'1.0.0',manifestPath:'packages/'+name+'/package.json'}));
  for(const row of primary) {const file=path.join(root,row.manifestPath);fs.mkdirSync(path.dirname(file),{recursive:true});files[row.manifestPath]=JSON.stringify({name:row.name,scripts:{build:'node build.mjs'}});fs.writeFileSync(file,files[row.manifestPath]);}
  const selection={build:['npm:a','npm:b','npm:c'],buildNodes:['npm:packages/a','npm:packages/b','npm:packages/c'],buildEdges:{'npm:packages/a':['npm:packages/b'],'npm:packages/b':[],'npm:packages/c':[]}};
  const calls=[];const result=await runReleaseJsBuilds({npm:{primary,payloads:[]}},selection,{root,pnpm:'fixture',bestEffort:true,verifyManifest:file=>files[file],execute:async(_node,_args,options)=>{const name=path.basename(options.cwd);calls.push(name);if(name==='b')throw Error('fixture failure');}});
  assert.deepEqual(calls,['b','c']);assert.deepEqual(result.completed,['packages/c']);
  assert.deepEqual(Object.keys(result.failures).sort(),['npm:packages/a','npm:packages/b']);
 } finally {fs.rmSync(root,{recursive:true,force:true});}
});
test('per-package packing seals successful new subset and records failures without dropping scope',async()=>{
 const {packPresenceCandidates}=await import('./incremental-release-producer.mjs');
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'farm-presence-pack-'));
 try {
  const rows=['failed','working'].map(name=>({name,version:'1.0.0',manifestPath:'packages/'+name+'/package.json'}));
  const candidate={npm:{primary:rows,payloads:[]},publication:{npm:{primary:rows,payloads:[]},cargo:[]},incremental:{selectionPolicy:'registry-exact-version-v2',publish:['npm:failed','npm:working'],buildFailures:{},registryBuildMaterials:[{origin:'registry-present'}]}};
  const calls=[];const receipt=packPresenceCandidates({root,candidate,assembly:{sourceCommit:sha,records:[]},artifacts:[],identity:{sourceCommit:sha,testedCommit:sha,testedTree:tree,runId:10},output:path.join(root,'output'),pnpm:'fixture',
   pack:({plan,output})=>{const row=plan.publication.npm.primary[0];calls.push(row.name);if(row.name==='failed')throw Error('failed pack');fs.mkdirSync(output);fs.writeFileSync(path.join(output,'working.tgz'),'bytes');return {status:'sealed',records:[{name:row.name,version:row.version,file:'working.tgz',sha256:sha256(Buffer.from('bytes'))}]};},
   verify:(plan,receipt)=>{assert.deepEqual(plan.publication.npm.primary.map(row=>row.name),['working']);assert.equal(receipt.records.length,1);return {candidateArchivesVerified:true};}});
  assert.deepEqual(calls,['failed','working']);assert.deepEqual(receipt.buildFailures,{'npm:failed':'package-pack-or-seal-failed'});
  assert.deepEqual(candidate.incremental.publish,['npm:failed','npm:working']);assert.equal(receipt.registryBuildMaterials.length,1);
  assert.equal(receipt.records[0].name,'working');assert.equal(receipt.records.some(row=>row.origin==='registry-present'),false);
 } finally {fs.rmSync(root,{recursive:true,force:true});}
});
test('native-only version absence never schedules source JS prerequisite builds',()=>{
 const value=presenceFixture();value.selection.build=['npm:native'];value.selection.buildNodes=['npm:packages/core'];
 assert.deepEqual(releaseBuildDirectories(value.plan,value.selection,'/unused'),[]);
});
test('workflow schedules exact ABI owner cells, observation transactions and independent failure reporting',()=>{
 const workflow=fs.readFileSync(new URL('../.github/workflows/incremental-release.yaml',import.meta.url),'utf8');
 assert.ok(workflow.includes('plan --version-presence --source'));
 for(const group of ['primary','extra'])assert.ok(workflow.includes('matrix: ${{ fromJSON(needs.plan.outputs.native_'+group+'_matrix) }}'));
 assert.equal((workflow.match(/native_owners: \$\{\{ matrix.native_owners \}\}/g)??[]).length,2);
 const producer=workflow.split('  producer:')[1].split('  candidate-identity:')[0];
 assert.ok(producer.includes("needs.plan.outputs.transaction_required == 'true'"));
 assert.ok(!producer.includes("needs.release-native-primary.result == 'success'"));
 assert.ok(workflow.includes('  release-report:'));
 assert.ok(workflow.includes('artifact-ids: ${{ needs.publisher.outputs.results_id }}'));
 assert.ok(workflow.includes('EXPECTED_PUBLISHER_RESULT_ARTIFACT_DIGEST: ${{ needs.publisher.outputs.results_digest }}'));
 assert.ok(workflow.includes('--publisher-results publication-results/results.json'));
});


test('retained version prerequisites carry metadata identity without fake archive or source byte proof',()=>{
 const value=presenceFixture();value.plan.npm.primary.push({name:'retained',version:'1.0.0',manifestPath:'packages/retained/package.json'});
 value.selection.registryRetained={'npm:retained':{name:'retained',version:'1.0.0',status:'published',evidenceKind:'registry-metadata',integrity:'sha512-'+Buffer.alloc(64).toString('base64')}};
 const candidate=candidatePlanForSelection(value.plan,value.selection);
 assert.deepEqual(candidate.validationDependencies.npm,[{name:'retained',version:'1.0.0',integrity:value.selection.registryRetained['npm:retained'].integrity,evidenceKind:'registry-metadata',origin:'registry-present'}]);
 assert.equal(candidate.validationDependencies.npm[0].sha256,undefined);
 assert.equal(candidate.publication.npm.primary.some(row=>row.name==='retained'),false);
 value.selection.registryRetained['npm:retained'].archiveSha256='a'.repeat(64);
 assert.throws(()=>candidatePlanForSelection(value.plan,value.selection));
});


test('registry host material binds measured execution bytes without a planned archive hash or source-native claim',async()=>{
 const {verifyRegistryHostMaterial}=await import('./incremental-release-producer.mjs');
 const selection=presenceFixture().selection;
 const input={name:'@farmfe/core-linux-x64-gnu',version:'2.0.0',platform:'linux-x64-gnu',origin:'registry-present',evidenceKind:'registry-metadata',integrity:'sha512-'+Buffer.alloc(64).toString('base64'),tarballURL:'https://registry.npmjs.org/host.tgz'};
 selection.registryBuildInputs={'npm:@farmfe/core-linux-x64-gnu':input};
 const binary=Buffer.from('measured execution binary');
 const material={schemaVersion:1,kind:'registry-native-host-build-material',...input,sourceSHA:sha,sourceTree:tree,planHash:selection.planHash,
  evidenceKind:'registry-archive',registryBytesVerified:true,sourceNativeArtifact:false,publicationCandidate:false,
  destination:'packages/core/binding/farm.linux-x64-gnu.node',archiveSha256:'a'.repeat(64),archiveBytes:100,binarySha256:sha256(binary),binaryBytes:binary.length,
  smoke:{node:'v22.20.0',platform:'linux',arch:'x64',libc:'glibc',napi:'10',requiredExports:['Compiler','DocsSnapshot','OutputDirectoryLock','Resolver'],resolver:true,compile:true,tsTransformed:true,outputExecuted:true}};
 assert.equal(input.archiveSha256,undefined);assert.equal(verifyRegistryHostMaterial(material,selection,()=>binary),material);
 for(const change of [{sourceNativeArtifact:true},{publicationCandidate:true},{binarySha256:'0'.repeat(64)},{archiveBytes:0},{sourceSHA:'0'.repeat(40)},{smoke:{...material.smoke,node:'v24.22.1'}}])
  assert.throws(()=>verifyRegistryHostMaterial({...material,...change},selection,()=>binary));
 assert.throws(()=>verifyRegistryHostMaterial(material,selection,()=>Buffer.from('changed')));
});

test('host plugin closure follows execution recipes rather than runtime imports',async()=>{
 const {releaseHostPlugins}=await import('./incremental-release-producer.mjs');
 assert.deepEqual(releaseHostPlugins('packages/core'),['@farmfe/plugin-replace-dirname']);
 assert.deepEqual(releaseHostPlugins('js-plugins/visualizer'),['@farmfe/plugin-replace-dirname','@farmfe/plugin-worker']);
 for(const directory of ['packages/docs','packages/docs-theme-default','packages/runtime','packages/cli'])assert.deepEqual(releaseHostPlugins(directory),[]);
});

test('failed worker host blocks visualizer without preventing an independent JS build',async()=>{
 const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'farm-plugin-isolation-')));
 try {
  const rows=[...['visualizer','less'].map(slug=>({name:'@farmfe/js-plugin-'+slug,version:'2.0.0',manifestPath:`js-plugins/${slug}/package.json`})),{name:'@farmfe/docs',version:'2.0.0',manifestPath:'packages/docs/package.json'}];
  const contents={};
  for(const row of rows){const file=row.manifestPath;contents[file]=JSON.stringify({name:row.name,scripts:{build:'farm build'}});fs.mkdirSync(path.dirname(path.join(root,file)),{recursive:true});fs.writeFileSync(path.join(root,file),contents[file]);}
  const selection={build:rows.map(row=>'npm:'+row.name),buildNodes:rows.map(row=>'npm:'+path.dirname(row.manifestPath)),buildEdges:{'npm:js-plugins/visualizer':[],'npm:js-plugins/less':[],'npm:packages/docs':[]}};
  const calls=[];
  const result=await runReleaseJsBuilds({npm:{primary:rows,payloads:[]}},selection,{root,pnpm:'fixture',bestEffort:true,unavailableHostPlugins:['@farmfe/plugin-worker'],verifyManifest:file=>contents[file],execute:async(_,__,options)=>calls.push(path.basename(options.cwd))});
  assert.deepEqual(calls,['less','docs']);assert.deepEqual(result.completed,['js-plugins/less','packages/docs']);
  assert.equal(result.failures['npm:js-plugins/visualizer'],'js-build-or-prerequisite-failed');
  calls.length=0;
  const noReplace=await runReleaseJsBuilds({npm:{primary:rows,payloads:[]}},selection,{root,pnpm:'fixture',bestEffort:true,unavailableHostPlugins:['@farmfe/plugin-replace-dirname'],verifyManifest:file=>contents[file],execute:async(_,__,options)=>calls.push(path.basename(options.cwd))});
  assert.deepEqual(calls,['docs']);assert.deepEqual(noReplace.completed,['packages/docs']);
 }finally{fs.rmSync(root,{recursive:true,force:true});}
});

test('registry plugin material binds exact tool, ABI smoke and unchanged core binary',async()=>{
 const {verifyRegistryHostMaterial}=await import('./incremental-release-producer.mjs');
 for(const slug of ['replace-dirname','worker']){
  const selection=presenceFixture().selection;
  const input={name:`@farmfe/plugin-${slug}-linux-x64-gnu`,version:'2.0.0',platform:'linux-x64-gnu',origin:'registry-present',evidenceKind:'registry-metadata',integrity:'sha512-'+Buffer.alloc(64).toString('base64'),tarballURL:'https://registry.npmjs.org/host.tgz'};
  selection.registryBuildInputs={['npm:'+input.name]:input,'npm:@farmfe/core-linux-x64-gnu':{name:'@farmfe/core-linux-x64-gnu',platform:'linux-x64-gnu',origin:'registry-present',evidenceKind:'registry-metadata'}};
  const binary=Buffer.from('plugin fixture'),core=Buffer.from('core fixture');
  const material={schemaVersion:1,kind:'registry-native-host-build-material',...input,owner:'@farmfe/plugin-'+slug,sourceSHA:sha,sourceTree:tree,planHash:selection.planHash,
   evidenceKind:'registry-archive',registryBytesVerified:true,sourceNativeArtifact:false,publicationCandidate:false,
   destination:`rust-plugins/${slug}/npm/linux-x64-gnu/index.farm`,archiveSha256:'a'.repeat(64),archiveBytes:100,binarySha256:sha256(binary),binaryBytes:binary.length,coreBinarySha256:sha256(core),
   compiler:{name:'@farmfe/core-linux-x64-gnu',origin:'registry-present',destination:'packages/core/binding/farm.linux-x64-gnu.node',binarySha256:sha256(core),binaryBytes:core.length},
   wrapperResolution:{path:`rust-plugins/${slug}/node_modules/@farmfe/plugin-${slug}-linux-x64-gnu`,destination:`rust-plugins/${slug}/npm/linux-x64-gnu`,workspacePlatformVerified:true},
   smoke:{node:'v22.20.0',platform:'linux',arch:'x64',libc:'glibc',napi:'10',requiredExports:['Compiler','DocsSnapshot','OutputDirectoryLock','Resolver'],resolver:true,compile:true,tsTransformed:true,outputExecuted:true,plugin:input.name,rustPluginLoaded:true,pluginFixture:true}};
  const read=file=>file.endsWith('.node')?core:binary;
  assert.equal(verifyRegistryHostMaterial(material,selection,read),material);
  for(const change of [{coreBinarySha256:'0'.repeat(64)},{owner:'@farmfe/plugin-other'},{destination:'../../escape'},{smoke:{...material.smoke,rustPluginLoaded:false}},{smoke:{...material.smoke,pluginFixture:false}},{smoke:{...material.smoke,plugin:'@farmfe/plugin-other-linux-x64-gnu'}},{wrapperResolution:{...material.wrapperResolution,workspacePlatformVerified:false}},{wrapperResolution:{...material.wrapperResolution,path:'wrong'}},{compiler:{...material.compiler,origin:'source-build'}},{compiler:{...material.compiler,binarySha256:'0'.repeat(64)}}])assert.throws(()=>verifyRegistryHostMaterial({...material,...change},selection,read));
  delete selection.registryBuildInputs['npm:@farmfe/core-linux-x64-gnu'];
  selection.registrySnapshot.packages['npm:@farmfe/core-linux-x64-gnu']={status:'missing'};
  const cell={package:'npm:@farmfe/core-linux-x64-gnu',owner:'@farmfe/core',platform:'linux-x64-gnu'};
  selection.nativeBuildCells=[cell];selection.preparationNativeCells=[cell];
  const sourceMaterial={...material,compiler:{...material.compiler,origin:'source-build'}};
  assert.equal(verifyRegistryHostMaterial(sourceMaterial,selection,read),sourceMaterial);
  selection.preparationNativeCells=[{...cell,platform:'darwin-arm64'}];
  assert.throws(()=>verifyRegistryHostMaterial(sourceMaterial,selection,read));
 }
});
