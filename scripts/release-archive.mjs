import { gunzipSync } from 'node:zlib';
import crypto from 'node:crypto';
import { assertStableManifest, verifyPublicEntries } from './v2-normal-pack-contract.mjs';

export const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
export function nativePlatformMatches(bytes, platform) {
  const [os, arch] = platform.split('-');
  if (os === 'win32') {
    if (bytes.length < 64 || bytes.toString('ascii', 0, 2) !== 'MZ') return false;
    const offset = bytes.readUInt32LE(60);
    return offset + 6 <= bytes.length && bytes.toString('ascii', offset, offset + 4) === 'PE\0\0' && bytes.readUInt16LE(offset + 4) === { x64: 0x8664, ia32: 0x14c, arm64: 0xaa64 }[arch];
  }
  if (os === 'linux') return bytes.length >= 20 && bytes.subarray(0, 4).equals(Buffer.from([127, 69, 76, 70])) && bytes[4] === 2 && bytes[5] === 1 && bytes.readUInt16LE(18) === { x64: 62, arm64: 183 }[arch];
  if (os === 'darwin') return bytes.length >= 8 && bytes.readUInt32LE(0) === 0xfeedfacf && bytes.readUInt32LE(4) === { x64: 0x1000007, arm64: 0x100000c }[arch];
  return false;
}
// Inspect regular tar members in memory. Never extract archive-controlled paths.
export function archiveFiles(bytes) {
  const tar = gunzipSync(bytes, { maxOutputLength: 512 * 1024 * 1024 });
  const files = new Map();
  const text = (block, start, size) => block.subarray(start, start + size).toString('utf8').replace(/\0.*$/s, '');
  for (let offset = 0; offset + 512 <= tar.length;) {
    const header = tar.subarray(offset, offset + 512);
    if (header.every(byte => byte === 0)) return files;
    const octal = (start, size) => {
      const value = text(header, start, size).trim();
      if (!/^[0-7]+$/.test(value)) throw new Error('Invalid archive numeric field');
      return Number.parseInt(value, 8);
    };
    const expected = octal(148, 8);
    const actual = [...header].reduce((sum, byte, index) => sum + (index >= 148 && index < 156 ? 32 : byte), 0);
    if (expected !== actual) throw new Error('Archive header checksum mismatch');
    const prefix = text(header, 345, 155);
    const name = `${prefix ? prefix + '/' : ''}${text(header, 0, 100)}`;
    const size = octal(124, 12);
    const end = offset + 512 + size;
    if (end > tar.length) throw new Error('Truncated archive member');
    if (name.startsWith('/') || name.includes('\\') || name.split('/').some(part => part === '..' || part === '.')) throw new Error('Unsafe archive path');
    if (![0, 48, 53].includes(header[156])) throw new Error(`Unsupported archive member type: ${header[156]}`);
    if (header[156] !== 53) {
      if (files.has(name)) throw new Error('Duplicate archive member');
      files.set(name, tar.subarray(offset + 512, end));
    }
    offset += 512 + Math.ceil(size / 512) * 512;
  }
  throw new Error('Archive has no terminating block');
}

export function inspectNpmArchive(bytes, expected) {
  const files = archiveFiles(bytes);
  const manifest = JSON.parse(files.get('package/package.json')?.toString('utf8') ?? 'null');
  if (manifest?.name !== expected.name || manifest?.version !== expected.version) throw new Error('Packed package identity mismatch');
  const issues = [];
  for (const section of ['dependencies', 'optionalDependencies', 'peerDependencies']) {
    for (const [name, version] of Object.entries(manifest[section] ?? {})) {
      if (/^(workspace:|file:|link:|git\+|https?:\/\/)/.test(version)) issues.push(`Non-registry dependency ${name}: ${version}`);
    }
  }
  try { assertStableManifest(manifest); } catch (error) { issues.push(error.message); }
  try { verifyPublicEntries(manifest, [...files.keys()].map(name => ({ path: name.replace(/^package\//, '') }))); } catch (error) { issues.push(error.message); }
  const legal = [...files].filter(([name]) => /\/(LICENSE(?:\.[^/]*)?|THIRD_PARTY_NOTICES\.md)$/.test(name) || name.startsWith('package/licenses/')).map(([name, content]) => ({ path: name.slice(8), sha256: sha256(content), bytes: content.length }));
  if (!legal.some(row => /^LICENSE(?:\.|$)/.test(row.path) && row.bytes > 0)) issues.push('Missing package license text');
  return { manifest, files, legal, issues };
}
