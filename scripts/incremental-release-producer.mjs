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
import {verifyCandidateSeal} from './verify-v2-candidate-seal.mjs';

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
  const required=new Set(selection.build),prior=selection.registryRetained??selection.baselinePackages??{};
  const dependencies = (rows,kind) => rows.filter(row=>!required.has(kind+':'+row.name)).map(row=>{
    const original=prior[kind+':'+row.name];
    assert.ok(original&&original.version===row.version,'Unchanged validation dependency lacks verified registry evidence');
    assert.match(original.archiveSha256,/^[a-f0-9]{64}$/);
    if(kind==='npm')assert.match(original.integrity??'',/^sha512-[A-Za-z0-9+/]+={0,2}$/);
    return {name:row.name,version:row.version,sha256:original.archiveSha256,...(kind==='npm'?{integrity:original.integrity}:{})};
  });
  return {...family,preparation:{...family.preparation,mode:selection.publicationScope==='npm'?'incremental-npm-source-release':'incremental-source-release'},publication:{
    npm:{primary:family.npm.primary.filter(row=>required.has('npm:'+row.name)),payloads:family.npm.payloads.filter(row=>required.has('npm:'+row.name))},
    cargo:selection.publicationScope==='npm'?[]:family.cargo.filter(row=>required.has('cargo:'+row.name))},
    // These immutable public dependencies are for isolated consumer validation
    // only. No producer may restore them into a source build or seal them as a
    // newly built candidate. Cargo dependency sources are compiled by consumers.
    validationDependencies:{npm:dependencies([...family.npm.primary,...family.npm.payloads],'npm'),cargo:selection.publicationScope==='npm'?[]:dependencies(family.cargo,'cargo')},
    incremental:{sourceSHA:selection.sourceSHA,sourceTree:selection.sourceTree,planHash:selection.planHash,graphHash:selection.graphHash,
      publicationScope:selection.publicationScope??'all',build:[...selection.build],publish:[...selection.publish],preparationNodes:selection.buildNodes??[],
      preparationNativeOwners:selection.preparationNativeOwners??[],baselineBinding:selection.baselineBinding??null}};
}

export function releaseBuildDirectories(family, selection, root) {
  const wanted = new Set(selection.build);
  const publicDirectories = new Map(family.npm.primary.map(row => [path.dirname(row.manifestPath).replaceAll('\\', '/'), row]));
  const selected = candidateJsBuilds(family, {root}).filter(directory => publicDirectories.has(directory) && (wanted.has('npm:' + publicDirectories.get(directory).name) || selection.buildNodes?.includes('npm:' + directory)));
  const privateDirectories = new Set();
  for (const node of selection.buildNodes ?? []) {
    if (!node.startsWith('npm:')) continue;
    const directory = node.slice(4);
    assert.match(directory, /^(?:packages|js-plugins|rust-plugins|examples)\/[a-zA-Z0-9_.-]+(?:\/npm\/[a-zA-Z0-9_.-]+)?$/, 'Invalid release build node');
    if (publicDirectories.has(directory) || directory.startsWith('rust-plugins/') || directory.includes('/npm/')) continue;
    const manifest = JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json')));
    assert.equal(manifest.private, true, 'Non-public prerequisite must be explicitly private');
    assert.ok(manifest.scripts?.build, 'Private prerequisite has no reviewed build command');
    privateDirectories.add(directory);
  }
  const directories = [...selected, ...privateDirectories];
  const manifests = new Map(directories.map(directory => [directory, JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json')))]));
  const byName = new Map(directories.map(directory => [manifests.get(directory).name, directory]));
  assert.equal(byName.size, directories.length, 'Ambiguous build prerequisite name');
  // These existing joint bootstrap recipes resolve their own project-reference
  // or peer cycles. External prerequisites of ANY cohort member run before the
  // entire cohort; all other resolved public/private edges are ordered normally.
  const bootstrap = new Set(['packages/cli', 'packages/core', 'packages/runtime', 'packages/runtime-plugin-hmr', 'packages/runtime-plugin-import-meta', 'packages/plugin-tools', 'packages/utils']);
  const docs = new Set(['packages/docs', 'packages/docs-theme-default']);
  const groupFor = directory => bootstrap.has(directory) ? 'bootstrap' : docs.has(directory) ? 'docs' : directory;
  const groups = new Map();
  for (const directory of directories) {
    const group = groupFor(directory); groups.set(group, [...(groups.get(group) ?? []), directory]);
  }
  const active = new Set(), done = new Set(), ordered = [];
  function visit(group) {
    if (done.has(group)) return;
    assert.ok(!active.has(group), 'Cyclic release build prerequisite'); active.add(group);
    for (const directory of groups.get(group)) if (selection.buildNodes) {
      const node = 'npm:' + directory;
      assert.ok(selection.buildEdges && Array.isArray(selection.buildEdges[node]), 'Resolved release build edges are missing');
      for (const dependencyNode of selection.buildEdges[node]) {
        assert.ok(selection.buildNodes.includes(dependencyNode), 'Release dependency is outside the bound build closure');
        const dependency = dependencyNode.startsWith('npm:') ? dependencyNode.slice(4) : null;
        if (dependency && manifests.has(dependency) && groupFor(dependency) !== group) visit(groupFor(dependency));
      }
    }
    active.delete(group); done.add(group); ordered.push(...groups.get(group));
  }
  for (const group of groups.keys()) visit(group);
  return ordered;
}

export async function runReleaseJsBuilds(family, selection, {root, pnpm, env = process.env, execute = command, verifyManifest} = {}) {
  const directories = releaseBuildDirectories(family, selection, root), deadline = Date.now() + 30 * 60 * 1000;
  for (const directory of directories) {
    const file = directory + '/package.json', bytes = fs.readFileSync(path.join(root, file));
    const committed = verifyManifest ? verifyManifest(file) : execFileSync('git', ['show', 'HEAD:' + file], {cwd: root});
    assert.ok(bytes.equals(Buffer.from(committed)), 'Release build manifest differs from committed source');
    const manifest = JSON.parse(bytes);
    assert.ok(manifest.scripts?.build && !/\bcargo\b|\bnapi\b|farm-plugin-tools build/.test(manifest.scripts.build), 'Unexpected native command in JS release recipe');
    assert.ok(Date.now() < deadline, 'Release JS build deadline exceeded');
    await execute(process.execPath, [pnpm, 'run', 'build'], {cwd: path.join(root, directory), env,
      log: path.join(root, 'release-work', directory.replaceAll('/', '-') + '.log'), phase: 'release-js/' + manifest.name,
      timeoutMs: Math.min(300000, deadline - Date.now())});
  }
  return directories;
}

export function verifyReleaseSelection(selection, source, expectedPlanHash) {
  assert.match(expectedPlanHash ?? '', /^[a-f0-9]{64}$/, 'Trusted release plan hash is required');
  const {planHash, ...unsigned} = selection;
  assert.equal(publicationManifestDigest(unsigned), planHash, 'Release selection digest differs');
  assert.equal(planHash, expectedPlanHash, 'Release selection differs from trusted job output');
  assert.equal(selection.chain, 'release'); assert.equal(selection.profile, 'release');
  assert.equal(selection.sourceSHA, source.sha); assert.equal(selection.sourceTree, source.tree);
  assert.equal(selection.publicationBlocked, false); assert.deepEqual(selection.versionRequired, []);
  return selection;
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

export function makePublicationManifest({plan, selection, receipt, source, runId, attempt, readArchive, readPackage=()=>({}), verificationOnly=false}) {
  assert.equal(selection.chain, 'release'); assert.equal(selection.profile, 'release');
  assert.equal(selection.sourceSHA, source.sha); assert.equal(selection.sourceTree, source.tree);
  assert.equal(receipt.status, 'complete-seal');
  assert.equal(receipt.testedCommit, source.sha); assert.equal(receipt.testedTree, source.tree);
  assert.notEqual(selection.publicationBlocked, true, 'Version or retirement requirements block publication');
  assert.equal((selection.versionRequired ?? []).length, 0, 'Required version changes are not prepared');
  assert.equal((selection.removedPackages ?? []).length, 0, 'Retirement needs an explicit reviewed contract');
  const fullFamily = [...(selection.publicationScope==='npm'?[]:plan.cargo ?? []).map(row => ({...row, kind:'cargo'})),
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
  if (verificationOnly) {
    assert.deepEqual(selection.publish, [], 'Rebuild verification cannot publish a delta');
    assert.ok(selection.baselineBinding && selection.build?.length, 'Rebuild verification needs original release bytes and a nonempty build');
    return {schemaVersion: 1, kind: 'release-rebuild-verification', chain: 'release', profile: 'release',
      sourceSHA: source.sha, sourceTree: source.tree, planHash: selection.planHash, graphHash: selection.graphHash, baseline: selection.baselineBinding,
      rebuilt: [...selection.build], producerRunId: runId, producerAttempt: attempt, originalBytesEqual: true,
      publicationExecuted: false, baselineAdvanced: false};
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
  return {schemaVersion: 1, chain: 'release', profile: 'release', namespace: 'farm-release-v1', publicationScope:selection.publicationScope??'all',
    sourceSHA: source.sha, sourceTree: source.tree, planHash: selection.planHash, graphHash: selection.graphHash, baseline: selection.baselineBinding ?? null,
    producerRunId: runId, producerAttempt: attempt, abi: plan.abi,
    tag: packages.some(row => row.kind === 'npm' && row.name === '@farmfe/core')
      ? `v${packages.find(row => row.kind === 'npm' && row.name === '@farmfe/core').version}` : `packages-${selection.planHash.slice(0,16)}`,
    family:fullFamily, rebuild:[...(selection.build ?? selection.publish)], versionRequired:[...(selection.versionRequired ?? [])],
    publish:[...selection.publish], buildDependencyClosure:[...(selection.buildDependencyClosure ?? [])],
    validationRequired:[...(selection.validationRequired ?? [])], ...(selection.registrySnapshot ? {registrySnapshot:selection.registrySnapshot} : {}), packages};
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
    const selection = verifyReleaseSelection(JSON.parse(fs.readFileSync(selectionFile)), source, process.env.FARM_RELEASE_PLAN_HASH);
    const candidate=candidatePlanForSelection(plan,selection);fs.mkdirSync(path.dirname(path.resolve(candidateFile)),{recursive:true});
    fs.writeFileSync(candidateFile,JSON.stringify(candidate,null,2)+'\n',{flag:'wx'});
    fs.mkdirSync(pool,{recursive:true});
    const selectedNative=(candidate.publication?.npm??candidate.npm).payloads;
    const preparationNative=plan.npm.payloads.filter(row=>(selection.preparationNativeOwners??[]).includes(row.owner));
    const nativePlan={...plan,npm:{...plan.npm,payloads:[...new Map([...selectedNative,...preparationNative].map(row=>[row.name,row])).values()]}};
    const records=assembleNativeArtifacts(root,path.resolve(pool),source.sha,nativePlan);
    fs.writeFileSync(assemblyFile,JSON.stringify({schemaVersion:1,sourceCommit:source.sha,count:selectedNative.length,records:records.filter(row=>selectedNative.some(pkg=>pkg.name===row.name)),preparationRecords:records.filter(row=>!selectedNative.some(pkg=>pkg.name===row.name)),published:false},null,2)+'\n',{flag:'wx'});
    const core=records.find(row=>row.owner==='@farmfe/core'&&row.platform==='linux-x64-gnu');
    if(core)fs.copyFileSync(path.join(root,core.file),path.join(root,'packages/core/binding',path.basename(core.file)),fs.constants.COPYFILE_EXCL);
    assert.equal(process.platform,'linux');assert.equal(process.arch,'x64');
    if(releaseBuildDirectories(plan,selection,root).length) {
      assert.ok(core,'Source-built GNU Core prerequisite missing');
      const native=createRequire(import.meta.url)(path.join(root,'packages/core/binding/farm.linux-x64-gnu.node'));
      assert.ok(native&&Object.keys(native).length>0,'Independent release prerequisite native load failed');
    }
    const rebuiltJs = await runReleaseJsBuilds(plan, selection, {root, pnpm: resolvePnpmCli()});
    return {rebuiltNative: records.length, rebuiltJs, profile: 'release'};
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
  if(stage==='seal-npm') {
    const [npmDirectory,output,candidateFile]=args;assert.ok(npmDirectory&&output&&candidateFile);
    const candidate=JSON.parse(fs.readFileSync(candidateFile));
    assert.equal(candidate.preparation.mode,'incremental-npm-source-release');
    assert.equal(candidate.incremental.publicationScope,'npm');
    assert.deepEqual(candidate.publication.cargo,[]);
    const receipt=JSON.parse(fs.readFileSync(path.join(npmDirectory,'receipt.json')));
    assert.equal(receipt.status,'sealed');assert.equal(receipt.testedCommit,source.sha);assert.equal(receipt.testedTree,source.tree);
    assert.equal(receipt.planSha256,sha256(Buffer.from(JSON.stringify(candidate))));
    assert.equal(verifyCandidateSeal(candidate,receipt,npmDirectory,root).candidateArchivesVerified,true);
    assert.ok(!fs.existsSync(path.join(root,'.changeset/pre.json')),'Npm version preparation is incomplete');
    assert.equal(fs.readdirSync(path.join(root,'.changeset')).filter(file=>file.endsWith('.md')&&file!=='README.md').length,0);
    fs.mkdirSync(output);
    for(const row of receipt.records) {
      assert.equal(path.basename(row.file),row.file);
      fs.copyFileSync(path.join(npmDirectory,row.file),path.join(output,row.file),fs.constants.COPYFILE_EXCL);
    }
    fs.writeFileSync(path.join(output,'candidate-plan.json'),JSON.stringify(candidate,null,2)+'\n',{flag:'wx'});
    fs.writeFileSync(path.join(output,'receipt.json'),JSON.stringify({...receipt,rust:[],status:'complete-seal',publicationScope:'npm'},null,2)+'\n',{flag:'wx'});
    return {npm:receipt.records.length,publicationScope:'npm',publicationExecuted:false};
  }
  if (stage === 'manifest' || stage === 'verify-rebuild') {
    const [directory, selectionFile] = args; assert.ok(directory && selectionFile);
    const candidate = path.resolve(directory), receipt = JSON.parse(fs.readFileSync(path.join(candidate,'receipt.json')));
    const selection = verifyReleaseSelection(JSON.parse(fs.readFileSync(selectionFile)), source, process.env.FARM_RELEASE_PLAN_HASH);
    const manifest = makePublicationManifest({verificationOnly: stage === 'verify-rebuild', plan, selection, receipt, source, runId: identity.runId,
      attempt: Number(process.env.GITHUB_RUN_ATTEMPT), readArchive: file => fs.readFileSync(path.join(candidate,file)),
      readPackage: row=>{
        const pkg=[...plan.npm.primary,...plan.npm.payloads].find(pkg=>pkg.name===row.name);assert.ok(pkg,'Unknown publication manifest');
        return JSON.parse(fs.readFileSync(path.join(root,pkg.manifestPath)));
      }});
    if (stage === 'verify-rebuild') {
      fs.writeFileSync(path.join(candidate, 'rebuild-verification.json'), JSON.stringify(manifest, null, 2) + '\n', {flag: 'wx'});
      return manifest;
    }
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
