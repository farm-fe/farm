import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { cargoInventory, farmAbiOwners } from './v2-release-inventory.mjs';
import { changesetStatus } from './check-changeset-status.mjs';

export const cargoChangesetId = 'farm-cargo-version-proposal';
export function cargoChangeset(names) {
  assert.ok(names.includes('@farmfe/core'));
  return `---\n${[...new Set(names)].sort().map(name => `${JSON.stringify(name)}: patch`).join('\n')}\n---\n\nPrepare native consumers for the reviewed Rust SDK version proposal.\n`;
}
export function assertVersionOnlyManifest(before, after) {
  const normalize = text => text.replaceAll('\r\n', '\n').replace(/((?:^|[\s,{])version\s*=\s*")\d+\.\d+\.\d+(?=")/g, '$1<version>');
  assert.equal(normalize(before), normalize(after), 'Cargo proposal changes more than stable version fields');
}
export function assertNewRustVersion(previous, next) {
  assert.match(previous, /^\d+\.\d+\.\d+$/);
  assert.match(next, /^(?:0|1|2)\.\d+\.\d+$/, 'Unreviewed Rust major version');
  const a = previous.split('.').map(Number), b = next.split('.').map(Number);
  const index = a.findIndex((value, i) => value !== b[i]);
  assert.ok(index >= 0 && b[index] > a[index], 'Rust proposal must advance the immutable version');
}
export function synchronizeNativePayloads(directory, baseline) {
  for (const old of baseline.npm.primary) {
    const file = path.join(directory, old.manifestPath), owner = JSON.parse(fs.readFileSync(file));
    if (owner.version === old.version) continue;
    const payloads = baseline.npm.payloads.filter(row => row.owner === owner.name);
    for (const payload of payloads) {
      const target = path.join(directory, payload.manifestPath), value = JSON.parse(fs.readFileSync(target));
      assert.equal(value.name, payload.name, 'Native wrapper identity changed');
      assert.ok(value.name in (owner.optionalDependencies ?? {}), 'Native optional dependency removed');
      value.version = owner.version; owner.optionalDependencies[value.name] = owner.version;
      fs.writeFileSync(target, JSON.stringify(value, null, 2) + '\n');
      const changelog = path.join(path.dirname(target), 'CHANGELOG.md');
      const existing = fs.existsSync(changelog) ? fs.readFileSync(changelog, 'utf8') : `# ${value.name}\n`;
      if (!existing.split(/\r?\n/).includes(`## ${owner.version}`)) {
        const a = old.version.split('.'), b = owner.version.split('.');
        const level = a[0] !== b[0] ? 'Major' : a[1] !== b[1] ? 'Minor' : 'Patch';
        const firstLine = existing.indexOf('\n');
        const entry = `\n\n## ${owner.version}\n\n### ${level} Changes\n\n- Native platform payload for ${owner.name}@${owner.version}. See the owner changelog for functional changes.\n`;
        fs.writeFileSync(changelog, firstLine < 0 ? existing + entry : existing.slice(0, firstLine) + entry + existing.slice(firstLine));
      }
    }
    if (payloads.length) fs.writeFileSync(file, JSON.stringify(owner, null, 2) + '\n');
  }
}
export function requiredNativeOwners(plan, metadata, changedNames, abi = plan.abi) {
  const changed = new Set(changedNames), byName = new Map(metadata.packages.map(pkg => [pkg.name, pkg]));
  const reaches = (pkg, seen = new Set()) => {
    if (changed.has(pkg.name)) return true;
    if (seen.has(pkg.name)) return false;
    seen.add(pkg.name);
    return pkg.dependencies.some(dep => dep.kind !== 'dev' && dep.path && byName.has(dep.name) && reaches(byName.get(dep.name), seen));
  };
  const names = new Set(['@farmfe/core']);
  if (abi !== plan.abi) for (const name of farmAbiOwners(plan)) names.add(name);
  for (const owner of plan.npm.primary.filter(row => plan.npm.payloads.some(payload => payload.owner === row.name))) {
    const crateDirectory = owner.name === '@farmfe/core' ? 'crates/node' : owner.name === 'create-farm' ? 'crates/create-farm-rs' : path.dirname(owner.manifestPath);
    const pkg = metadata.packages.find(row => path.resolve(row.manifest_path) === path.resolve(crateDirectory, 'Cargo.toml'));
    assert.ok(pkg, `Missing native owner Cargo metadata: ${owner.name}`);
    if (reaches(pkg)) names.add(owner.name);
  }
  return [...names].sort();
}
export function validateCargoPreparation(record, baseline, metadata, { baseSHA, readGitFile, changedFiles, readCurrent }) {
  assert.equal(record.schemaVersion, 1); assert.equal(record.baseSHA, baseSHA);
  assert.match(record.headSHA, /^[a-f0-9]{40}$/); assert.match(record.branch, /^release-plz-/);
  assert.deepEqual(changedFiles(record.baseSHA, record.headSHA), record.files, 'Cargo proposal file set drift');
  const manifests = new Set(['Cargo.toml', ...baseline.cargo.map(row => row.manifestPath)]);
  const allowed = new Set(['Cargo.lock', ...manifests, ...baseline.cargo.map(row => `${path.dirname(row.manifestPath).replaceAll('\\', '/')}/CHANGELOG.md`)]);
  assert.ok(record.files.length && record.files.every(file => allowed.has(file)), 'Cargo proposal exceeds SDK preparation scope');
  for (const file of record.files) {
    const proposed = readGitFile(record.headSHA, file).replaceAll('\r\n', '\n');
    assert.equal(readCurrent(file).replaceAll('\r\n', '\n'), proposed, `Cargo proposal content differs: ${file}`);
    if (manifests.has(file)) assertVersionOnlyManifest(readGitFile(record.baseSHA, file), proposed);
  }
  const next = cargoInventory(metadata);
  const changed = next.filter(row => row.version !== baseline.cargo.find(old => old.name === row.name)?.version);
  assert.deepEqual(next.map(row => row.name), baseline.cargo.map(row => row.name), 'SDK closure changed outside version preparation');
  assert.deepEqual(changed.map(({ name, version }) => ({ name, version })), record.versions);
  assert.ok(changed.length, 'Cargo proposal has no SDK version changes');
  for (const row of changed) assertNewRustVersion(baseline.cargo.find(old => old.name === row.name).version, row.version);
  const abi = readCurrent('crates/core/src/lib.rs').match(/^pub const VERSION: &str = "([^"]+)";/m)?.[1];
  const names = requiredNativeOwners(baseline, metadata, changed.map(row => row.name), abi);
  assert.deepEqual(record.requiredNpm, names);
  assert.equal(record.changeset, cargoChangeset(names), 'Generated Cargo Changeset differs');
  return record.files;
}
const root = fileURLToPath(new URL('../', import.meta.url));
const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8', timeout: 120000 });
const gh = endpoint => JSON.parse(execFileSync('gh', ['api', endpoint], { cwd: root, encoding: 'utf8', timeout: 30000 }));
export function inspectCargoProposal() {
  const repository = process.env.GITHUB_REPOSITORY; assert.match(repository, /^[\w.-]+\/[\w.-]+$/);
  const baseSHA = git('rev-parse', 'HEAD').trim();
  const proposals = [];
  for (let page = 1; page <= 10; page++) {
    const rows = gh(`repos/${repository}/pulls?state=open&base=main&per_page=100&page=${page}`);
    proposals.push(...rows.filter(pr => pr.head.ref.startsWith('release-plz-')));
    if (rows.length < 100) break;
    assert.ok(page < 10, 'Cargo proposal search exceeded bound');
  }
  assert.ok(proposals.length <= 1, 'Multiple Cargo proposals require review');
  if (!proposals.length) return null;
  const pr = proposals[0];
  assert.equal(pr.user.login, 'github-actions[bot]', 'Cargo proposal must be created by the configured workflow token');
  assert.equal(pr.head.repo.full_name, repository); assert.equal(pr.base.repo.full_name, repository);
  assert.equal(pr.base.sha, baseSHA, 'Cargo proposal is stale; refresh release-plz first');
  return { number: pr.number, headSHA: pr.head.sha, baseSHA, branch: pr.head.ref };
}
export function applyCargoProposal(proposal) {
  if (!proposal) return;
  assert.equal(git('rev-parse', 'HEAD').trim(), proposal.baseSHA, 'Version base moved');
  assert.match(proposal.headSHA, /^[a-f0-9]{40}$/);
  const repository = process.env.GITHUB_REPOSITORY;
  const current = gh(`repos/${repository}/pulls/${proposal.number}`);
  assert.equal(current.state, 'open'); assert.equal(current.head.sha, proposal.headSHA);
  assert.equal(current.head.repo.full_name, repository); assert.equal(current.base.sha, proposal.baseSHA);
  assert.equal(current.head.ref, proposal.branch); assert.match(proposal.branch, /^release-plz-/);
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  git('fetch', 'origin', proposal.headSHA);
  const files = git('diff', '--name-only', proposal.baseSHA, proposal.headSHA).trim().split(/\r?\n/).filter(Boolean);
  const manifests = new Set(['Cargo.toml', ...plan.cargo.map(row => row.manifestPath)]);
  const allowed = new Set(['Cargo.lock', ...manifests, ...plan.cargo.map(row => `${path.dirname(row.manifestPath).replaceAll('\\', '/')}/CHANGELOG.md`)]);
  assert.ok(files.length && files.every(file => allowed.has(file)), 'Cargo proposal exceeds selected SDK version/changelog scope');
  for (const file of files.filter(file => manifests.has(file))) assertVersionOnlyManifest(git('show', `${proposal.baseSHA}:${file}`), git('show', `${proposal.headSHA}:${file}`));
  const patch = git('diff', '--binary', proposal.baseSHA, proposal.headSHA, '--', ...files);
  execFileSync('git', ['apply', '--check', '-'], { cwd: root, input: patch, encoding: 'utf8' });
  execFileSync('git', ['apply', '-'], { cwd: root, input: patch, encoding: 'utf8' });
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--locked', '--format-version', '1'], { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 ** 2 }));
  const next = cargoInventory(metadata);
  assert.deepEqual(next.map(row => row.name), plan.cargo.map(row => row.name), 'SDK closure changed outside version preparation');
  const changed = next.filter(row => row.version !== plan.cargo.find(old => old.name === row.name).version);
  assert.ok(changed.length, 'Cargo proposal has no selected version changes');
  for (const row of changed) assertNewRustVersion(plan.cargo.find(old => old.name === row.name).version, row.version);
  const abi = fs.readFileSync(path.join(root, 'crates/core/src/lib.rs'), 'utf8').match(/^pub const VERSION: &str = "([^"]+)";/m)?.[1];
  const requiredNpm = requiredNativeOwners(plan, metadata, changed.map(row => row.name), abi);
  const changeset = cargoChangeset(requiredNpm);
  const target = path.join(root, `.changeset/${cargoChangesetId}.md`);
  if (fs.existsSync(target)) assert.equal(fs.readFileSync(target, 'utf8'), cargoChangeset(['@farmfe/core']), 'Reserved Cargo proposal changeset has user content');
  fs.writeFileSync(target, changeset);
  fs.writeFileSync(path.join(root, 'docs/release-evidence/cargo-version-proposal.json'), JSON.stringify({ schemaVersion: 1, ...proposal, files, versions: changed.map(({ name, version }) => ({ name, version })), requiredNpm, changeset }, null, 2) + '\n');
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === '--seed') {
    const proposal = JSON.parse(process.env.FARM_CARGO_PROPOSAL ?? 'null');
    if (proposal) {
      assert.match(proposal.headSHA, /^[a-f0-9]{40}$/);
      fs.writeFileSync(path.join(root, `.changeset/${cargoChangesetId}.md`), cargoChangeset(['@farmfe/core']), { flag: 'wx' });
    }
  } else {
  assert.equal(process.argv[2], '--inspect');
  const proposal = inspectCargoProposal(), state = proposal ? 'pending' : changesetStatus(root).state;
  if (process.env.GITHUB_OUTPUT) fs.appendFileSync(process.env.GITHUB_OUTPUT, `state=${state}\nproposal=${JSON.stringify(proposal)}\n`);
  console.log(JSON.stringify({ state, proposal }));
  }
}
