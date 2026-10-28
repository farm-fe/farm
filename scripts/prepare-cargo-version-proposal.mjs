import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import crypto from 'node:crypto';
import { cargoInventory, farmAbiOwners } from './v2-release-inventory.mjs';
import { changesetStatus } from './check-changeset-status.mjs';

export const cargoChangesetId = 'farm-cargo-version-proposal';
export function cargoChangeset(names) {
  assert.ok(names.includes('@farmfe/core'));
  return `---\n${[...new Set(names)].sort().map(name => `${JSON.stringify(name)}: patch`).join('\n')}\n---\n\nPrepare native consumers for the reviewed Rust SDK version proposal.\n`;
}
export function assertVersionOnlyManifest(before, after) {
  // Retain the constraint prefix: a version bump must not widen an exact pin
  // into a caret/tilde range (or silently change the default constraint).
  const normalize = text => text.replaceAll('\r\n', '\n').replace(/((?:^|[\s,{])version\s*=\s*")([=^~]?)\d+\.\d+\.\d+(?=")/g, '$1$2<version>');
  assert.equal(normalize(before), normalize(after), 'Cargo proposal changes more than stable version fields');
}
export function assertNewRustVersion(previous, next) {
  assert.match(previous, /^\d+\.\d+\.\d+$/);
  assert.match(next, /^(?:0|1|2)\.\d+\.\d+$/, 'Unreviewed Rust major version');
  const a = previous.split('.').map(Number), b = next.split('.').map(Number);
  const index = a.findIndex((value, i) => value !== b[i]);
  assert.ok(index >= 0 && b[index] > a[index], 'Rust proposal must advance the immutable version');
}
const textHash = text => crypto.createHash('sha256').update(text.replaceAll('\r\n', '\n')).digest('hex');
export function affectedCargoConsumers(metadata, changedNames) {
  const affected = new Set(changedNames);
  const byPath = new Map(metadata.packages.map(pkg => [path.resolve(path.dirname(pkg.manifest_path)), pkg.name]));
  let expanded = true;
  while (expanded) {
    expanded = false;
    for (const pkg of metadata.packages) if (!affected.has(pkg.name) && pkg.dependencies.some(dep => dep.path && affected.has(byPath.get(path.resolve(dep.path))))) {
      affected.add(pkg.name); expanded = true;
    }
  }
  return affected;
}
export function updateConsumerDependencyVersions(source, pkg, versions) {
  const targets = new Map(pkg.dependencies.filter(dep => dep.path && versions.has(dep.name)).map(dep => [dep.rename ?? dep.name, versions.get(dep.name)]));
  let table = null;
  const update = (value, version) => {
    if (!/\bversion\s*=/.test(value)) return value; // path-only or inherited workspace dependency
    const pattern = /(\bversion\s*=\s*)(["'])([=^~]?)(\d+\.\d+\.\d+)\2/;
    assert.match(value, pattern, 'Consumer constraint needs explicit review');
    return value.replace(pattern, (_, field, quote, prefix) => `${field}${quote}${prefix}${version}${quote}`);
  };
  return source.replaceAll('\r\n', '\n').split('\n').map(line => {
    if (/^\s*\[/.test(line)) {
      const header = line.trim().match(/^\[(?:target\..+\.)?(?:dev-|build-)?dependencies(?:\.(.+))?\]$/);
      table = header ? { dependency: header[1]?.replace(/^["']|["']$/g, '') } : null;
      return line;
    }
    if (!table) return line;
    if (table.dependency) return targets.has(table.dependency) && /^\s*version\s*=/.test(line) ? update(line, targets.get(table.dependency)) : line;
    const entry = line.match(/^(\s*(?:"([^"]+)"|'([^']+)'|([\w-]+))\s*=\s*)(.*)$/);
    const key = entry?.[2] ?? entry?.[3] ?? entry?.[4];
    return entry && targets.has(key) ? entry[1] + update(entry[5], targets.get(key)) : line;
  }).join('\n');
}
function packageVersion(manifest, workspace) {
  const section = (text, name) => text.replaceAll('\r\n', '\n').split(`[${name}]`)[1]?.split(/^\[/m)[0] ?? '';
  const pkg = section(manifest, 'package');
  const own = pkg.match(/^\s*version\s*=\s*["'](\d+\.\d+\.\d+)["']/m)?.[1];
  if (own) return own;
  assert.match(pkg, /^\s*version\.workspace\s*=\s*true/m, 'Missing stable Cargo package version');
  const inherited = section(workspace, 'workspace.package').match(/^\s*version\s*=\s*["'](\d+\.\d+\.\d+)["']/m)?.[1];
  assert.ok(inherited, 'Missing stable workspace version'); return inherited;
}
export function cargoProposalScope(baseline, metadata, readBefore, readProposed, directory = process.cwd()) {
  const versions = new Map();
  for (const row of baseline.cargo) {
    const next = packageVersion(readProposed(row.manifestPath), readProposed('Cargo.toml'));
    if (next !== row.version) { assertNewRustVersion(row.version, next); versions.set(row.name, next); }
  }
  assert.ok(versions.size, 'Cargo proposal has no selected SDK version changes');
  const affected = affectedCargoConsumers(metadata, [...versions.keys()]);
  const manifests = new Set(['Cargo.toml', ...baseline.cargo.map(row => row.manifestPath)]), consumers = new Map();
  for (const pkg of metadata.packages.filter(row => affected.has(row.name))) {
    const relative = path.relative(directory, pkg.manifest_path).replaceAll('\\', '/');
    assert.ok(!relative.startsWith('..') && !path.isAbsolute(relative), 'Cargo consumer escapes repository');
    if (manifests.has(relative)) continue;
    const before = readBefore(relative);
    const after = updateConsumerDependencyVersions(before, pkg, versions);
    if (before.replaceAll('\r\n', '\n') !== after) consumers.set(relative, { before, after });
  }
  const allowed = new Set(['Cargo.lock', ...manifests, ...consumers.keys(), ...baseline.cargo.map(row => `${path.dirname(row.manifestPath).replaceAll('\\', '/')}/CHANGELOG.md`)]);
  return { manifests, consumers: new Map([...consumers].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)), allowed, versions };
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
  const affected = affectedCargoConsumers(metadata, changedNames);
  const names = new Set(['@farmfe/core']);
  if (abi !== plan.abi) for (const name of farmAbiOwners(plan)) names.add(name);
  for (const owner of plan.npm.primary.filter(row => plan.npm.payloads.some(payload => payload.owner === row.name))) {
    const crateDirectory = owner.name === '@farmfe/core' ? 'crates/node' : owner.name === 'create-farm' ? 'crates/create-farm-rs' : path.dirname(owner.manifestPath);
    const pkg = metadata.packages.find(row => path.resolve(row.manifest_path) === path.resolve(crateDirectory, 'Cargo.toml'));
    assert.ok(pkg, `Missing native owner Cargo metadata: ${owner.name}`);
    if (affected.has(pkg.name)) names.add(owner.name);
  }
  return [...names].sort();
}
export function validateCargoPreparation(record, baseline, metadata, { baseSHA, readGitFile, changedFiles, readCurrent, isAncestor, readProducerRun, repository }) {
  assert.equal(record.schemaVersion, 2); assert.equal(record.baseSHA, baseSHA);
  assert.match(record.headSHA, /^[a-f0-9]{40}$/); assert.match(record.branch, /^release-plz-/);
  assert.equal(typeof isAncestor, 'function'); assert.ok(isAncestor(baseSHA, record.headSHA), 'Cargo proposal does not contain its base');
  assert.equal(typeof readProducerRun, 'function');
  assert.ok(record.production && Number.isSafeInteger(record.production.runId), 'Missing Cargo producer evidence');
  validateCargoProduction(record.production, readProducerRun(record.production.runId), record, repository);
  assert.deepEqual(changedFiles(record.baseSHA, record.headSHA), record.files, 'Cargo proposal file set drift');
  const { manifests, consumers, allowed } = cargoProposalScope(baseline, metadata, file => readGitFile(baseSHA, file), file => readGitFile(record.headSHA, file));
  assert.ok(record.files.length && record.files.every(file => allowed.has(file)), 'Cargo proposal exceeds SDK preparation scope');
  for (const file of record.files) {
    const proposed = readGitFile(record.headSHA, file).replaceAll('\r\n', '\n');
    if (consumers.has(file)) assert.equal(proposed, consumers.get(file).after, `Consumer proposal changes more than dependency versions: ${file}`);
    else assert.equal(readCurrent(file).replaceAll('\r\n', '\n'), proposed, `Cargo proposal content differs: ${file}`);
    if (manifests.has(file)) assertVersionOnlyManifest(readGitFile(record.baseSHA, file), proposed);
  }
  const consumerRecords = [...consumers].map(([file, value]) => ({ file, beforeSha256: textHash(value.before), afterSha256: textHash(value.after) }));
  assert.deepEqual(record.consumerManifests, consumerRecords, 'Cargo consumer evidence differs');
  for (const [file, value] of consumers) assert.equal(readCurrent(file).replaceAll('\r\n', '\n'), value.after, `Consumer dependency update differs: ${file}`);
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
  return [...new Set([...record.files, ...consumers.keys()])];
}
const root = fileURLToPath(new URL('../', import.meta.url));
const git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8', timeout: 120000 });
const gh = endpoint => JSON.parse(execFileSync('gh', ['api', endpoint], { cwd: root, encoding: 'utf8', timeout: 30000 }));
const producerPath = '.github/workflows/release-plz.yaml';
function productionRecord(body) {
  const matches = [...(body ?? '').matchAll(/<!-- farm-cargo-production:([^\n]+) -->/g)];
  assert.equal(matches.length, 1, 'Exactly one Cargo producer record required');
  assert.ok(matches[0][1].length <= 8192, 'Cargo producer record exceeds bound');
  return JSON.parse(matches[0][1]);
}
export function validateCargoProduction(record, run, proposal, repository) {
  assert.equal(record.schemaVersion, 1); assert.equal(record.workflowPath, producerPath);
  assert.equal(record.sourceSHA, proposal.baseSHA); assert.equal(record.proposalHeadSHA, proposal.headSHA);
  assert.equal(record.number, proposal.number); assert.equal(record.branch, proposal.branch);
  assert.equal(run.id, record.runId); assert.equal(run.run_attempt, record.runAttempt);
  assert.equal(run.repository.full_name, repository); assert.equal(run.path, producerPath);
  assert.equal(run.head_sha, record.sourceSHA); assert.equal(run.head_branch, 'main');
  assert.ok(['push', 'workflow_dispatch'].includes(run.event));
  assert.equal(run.status, 'completed'); assert.equal(run.conclusion, 'success');
}
function liveProposal(pr, baseSHA) {
  const repository = process.env.GITHUB_REPOSITORY;
  assert.equal(pr.state, 'open'); assert.equal(pr.user.login, 'github-actions[bot]');
  assert.equal(pr.head.repo.full_name, repository); assert.equal(pr.base.repo.full_name, repository);
  assert.equal(pr.base.ref, 'main'); assert.equal(pr.base.sha, baseSHA, 'Cargo proposal target moved');
  assert.match(pr.head.ref, /^release-plz-[\w.-]+$/);
  git('fetch', 'origin', pr.head.sha);
  git('merge-base', '--is-ancestor', baseSHA, pr.head.sha);
  const proposal = { number: pr.number, headSHA: pr.head.sha, baseSHA, branch: pr.head.ref };
  const production = productionRecord(pr.body);
  validateCargoProduction(production, gh(`repos/${repository}/actions/runs/${production.runId}`), proposal, repository);
  if (process.env.GITHUB_EVENT_NAME === 'workflow_run') {
    const event = JSON.parse(fs.readFileSync(process.env.GITHUB_EVENT_PATH));
    assert.equal(production.runId, event.workflow_run.id, 'Proposal belongs to another producer run');
    assert.equal(production.runAttempt, event.workflow_run.run_attempt);
  }
  return { ...proposal, production };
}
function recordCargoProduction(value) {
  if (!value) return;
  const repository = process.env.GITHUB_REPOSITORY, baseSHA = git('rev-parse', 'HEAD').trim();
  assert.equal(process.env.GITHUB_WORKFLOW_REF, `${repository}/${producerPath}@refs/heads/main`);
  assert.equal(baseSHA, process.env.GITHUB_SHA);
  assert.ok(Number.isSafeInteger(value.number) && value.number > 0);
  const pr = gh(`repos/${repository}/pulls/${value.number}`);
  assert.equal(pr.state, 'open'); assert.equal(pr.user.login, 'github-actions[bot]');
  assert.equal(pr.head.repo.full_name, repository); assert.equal(pr.base.repo.full_name, repository);
  assert.equal(pr.base.ref, 'main'); assert.equal(pr.base.sha, baseSHA);
  assert.equal(pr.head.ref, value.head_branch); assert.match(pr.head.ref, /^release-plz-[\w.-]+$/);
  git('fetch', 'origin', pr.head.sha); git('merge-base', '--is-ancestor', baseSHA, pr.head.sha);
  const record = { schemaVersion: 1, workflowPath: producerPath, sourceSHA: baseSHA,
    runId: Number(process.env.GITHUB_RUN_ID), runAttempt: Number(process.env.GITHUB_RUN_ATTEMPT),
    number: pr.number, proposalHeadSHA: pr.head.sha, branch: pr.head.ref };
  assert.ok(Number.isSafeInteger(record.runId) && record.runId > 0 && Number.isSafeInteger(record.runAttempt) && record.runAttempt > 0);
  const body = (pr.body ?? '').replace(/\n?<!-- farm-cargo-production:[^\n]+ -->/g, '') + `\n<!-- farm-cargo-production:${JSON.stringify(record)} -->`;
  assert.ok(Buffer.byteLength(body) <= 60000, 'Cargo proposal body exceeds safe bound');
  execFileSync('gh', ['api', '--method', 'PATCH', `repos/${repository}/pulls/${pr.number}`, '--input', '-'], { cwd: root, input: JSON.stringify({ body }), encoding: 'utf8', timeout: 30000 });
}
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
  return liveProposal(proposals[0], baseSHA);
}
export function applyCargoProposal(proposal) {
  if (!proposal) return;
  assert.equal(git('rev-parse', 'HEAD').trim(), proposal.baseSHA, 'Version base moved');
  assert.match(proposal.headSHA, /^[a-f0-9]{40}$/);
  const repository = process.env.GITHUB_REPOSITORY;
  const current = gh(`repos/${repository}/pulls/${proposal.number}`);
  assert.deepEqual(liveProposal(current, proposal.baseSHA), proposal, 'Cargo proposal identity or producer moved');
  const plan = JSON.parse(fs.readFileSync(path.join(root, 'docs/releases/v2-candidate.json')));
  git('fetch', 'origin', proposal.headSHA);
  const files = git('diff', '--name-only', proposal.baseSHA, proposal.headSHA).trim().split(/\r?\n/).filter(Boolean);
  const beforeMetadata = JSON.parse(execFileSync('cargo', ['metadata', '--no-deps', '--locked', '--format-version', '1'], { cwd: root, encoding: 'utf8', timeout: 120000, maxBuffer: 16 * 1024 ** 2 }));
  const { manifests, consumers, allowed } = cargoProposalScope(plan, beforeMetadata, file => git('show', `${proposal.baseSHA}:${file}`), file => git('show', `${proposal.headSHA}:${file}`), root);
  assert.ok(files.length && files.every(file => allowed.has(file)), 'Cargo proposal exceeds selected SDK version/changelog scope');
  for (const file of files.filter(file => manifests.has(file))) assertVersionOnlyManifest(git('show', `${proposal.baseSHA}:${file}`), git('show', `${proposal.headSHA}:${file}`));
  for (const file of files.filter(file => consumers.has(file))) assert.equal(git('show', `${proposal.headSHA}:${file}`).replaceAll('\r\n', '\n'), consumers.get(file).after, `Consumer proposal changes more than dependency versions: ${file}`);
  const patch = git('diff', '--binary', proposal.baseSHA, proposal.headSHA, '--', ...files);
  execFileSync('git', ['apply', '--check', '-'], { cwd: root, input: patch, encoding: 'utf8' });
  execFileSync('git', ['apply', '-'], { cwd: root, input: patch, encoding: 'utf8' });
  for (const [file, value] of consumers) fs.writeFileSync(path.join(root, file), value.after);
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
  const consumerManifests = [...consumers].map(([file, value]) => ({ file, beforeSha256: textHash(value.before), afterSha256: textHash(value.after) }));
  fs.writeFileSync(path.join(root, 'docs/release-evidence/cargo-version-proposal.json'), JSON.stringify({ schemaVersion: 2, ...proposal, files, consumerManifests, versions: changed.map(({ name, version }) => ({ name, version })), requiredNpm, changeset }, null, 2) + '\n');
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === '--record-production') {
    recordCargoProduction(JSON.parse(process.env.FARM_CARGO_ACTION_PR ?? 'null'));
  } else if (process.argv[2] === '--seed') {
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
