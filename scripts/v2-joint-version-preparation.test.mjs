import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import crypto from 'node:crypto';
import { assertVersionOnlyManifest, assertNewRustVersion, cargoChangeset, synchronizeNativePayloads, validateCargoPreparation, cargoProposalScope, updateConsumerDependencyVersions, affectedCargoConsumers, validateCargoProduction } from './prepare-cargo-version-proposal.mjs';
import { changesetsPublication } from './v2-release-inventory.mjs';

test('one real Core patch Changeset generates an official owner bump then synchronizes platform wrappers', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-official-version-'));
  try {
    const cliDirectory = path.dirname(fs.realpathSync(fileURLToPath(new URL('../node_modules/@changesets/cli/package.json', import.meta.url))));
    const cli = JSON.parse(fs.readFileSync(path.join(cliDirectory, 'package.json')));
    const changelog = path.resolve(cliDirectory, cli.exports['./changelog'].default);
    const write = (file, value) => { const target = path.join(directory, file); fs.mkdirSync(path.dirname(target), { recursive: true }); fs.writeFileSync(target, typeof value === 'string' ? value : JSON.stringify(value)); };
    write('package.json', { name: 'version-fixture', private: true, workspaces: ['packages/*'] });
    write('.changeset/config.json', { changelog, commit: false, access: 'public', baseBranch: 'main', updateInternalDependencies: 'patch', ignore: [] });
    write('.changeset/core-patch.md', '---\n"@farmfe/core": patch\n---\n\nCore patch fixture.\n');
    write('packages/core/package.json', { name: '@farmfe/core', version: '2.0.0', optionalDependencies: { '@farmfe/core-linux-x64-gnu': '2.0.0' } });
    write('packages/payload/package.json', { name: '@farmfe/core-linux-x64-gnu', version: '2.0.0' });
    // The official default generator uses real Git provenance for its notes.
    execFileSync('git', ['init', '--quiet', '--initial-branch=main', directory], { timeout: 10000 });
    execFileSync('git', ['add', '.'], { cwd: directory, timeout: 10000 });
    execFileSync('git', ['-c', 'user.name=Version fixture', '-c', 'user.email=fixture@example.invalid', '-c', 'commit.gpgsign=false', 'commit', '--quiet', '-m', 'test: seed Changesets fixture'], { cwd: directory, timeout: 10000 });
    execFileSync(process.execPath, [path.resolve(cliDirectory, typeof cli.bin === 'string' ? cli.bin : Object.values(cli.bin)[0]), 'version'], { cwd: directory, timeout: 30000, encoding: 'utf8' });
    const read = file => JSON.parse(fs.readFileSync(path.join(directory, file)));
    assert.equal(read('packages/core/package.json').version, '2.0.1');
    assert.equal(read('packages/payload/package.json').version, '2.0.0');
    synchronizeNativePayloads(directory, { npm: { primary: [{ name: '@farmfe/core', version: '2.0.0', manifestPath: 'packages/core/package.json' }], payloads: [{ name: '@farmfe/core-linux-x64-gnu', owner: '@farmfe/core', manifestPath: 'packages/payload/package.json' }] } });
    assert.equal(read('packages/payload/package.json').version, '2.0.1');
    assert.equal(read('packages/core/package.json').optionalDependencies['@farmfe/core-linux-x64-gnu'], '2.0.1');
    assert.ok(!fs.existsSync(path.join(directory, '.changeset/core-patch.md')));
    assert.ok(fs.readFileSync(path.join(directory, 'packages/core/CHANGELOG.md'), 'utf8').includes('## 2.0.1'));
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});

test('Cargo proposal permits stable version edits but rejects source or dependency rewrites', () => {
  const before = '[package]\nname = "example"\nversion = "2.2.0"\n[dependencies]\nother = { version = "2.1.0", path = "../other" }\n';
  assertVersionOnlyManifest(before, before.replace('2.2.0', '2.2.1').replace('2.1.0', '2.1.1'));
  assert.throws(() => assertVersionOnlyManifest(before, before.replace('../other', '../unreviewed')), /more than stable version/);
  assert.throws(() => assertVersionOnlyManifest(before, before.replace('2.2.0', '2.2.1-beta.0')), /more than stable version/);
  assert.equal(cargoChangeset(['@farmfe/core', '@farmfe/plugin-example', '@farmfe/core']).match(/"@farmfe\/core": patch/g).length, 1);
  assert.throws(() => cargoChangeset(['@farmfe/plugin-example']));
  assertNewRustVersion('2.2.0', '2.2.1');
  assert.throws(() => assertNewRustVersion('2.2.0', '2.1.9'), /advance/);
  assert.throws(() => assertNewRustVersion('2.2.0', '3.0.0'), /major version/);
});

test('real Core to utils exact dependency can advance without changing its constraint policy', () => {
  const source = fs.readFileSync(new URL('../crates/core/Cargo.toml', import.meta.url), 'utf8');
  const pattern = /(farmfe_utils\s*=\s*\{\s*version\s*=\s*")=(\d+)\.(\d+)\.(\d+)(")/;
  assert.match(source, pattern, 'Fixture must exercise the actual exact Core to utils dependency');
  const upgraded = source.replace(pattern, (_, start, major, minor, patch, end) => `${start}=${major}.${minor}.${Number(patch) + 1}${end}`);
  assert.notEqual(upgraded, source);
  assertVersionOnlyManifest(source, upgraded);
  for (const prefix of ['^', '~', '']) {
    assert.throws(() => assertVersionOnlyManifest(source, upgraded.replace(pattern, (_, start, major, minor, patch, end) => `${start}${prefix}${major}.${minor}.${patch}${end}`)), /more than stable version/);
  }
  for (const prefix of ['', '=', '^', '~']) {
    const before = `[dependencies]\nutils = { version = "${prefix}2.1.1", path = "../utils" }\n`;
    assertVersionOnlyManifest(before, before.replace('2.1.1', '2.1.2'));
    for (const changed of ['', '=', '^', '~'].filter(value => value !== prefix)) {
      assert.throws(() => assertVersionOnlyManifest(before, before.replace(`${prefix}2.1.1`, `${changed}2.1.2`)), /more than stable version/);
    }
  }
});

test('joint Cargo receipt binds exact proposal files and rejects source, stale base and unversioned consumers', () => {
  const names = ['farmfe_compiler', 'farmfe_core', 'farmfe_ecosystem_glob', 'farmfe_toolkit'];
  const manifestPaths = { farmfe_compiler: 'crates/compiler/Cargo.toml', farmfe_core: 'crates/core/Cargo.toml', farmfe_ecosystem_glob: 'rust-ecosystem/glob/Cargo.toml', farmfe_toolkit: 'crates/toolkit/Cargo.toml' };
  const metadata = { packages: names.map(name => ({ name, version: name === 'farmfe_core' ? '2.2.1' : '2.2.0', manifest_path: path.resolve(manifestPaths[name]), dependencies: [], publish: null })) };
  for (const [name, manifest] of [['farmfe_node', 'crates/node/Cargo.toml'], ['farmfe_plugin_react', 'rust-plugins/react/Cargo.toml']]) metadata.packages.push({ name, version: '0.0.0', manifest_path: path.resolve(manifest), dependencies: [{ name: 'farmfe_core', path: path.resolve('crates/core'), kind: null }], publish: [] });
  const baseline = { abi: '0.5.4', npm: { primary: [{ name: '@farmfe/core', manifestPath: 'packages/core/package.json' }, { name: '@farmfe/plugin-react', manifestPath: 'rust-plugins/react/package.json' }], payloads: [{ owner: '@farmfe/core' }, { owner: '@farmfe/plugin-react' }] }, cargo: names.map(name => ({ name, version: '2.2.0', manifestPath: manifestPaths[name] })) };
  const beforeFiles = Object.fromEntries(names.map(name => [manifestPaths[name], `[package]\nname = "${name}"\nversion = "2.2.0"\n`]));
  beforeFiles['Cargo.toml'] = '[workspace]\nmembers = []\n';
  for (const file of ['crates/node/Cargo.toml', 'rust-plugins/react/Cargo.toml']) {
    const actual = fs.readFileSync(new URL('../' + file, import.meta.url), 'utf8');
    const coreDependency = /(farmfe_core\s*=\s*\{\s*version\s*=\s*")=\d+\.\d+\.\d+(")/;
    assert.match(actual, coreDependency);
    beforeFiles[file] = actual.replace(coreDependency, '$1=2.2.0$2');
  }
  const proposedFiles = { ...beforeFiles, 'crates/core/Cargo.toml': beforeFiles['crates/core/Cargo.toml'].replace('2.2.0', '2.2.1') };
  const scope = cargoProposalScope(baseline, metadata, file => beforeFiles[file], file => proposedFiles[file]);
  assert.deepEqual([...scope.consumers.keys()], ['crates/node/Cargo.toml', 'rust-plugins/react/Cargo.toml']);
  const hash = text => crypto.createHash('sha256').update(text.replaceAll('\r\n', '\n')).digest('hex');
  const requiredNpm = ['@farmfe/core', '@farmfe/plugin-react'];
  const record = { schemaVersion: 2, number: 7, baseSHA: 'a'.repeat(40), headSHA: 'b'.repeat(40), branch: 'release-plz-example', files: ['crates/core/Cargo.toml'], versions: [{ name: 'farmfe_core', version: '2.2.1' }], requiredNpm, changeset: cargoChangeset(requiredNpm), consumerManifests: [...scope.consumers].map(([file, value]) => ({ file, beforeSha256: hash(value.before), afterSha256: hash(value.after) })) };
  record.production = { schemaVersion: 1, workflowPath: '.github/workflows/release-plz.yaml', sourceSHA: record.baseSHA, proposalHeadSHA: record.headSHA, number: record.number, branch: record.branch, runId: 44, runAttempt: 1 };
  const run = { id: 44, run_attempt: 1, path: record.production.workflowPath, repository: { full_name: 'farm-fe/farm' }, head_sha: record.baseSHA, head_branch: 'main', event: 'push', status: 'completed', conclusion: 'success' };
  const options = { baseSHA: record.baseSHA, repository: 'farm-fe/farm', isAncestor: () => true, readProducerRun: () => run, changedFiles: () => record.files, readGitFile: (sha, file) => (sha === record.baseSHA ? beforeFiles : proposedFiles)[file],
    readCurrent: file => file.endsWith('lib.rs') ? 'pub const VERSION: &str = "0.5.4";' : scope.consumers.get(file)?.after ?? proposedFiles[file] };
  assert.deepEqual(validateCargoPreparation(record, baseline, metadata, options), [...record.files, ...scope.consumers.keys()]);
  assert.equal(baseline.cargo.length, 4, 'Private native consumers are not publication records');
  for (const value of scope.consumers.values()) {
    assert.ok(value.after.includes('version = "=2.2.1"'));
    assert.equal(value.before.match(/^version = .+$/m)[0], value.after.match(/^version = .+$/m)[0], 'Private package version is preserved');
  }
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, baseSHA: 'c'.repeat(40) }));
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, changedFiles: () => ['crates/core/src/lib.rs'] }), /file set drift/);
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, readCurrent: () => 'changed' }), /content differs/);
  assert.throws(() => validateCargoPreparation({ ...record, requiredNpm: [] }, baseline, metadata, options));
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, isAncestor: () => false }), /does not contain/);
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, readProducerRun: () => ({ ...run, path: '.github/workflows/ci.yaml' }) }));
  assert.throws(() => validateCargoProduction(record.production, { ...run, head_sha: 'c'.repeat(40) }, record, 'farm-fe/farm'));
  for (const change of [{ path: '.github/workflows/ci.yaml' }, { event: 'pull_request' }, { head_branch: 'feature' }, { run_attempt: 2 }, { conclusion: 'failure' }, { repository: { full_name: 'other/farm' } }]) assert.throws(() => validateCargoProduction(record.production, { ...run, ...change }, record, 'farm-fe/farm'));
  const nodeFile = 'crates/node/Cargo.toml';
  assert.throws(() => validateCargoPreparation(record, baseline, metadata, { ...options, readCurrent: file => file === nodeFile ? scope.consumers.get(file).after.replace(/^version = "[^"]+"/m, 'version = "99.0.0"') : options.readCurrent(file) }), /Consumer dependency update differs/);
  const changedFiles = [...record.files, nodeFile];
  assert.throws(() => validateCargoPreparation({ ...record, files: changedFiles }, baseline, metadata, { ...options, changedFiles: () => changedFiles,
    readGitFile: (sha, file) => sha === record.headSHA && file === nodeFile ? scope.consumers.get(file).after.replace('=2.2.1', '^2.2.1') : options.readGitFile(sha, file) }), /Consumer proposal changes more than dependency versions/);
});

test('consumer dependency-only rewrite preserves private metadata, prefixes, unrelated dependencies and reverse edges', () => {
  const pkg = { name: 'private', dependencies: [{ name: 'core', path: path.resolve('core') }] };
  const before = '[package]\nname = "private"\nversion = "0.0.0"\n[dependencies]\ncore = { version = "=2.2.0", path = "../core" }\nother = { version = "1.0.0" }\n';
  const after = updateConsumerDependencyVersions(before, pkg, new Map([['core', '2.2.1']]));
  assert.equal(after, before.replace('=2.2.0', '=2.2.1'));
  const table = '[dependencies.core]\nversion = "~2.2.0"\npath = "../core"\n';
  assert.equal(updateConsumerDependencyVersions(table, pkg, new Map([['core', '2.2.1']])), table.replace('~2.2.0', '~2.2.1'));
  const metadata = { packages: [{ name: 'core', manifest_path: path.resolve('core/Cargo.toml'), dependencies: [] }, { name: 'bridge', manifest_path: path.resolve('bridge/Cargo.toml'), dependencies: [{ name: 'core', path: path.resolve('core') }] }, { name: 'node', manifest_path: path.resolve('node/Cargo.toml'), dependencies: [{ name: 'bridge', path: path.resolve('bridge') }] }, { name: 'unrelated', manifest_path: path.resolve('unrelated/Cargo.toml'), dependencies: [] }] };
  assert.deepEqual([...affectedCargoConsumers(metadata, ['core'])].sort(), ['bridge', 'core', 'node']);
});

test('official owner bump synchronizes every native manifest and exact optional dependency only for changed owners', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-version-contract-'));
  try {
    const write = (file, value) => fs.writeFileSync(path.join(directory, file), JSON.stringify(value));
    write('core.json', { name: '@farmfe/core', version: '2.0.1', optionalDependencies: { payload: '2.0.0', payload2: '2.0.0' } });
    write('payload.json', { name: 'payload', version: '2.0.0' });
    write('payload2.json', { name: 'payload2', version: '2.0.0' });
    write('js.json', { name: 'js', version: '1.0.0' });
    const originalJs = fs.readFileSync(path.join(directory, 'js.json'));
    const baseline = { npm: { primary: [{ name: '@farmfe/core', version: '2.0.0', manifestPath: 'core.json' }, { name: 'js', version: '1.0.0', manifestPath: 'js.json' }], payloads: [
      { owner: '@farmfe/core', name: 'payload', manifestPath: 'payload.json' }, { owner: '@farmfe/core', name: 'payload2', manifestPath: 'payload2.json' }
    ] } };
    synchronizeNativePayloads(directory, baseline);
    const read = file => JSON.parse(fs.readFileSync(path.join(directory, file)));
    assert.deepEqual(read('core.json').optionalDependencies, { payload: '2.0.1', payload2: '2.0.1' });
    assert.equal(read('payload.json').version, '2.0.1'); assert.equal(read('payload2.json').version, '2.0.1');
    assert.deepEqual(fs.readFileSync(path.join(directory, 'js.json')), originalJs);
    write('payload.json', { name: 'wrong', version: '2.0.0' });
    assert.throws(() => synchronizeNativePayloads(directory, baseline), /identity changed/);
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});

test('ABI changes cannot reuse an old plugin binary; independent Node-API generator stays outside this cohort', () => {
  const old = { abi: '0.5.4', npm: { primary: ['@farmfe/core', '@farmfe/plugin-example', 'create-farm'].map(name => ({ name, version: '2.0.0' })), payloads: ['@farmfe/core', '@farmfe/plugin-example', 'create-farm'].map(owner => ({ name: owner + '-platform', owner, version: '2.0.0' })) }, cargo: [] };
  const next = structuredClone(old); next.abi = '0.5.5'; next.npm.primary[0].version = '2.0.1'; next.npm.payloads[0].version = '2.0.1';
  const official = [{ name: '@farmfe/core', newVersion: '2.0.1' }];
  assert.throws(() => changesetsPublication(next, old, official), /ABI change requires a new native owner version/);
  next.npm.primary[1].version = '2.0.1'; next.npm.payloads[1].version = '2.0.1'; official.push({ name: '@farmfe/plugin-example', newVersion: '2.0.1' });
  const delta = changesetsPublication(next, old, official);
  assert.equal(delta.counts.total, 4); assert.ok(!delta.npm.primary.some(row => row.name === 'create-farm'));
});
