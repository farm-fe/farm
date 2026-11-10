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
