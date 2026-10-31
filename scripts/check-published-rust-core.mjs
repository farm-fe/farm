// Adapted from 57a7cfd0: verify the intended Cargo version before stable npm publication.
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { gunzipSync } from 'node:zlib';

const repository = fileURLToPath(new URL('../', import.meta.url));
const registry = 'https://crates.io/api/v1/crates/farmfe_core';

export function assertPublishedRustCore(evidence) {
  for (const key of ['expectedVersion', 'expectedAbi', 'publishedVersion', 'publishedAbi']) {
    if (typeof evidence[key] !== 'string' || !evidence[key].trim()) throw new Error(`Rust publication evidence missing ${key}`);
  }
  if (evidence.publishedVersion !== evidence.expectedVersion) throw new Error(`Published Rust core version ${evidence.publishedVersion}; expected ${evidence.expectedVersion}`);
  if (evidence.publishedAbi !== evidence.expectedAbi) throw new Error(`Published Rust core ABI ${evidence.publishedAbi}; expected ${evidence.expectedAbi}`);
}

async function getBytes(url, fetchImpl, limit) {
  const response = await fetchImpl(url, { headers: { 'User-Agent': 'Farm-release-ABI-check' }, signal: AbortSignal.timeout(30000) });
  if (!response.ok) throw new Error(`Rust publication lookup failed: HTTP ${response.status}`);
  if (!response.body) throw new Error('Rust publication response has no body');
  const chunks = [];
  let size = 0;
  const reader = response.body.getReader();
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > limit) { await reader.cancel(); throw new Error('Rust publication response exceeds size limit'); }
      chunks.push(Buffer.from(value));
    }
  } finally { reader.releaseLock(); }
  return Buffer.concat(chunks);
}

// Read only a named regular tar member in memory; no registry path is extracted.
export function readRustCoreAbiFromArchive(archive, version) {
  const tar = gunzipSync(archive, { maxOutputLength: 32 * 1024 * 1024 });
  const target = `farmfe_core-${version}/src/lib.rs`;
  const field = (header, start, length) => header.subarray(start, start + length).toString('utf8').replace(/\0.*$/s, '');
  let source;
  for (let offset = 0; offset < tar.length;) {
    if (offset + 512 > tar.length) throw new Error('Truncated Rust crate archive header');
    const header = tar.subarray(offset, offset + 512);
    if (header.every(byte => byte === 0)) break;
    const sizeText = field(header, 124, 12).trim();
    if (!/^[0-7]+$/.test(sizeText)) throw new Error('Invalid Rust crate archive member size');
    const size = Number.parseInt(sizeText, 8);
    const end = offset + 512 + size;
    if (!Number.isSafeInteger(size) || end > tar.length) throw new Error('Truncated Rust crate archive member');
    const prefix = field(header, 345, 155);
    const name = (prefix ? prefix + '/' : '') + field(header, 0, 100);
    if (name === target) {
      if (source !== undefined || ![0, 48].includes(header[156])) throw new Error('Ambiguous Rust crate source member');
      source = tar.subarray(offset + 512, end).toString('utf8');
    }
    offset += 512 + Math.ceil(size / 512) * 512;
  }
  const abi = source?.match(/^pub const VERSION: &str = "([^"]+)";/m)?.[1];
  if (!abi) throw new Error('Published Rust core ABI source is missing');
  return abi;
}

export async function checkPublishedRustCore({ root = repository, fetchImpl = globalThis.fetch } = {}) {
  const cargo = await fs.readFile(path.join(root, 'crates/core/Cargo.toml'), 'utf8');
  const library = await fs.readFile(path.join(root, 'crates/core/src/lib.rs'), 'utf8');
  const expectedVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  const expectedAbi = library.match(/^pub const VERSION: &str = "([^"]+)";/m)?.[1];
  if (!/^\d+\.\d+\.\d+$/.test(expectedVersion ?? '') || !expectedAbi) throw new Error('Local Rust core version or ABI is missing or not stable');
  const source = `${registry}/${expectedVersion}`;
  const metadata = JSON.parse((await getBytes(source, fetchImpl, 1024 * 1024)).toString('utf8'));
  const published = metadata.version;
  if (published?.num !== expectedVersion || published.yanked || !/^[a-f0-9]{64}$/i.test(published.checksum ?? '')) throw new Error('Intended Rust core publication is missing, yanked or invalid');
  const archive = await getBytes(`${source}/download`, fetchImpl, 16 * 1024 * 1024);
  const archiveSha256 = createHash('sha256').update(archive).digest('hex');
  if (archiveSha256 !== published.checksum.toLowerCase()) throw new Error('Published Rust core archive checksum mismatch');
  const evidence = { expectedVersion, expectedAbi, publishedVersion: published.num, publishedAbi: readRustCoreAbiFromArchive(archive, expectedVersion), archiveSha256, source };
  assertPublishedRustCore(evidence);
  return evidence;
}

export async function checkBeforeStableNpmRelease({ manifestPath = path.join(repository, 'packages/core/package.json'), checkPublished = checkPublishedRustCore } = {}) {
  const manifest = JSON.parse(await fs.readFile(manifestPath, 'utf8'));
  if (manifest.version.includes('-')) return { skipped: true, reason: 'Prerelease npm core; stable publication gate does not apply.' };
  if (!/^\d+\.\d+\.\d+$/.test(manifest.version)) throw new Error('Invalid npm core version');
  return checkPublished();
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  checkPublishedRustCore().then(evidence => console.log(JSON.stringify(evidence))).catch(error => { console.error(error.message); process.exitCode = 1; });
}
