// cspell:words ustar
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { gzipSync } from 'node:zlib';
import { archiveFiles, sha256 } from './release-archive.mjs';
import { assertCandidateCheckout, assertNativeAssembly, assertSourceLicense, assertProvenanceRepository, nativePackManifest, packNpmCandidates } from './pack-v2-npm-candidates.mjs';
test('native producer rejects absent, duplicate or different-platform native cohorts', () => {
  const plan = { npm: { payloads: [{ name: 'core-linux', version: '2.0.0', platform: 'linux-x64-gnu' }] } };
  const assembly = { sourceCommit: 'a'.repeat(40), count: 1, records: [{ ...plan.npm.payloads[0], artifact: 'exact-artifact' }] };
  const artifactIndex = [{ name: 'exact-artifact', id: 1 }];
  assertNativeAssembly(plan, assembly, artifactIndex, assembly.sourceCommit);
  assert.throws(() => assertNativeAssembly(plan, assembly, [], assembly.sourceCommit));
  assert.throws(() => assertNativeAssembly(plan, assembly, [...artifactIndex, ...artifactIndex], assembly.sourceCommit));
  assert.throws(() => assertNativeAssembly(plan, assembly, artifactIndex, 'b'.repeat(40)));
});
test('source legal gate does not silently relabel ISC or borrow a root license', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-license-gate-'));
  try {
    assert.throws(() => assertSourceLicense(root, { name: 'generator', license: 'ISC' }));
    fs.writeFileSync(path.join(root, 'LICENSE'), 'MIT License\nPermission is hereby granted, free of charge');
    assertSourceLicense(root, { name: 'core', license: 'MIT' });
    assert.throws(() => assertSourceLicense(root, { name: 'generator', license: 'ISC' }), /ISC declaration differs/);
    assert.throws(() => assertSourceLicense(root, { name: 'unlabeled' }), /Missing license declaration/);
  } finally { fs.rmSync(root, { recursive: true }); }
});

test('PR candidate identity binds the actual checkout while retaining the distinct head identity', () => {
  const head = 'a'.repeat(40), merge = 'b'.repeat(40), base = 'c'.repeat(40);
  assertCandidateCheckout({ testedCommit: head, workflowSHA: head, sourceCommit: head, eventName: 'workflow_dispatch', parents: [base] });
  const current = { testedCommit: merge, workflowSHA: merge, sourceCommit: head, eventName: 'pull_request', parents: [base, head] };
  assertCandidateCheckout(current);
  for (const change of [
    { workflowSHA: head }, { sourceCommit: 'd'.repeat(40) }, { parents: [base] },
    { parents: [base, head, 'd'.repeat(40)] }, { eventName: 'workflow_dispatch' }
  ]) assert.throws(() => assertCandidateCheckout({ ...current, ...change }));
});

const repositoryRoot = 'https://github.com/farm-fe/farm';
function nativeFixture() {
  const owner = { name: 'create-farm', version: '2.0.0', manifestPath: 'packages/create-farm/package.json' };
  const pkg = { name: 'create-farm-linux-x64-gnu', version: owner.version, owner: owner.name,
    platform: 'linux-x64-gnu', manifestPath: 'packages/create-farm/npm/linux-x64-gnu/package.json' };
  const ownerManifest = { name: owner.name, version: owner.version, license: 'MIT', main: 'index.js',
    repository: { type: 'git', url: repositoryRoot, directory: 'packages/create-farm' },
    optionalDependencies: { [pkg.name]: pkg.version } };
  const manifest = { name: pkg.name, version: pkg.version, license: 'MIT', main: 'create-farm.node' };
  return { owner, pkg, ownerManifest, manifest };
}

test('provenance accepts only unambiguous Farm repository roots', () => {
  for (const url of [repositoryRoot, repositoryRoot + '.git', 'git+' + repositoryRoot, 'git+' + repositoryRoot + '.git']) {
    assertProvenanceRepository({ name: 'fixture', repository: url });
    assertProvenanceRepository({ name: 'fixture', repository: { type: 'git', url, directory: 'packages/fixture' } });
  }
  for (const repository of [undefined, null, [], {}, repositoryRoot + '/', 'farm-fe/farm',
    repositoryRoot + '/tree/main/packages/create-farm', repositoryRoot + '?query=1', repositoryRoot + '#main',
    'https://github.com/other/farm', 'https://github.com/farm-fe/farm-other',
    'https://github.com@evil.example/farm-fe/farm', { type: 'svn', url: repositoryRoot }]) {
    assert.throws(() => assertProvenanceRepository({ name: 'fixture', repository }));
  }
  assert.throws(() => assertProvenanceRepository({ name: 'fixture', repository: repositoryRoot, repositories: [repositoryRoot] }), /Ambiguous/);
});

test('native staging deterministically derives source repository without mutating source', () => {
  const { owner, pkg, ownerManifest, manifest } = nativeFixture();
  const before = structuredClone({ owner, pkg, ownerManifest, manifest });
  const staged = nativePackManifest(manifest, ownerManifest, pkg, owner);
  assert.deepEqual(staged.repository, { type: 'git', url: repositoryRoot, directory: 'packages/create-farm/npm/linux-x64-gnu' });
  assert.deepEqual(staged, nativePackManifest(manifest, ownerManifest, pkg, owner));
  assert.deepEqual({ owner, pkg, ownerManifest, manifest }, before);
  assert.deepEqual({ ...staged, repository: undefined }, { ...manifest, repository: undefined });
  assertProvenanceRepository(staged);
  for (const repository of [repositoryRoot, { type: 'git', url: 'git+' + repositoryRoot + '.git' },
    repositoryRoot + '/tree/main/packages/create-farm']) {
    assert.deepEqual(nativePackManifest({ ...manifest, repository }, ownerManifest, pkg, owner), staged);
  }
});

test('native staging rejects identity, version, owner, dependency and repository mismatches', () => {
  const value = nativeFixture();
  const apply = ({ manifest, ownerManifest, pkg, owner }) => nativePackManifest(manifest, ownerManifest, pkg, owner);
  for (const change of [
    { manifest: { name: 'other' } }, { manifest: { version: '2.0.1' } },
    { manifest: { repository: null } }, { manifest: { repositories: [] } },
    { manifest: { repository: 'https://github.com/other/farm' } },
    { manifest: { repository: repositoryRoot + '/tree/other/packages/create-farm' } },
    { manifest: { repository: { type: 'svn', url: repositoryRoot } } },
    { manifest: { repository: { url: repositoryRoot, directory: 'packages/other' } } },
    { ownerManifest: { name: 'other' } }, { ownerManifest: { version: '2.0.1' } },
    { ownerManifest: { optionalDependencies: {} } }, { ownerManifest: { repository: undefined } },
    { ownerManifest: { repository: 'https://github.com/other/farm' } },
    { ownerManifest: { repository: { url: repositoryRoot, directory: 'packages/other' } } },
    { pkg: { owner: 'other' } }, { pkg: { version: '2.0.1' } }, { pkg: { platform: '../linux-x64-gnu' } },
    { pkg: { manifestPath: 'packages/other/npm/linux-x64-gnu/package.json' } },
    { owner: { manifestPath: 'packages/../create-farm/package.json' } }
  ]) {
    const changed = structuredClone(value);
    for (const [key, fields] of Object.entries(change)) Object.assign(changed[key], fields);
    assert.throws(() => apply(changed), JSON.stringify(change));
  }
  assert.throws(() => apply({ ...value, owner: undefined }), /Missing native source owner/);
  const { manifest, ownerManifest, pkg, owner } = value;
  const otherName = '@farmfe/core';
  assert.throws(() => nativePackManifest({ ...manifest, repository: repositoryRoot + '/tree/main/packages/create-farm' },
    { ...ownerManifest, name: otherName }, { ...pkg, owner: otherName }, { ...owner, name: otherName }), /Native repository differs/);
});

// Deterministic stand-in for pnpm's tar writer, used only to exercise producer
// staging, actual archive inspection, sealing and fail-before-pack behavior.
function fixtureTar(files) {
  const parts = [];
  for (const [name, value] of Object.entries(files)) {
    const body = Buffer.from(value), header = Buffer.alloc(512);
    header.write('package/' + name); header.write('0000644\0', 100); header.write('0000000\0', 108); header.write('0000000\0', 116);
    header.write(body.length.toString(8).padStart(11, '0') + '\0', 124); header.write('00000000000\0', 136);
    header.fill(32, 148, 156); header[156] = 48; header.write('ustar', 257); header.write('00', 263);
    header.write([...header].reduce((sum, byte) => sum + byte, 0).toString(8).padStart(6, '0') + '\0 ', 148);
    parts.push(header, body, Buffer.alloc((-body.length % 512 + 512) % 512));
  }
  return gzipSync(Buffer.concat([...parts, Buffer.alloc(1024)]));
}
function packFixture(t, packedRepository) {
  t.mock.method(fs, 'statfsSync', () => ({ bavail: 4 * 1024 * 1024, bsize: 4096 }));
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-provenance-pack-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const write = (file, content) => { fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true }); fs.writeFileSync(path.join(root, file), content); };
  const value = nativeFixture(), { owner, pkg, ownerManifest, manifest } = value;
  const native = Buffer.alloc(64); Buffer.from([127, 69, 76, 70, 2, 1]).copy(native); native.writeUInt16LE(62, 18);
  write('package.json', JSON.stringify({ packageManager: 'pnpm@9.4.0' }));
  write(owner.manifestPath, JSON.stringify(ownerManifest)); write(pkg.manifestPath, JSON.stringify(manifest));
  write('packages/create-farm/LICENSE', 'MIT License\nPermission is hereby granted, free of charge');
  write('packages/create-farm/index.js', 'module.exports = {};'); write('native.node', native);
  write('fixture-pnpm/package.json', JSON.stringify({ name: 'pnpm', version: '9.4.0', bin: { pnpm: 'bin/pnpm.cjs' } }));
  write('fixture-pnpm/bin/pnpm.cjs', `const fs = require('node:fs'), path = require('node:path'), { gzipSync } = require('node:zlib');
    const fixtureTar = ${fixtureTar.toString()};
    const manifest = JSON.parse(fs.readFileSync('package.json'));
    ${packedRepository === undefined ? '' : 'manifest.repository = ' + JSON.stringify(packedRepository) + ';'}
    const files = { 'package.json': JSON.stringify(manifest), LICENSE: fs.readFileSync('LICENSE'), [manifest.main]: fs.readFileSync(manifest.main) };
    fs.writeFileSync(path.join(process.argv[process.argv.indexOf('--pack-destination') + 1], 'fixture.tgz'), fixtureTar(files));`);
  const plan = { abi: '0.5.4', npm: { primary: [owner], payloads: [pkg] } };
  const assembly = { sourceCommit: 'a'.repeat(40), count: 1, records: [{ ...pkg, file: 'native.node', sha256: sha256(native), artifact: 'fixture-native' }] };
  return { ...value, root, write, native, options: { root, plan, assembly, artifactIndex: [{ name: 'fixture-native', id: 1 }],
    sourceCommit: assembly.sourceCommit, testedCommit: assembly.sourceCommit, testedTree: 'b'.repeat(40), runId: 10,
    output: path.join(root, 'output'), pnpm: path.join(root, 'fixture-pnpm/bin/pnpm.cjs') } };
}

test('producer seals repository-complete archives and preserves native and source bytes', t => {
  const fixture = packFixture(t), { root, manifest, ownerManifest, options, native } = fixture;
  const result = packNpmCandidates(options);
  assert.equal(result.status, 'sealed'); assert.equal(result.records.length, 2);
  for (const row of result.records) {
    const bytes = fs.readFileSync(path.join(options.output, row.file));
    assert.equal(sha256(bytes), row.sha256);
    const files = archiveFiles(bytes), packed = JSON.parse(files.get('package/package.json'));
    assertProvenanceRepository(packed);
    if (row.name === manifest.name) {
      assert.deepEqual(packed.repository, { type: 'git', url: repositoryRoot, directory: 'packages/create-farm/npm/linux-x64-gnu' });
      assert.deepEqual(files.get('package/' + manifest.main), native); assert.deepEqual(packed.libc, ['glibc']);
    }
  }
  assert.equal(fs.readFileSync(path.join(root, fixture.pkg.manifestPath), 'utf8'), JSON.stringify(manifest));
  assert.equal(fs.readFileSync(path.join(root, fixture.owner.manifestPath), 'utf8'), JSON.stringify(ownerManifest));
  const original = result.records.map(row => fs.readFileSync(path.join(options.output, row.file)));
  assert.throws(() => packNpmCandidates(options), /EEXIST/);
  assert.deepEqual(result.records.map(row => fs.readFileSync(path.join(options.output, row.file))), original);
});

test('native repository mismatch aborts the whole producer before any primary packing', t => {
  const fixture = packFixture(t);
  fixture.write(fixture.pkg.manifestPath, JSON.stringify({ ...fixture.manifest, repository: 'https://github.com/other/farm' }));
  assert.throws(() => packNpmCandidates(fixture.options), /Native repository differs/);
  assert.deepEqual(fs.readdirSync(fixture.options.output), ['receipt.json']);
  const receipt = JSON.parse(fs.readFileSync(path.join(fixture.options.output, 'receipt.json')));
  assert.deepEqual(receipt.records, []); assert.equal(receipt.status, 'incomplete');
});

test('producer checks packed metadata again before recording or sealing archive bytes', t => {
  const fixture = packFixture(t, 'https://github.com/other/farm');
  assert.throws(() => packNpmCandidates(fixture.options), /Provenance repository/);
  const receipt = JSON.parse(fs.readFileSync(path.join(fixture.options.output, 'receipt.json')));
  assert.deepEqual(receipt.records, []); assert.equal(receipt.status, 'incomplete');
  assert.deepEqual(fs.readdirSync(fixture.options.output).filter(file => file.endsWith('.tgz')), []);
});

test('source inventory has repository-complete primary and staged native metadata', () => {
  const root = new URL('../', import.meta.url);
  const read = file => JSON.parse(fs.readFileSync(new URL(file, root)));
  const plan = read('docs/releases/v2-candidate.json');
  for (const pkg of plan.npm.primary) assertProvenanceRepository(read(pkg.manifestPath));
  for (const pkg of plan.npm.payloads) {
    const owner = plan.npm.primary.find(row => row.name === pkg.owner);
    const manifest = read(pkg.manifestPath), before = structuredClone(manifest);
    const staged = nativePackManifest(manifest, read(owner.manifestPath), pkg, owner);
    assertProvenanceRepository(staged); assert.deepEqual(manifest, before);
  }
});

test('payload-only selection derives metadata from the full-family source owner', t => {
  const fixture = packFixture(t);
  const manifest = { ...fixture.manifest, repository: repositoryRoot + '/tree/main/packages/create-farm' };
  fixture.write(fixture.pkg.manifestPath, JSON.stringify(manifest));
  fixture.options.plan.publication = { npm: { primary: [], payloads: [fixture.pkg] } };
  const result = packNpmCandidates(fixture.options);
  assert.equal(result.status, 'sealed'); assert.deepEqual(result.records.map(row => row.name), [fixture.pkg.name]);
  const bytes = fs.readFileSync(path.join(fixture.options.output, result.records[0].file));
  assertProvenanceRepository(JSON.parse(archiveFiles(bytes).get('package/package.json')));
  assert.equal(fs.readFileSync(path.join(fixture.root, fixture.pkg.manifestPath), 'utf8'), JSON.stringify(manifest));
});
