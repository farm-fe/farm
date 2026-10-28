import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import test from 'node:test';
import { gzipSync } from 'node:zlib';
import { archiveFiles, inspectNpmArchive, nativePlatformMatches, sha256 } from './release-archive.mjs';
import { verifyCandidateSeal } from './verify-v2-candidate-seal.mjs';
import { publicationOrder, verifyExistingNpm } from './release.mjs';
import crypto from 'node:crypto';
import { assertStableReleaseState } from './check-stable-release-state.mjs';

function tar(files) {
  const chunks = [];
  for (const [name, text] of Object.entries(files)) {
    const bytes = Buffer.from(text);
    const header = Buffer.alloc(512);
    header.write(name);
    header.write(bytes.length.toString(8).padStart(11, '0') + '\0', 124);
    header.fill(32, 148, 156);
    header[156] = 48;
    header.write([...header].reduce((sum, byte) => sum + byte, 0).toString(8).padStart(6, '0') + '\0 ', 148);
    chunks.push(header, bytes, Buffer.alloc((512 - bytes.length % 512) % 512));
  }
  return gzipSync(Buffer.concat([...chunks, Buffer.alloc(1024)]));
}
const expected = { name: 'example-candidate', version: '2.0.0', manifestPath: 'pkg/package.json' };
test('archive uses normal-pack bin, wildcard and stable dependency contracts', () => {
  const manifest = { ...expected, license: 'MIT', bin: { example: './bin/run.js' }, exports: { './*': './dist/*.js' } };
  const files = { 'package/package.json': JSON.stringify(manifest), 'package/LICENSE': 'MIT License', 'package/bin/run.js': 'run', 'package/dist/a.js': 'export {}' };
  assert.deepEqual(inspectNpmArchive(tar(files), expected).issues, []);
  for (const missing of ['package/bin/run.js', 'package/dist/a.js', 'package/LICENSE']) {
    const broken = { ...files }; delete broken[missing];
    assert.ok(inspectNpmArchive(tar(broken), expected).issues.length, missing);
  }
  for (const dependencies of [{ internal: 'workspace:*' }, { '@farmfe/core': '2.0.0-beta.1' }]) {
    assert.ok(inspectNpmArchive(tar({ ...files, 'package/package.json': JSON.stringify({ ...manifest, dependencies }) }), expected).issues.length);
  }
});
test('existing npm versions are reusable only when exact sealed bytes match registry integrity', () => {
  const bytes = Buffer.from('sealed');
  const metadata = { name: expected.name, version: expected.version, dist: { integrity: `sha512-${crypto.createHash('sha512').update(bytes).digest('base64')}` } };
  assert.equal(verifyExistingNpm(expected, bytes, null), false);
  assert.equal(verifyExistingNpm(expected, bytes, metadata), true);
  assert.throws(() => verifyExistingNpm(expected, Buffer.from('other'), metadata), /Immutable/);
  assert.throws(() => verifyExistingNpm(expected, bytes, { ...metadata, name: 'other' }), /identity/);
});
const base = { 'package/package.json': JSON.stringify({ name: expected.name, version: expected.version, main: 'index.js', license: 'MIT' }), 'package/index.js': 'export default 42;', 'package/LICENSE': 'Existing license text\n' };

test('inspects actual package identity, entries, dependencies and legal bytes', () => {
  assert.deepEqual(inspectNpmArchive(tar(base), expected).issues, []);
  const bad = { ...base, 'package/package.json': JSON.stringify({ name: expected.name, version: expected.version, main: 'missing.js', dependencies: { dep: 'workspace:*' } }) };
  const issues = inspectNpmArchive(tar(bad), expected).issues;
  assert.ok(issues.some(issue => issue.includes('workspace')));
  assert.ok(issues.some(issue => issue.includes('missing.js')));
  assert.throws(() => archiveFiles(tar({ '../escape': 'bad' })), /Unsafe/);
  assert.throws(() => inspectNpmArchive(tar(base), { ...expected, version: '3.0.0' }), /identity/);
});
test('a complete valid archive set can pass; missing, changed and unlicensed packages fail', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-seal-test-'));
  try {
    fs.mkdirSync(path.join(directory, 'pkg'));
    fs.writeFileSync(path.join(directory, 'pkg/LICENSE'), base['package/LICENSE']);
    const bytes = tar(base);
    fs.writeFileSync(path.join(directory, 'pkg.tgz'), bytes);
    const plan = { npm: { primary: [expected], payloads: [] } };
    const receipt = { records: [{ ...expected, file: 'pkg.tgz', sha256: sha256(bytes) }] };
    assert.equal(verifyCandidateSeal(plan, receipt, directory, directory).candidateArchivesVerified, true);
    assert.equal(verifyCandidateSeal(plan, { records: [] }, directory, directory).candidateArchivesVerified, false);
    fs.writeFileSync(path.join(directory, 'pkg/LICENSE'), 'Changed source license');
    assert.equal(verifyCandidateSeal(plan, receipt, directory, directory).candidateArchivesVerified, false);
    fs.writeFileSync(path.join(directory, 'pkg.tgz'), 'corrupted');
    assert.equal(verifyCandidateSeal(plan, receipt, directory, directory).errors.length, 1);
  } finally { fs.rmSync(directory, { recursive: true }); }
});
test('native headers must match the advertised OS and architecture', () => {
  const bytes = Buffer.alloc(128);
  bytes.write('MZ'); bytes.writeUInt32LE(64, 60); bytes.write('PE\0\0', 64); bytes.writeUInt16LE(0x8664, 68);
  assert.equal(nativePlatformMatches(bytes, 'win32-x64-msvc'), true);
  assert.equal(nativePlatformMatches(bytes, 'win32-arm64-msvc'), false);
  assert.equal(nativePlatformMatches(bytes, 'linux-x64-gnu'), false);
});
test('publication orders sealed native payloads and runtime prerequisites before consumers', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-publish-order-'));
  try {
    const records = [];
    for (const [name, dependencies] of [['consumer', { runtime: '2.0.0' }], ['runtime', {}]]) {
      const file = name + '.tgz';
      fs.writeFileSync(path.join(directory, file), tar({ 'package/package.json': JSON.stringify({ name, version: '2.0.0', dependencies }) }));
      records.push({ name, file });
    }
    records.push({ name: 'native', file: 'native.tgz' });
    const plan = { npm: { primary: [{ name: 'consumer' }, { name: 'runtime' }], payloads: [{ name: 'native' }] }, counts: { total: 3 } };
    assert.deepEqual(publicationOrder(plan, records, directory).map(row => row.name), ['native', 'runtime', 'consumer']);
  } finally { fs.rmSync(directory, { recursive: true }); }
});
test('stable entry rejects beta state and unconsumed version changes before publishing', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-stable-gate-'));
  try {
    const changesets = path.join(directory, '.changeset'); fs.mkdirSync(changesets);
    fs.writeFileSync(path.join(changesets, 'pre.json'), '{"mode":"pre","tag":"beta"}');
    assert.throws(() => assertStableReleaseState(directory), /prerelease state/);
    fs.unlinkSync(path.join(changesets, 'pre.json')); fs.writeFileSync(path.join(changesets, 'pending.md'), 'reviewed version required');
    assert.throws(() => assertStableReleaseState(directory), /1 unconsumed/);
    fs.unlinkSync(path.join(changesets, 'pending.md')); assert.doesNotThrow(() => assertStableReleaseState(directory));
  } finally { fs.rmSync(directory, { recursive: true }); }
});
