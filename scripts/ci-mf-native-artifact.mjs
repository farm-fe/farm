// Private, same-run test inputs only. This is not a release or reuse inventory.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { normalFile } from './formal-native-scope.mjs';
import { nativePlatformMatches } from './release-archive.mjs';

export const mfTargets = Object.freeze({
  'linux-x64-gnu': 'x86_64-unknown-linux-gnu',
  'darwin-arm64': 'aarch64-apple-darwin',
  'win32-x64-msvc': 'x86_64-pc-windows-msvc'
});
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const sourceFiles = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml'];
const packageFile = abi => `rust-plugins/module-federation/npm/${abi}/index.farm`;
const bundleDirectory = abi => `logs/mf-native/${abi}`;
function context(abi, { root = process.cwd(), env = process.env, git = (...args) => execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim() } = {}) {
  assert.ok(Object.hasOwn(mfTargets, abi), 'Unsupported experimental MF ABI');
  assert.match(env.GITHUB_SHA ?? '', /^[a-f0-9]{40}$/, 'Missing current source SHA');
  assert.equal(git('rev-parse', 'HEAD'), env.GITHUB_SHA, 'MF source differs from current CI checkout');
  assert.equal(git('diff', '--name-only', 'HEAD', '--', ...sourceFiles, 'crates', 'rust-plugins/module-federation'), '', 'MF native source must match the committed checkout');
  assert.match(env.GITHUB_RUN_ID ?? '', /^\d+$/, 'Missing workflow run');
  assert.match(env.GITHUB_RUN_ATTEMPT ?? '', /^\d+$/, 'Missing workflow attempt');
  assert.match(env.GITHUB_REPOSITORY ?? '', /^[\w.-]+\/[\w.-]+$/, 'Missing repository');
  return { root, env, identity: {
    sourceSHA: env.GITHUB_SHA, runId: env.GITHUB_RUN_ID, repository: env.GITHUB_REPOSITORY,
    sourceInputs: Object.fromEntries(sourceFiles.map(file => [file, hash(fs.readFileSync(normalFile(root, file)))])),
    abi, recipe: { target: mfTargets[abi], profile: 'release', locked: true }
  } };
}
function destinationAt(root, relative) {
  let directory = fs.realpathSync(root);
  for (const part of relative.split('/').slice(0, -1)) {
    assert.ok(part && part !== '.' && part !== '..');
    directory = path.join(directory, part);
    if (!fs.existsSync(directory)) fs.mkdirSync(directory);
    assert.ok(fs.lstatSync(directory).isDirectory() && !fs.lstatSync(directory).isSymbolicLink(), 'MF destination redirects');
    assert.equal(fs.realpathSync(directory), directory, 'MF destination junction redirects');
  }
  const file = path.join(directory, path.basename(relative));
  if (fs.existsSync(file) || (() => { try { fs.lstatSync(file); return true; } catch { return false; } })()) normalFile(root, relative);
  return file;
}
function bytesAt(root, relative, abi) {
  const bytes = fs.readFileSync(normalFile(root, relative));
  assert.ok(nativePlatformMatches(bytes, abi), 'MF native architecture differs');
  return bytes;
}
export function recordMfNative(abi, options) {
  const { root, env, identity } = context(abi, options);
  const bytes = bytesAt(root, packageFile(abi), abi);
  const row = { schemaVersion: 1, purpose: 'private-mf-test-artifact', releaseBytesAuthorized: false,
    ...identity, producerAttempt: env.GITHUB_RUN_ATTEMPT, bytes: bytes.length, sha256: hash(bytes) };
  fs.writeFileSync(destinationAt(root, `${bundleDirectory(abi)}/index.farm`), bytes);
  fs.writeFileSync(destinationAt(root, `${bundleDirectory(abi)}/receipt.json`), JSON.stringify(row, null, 2) + '\n');
  return row;
}
export function consumeMfNative(abi, options) {
  const { root, env, identity } = context(abi, options);
  assert.match(env.FARM_MF_PRODUCER_ATTEMPT ?? '', /^\d+$/, 'Missing bound producer attempt');
  const receiptBytes = fs.readFileSync(normalFile(root, `${bundleDirectory(abi)}/receipt.json`));
  assert.ok(receiptBytes.length < 65536, 'MF receipt exceeds bound');
  const row = JSON.parse(receiptBytes);
  assert.equal(row.schemaVersion, 1); assert.equal(row.purpose, 'private-mf-test-artifact');
  assert.equal(row.releaseBytesAuthorized, false, 'MF test inputs cannot authorize publication');
  for (const [key, value] of Object.entries(identity)) assert.deepEqual(row[key], value, `MF ${key} differs`);
  assert.equal(row.producerAttempt, env.FARM_MF_PRODUCER_ATTEMPT, 'MF producer attempt differs');
  const file = `${bundleDirectory(abi)}/index.farm`;
  const bytes = fs.readFileSync(normalFile(root, file));
  assert.equal(bytes.length, row.bytes, 'MF artifact bytes differ');
  assert.equal(hash(bytes), row.sha256, 'MF artifact digest differs');
  assert.ok(nativePlatformMatches(bytes, abi), 'MF native architecture differs');
  const destination = destinationAt(root, packageFile(abi));
  fs.copyFileSync(normalFile(root, file), destination);
  return row;
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, abi] = process.argv.slice(2);
  assert.ok(Object.hasOwn(mfTargets, abi), 'Unsupported experimental MF ABI');
  if (mode === 'target') console.log(mfTargets[abi]);
  else if (mode === 'record') console.log(JSON.stringify(recordMfNative(abi)));
  else if (mode === 'consume') console.log(JSON.stringify(consumeMfNative(abi)));
  else throw new Error('Expected target, record or consume');
}
