import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
import { rustPublicationOrder } from './release-rust-metadata.mjs';
import { execFileSync } from 'node:child_process';
import { npmInventory, cargoInventory, changesetsPublication } from './v2-release-inventory.mjs';
import { cargoChangesetId, validateCargoPreparation, readCargoArtifact } from './prepare-cargo-version-proposal.mjs';
// Intent is reviewed version-preparation data. Only the main release workflow's
// frozen source, verified release baseline and explicit switch authorize writes.
export const independentReleaseWorkflow = '.github/workflows/incremental-release.yaml';
export function independentPublicationPolicy() {
  return {workflowPath: independentReleaseWorkflow, event: 'workflow_dispatch', ref: 'refs/heads/main',
    publishDefault: false, explicitAuthorizationRequired: true, sourceBinding: 'fixed-main-dispatch-snapshot',
    chain: 'release', profile: 'release', namespace: 'farm-release-v1',
    baselineAuthority: 'completed-public-registry-and-tag-release-acceptance', ciBuildBytesAccepted: false};
}
const requiredWorkflows = ['.github/workflows/ci.yaml', '.github/workflows/rust-test.yaml', independentReleaseWorkflow];
const platforms = ['ubuntu-24.04', 'macos-latest', 'windows-latest'];
const requiredCiJobs = ['Release source contracts', 'Check declared native inputs (not ABI execution)', 'Type Check & Changesets',
  ...['Examples Test', 'Typescript Test', 'Commerce Performance'].flatMap(name => platforms.map(platform => `${name} (${platform})`)),
  'Docs / windows-latest', 'Docs / ubuntu-24.04', 'Aggregate informational performance report'];
const requiredReleaseJobs = ['Build independent release candidates', ...platforms.map(platform => `Independent release candidate consumer (${platform})`)];
export function serializeReleaseIntent(intent) {
  assert.deepEqual(intent.publication, independentPublicationPolicy(), 'Intent cannot enable publication or select another build chain');
  assert.equal(intent.seal?.workflowPath, independentReleaseWorkflow, 'Release seal must come from its independent release producer');
  return JSON.stringify(intent, null, 2) + '\n';
}
export function validationDispatches(number, baseSHA) {
  assert.ok(Number.isSafeInteger(number) && number > 0); assert.match(baseSHA, /^[a-f0-9]{40}$/);
  return ['ci.yaml', 'rust-test.yaml', 'lint.yaml', 'code-spell-check.yaml'].map(workflow =>
    ({workflow, inputs: workflow === 'ci.yaml' ? [`pr_number=${number}`, `base_sha=${baseSHA}`] : [`release_pr=${number}`, `release_base=${baseSHA}`]}));
}
export function createReleaseIntent(root, plan, metadata, pullRequest = 2329) {
  assert.ok(Number.isSafeInteger(pullRequest) && pullRequest > 0, 'Reviewed version PR number is required');
  const compatibility = { farmfe_macro_cache_item: 'published-macro-compat.json', farmfe_macro_plugin: 'published-plugin-macro-compat.json' };
  const changesets = plan.preparation?.mode === 'changesets';
  const rust = rustPublicationOrder(plan, metadata, { includeUploadMetadata: false }).map(({ name, version }) => {
    const record = { name, version, action: 'publish' };
    if (changesets && !plan.publication.cargo.some(row => row.name === name)) {
      const reused = plan.publication.reuseRust.find(row => row.name === name && row.version === version);
      assert.ok(reused, `Missing immutable registry reuse: ${name}`);
      const evidence = JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence/changesets-preparation.json')));
      return { ...record, action: 'reuse', registryChecksum: reused.sha256, reuseEvidenceFile: 'docs/release-evidence/changesets-preparation.json', reuseEvidenceSha256: digest(JSON.stringify(evidence)) };
    }
    if (!changesets && compatibility[name]) {
      const evidence = JSON.parse(fs.readFileSync(path.join(root, 'docs/release-evidence', compatibility[name])));
      assert.equal(evidence.status, 'passed'); assert.equal(evidence.registryPackage.name, name); assert.equal(evidence.registryPackage.version, version);
      return { ...record, action: 'reuse', registryChecksum: evidence.registryPackage.sha256, reuseEvidenceFile: `docs/release-evidence/${compatibility[name]}`, reuseEvidenceSha256: digest(JSON.stringify(evidence)) };
    }
    return record;
  });
  const script = 'scripts/verify-published-v2-consumers.mjs';
  const abi = fs.readFileSync(path.join(root, 'crates/core/src/lib.rs'), 'utf8').match(/^pub const VERSION: &str = "([^"]+)";/m)?.[1];
  return { schemaVersion: 1, preparationMode: changesets ? 'changesets' : 'first-stable', pullRequest, baseBranch: 'main', tag: plan.publication?.releaseIdentity ?? `v${plan.npm.primary.find(pkg => pkg.name === '@farmfe/core').version}`,
    abi, rustCoreVersion: plan.cargo.find(row => row.name === 'farmfe_core').version, candidatePlanSha256: digest(JSON.stringify(plan)), counts: plan.publication?.counts ?? plan.counts,
    requiredWorkflows, requiredCiJobs, requiredReleaseJobs, performanceRequired: true, publication: independentPublicationPolicy(), seal: { workflowPath: independentReleaseWorkflow }, candidateSeal: 'receipt.json', sdkProof: 'sdk-consumer.json',
    rust, registryVerification: { script, sha256: digest(fs.readFileSync(path.join(root, script), 'utf8').replaceAll('\r\n', '\n')) }, websiteDeployment: false };
}
export async function refreshChangesetsVersionPr(root, number) {
  const repository = process.env.GITHUB_REPOSITORY;
  assert.match(repository, /^[\w.-]+\/[\w.-]+$/); assert.ok(Number.isSafeInteger(number) && number > 0);
  const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8', timeout: 120000 }).trim();
  const gh = (...args) => JSON.parse(execFileSync('gh', args, { cwd: root, encoding: 'utf8', timeout: 30000 }));
  const pr = gh('api', `repos/${repository}/pulls/${number}`);
  assert.equal(pr.head.repo.full_name, repository); assert.equal(pr.base.repo.full_name, repository);
  assert.equal(pr.base.ref, 'main'); assert.equal(pr.head.ref, 'changeset-release/main');
  assert.equal(pr.state, 'open'); assert.match(pr.title, /^chore: release packages/);
  assert.equal(git('status', '--porcelain'), '', 'Version action left uncommitted files; preserve them');
  git('fetch', 'origin', pr.head.sha, pr.base.sha); git('checkout', '--detach', pr.head.sha);
  const read = file => JSON.parse(fs.readFileSync(path.join(root, file)));
  const show = file => git('show', `${pr.base.sha}:${file}`);
  const official = read('docs/release-evidence/changesets-version-plan.json');
  assert.ok(official.changesets?.length && official.releases?.length, 'Official Changesets plan required');
  const baseline = JSON.parse(show('docs/releases/v2-candidate.json'));
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--locked', '--format-version', '1'], { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 ** 2 }));
  const cargoRecordFile = 'docs/release-evidence/cargo-version-proposal.json';
  const cargoRecord = official.changesets.some(row => row.id === cargoChangesetId) ? read(cargoRecordFile) : null;
  let cargoFiles = [];
  if (cargoRecord) {
    git('fetch', 'origin', cargoRecord.headSHA);
    cargoFiles = validateCargoPreparation(cargoRecord, baseline, metadata, { baseSHA: pr.base.sha,
      repository, readProducerRun: id => gh('api', `repos/${repository}/actions/runs/${id}`),
      readProducerArtifact: readCargoArtifact,
      isAncestor: (base, head) => { git('merge-base', '--is-ancestor', base, head); return true; },
      readGitFile: (sha, file) => execFileSync('git', ['show', `${sha}:${file}`], { cwd: root, encoding: 'utf8' }),
      changedFiles: (base, head) => git('diff', '--name-only', base, head).split(/\r?\n/).filter(Boolean),
      readCurrent: file => fs.readFileSync(path.join(root, file), 'utf8') });
  }
  const allowedBeforeRefresh = new Set(['pnpm-lock.yaml', 'docs/release-evidence/changesets-version-plan.json', 'docs/release-evidence/changesets-preparation.json', 'docs/releases/v2-candidate.json', 'docs/releases/v2-release-intent.json', 'scripts/release-fixtures/sdk-consumer/Cargo.toml',
    ...baseline.npm.primary.flatMap(row => [row.manifestPath, `${path.dirname(row.manifestPath).replaceAll('\\', '/')}/CHANGELOG.md`]),
    ...baseline.npm.payloads.flatMap(row => [row.manifestPath, `${path.dirname(row.manifestPath).replaceAll('\\', '/')}/CHANGELOG.md`]), ...official.changesets.map(row => `.changeset/${row.id}.md`)]);
  if (cargoRecord) for (const file of [cargoRecordFile, ...cargoFiles]) allowedBeforeRefresh.add(file);
  const modified = git('diff', '--name-only', pr.base.sha, pr.head.sha).split(/\r?\n/).filter(Boolean);
  assert.ok(modified.every(file => allowedBeforeRefresh.has(file)), 'Version PR contains non-generated source changes; review before granting validation execution');
  const componentFiles = [];
  for (const owner of baseline.npm.primary) {
    const file = `${path.dirname(owner.manifestPath).replaceAll('\\', '/')}/license-components.json`;
    if (!fs.existsSync(path.join(root, file))) continue;
    const components = read(file);
    // Only a versioned manifest is rewritten by Changesets. Source/license
    // attribution is never refreshed to hide an unreviewed source change.
    for (const component of components.components) for (const source of component.sourceFiles ?? []) {
      if (source.file === 'package.json') source.sha256 = digest(fs.readFileSync(path.join(root, owner.manifestPath), 'utf8').replaceAll('\r\n', '\n'));
    }
    fs.writeFileSync(path.join(root, file), JSON.stringify(components, null, 2) + '\n');
    componentFiles.push(file);
  }
  const npm = npmInventory(), cargo = cargoInventory(metadata);
  const sdkFixture = 'scripts/release-fixtures/sdk-consumer/Cargo.toml';
  let fixtureText = fs.readFileSync(path.join(root, sdkFixture), 'utf8');
  for (const row of cargo) fixtureText = fixtureText.replace(new RegExp(`^(${row.name} = )"=[^"]+"`, 'm'), `$1"=${row.version}"`);
  fs.writeFileSync(path.join(root, sdkFixture), fixtureText);
  const plan = { ...baseline, npm, cargo, abi: fs.readFileSync(path.join(root, 'crates/core/src/lib.rs'), 'utf8').match(/^pub const VERSION: &str = "([^"]+)";/m)?.[1], preparation: { mode: 'changesets', receipt: 'docs/release-evidence/changesets-preparation.json' },
    counts: { primary: npm.primary.length, payloads: npm.payloads.length, total: npm.primary.length + npm.payloads.length } };
  plan.publication = changesetsPublication(plan, baseline, official.releases);
  async function registry(url) {
    const response = await fetch(url, { signal: AbortSignal.timeout(30000) });
    assert.ok(response.ok, `Immutable registry prerequisite absent: ${url} (${response.status})`);
    return response.json();
  }
  const selected = new Set([...plan.publication.npm.primary, ...plan.publication.npm.payloads].map(row => row.name));
  plan.publication.reuseNpm = [];
  for (const row of [...npm.primary, ...npm.payloads].filter(row => !selected.has(row.name))) {
    const value = await registry(`https://registry.npmjs.org/${encodeURIComponent(row.name)}/${row.version}`);
    assert.equal(value.name, row.name); assert.equal(value.version, row.version);
    assert.match(value.dist?.integrity ?? '', /^sha512-[A-Za-z0-9+/]+=*$/);
    plan.publication.reuseNpm.push({ name: row.name, version: row.version, integrity: value.dist.integrity });
  }
  plan.publication.reuseRust = [];
  for (const row of cargo.filter(row => !plan.publication.cargo.some(item => item.name === row.name))) {
    const value = (await registry(`https://crates.io/api/v1/crates/${encodeURIComponent(row.name)}/${row.version}`)).version;
    assert.equal(value.num, row.version); assert.equal(value.yanked, false); assert.match(value.checksum, /^[a-f0-9]{64}$/);
    plan.publication.reuseRust.push({ name: row.name, version: row.version, sha256: value.checksum });
  }
  const write = (file, value) => fs.writeFileSync(path.join(root, file), JSON.stringify(value, null, 2) + '\n');
  write('docs/releases/v2-candidate.json', plan);
  const record = { schemaVersion: 1, mode: 'changesets', baseSHA: pr.base.sha, sourceHeadSHA: pr.head.sha,
    officialPlanSha256: digest(JSON.stringify(official)), planSha256: digest(JSON.stringify(plan)),
    consumed: official.changesets.map(change => ({ id: change.id, sha256: digest(change.id === cargoChangesetId && cargoRecord ? cargoRecord.changeset : execFileSync('git', ['show', `${pr.base.sha}:.changeset/${change.id}.md`], { cwd: root, encoding: 'utf8' }).replaceAll('\r\n', '\n')) })) };
  write('docs/release-evidence/changesets-preparation.json', record);
  const intent = createReleaseIntent(root, plan, metadata, number);
  fs.writeFileSync(path.join(root, 'docs/releases/v2-release-intent.json'), serializeReleaseIntent(intent));
  const files = ['docs/releases/v2-candidate.json', 'docs/releases/v2-release-intent.json', 'docs/release-evidence/changesets-preparation.json', sdkFixture, ...componentFiles];
  git('add', '--', ...files);
  git('-c', 'user.name=github-actions[bot]', '-c', 'user.email=41898282+github-actions[bot]@users.noreply.github.com', 'commit', '-m', 'chore: seal version preparation metadata');
  git('push', 'origin', 'HEAD:refs/heads/changeset-release/main');
  const head = git('rev-parse', 'HEAD');
  assert.equal(gh('api', `repos/${repository}/pulls/${number}`).head.sha, head, 'Version PR moved concurrently');
  for (const {workflow: file, inputs} of validationDispatches(number, pr.base.sha)) {
    try { execFileSync('gh', ['workflow', 'run', file, '--repo', repository, '--ref', pr.head.ref, ...inputs.flatMap(value => ['-f', value])], { cwd: root, stdio: 'inherit', timeout: 30000 }); }
    catch (error) { throw new Error(`Explicit validation dispatch failed for ${file}. This version-preparation job requires existing GITHUB_TOKEN actions:write; no new credential is created. ${error.message}`); }
  }
  console.log(JSON.stringify({ sourceHeadSHA: pr.head.sha, validationHeadSHA: head, publication: plan.publication.counts, registryReused: plan.publication.reuseNpm.length }));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === '--changesets-pr') {
    await refreshChangesetsVersionPr(fileURLToPath(new URL('../', import.meta.url)), Number(process.argv[3]));
  } else {
  const [metadataFile, destination, prNumber = '2329'] = process.argv.slice(2); assert.ok(metadataFile && destination);
  const root = fileURLToPath(new URL('../', import.meta.url));
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  const intent = createReleaseIntent(root, plan, JSON.parse(fs.readFileSync(metadataFile)), Number(prNumber));
  const text = serializeReleaseIntent(intent);
  fs.writeFileSync(destination, text, { flag: 'wx' }); console.log(digest(text.replaceAll('\r\n', '\n')));
  }
}
