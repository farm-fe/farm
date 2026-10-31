// cspell:words npmjs
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {assembleNativeArtifacts} from './assemble-v2-native-artifacts.mjs';
import {candidateJsBuilds} from './prepare-v2-candidate-js.mjs';
import {command} from './performance-report.mjs';
import {packNpmCandidates} from './pack-v2-npm-candidates.mjs';
import {resolvePnpmCli} from './v2-pnpm-cli.mjs';
import {sha256} from './release-archive.mjs';

export function publicationManifestDigest(value) {
  const ordered = input => {
    if(Array.isArray(input))return input.map(ordered);
    if(input !== null && typeof input === 'object')return Object.fromEntries(Object.keys(input).sort().map(key=>[key,ordered(input[key])]));
    assert.ok(input===null||['string','boolean'].includes(typeof input)||Number.isSafeInteger(input),'Manifest has a non-canonical value');
    return input;
  };
  return sha256(Buffer.from(JSON.stringify(ordered(value)).replace(/[\u007f-\uffff]/g,character=>'\\u'+character.charCodeAt(0).toString(16).padStart(4,'0'))));
}

export function candidatePlanForSelection(family, selection) {
  assert.equal(selection.chain,'release');assert.equal(selection.profile,'release');assert.notEqual(selection.publicationBlocked,true);
  assert.equal((selection.versionRequired??[]).length,0);
  if(!selection.baselineBinding)return family;
  const required=new Set(selection.build),prior=selection.baselinePackages??{};
  const reuseNpm=[...family.npm.primary,...family.npm.payloads].filter(row=>!required.has('npm:'+row.name)).map(row=>{
    const original=prior['npm:'+row.name];assert.ok(original&&original.version===row.version,'Unchanged npm package lacks original release bytes');
    assert.match(original.integrity??'',/^sha512-[A-Za-z0-9+/]+={0,2}$/);assert.match(original.archiveSha256,/^[a-f0-9]{64}$/);
    return {name:row.name,version:row.version,integrity:original.integrity,sha256:original.archiveSha256};
  });
  const reuseRust=family.cargo.filter(row=>!required.has('cargo:'+row.name)).map(row=>{
    const original=prior['cargo:'+row.name];assert.ok(original&&original.version===row.version,'Unchanged Rust package lacks original release bytes');
    assert.match(original.archiveSha256,/^[a-f0-9]{64}$/);return {name:row.name,version:row.version,sha256:original.archiveSha256};
  });
  return {...family,preparation:{...family.preparation,mode:'incremental-release'},publication:{
    npm:{primary:family.npm.primary.filter(row=>required.has('npm:'+row.name)),payloads:family.npm.payloads.filter(row=>required.has('npm:'+row.name))},
    reuseNpm,reuseRust},incremental:{sourceSHA:selection.sourceSHA,sourceTree:selection.sourceTree,planHash:selection.planHash,baselineBinding:selection.baselineBinding}};
}

export function releaseBuildDirectories(family, selection, root) {
  const wanted=new Set(selection.build);
  return candidateJsBuilds(family,{root}).filter(directory=>family.npm.primary.some(row=>path.dirname(row.manifestPath).replaceAll('\\','/')===directory&&wanted.has('npm:'+row.name)));
}

export function releaseProducerIdentity(env, checkout) {
  assert.equal(env.GITHUB_ACTIONS, 'true');
  assert.equal(env.GITHUB_REF, 'refs/heads/main');
  assert.ok(['push', 'workflow_dispatch'].includes(env.GITHUB_EVENT_NAME));
  assert.equal(env.GITHUB_WORKFLOW_REF, `${env.GITHUB_REPOSITORY}/.github/workflows/incremental-release.yaml@refs/heads/main`);
  assert.match(env.GITHUB_SHA, /^[a-f0-9]{40}$/);
  assert.equal(checkout.sha, env.GITHUB_SHA);
  assert.match(checkout.tree, /^[a-f0-9]{40}$/);
  assert.equal(env.FARM_BUILD_PROFILE, 'release');
  assert.ok(Number.isSafeInteger(Number(env.GITHUB_RUN_ID)) && Number(env.GITHUB_RUN_ID) > 0);
  return {sourceCommit: checkout.sha, testedCommit: checkout.sha, testedTree: checkout.tree, runId: Number(env.GITHUB_RUN_ID)};
}

export function makePublicationManifest({plan, selection, receipt, source, runId, attempt, readArchive, readPackage=()=>({})}) {
  assert.equal(selection.chain, 'release'); assert.equal(selection.profile, 'release');
  assert.equal(selection.sourceSHA, source.sha); assert.equal(selection.sourceTree, source.tree);
  assert.equal(receipt.status, 'complete-seal');
  assert.equal(receipt.testedCommit, source.sha); assert.equal(receipt.testedTree, source.tree);
  assert.notEqual(selection.publicationBlocked, true, 'Version or retirement requirements block publication');
  assert.equal((selection.versionRequired ?? []).length, 0, 'Required version changes are not prepared');
  assert.equal((selection.removedPackages ?? []).length, 0, 'Retirement needs an explicit reviewed contract');
  const fullFamily = [...(plan.cargo ?? []).map(row => ({...row, kind:'cargo'})),
    ...(plan.npm?.primary ?? []).map(row => ({...row,kind:'npm'})), ...(plan.npm?.payloads ?? []).map(row => ({...row,kind:'npm'}))]
    .map(({kind,name,version,manifestPath,owner,platform})=>({kind,name,version,manifestPath,...(owner?{owner}:{}),...(platform?{platform}:{})}))
    .sort((a,b)=>(a.kind+':'+a.name).localeCompare(b.kind+':'+b.name));
  assert.ok(fullFamily.length > 0 && new Set(fullFamily.map(row=>row.kind+':'+row.name)).size===fullFamily.length, 'Full source family is missing or duplicated');
  if(selection.fullFamily) {
    const actual = selection.fullFamily.map(({kind,name,version,manifestPath,owner,platform})=>({kind,name,version,manifestPath,...(owner?{owner}:{}),...(platform?{platform}:{})}))
      .sort((a,b)=>(a.kind+':'+a.name).localeCompare(b.kind+':'+b.name));
    assert.deepEqual(actual,fullFamily,'Source family differs from independent plan');
  }
  const completeRows = [...receipt.rust.map(row => ({...row, kind: 'cargo'})), ...receipt.records.map(row => ({...row, kind: 'npm'}))];
  for(const row of completeRows) assert.ok(fullFamily.some(item=>item.kind===row.kind&&item.name===row.name&&item.version===row.version),'Sealed record is outside the exact source family');
  for(const key of selection.build ?? []) {
    if(selection.publish.includes(key))continue;
    const original=selection.baselinePackages?.[key],rebuilt=completeRows.filter(row=>row.kind+':'+row.name===key);
    assert.ok(original&&rebuilt.length===1,'Same-version rebuild requires its independently verified original archive');
    const row=rebuilt[0];assert.equal(row.version,original.version);
    assert.equal(path.basename(row.file),row.file);
    const actual=readArchive(row.file);
    assert.equal(sha256(actual),row.sha256,'Rebuilt archive bytes differ from their seal');
    assert.equal(sha256(actual),original.archiveSha256,'Same-version rebuild changed bytes; prepare a new version before publication');
  }
  assert.ok(selection.publish.length>0,'No publication delta; no publisher transaction is needed');
  const rows = completeRows
    .filter(row=>selection.publish.includes(row.kind+':'+row.name));
  const byName=new Map(rows.filter(row=>row.kind==='npm').map(row=>[row.name,row]));
  const ordered=rows.filter(row=>row.kind==='cargo'),active=new Set(),done=new Set();
  function visit(name){
    if(done.has(name))return;
    assert.ok(!active.has(name),'Cyclic npm publication dependencies');active.add(name);
    const row=byName.get(name),manifest=readPackage(row);
    for(const dependency of [...new Set(['dependencies','peerDependencies','optionalDependencies'].flatMap(field=>Object.keys(manifest[field]??{})))].sort())
      if(byName.has(dependency))visit(dependency);
    active.delete(name);done.add(name);ordered.push(row);
  }
  for(const name of [...byName.keys()].sort())visit(name);
  const packages = ordered.map(row => {
    assert.equal(path.basename(row.file), row.file);
    const bytes = readArchive(row.file); assert.equal(sha256(bytes), row.sha256);
    return {kind: row.kind, name: row.name, version: row.version, archive: row.file,
      archiveBytes: bytes.length, archiveSha256: row.sha256,
      ...(row.kind === 'cargo' ? {metadata: row.metadata, metadataSha256: row.metadataSha256} : {})};
  });
  assert.deepEqual(packages.map(row => `${row.kind}:${row.name}`).sort(), [...selection.publish].sort(), 'Sealed publication delta differs from independent plan');
  assert.equal(new Set(packages.map(row => `${row.kind}:${row.name}`)).size, packages.length);
  return {schemaVersion: 1, chain: 'release', profile: 'release', namespace: 'farm-release-v1',
    sourceSHA: source.sha, sourceTree: source.tree, planHash: selection.planHash, baseline: selection.baselineBinding ?? null,
    producerRunId: runId, producerAttempt: attempt, abi: plan.abi,
    tag: packages.some(row => row.kind === 'npm' && row.name === '@farmfe/core')
      ? `v${packages.find(row => row.kind === 'npm' && row.name === '@farmfe/core').version}` : `packages-${selection.planHash.slice(0,16)}`,
    family:fullFamily, rebuild:[...(selection.build ?? selection.publish)], versionRequired:[...(selection.versionRequired ?? [])],
    publish:[...selection.publish], buildDependencyClosure:[...(selection.buildDependencyClosure ?? [])],
    validationRequired:[...(selection.validationRequired ?? [])], packages};
}

async function main() {
  const root = fileURLToPath(new URL('../', import.meta.url));
  const git = (...args) => execFileSync('git', args, {cwd: root, encoding: 'utf8'}).trim();
  const source = {sha: git('rev-parse','HEAD'), tree: git('rev-parse','HEAD^{tree}')};
  const identity = releaseProducerIdentity(process.env, source);
  const [stage, ...args] = process.argv.slice(2);
  const plan = JSON.parse(fs.readFileSync(path.join(root,'docs/releases/v2-candidate.json')));
  if(stage==='prepare') {
    const [selectionFile,pool,assemblyFile,candidateFile]=args;assert.ok(selectionFile&&pool&&assemblyFile&&candidateFile);
    assert.equal(path.resolve(candidateFile),path.join(root,'release-work/candidate-plan.json'));
    const selection=JSON.parse(fs.readFileSync(selectionFile));assert.equal(selection.sourceSHA,source.sha);assert.equal(selection.sourceTree,source.tree);
    const {planHash,...unsigned}=selection;assert.equal(publicationManifestDigest(unsigned),planHash,'Release selection digest differs');
    const candidate=candidatePlanForSelection(plan,selection);fs.mkdirSync(path.dirname(path.resolve(candidateFile)),{recursive:true});
    fs.writeFileSync(candidateFile,JSON.stringify(candidate,null,2)+'\n',{flag:'wx'});
    if(selection.baselineBinding) execFileSync(process.env.FARM_RELEASE_PYTHON??'python',['-B','scripts/incremental-release-materialize.py','--selection',selectionFile,'--output','release-work/prerequisites.json'],{cwd:root,stdio:'inherit',timeout:20*60*1000});
    fs.mkdirSync(pool,{recursive:true});
    const nativePlan={...plan,npm:{...plan.npm,payloads:(candidate.publication?.npm??candidate.npm).payloads}};
    const records=assembleNativeArtifacts(root,path.resolve(pool),source.sha,nativePlan);
    fs.writeFileSync(assemblyFile,JSON.stringify({schemaVersion:1,sourceCommit:source.sha,count:records.length,records,published:false},null,2)+'\n',{flag:'wx'});
    const core=records.find(row=>row.owner==='@farmfe/core'&&row.platform==='linux-x64-gnu');
    if(core)fs.copyFileSync(path.join(root,core.file),path.join(root,'packages/core/binding',path.basename(core.file)),fs.constants.COPYFILE_EXCL);
    assert.equal(process.platform,'linux');assert.equal(process.arch,'x64');
    const native=createRequire(import.meta.url)(path.join(root,'packages/core/binding/farm.linux-x64-gnu.node'));
    assert.ok(native&&Object.keys(native).length>0,'Independent release prerequisite native load failed');
    const pnpm=resolvePnpmCli(),deadline=Date.now()+30*60*1000;
    for(const directory of releaseBuildDirectories(plan,selection,root)) {
      const manifest=JSON.parse(fs.readFileSync(path.join(root,directory,'package.json')));
      assert.ok(manifest.scripts?.build&&!/\bcargo\b|\bnapi\b|farm-plugin-tools build/.test(manifest.scripts.build),'Unexpected native command in JS release recipe');
      assert.ok(Date.now()<deadline,'Release JS build deadline exceeded');
      await command(process.execPath,[pnpm,'run','build'],{cwd:path.join(root,directory),env:process.env,
        log:path.join(root,'release-work',directory.replaceAll('/','-')+'.log'),phase:'release-js/'+manifest.name,timeoutMs:Math.min(300000,deadline-Date.now())});
    }
    return {rebuiltNative:records.length,rebuiltJs:releaseBuildDirectories(plan,selection,root),profile:'release'};
  }
  if (stage === 'pack') {
    const [assembly, index, output,candidateFile] = args; assert.ok(assembly && index && output);
    const candidate=candidateFile?JSON.parse(fs.readFileSync(candidateFile)):plan;
    assert.deepEqual(candidate.npm,plan.npm);assert.deepEqual(candidate.cargo,plan.cargo);assert.equal(candidate.abi,plan.abi);
    const pages = JSON.parse(fs.readFileSync(index));
    const artifacts = Array.isArray(pages) ? pages.flatMap(page => page.artifacts) : pages.artifacts;
    assert.ok(artifacts.every(row => row.workflow_run?.id === identity.runId), 'Cross-run artifacts cannot enter a release build');
    return packNpmCandidates({root, plan:candidate, assembly: JSON.parse(fs.readFileSync(assembly)), artifactIndex: artifacts,
      ...identity, output: path.resolve(output), pnpm: resolvePnpmCli()});
  }
  if (stage === 'manifest') {
    const [directory, selectionFile] = args; assert.ok(directory && selectionFile);
    const candidate = path.resolve(directory), receipt = JSON.parse(fs.readFileSync(path.join(candidate,'receipt.json')));
    const selection = JSON.parse(fs.readFileSync(selectionFile));
    const manifest = makePublicationManifest({plan, selection, receipt, source, runId: identity.runId,
      attempt: Number(process.env.GITHUB_RUN_ATTEMPT), readArchive: file => fs.readFileSync(path.join(candidate,file)),
      readPackage: row=>{
        const pkg=[...plan.npm.primary,...plan.npm.payloads].find(pkg=>pkg.name===row.name);assert.ok(pkg,'Unknown publication manifest');
        return JSON.parse(fs.readFileSync(path.join(root,pkg.manifestPath)));
      }});
    if(process.env.FARM_FUNCTIONAL_EVIDENCE) manifest.functionalEvidence=JSON.parse(process.env.FARM_FUNCTIONAL_EVIDENCE);
    fs.writeFileSync(path.join(candidate,'manifest.json'), JSON.stringify(manifest,null,2)+'\n',{flag:'wx'});
    const archiveDirectory = path.join(candidate,'archives'); fs.mkdirSync(archiveDirectory);
    for (const row of manifest.packages) {
      fs.copyFileSync(path.join(candidate,row.archive),path.join(archiveDirectory,row.archive),fs.constants.COPYFILE_EXCL);
      if (row.metadata) fs.copyFileSync(path.join(candidate,row.metadata),path.join(archiveDirectory,row.metadata),fs.constants.COPYFILE_EXCL);
    }
    return {count:manifest.packages.length,publicationExecuted:false};
  }
  throw Error('Unknown release producer stage');
}
if(process.argv[1] && path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) console.log(JSON.stringify(await main()));
