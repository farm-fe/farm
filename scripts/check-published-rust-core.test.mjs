import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';
import { assertPublishedRustCore, checkPublishedRustCore, checkBeforeStableNpmRelease, readRustCoreAbiFromArchive } from './check-published-rust-core.mjs';

function archive(abi = '0.5.4') {
  const content = Buffer.from(`pub const VERSION: &str = "${abi}";\n`);
  const header = Buffer.alloc(512);
  header.write('farmfe_core-2.2.0/src/lib.rs');
  header.write(content.length.toString(8).padStart(11, '0'), 124);
  header[156] = 48;
  return gzipSync(Buffer.concat([header, content, Buffer.alloc(512 - content.length), Buffer.alloc(1024)]));
}

async function fixture(t, bytes = archive()) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-publication-test-'));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  await fs.mkdir(path.join(root, 'crates/core/src'), { recursive: true });
  await fs.writeFile(path.join(root, 'crates/core/Cargo.toml'), '[package]\nversion = "2.2.0"\n');
  await fs.writeFile(path.join(root, 'crates/core/src/lib.rs'), 'pub const VERSION: &str = "0.5.4";\n');
  const metadata = { version: { num: '2.2.0', yanked: false, checksum: createHash('sha256').update(bytes).digest('hex') }, crate: { max_stable_version: '9.0.0' } };
  const calls = [];
  const fetchImpl = async url => { calls.push(url); return new Response(url.endsWith('/download') ? bytes : JSON.stringify(metadata)); };
  return { root, metadata, calls, fetchImpl };
}

test('checks the intended Cargo publication even when a newer stable version exists', async t => {
  const f = await fixture(t);
  const evidence = await checkPublishedRustCore(f);
  assert.equal(evidence.publishedVersion, '2.2.0');
  assert.equal(evidence.publishedAbi, '0.5.4');
  assert.deepEqual(f.calls, ['https://crates.io/api/v1/crates/farmfe_core/2.2.0', 'https://crates.io/api/v1/crates/farmfe_core/2.2.0/download']);
});

test('rejects a missing or yanked target before using an archive', async t => {
  const f = await fixture(t);
  await assert.rejects(checkPublishedRustCore({ ...f, fetchImpl: async () => new Response('', { status: 404 }) }), /HTTP 404/);
  f.metadata.version.yanked = true;
  await assert.rejects(checkPublishedRustCore(f), /yanked/);
  assert.equal(f.calls.length, 1);
});

test('verifies registry checksum and the archive ABI independently', async t => {
  const f = await fixture(t);
  f.metadata.version.checksum = '0'.repeat(64);
  await assert.rejects(checkPublishedRustCore(f), /checksum mismatch/);
  const incompatible = await fixture(t, archive('0.5.0'));
  await assert.rejects(checkPublishedRustCore(incompatible), /ABI 0\.5\.0; expected 0\.5\.4/);
});

test('requires complete matching publication evidence and an actual crate source member', () => {
  const evidence = { expectedVersion: '2.2.0', expectedAbi: '0.5.4', publishedVersion: '2.2.0', publishedAbi: '0.5.4' };
  assert.doesNotThrow(() => assertPublishedRustCore(evidence));
  for (const field of Object.keys(evidence)) assert.throws(() => assertPublishedRustCore({ ...evidence, [field]: undefined }), /missing/);
  assert.throws(() => assertPublishedRustCore({ ...evidence, publishedVersion: '3.0.0' }), /version/);
  assert.throws(() => readRustCoreAbiFromArchive(gzipSync(Buffer.alloc(1024)), '2.2.0'), /source is missing/);
  assert.throws(() => readRustCoreAbiFromArchive(gzipSync(Buffer.alloc(20, 1)), '2.2.0'), /Truncated/);
});

test('stable release awaits failure while prerelease skips the registry check', async t => {
  const f = await fixture(t);
  const manifestPath = path.join(f.root, 'package.json');
  let calls = 0;
  const checkPublished = async () => { calls++; await Promise.resolve(); throw new Error('publication refused'); };
  await fs.writeFile(manifestPath, JSON.stringify({ version: '2.0.0' }));
  await assert.rejects(checkBeforeStableNpmRelease({ manifestPath, checkPublished }), /publication refused/);
  assert.equal(calls, 1);
  await fs.writeFile(manifestPath, JSON.stringify({ version: '2.0.0-beta.11' }));
  assert.equal((await checkBeforeStableNpmRelease({ manifestPath, checkPublished })).skipped, true);
  assert.equal(calls, 1);
});
