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

export function versionPresenceMode(selection) {
  return selection.selectionPolicy === 'registry-exact-version-v2' && selection.registrySnapshot?.schemaVersion === 2;
}

export function candidatePlanForSelection(family, selection) {
  assert.equal(selection.chain,'release');assert.equal(selection.profile,'release');assert.notEqual(selection.publicationBlocked,true);
  assert.equal((selection.versionRequired??[]).length,0);
  const required=new Set(selection.build),prior=selection.registryRetained??selection.baselinePackages??{};
  const presence = versionPresenceMode(selection);
  const dependencies = (rows,kind) => rows.filter(row=>!required.has(kind+':'+row.name) && (!presence || prior[kind+':'+row.name])).map(row=>{
    const original=prior[kind+':'+row.name];
    assert.ok(original&&original.version===row.version,'Unchanged validation dependency lacks verified registry evidence');
    if(kind==='npm')assert.match(original.integrity??'',/^sha512-[A-Za-z0-9+/]+={0,2}$/);
    if (presence) {
      assert.equal(kind,'npm');assert.equal(original.evidenceKind,'registry-metadata');
      assert.equal(original.archiveSha256,undefined);assert.equal(original.archiveBytes,undefined);
      return {name:row.name,version:row.version,integrity:original.integrity,evidenceKind:'registry-metadata',origin:'registry-present'};
    }
    assert.match(original.archiveSha256,/^[a-f0-9]{64}$/);
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
      preparationNativeOwners:selection.preparationNativeOwners??[],baselineBinding:selection.baselineBinding??null,
      ...(presence ? {selectionPolicy:selection.selectionPolicy,registrySnapshot:selection.registrySnapshot,
        nativeBuildCells:selection.nativeBuildCells??[],preparationNativeCells:selection.preparationNativeCells??[],
        registryBuildInputs:selection.registryBuildInputs??{},buildFailures:Object.fromEntries(Object.keys(selection.buildBlocked??{}).map(key=>[key,'source-input-unknown'])),registryBuildMaterials:[]} : {})}};
}

export function releaseBuildDirectories(family, selection, root) {
  const wanted = new Set(selection.build);
  if (versionPresenceMode(selection) && !family.npm.primary.some(row=>wanted.has('npm:'+row.name) && row.name!=='create-farm' && !row.manifestPath.startsWith('rust-plugins/'))) return [];
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

export async function runReleaseJsBuilds(family, selection, {root, pnpm, env = process.env, execute = command, verifyManifest, bestEffort = false, hostAvailable = true} = {}) {
  const directories = releaseBuildDirectories(family, selection, root), deadline = Date.now() + 30 * 60 * 1000;
  const failures = {}, completed = [];
  const dependsOnFailure = (node, seen = new Set()) => {
    if (failures[node]) return true;
    if (seen.has(node)) return false;
    seen.add(node);
    return (selection.buildEdges?.[node] ?? []).some(dependency=>dependsOnFailure(dependency,seen));
  };
  for (const directory of directories) {
    const node = 'npm:' + directory;
    try {
      assert.ok(hostAvailable, 'Validated GNU Core prerequisite missing');
      assert.ok(!dependsOnFailure(node), 'Release build prerequisite failed');
      const file = directory + '/package.json', bytes = fs.readFileSync(path.join(root, file));
      const committed = verifyManifest ? verifyManifest(file) : execFileSync('git', ['show', 'HEAD:' + file], {cwd: root});
      assert.ok(bytes.equals(Buffer.from(committed)), 'Release build manifest differs from committed source');
      const manifest = JSON.parse(bytes);
      assert.ok(manifest.scripts?.build && !/\bcargo\b|\bnapi\b|farm-plugin-tools build/.test(manifest.scripts.build), 'Unexpected native command in JS release recipe');
      assert.ok(Date.now() < deadline, 'Release JS build deadline exceeded');
      await execute(process.execPath, [pnpm, 'run', 'build'], {cwd: path.join(root, directory), env,
        log: path.join(root, 'release-work', directory.replaceAll('/', '-') + '.log'), phase: 'release-js/' + manifest.name,
        timeoutMs: Math.min(300000, deadline - Date.now())});
      completed.push(directory);
    } catch (error) {
      if (!bestEffort) throw error;
      // Logs stay local to the build. Signed outcomes use bounded reason codes,
      // never arbitrary command output or environment-bearing diagnostics.
      failures[node] = hostAvailable ? 'js-build-or-prerequisite-failed' : 'registry-or-source-host-tool-unavailable';
    }
  }
  if (!bestEffort) return directories;
  // Joint bootstrap recipes resolve project-reference/peer cycles as a cohort.
  // A later failure also invalidates an earlier member's partial output.
  const bootstrap = new Set(['packages/cli','packages/core','packages/runtime','packages/runtime-plugin-hmr','packages/runtime-plugin-import-meta','packages/plugin-tools','packages/utils']);
  for (const cohort of [bootstrap,new Set(['packages/docs','packages/docs-theme-default'])]) {
    if (directories.some(directory=>cohort.has(directory)&&failures['npm:'+directory]))
      for (const directory of directories) if (cohort.has(directory)) failures['npm:'+directory]='js-bootstrap-cohort-failed';
  }
  for (const directory of directories) if (dependsOnFailure('npm:'+directory)) failures['npm:'+directory] ??= 'js-build-prerequisite-failed';
  return {completed:completed.filter(directory=>!failures['npm:'+directory]),failures};
}

export function verifyRegistryHostMaterial(material, selection, readBinary) {
  const input=selection.registryBuildInputs?.['npm:@farmfe/core-linux-x64-gnu'];
  assert.ok(input && input.origin==='registry-present' && input.evidenceKind==='registry-metadata','Registry host input must be a retained version observation');
  assert.equal(material.schemaVersion,1);assert.equal(material.kind,'registry-native-host-build-material');
  for (const field of ['sourceSHA','sourceTree','planHash']) assert.equal(material[field],selection[field]);
  for (const field of ['name','version','platform','integrity','tarballURL']) assert.equal(material[field],input[field]);
  assert.equal(material.name,'@farmfe/core-linux-x64-gnu');assert.equal(material.platform,'linux-x64-gnu');
  assert.equal(material.origin,'registry-present');assert.equal(material.evidenceKind,'registry-archive');
  assert.equal(material.registryBytesVerified,true);assert.equal(material.sourceNativeArtifact,false);assert.equal(material.publicationCandidate,false);
  assert.equal(material.destination,'packages/core/binding/farm.linux-x64-gnu.node');
  for (const field of ['archiveSha256','binarySha256']) assert.match(material[field]??'',/^[a-f0-9]{64}$/);
  for (const field of ['archiveBytes','binaryBytes']) assert.ok(Number.isSafeInteger(material[field]) && material[field]>0 && material[field]<=100*1024**2,'Registry host material size invalid');
  assert.equal(material.smoke?.node,'v22.20.0');assert.equal(material.smoke.platform,'linux');assert.equal(material.smoke.arch,'x64');assert.equal(material.smoke.libc,'glibc');
  assert.match(material.smoke.napi??'',/^[1-9][0-9]*$/);
  assert.deepEqual(material.smoke.requiredExports,['Compiler','DocsSnapshot','OutputDirectoryLock','Resolver']);
  for (const field of ['resolver','compile','tsTransformed','outputExecuted']) assert.equal(material.smoke[field],true);
  const bytes=readBinary(material.destination);
  assert.equal(bytes.length,material.binaryBytes);assert.equal(sha256(bytes),material.binarySha256,'Registry host changed after its isolated smoke');
  return material;
}

export function presenceNativePayloads(family, selection) {
  assert.ok(versionPresenceMode(selection), 'Version-presence selection required');
  const cells = [...(selection.nativeBuildCells??[]),...(selection.preparationNativeCells??[])];
  const selected = new Map();
  for (const cell of cells) {
    const pkg = family.npm.payloads.find(row=>'npm:'+row.name===cell.package);
    assert.ok(pkg && pkg.owner===cell.owner && pkg.platform===cell.platform, 'Native cell differs from the source family');
    assert.equal(selection.registrySnapshot.packages[cell.package]?.status,'missing','Published or unknown native cells cannot be rebuilt');
    selected.set(pkg.name,pkg);
  }
  return [...selected.values()];
}

export function assemblePresenceNative(family, selection, {root,pool,sourceSHA,assemble=assembleNativeArtifacts}) {
  const records=[], failures={};
  for (const pkg of presenceNativePayloads(family,selection)) {
    try { records.push(...assemble(root,pool,sourceSHA,{...family,npm:{...family.npm,payloads:[pkg]}})); }
    catch { failures['npm:'+pkg.name]='native-build-or-artifact-unavailable'; }
  }
  return {records,failures};
}

export function packPresenceCandidates({root,candidate,assembly,artifacts,identity,output,pnpm,pack=packNpmCandidates,verify=verifyCandidateSeal}) {
  assert.equal(candidate.incremental.selectionPolicy,'registry-exact-version-v2');
  fs.mkdirSync(output);
  const failures={...candidate.incremental.buildFailures},records=[];
  const selected=[...candidate.publication.npm.primary,...candidate.publication.npm.payloads];
  for (const [index,pkg] of selected.entries()) {
    const key='npm:'+pkg.name;
    if (failures[key]) continue;
    const one={...candidate,publication:{npm:{primary:pkg.platform?[]:[pkg],payloads:pkg.platform?[pkg]:[]},cargo:[]}};
    const native=assembly.records.filter(row=>row.name===pkg.name);
    const part=path.join(output,'part-'+index);
    try {
      const receipt=pack({root,plan:one,assembly:{...assembly,count:native.length,records:native},artifactIndex:artifacts,...identity,output:part,pnpm});
      assert.equal(receipt.status,'sealed');assert.equal(receipt.records.length,1);
      const row=receipt.records[0];assert.equal(row.name,pkg.name);assert.equal(path.basename(row.file),row.file);
      fs.copyFileSync(path.join(part,row.file),path.join(output,row.file),fs.constants.COPYFILE_EXCL);
      records.push(row);
    } catch { failures[key]='package-pack-or-seal-failed'; }
  }
  const successful=new Set(records.map(row=>row.name));
  candidate.publication.npm={primary:candidate.publication.npm.primary.filter(row=>successful.has(row.name)),payloads:candidate.publication.npm.payloads.filter(row=>successful.has(row.name))};
  candidate.incremental.buildFailures=failures;
  assert.deepEqual(Object.keys(failures).sort(),candidate.incremental.publish.filter(key=>!successful.has(key.slice(4))).sort(),'Missing source package outcome');
  const receipt={schemaVersion:1,sourceCommit:identity.sourceCommit,testedCommit:identity.testedCommit,testedTree:identity.testedTree,
    planSha256:sha256(Buffer.from(JSON.stringify(candidate))),published:false,status:'sealed',records,errors:[],buildFailures:failures,
    registryBuildMaterials:candidate.incremental.registryBuildMaterials};
  const verified=verify(candidate,receipt,output,root);
  assert.equal(verified.candidateArchivesVerified,true,'Successful source subset failed final verification');
  fs.writeFileSync(path.join(output,'receipt.json'),JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});
  fs.writeFileSync(path.join(output,'verification.json'),JSON.stringify(verified,null,2)+'\n',{flag:'wx'});
  return receipt;
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
  if (versionPresenceMode(selection)) for (const row of completeRows) assert.ok(selection.publish.includes(row.kind+':'+row.name),'Registry-present package was sealed as a source build');
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
  const presence = versionPresenceMode(selection);
  assert.ok(presence || selection.publish.length>0,'No publication delta; no publisher transaction is needed');
  const rows = completeRows
    .filter(row=>selection.publish.includes(row.kind+':'+row.name));
  const byName=new Map(rows.filter(row=>row.kind==='npm').map(row=>[row.name,row]));
  const ordered=rows.filter(row=>row.kind==='cargo'),active=new Set(),done=new Set();
  function visit(name){
    if(done.has(name))return;
    assert.ok(!active.has(name),'Cyclic npm publication dependencies');active.add(name);
    const row=byName.get(name),manifest=readPackage(row);
    // Peers constrain consumer compatibility, not upload order (Docs/theme peers form a valid cycle).
    // Keep peer declarations in archive/family/consumer checks; hard and optional cycles still fail.
    for(const dependency of [...new Set(['dependencies','optionalDependencies'].flatMap(field=>Object.keys(manifest[field]??{})))].sort())
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
  const buildFailures = receipt.buildFailures ?? {};
  if (presence) {
    assert.equal(selection.baselineBinding ?? null, null, 'Version presence cannot use a historical baseline');
    assert.deepEqual([...selection.build].sort(), [...selection.publish].sort());
    const sealed = new Set(packages.map(row => `${row.kind}:${row.name}`));
    assert.deepEqual(Object.keys(buildFailures).sort(), selection.publish.filter(key=>!sealed.has(key)).sort(), 'Build failures must account for every unsealed missing version');
    for (const [key, reason] of Object.entries(buildFailures)) assert.ok(selection.publish.includes(key) && typeof reason === 'string' && /^[a-z0-9-]{1,100}$/.test(reason), 'Invalid package build failure');
    for (const key of Object.keys(selection.buildBlocked ?? {})) assert.ok(Object.hasOwn(buildFailures,key), 'Blocked source input was published');
  } else assert.deepEqual(packages.map(row => `${row.kind}:${row.name}`).sort(), [...selection.publish].sort(), 'Sealed publication delta differs from independent plan');
  assert.equal(new Set(packages.map(row => `${row.kind}:${row.name}`)).size, packages.length);
  return {schemaVersion: 1, releaseValidation: 'archive-integrity-v1', chain: 'release', profile: 'release', namespace: 'farm-release-v1', publicationScope:selection.publicationScope??'all',
    sourceSHA: source.sha, sourceTree: source.tree, planHash: selection.planHash, graphHash: selection.graphHash, baseline: selection.baselineBinding ?? null,
    producerRunId: runId, producerAttempt: attempt, abi: plan.abi,
    tag: packages.some(row => row.kind === 'npm' && row.name === '@farmfe/core')
      ? `v${packages.find(row => row.kind === 'npm' && row.name === '@farmfe/core').version}` : `packages-${selection.planHash.slice(0,16)}`,
    family:fullFamily, rebuild:[...(selection.build ?? selection.publish)], versionRequired:[...(selection.versionRequired ?? [])],
    publish:[...selection.publish], buildDependencyClosure:[...(selection.buildDependencyClosure ?? [])],
    validationRequired:[...(selection.validationRequired ?? [])], ...(selection.registrySnapshot ? {registrySnapshot:selection.registrySnapshot} : {}),
    ...(presence ? {selectionPolicy:selection.selectionPolicy,registryRetained:selection.registryRetained??{},registryUnknown:selection.registryUnknown??{},
      nativeBuildCells:selection.nativeBuildCells??[],preparationNativeCells:selection.preparationNativeCells??[],registryBuildInputs:selection.registryBuildInputs??{},
      registryBuildMaterials:receipt.registryBuildMaterials??[],buildFailures} : {}), packages};
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
    if (versionPresenceMode(selection)) {
      const {records,failures}=assemblePresenceNative(plan,selection,{root,pool:path.resolve(pool),sourceSHA:source.sha});
      for (const [key,reason] of Object.entries(failures)) if (selection.publish.includes(key)) candidate.incremental.buildFailures[key]=reason;
      const directories=releaseBuildDirectories(plan,selection,root);
      let hostAvailable=directories.length===0;
      if (directories.length) {
        try {
          assert.equal(process.platform,'linux');assert.equal(process.arch,'x64');
          const core=records.find(row=>row.owner==='@farmfe/core'&&row.platform==='linux-x64-gnu');
          const inputs=Object.values(selection.registryBuildInputs??{});
          const registered=inputs.find(row=>row.owner==='@farmfe/core'&&row.platform==='linux-x64-gnu');
          assert.ok(!(core&&registered),'Native host cannot have source and registry origins');
          if (registered) {
            const material=path.join(root,'release-work/registry-host-material.json');
            execFileSync(process.env.FARM_RELEASE_PYTHON??'python3',['-B',path.join(root,'scripts/release-registry-host-tool.py'),
              '--selection',path.resolve(selectionFile),'--output',material,'--destination',path.join(root,'packages/core/binding/farm.linux-x64-gnu.node'),'--node',process.execPath],
              {cwd:root,env:{PATH:process.env.PATH??'/usr/bin:/bin',FARM_RELEASE_PLAN_HASH:selection.planHash},timeout:180000,stdio:['ignore','pipe','pipe']});
            candidate.incremental.registryBuildMaterials.push(verifyRegistryHostMaterial(JSON.parse(fs.readFileSync(material)),selection,file=>fs.readFileSync(path.join(root,file))));
          } else {
            assert.ok(core,'Missing exact source GNU Core prerequisite');
            fs.copyFileSync(path.join(root,core.file),path.join(root,'packages/core/binding',path.basename(core.file)),fs.constants.COPYFILE_EXCL);
            const native=createRequire(import.meta.url)(path.join(root,'packages/core/binding/farm.linux-x64-gnu.node'));
            assert.equal(typeof native.Compiler,'function');assert.equal(typeof native.Resolver,'function');
          }
          hostAvailable=true;
        } catch { hostAvailable=false; }
      }
      const js=await runReleaseJsBuilds(plan,selection,{root,pnpm:directories.length?resolvePnpmCli():null,bestEffort:true,hostAvailable});
      for (const pkg of candidate.publication.npm.primary) {
        const failure=js.failures['npm:'+path.posix.dirname(pkg.manifestPath)];
        if (failure) candidate.incremental.buildFailures['npm:'+pkg.name]=failure;
      }
      fs.writeFileSync(candidateFile,JSON.stringify(candidate,null,2)+'\n');
      fs.writeFileSync(assemblyFile,JSON.stringify({schemaVersion:1,sourceCommit:source.sha,
        count:records.filter(row=>selectedNative.some(pkg=>pkg.name===row.name)).length,
        records:records.filter(row=>selectedNative.some(pkg=>pkg.name===row.name)),
        preparationRecords:records.filter(row=>!selectedNative.some(pkg=>pkg.name===row.name)),published:false},null,2)+'\n',{flag:'wx'});
      return {rebuiltNative:records.length,rebuiltJs:js.completed,buildFailures:candidate.incremental.buildFailures,profile:'release'};
    }
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
    if (candidate.incremental?.selectionPolicy==='registry-exact-version-v2') {
      const receipt=packPresenceCandidates({root,candidate,assembly:JSON.parse(fs.readFileSync(assembly)),artifacts,identity,
        output:path.resolve(output),pnpm:candidate.publication.npm.primary.length+candidate.publication.npm.payloads.length?resolvePnpmCli():null});
      fs.writeFileSync(candidateFile,JSON.stringify(candidate,null,2)+'\n');
      return receipt;
    }
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
