import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { snapshotUnpublishedManifests, restoreUnchangedManifestFormatting } from './changesets-manifest-format.mjs';

function fixture(t) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-manifest-format-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const put = (file, text) => { fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true }); fs.writeFileSync(path.join(root, file), text); };
  const read = file => fs.readFileSync(path.join(root, file), 'utf8');
  const files = ['examples/docs-features/package.json', 'examples/docs-minimal/package.json', 'packages/core/package.json', 'packages/core/npm/darwin/package.json'];
  for (const file of files) put(file, '{"name":"' + file + '", "version":"1.0.0", "private":true}\n');
  execFileSync('git', ['init', '-q', root]);
  execFileSync('git', ['add', '.'], { cwd: root });
  const baseline = { npm: { primary: [{ manifestPath: files[2] }], payloads: [{ manifestPath: files[3] }] } };
  const snapshots = snapshotUnpublishedManifests(root, baseline);
  return { root, put, read, files, snapshots };
}

test('restores formatting-only example rewrites byte-for-byte, preserving published manifests', t => {
  const f = fixture(t), originals = f.files.map(f.read);
  for (const file of f.files) f.put(file, JSON.stringify(JSON.parse(f.read(file)), null, 2) + '\n');
  assert.deepEqual(restoreUnchangedManifestFormatting(f.root, f.snapshots), f.files.slice(0, 2));
  f.files.forEach((file, i) => i < 2 ? assert.equal(f.read(file), originals[i]) : assert.notEqual(f.read(file), originals[i]));
  assert.deepEqual(restoreUnchangedManifestFormatting(f.root, f.snapshots), []);
});

test('semantic dependency, script and version changes are not erased', t => {
  const f = fixture(t);
  for (const [i, file] of f.files.slice(0, 2).entries()) {
    const value = JSON.parse(f.read(file));
    if (i === 0) value.dependencies = { '@farmfe/core': '2.0.1' };
    else { value.version = '2.0.1'; value.scripts = { build: 'changed' }; }
    f.put(file, JSON.stringify(value));
  }
  assert.deepEqual(restoreUnchangedManifestFormatting(f.root, f.snapshots), []);
  assert.equal(JSON.parse(f.read(f.files[0])).dependencies['@farmfe/core'], '2.0.1');
  assert.equal(JSON.parse(f.read(f.files[1])).scripts.build, 'changed');
});

test('does not overwrite deletions, new files, symlinks or mode changes', t => {
  const f = fixture(t);
  fs.unlinkSync(path.join(f.root, f.files[0]));
  fs.symlinkSync(path.join(f.root, f.files[2]), path.join(f.root, f.files[0]));
  f.put(f.files[1], JSON.stringify(JSON.parse(f.read(f.files[1])), null, 2));
  fs.chmodSync(path.join(f.root, f.files[1]), 0o755);
  f.put('new/package.json', '{}');
  assert.deepEqual(restoreUnchangedManifestFormatting(f.root, f.snapshots), []);
  assert.ok(fs.lstatSync(path.join(f.root, f.files[0])).isSymbolicLink());
  assert.equal(f.read('new/package.json'), '{}');
  fs.unlinkSync(path.join(f.root, f.files[1]));
  assert.deepEqual(restoreUnchangedManifestFormatting(f.root, f.snapshots), []);
});

test('invalid generated JSON fails closed', t => {
  const f = fixture(t);
  f.put(f.files[0], '{bad');
  assert.throws(() => restoreUnchangedManifestFormatting(f.root, f.snapshots), SyntaxError);
  assert.equal(f.read(f.files[0]), '{bad');
});
