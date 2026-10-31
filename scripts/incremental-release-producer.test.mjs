import assert from 'node:assert/strict';
import {test} from 'node:test';
import {makePublicationManifest,releaseProducerIdentity,candidatePlanForSelection,releaseBuildDirectories,publicationManifestDigest} from './incremental-release-producer.mjs';
import {sha256} from './release-archive.mjs';
import fs from 'node:fs';
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

test('JS-only delta restores release prerequisites and packages no native or SDK rebuild',()=>{
 const root=fileURLToPath(new URL('../',import.meta.url));
 const primary=['packages/core','js-plugins/dts','js-plugins/babel'].map(directory=>{
  const pkg=JSON.parse(fs.readFileSync(path.join(root,directory,'package.json')));return {name:pkg.name,version:pkg.version,manifestPath:directory+'/package.json'};
 });
 const family={abi:'0.5.4',cargo:[{name:'farmfe_core',version:'2.2.0',manifestPath:'crates/core/Cargo.toml'}],npm:{primary,payloads:[]}};
 const selection={chain:'release',profile:'release',sourceSHA:sha,sourceTree:tree,planHash:'d'.repeat(64),baselineBinding:'e'.repeat(64),
  build:['npm:@farmfe/js-plugin-babel'],publish:['npm:@farmfe/js-plugin-babel'],versionRequired:[],baselinePackages:{}};
 for(const row of primary)selection.baselinePackages['npm:'+row.name]={version:row.version,archiveSha256:'a'.repeat(64),integrity:'sha512-'+Buffer.alloc(64).toString('base64')};
 selection.baselinePackages['cargo:farmfe_core']={version:'2.2.0',archiveSha256:'b'.repeat(64)};
 const candidate=candidatePlanForSelection(family,selection);
 assert.deepEqual(candidate.publication.npm.primary.map(row=>row.name),['@farmfe/js-plugin-babel']);
 assert.deepEqual(candidate.publication.npm.payloads,[]);
 assert.deepEqual(candidate.publication.reuseNpm.map(row=>row.name),['@farmfe/core','@farmfe/js-plugin-dts']);
 assert.deepEqual(releaseBuildDirectories(family,selection,root),['js-plugins/babel']);
 assert.deepEqual(rustRegistryReuse(candidate),[{name:'farmfe_core',version:'2.2.0',sha256:'b'.repeat(64)}]);
 delete selection.baselinePackages['npm:@farmfe/core'];assert.throws(()=>candidatePlanForSelection(family,selection));
});

test('portable candidate digest matches Python canonical manifest encoding including Unicode',()=>{
 assert.equal(publicationManifestDigest({z:'中文 😀',a:[1,null,true]}),'cf887cc07ffa09c406c81cfc551c3e45d27f111fbf431032bc82846b8f020428');
 assert.throws(()=>publicationManifestDigest({bad:1.5}));
 assert.equal(sdkConsumerManifest({cargo:[{name:'farmfe_core',version:'2.2.1'}]},'[dependencies]\nfarmfe_core = "=2.2.0"\n'), '[dependencies]\nfarmfe_core = "=2.2.1"\n');
 assert.throws(()=>sdkConsumerManifest({cargo:[]},'[patch.crates-io]\n'));
});
