// Prepublication artifact preparation only. No npm authentication or writer.
// cspell:words RUSTFLAGS
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { archiveFiles, inspectNpmArchive, nativePlatformMatches, sha256 } from './release-archive.mjs';
import { resolveNpmCli } from './v2-npm-cli.mjs';
import { hostNativeTarget } from '../rust-plugins/module-federation/scripts/native-platform.mjs';

export const owner = '@farmfe/plugin-module-federation';
export const version = '0.1.0';
export const corePeer = '~2.0.2';
export const targets = Object.freeze({
  'darwin-arm64': 'aarch64-apple-darwin',
  'linux-x64-gnu': 'x86_64-unknown-linux-gnu',
  'win32-x64-msvc': 'x86_64-pc-windows-msvc',
});
const project = 'rust-plugins/module-federation';
const root = fileURLToPath(new URL('../', import.meta.url));
const regular = (directory, relative) => {
  assert.equal(fs.realpathSync(directory), path.resolve(directory), 'Artifact root redirects');
  let current = directory;
  for (const part of relative.split('/')) {
    assert.ok(part && part !== '.' && part !== '..');
    current = path.join(current, part);
    assert.ok(!fs.lstatSync(current).isSymbolicLink(), 'Artifact path redirects');
  }
  assert.ok(fs.statSync(current).isFile());
  return fs.readFileSync(current);
};
const assertSHA = value => assert.match(value, /^[0-9a-f]{40}$/);
// Use exact committed bytes, not checkout transformations such as CRLF.
function sourceBytes(directory, sourceCommit, relative) {
  assertSHA(sourceCommit);
  assert.ok(relative.split('/').every(part => part && part !== '.' && part !== '..'));
  const git = (...args) => execFileSync('git', args, { cwd: directory, maxBuffer: 32 * 1024 ** 2 });
  const entry = git('ls-tree', sourceCommit, '--', relative).toString('utf8');
  assert.match(entry, /^100(?:644|755) blob [0-9a-f]{40}\t/, 'Package source must be a regular committed file');
  return git('show', `${sourceCommit}:${relative}`);
}
export function buildIdentity(directory = root) {
  const git = (...args) => execFileSync('git', args, { cwd: directory, encoding: 'utf8' }).trim();
  const sourceCommit = git('rev-parse', 'HEAD'), sourceTree = git('rev-parse', 'HEAD^{tree}');
  assertSHA(sourceCommit); assertSHA(sourceTree);
  assert.equal(git('status', '--porcelain', '--untracked-files=no'), '', 'Tracked source differs from build identity');
  const run = process.env.GITHUB_ACTIONS === 'true' ? {
    runId: process.env.GITHUB_RUN_ID, runAttempt: process.env.GITHUB_RUN_ATTEMPT,
  } : null;
  if (run) {
    assert.equal(sourceCommit, process.env.GITHUB_SHA, 'Bundle source differs from workflow SHA');
    for (const value of Object.values(run)) assert.match(value, /^[1-9][0-9]*$/);
  }
  return { sourceCommit, sourceTree, run, toolchainSha256: sha256(sourceBytes(directory, sourceCommit, 'rust-toolchain.toml')),
    lockSha256: sha256(sourceBytes(directory, sourceCommit, 'Cargo.lock')) };
}
function nativeReceipt(identity, abi, bytes, legalBytes) {
  assert.ok(Object.hasOwn(targets, abi), 'Unsupported MF bundle platform');
  assert.ok(bytes.length > 0 && bytes.length <= 32 * 1024 ** 2, 'MF binary exceeds 32 MiB input budget');
  assert.ok(nativePlatformMatches(bytes, abi), 'MF binary architecture differs');
  return { schemaVersion: 1, kind: 'mf-bundle-build-input-not-publication-authorization', ...identity,
    abi, target: targets[abi], profile: 'release', binarySha256: sha256(bytes), binarySize: bytes.length,
    legalInputsSha256: sha256(legalBytes) };
}

export function cargoLegalInputs(directory, abi) {
  const { sourceCommit } = buildIdentity(directory);
  assert.ok(Object.hasOwn(targets, abi));
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--format-version', '1', '--locked',
    '--offline', '--filter-platform', targets[abi]], { cwd: directory, encoding: 'utf8', maxBuffer: 32 * 1024 ** 2 }));
  const packages = new Map(metadata.packages.map(row => [row.id, row]));
  const graph = new Map(metadata.resolve.nodes.map(row => [row.id, row.deps]));
  const entry = metadata.packages.filter(row => row.name === 'farmfe_plugin_module_federation' && row.source === null);
  assert.equal(entry.length, 1);
  const pending = [entry[0].id], seen = new Set(), rows = [];
  while (pending.length) {
    const id = pending.pop();
    if (seen.has(id)) continue;
    seen.add(id);
    const row = packages.get(id);
    assert.ok(row);
    pending.push(...graph.get(id).filter(dep => dep.dep_kinds.some(kind => kind.kind !== 'dev')).map(dep => dep.pkg));
    const home = path.dirname(row.manifest_path);
    const available = fs.readdirSync(home).filter(file => /^(LICENSE|COPYING|NOTICE|UNLICENSE)([-_.]|$)/i.test(file));
    if (row.license_file && !available.includes(row.license_file)) available.push(row.license_file);
    rows.push({ name: row.name, version: row.version, source: row.source, declaration: row.license,
      materialHashes: available.sort().map(file => ({ path: file, sha256: sha256(row.source === null
        ? sourceBytes(directory, sourceCommit, path.relative(directory, path.join(home, file)).split(path.sep).join('/'))
        : regular(home, file)) })) });
  }
  rows.sort((a, b) => JSON.stringify([a.name, a.version, a.source]).localeCompare(JSON.stringify([b.name, b.version, b.source])));
  return { schemaVersion: 1, kind: 'locked-cargo-license-review-input', target: targets[abi],
    scope: 'Target-filtered non-dev prerequisite closure; may include build-only code',
    reviewComplete: false, components: rows };
}

export function buildNative({ directory = root, abi, output, npm = resolveNpmCli() }) {
  assert.equal(hostNativeTarget().abi, abi, 'Run the locked recipe on the native host');
  const identity = buildIdentity(directory);
  const binary = `${project}/npm/${abi}/index.farm`;
  assert.ok(!fs.existsSync(path.join(directory, binary)), 'Native build requires a fresh checkout output');
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) =>
    !/^(RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|CARGO_PROFILE_)/.test(key)));
  Object.assign(env, { CARGO_BUILD_JOBS: '2', ...(abi === 'darwin-arm64' ? { CARGO_PROFILE_RELEASE_STRIP: 'none' } : {}),
    ...(abi === 'linux-x64-gnu' ? { CARGO_INCREMENTAL: '0', CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER: 'gcc' } : {}) });
  execFileSync(process.execPath, [npm, '--prefix', project, 'run', 'build', '--',
    '--target', targets[abi], '--abi', abi, '--profile', 'release', '--locked'], { cwd: directory, env, stdio: 'inherit' });
  assert.deepEqual(buildIdentity(directory), identity, 'Tracked source moved while building');
  const bytes = regular(directory, binary);
  const legal = Buffer.from(JSON.stringify(cargoLegalInputs(directory, abi), null, 2) + '\n');
  const receipt = nativeReceipt(identity, abi, bytes, legal);
  fs.mkdirSync(output);
  fs.writeFileSync(path.join(output, 'index.farm'), bytes, { flag: 'wx' });
  fs.writeFileSync(path.join(output, 'legal-inputs.json'), legal, { flag: 'wx' });
  fs.writeFileSync(path.join(output, 'receipt.json'), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
  return receipt;
}

export function readBundleInputs(identity, directory) {
  assertSHA(identity.sourceCommit); assertSHA(identity.sourceTree);
  assert.deepEqual(fs.readdirSync(directory).sort(), Object.keys(targets).sort(), 'Exact three-ABI artifact set required');
  return Object.keys(targets).map(abi => {
    const folder = path.join(directory, abi);
    assert.deepEqual(fs.readdirSync(folder).sort(), ['index.farm', 'legal-inputs.json', 'receipt.json'], 'Unexpected native artifact members');
    const bytes = regular(directory, abi + '/index.farm');
    const receipt = JSON.parse(regular(directory, abi + '/receipt.json'));
    const legal = regular(directory, abi + '/legal-inputs.json');
    const review = JSON.parse(legal);
    assert.equal(review.kind, 'locked-cargo-license-review-input');
    assert.equal(review.target, targets[abi]);
    assert.equal(review.reviewComplete, false);
    assert.ok(Array.isArray(review.components) && review.components.length > 0);
    assert.deepEqual(receipt, nativeReceipt(identity, abi, bytes, legal), 'Native source/target/integrity receipt differs');
    return { abi, bytes, legal, receipt };
  });
}

const sourceMembers = [
  ['scripts/index.mjs', project + '/scripts/bundled-index.mjs'],
  ['scripts/native-platform.mjs', project + '/scripts/native-platform.mjs'],
  ['LICENSE', 'LICENSE'], ['README.md', project + '/BUNDLED-README.md'],
  ['THIRD_PARTY_NOTICES.md', project + '/BUNDLED-THIRD-PARTY-NOTICES.md'],
];
const contents = [
  'package.json', ...sourceMembers.map(([member]) => member),
  ...Object.keys(targets).map(abi => 'native/' + abi + '/index.farm'),
];
function expectedManifest() {
  return { name: owner, version, type: 'module', license: 'MIT',
    description: 'Native Module Federation plugin for Farm',
    repository: { type: 'git', url: 'https://github.com/farm-fe/farm', directory: project },
    exports: { '.': './scripts/index.mjs', './package.json': './package.json' },
    engines: { node: '>=20.6.0' },
    dependencies: { '@module-federation/runtime': '2.9.2' },
    peerDependencies: { '@farmfe/core': corePeer },
    files: ['scripts/index.mjs', 'scripts/native-platform.mjs', 'native', 'THIRD_PARTY_NOTICES.md'],
  };
}

export function readMfBundleForTest(directory, receiptDirectory) {
  const receipt = JSON.parse(regular(receiptDirectory, 'receipt.json'));
  const identity = buildIdentity(directory);
  for (const [key, value] of Object.entries(identity)) assert.deepEqual(receipt[key], value);
  assert.equal(receipt.name, owner);
  assert.equal(receipt.version, version);
  assert.equal(receipt.published, false);
  assert.equal(receipt.licenseReviewComplete, false);
  assert.equal(receipt.publicationAuthorized, false);
  const file = 'farmfe-plugin-module-federation-0.1.0.tgz';
  assert.equal(receipt.file, file);
  const bytes = regular(receiptDirectory, file);
  assert.equal(receipt.sha256, sha256(bytes));
  assert.equal(receipt.size, bytes.length);
  const { files, issues } = inspectNpmArchive(bytes, { name: owner, version });
  assert.deepEqual(issues, []);
  assert.deepEqual([...files.keys()].sort(), contents.map(name => 'package/' + name).sort());
  assert.deepEqual(JSON.parse(files.get('package/package.json')), expectedManifest());
  for (const [member, source] of sourceMembers) {
    assert.deepEqual(files.get('package/' + member), sourceBytes(directory, identity.sourceCommit, source));
  }
  assert.deepEqual(receipt.native, Object.keys(targets).map(abi =>
    nativeReceipt(identity, abi, files.get('package/native/' + abi + '/index.farm'),
      regular(receiptDirectory, 'legal-inputs-' + abi + '.json'))));
  return receipt;
}

export function prepareMfBundle({ directory = root, input, output, npm = resolveNpmCli() }) {
  const identity = buildIdentity(directory);
  const binaries = readBundleInputs(identity, input);
  const original = JSON.parse(sourceBytes(directory, identity.sourceCommit, project + '/package.json'));
  assert.equal(original.name, owner);
  assert.equal(original.private, true, 'Standalone bundle must not enter ordinary registry-family production');
  assert.equal(original.dependencies['@module-federation/runtime'], '2.9.2');
  const manifest = expectedManifest();
  const stage = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-mf-package-'));
  try {
    fs.mkdirSync(path.join(stage, 'scripts'));
    for (const [member, source] of sourceMembers) {
      fs.writeFileSync(path.join(stage, member), sourceBytes(directory, identity.sourceCommit, source));
    }
    fs.writeFileSync(path.join(stage, 'package.json'), JSON.stringify(manifest, null, 2) + '\n');
    for (const row of binaries) {
      const native = path.join(stage, 'native', row.abi);
      fs.mkdirSync(native, { recursive: true });
      fs.writeFileSync(path.join(native, 'index.farm'), row.bytes);
    }
    fs.mkdirSync(output);
    const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !/TOKEN|NPM_CONFIG/i.test(key)));
    const log = execFileSync(process.execPath, [npm, 'pack', '--ignore-scripts', '--json', '--pack-destination', output],
      { cwd: stage, env, encoding: 'utf8', timeout: 120000 });
    const packed = JSON.parse(log);
    assert.equal(packed.length, 1);
    const file = 'farmfe-plugin-module-federation-0.1.0.tgz';
    assert.equal(packed[0].filename, file);
    const bytes = regular(output, file), inspection = inspectNpmArchive(bytes, manifest);
    assert.ok(bytes.length <= 100 * 1024 ** 2, 'MF bundle exceeds registry archive budget');
    assert.deepEqual(inspection.issues, []);
    const files = archiveFiles(bytes);
    assert.deepEqual([...files.keys()].sort(), contents.map(name => 'package/' + name).sort(), 'Unexpected bundle member');
    assert.deepEqual(JSON.parse(files.get('package/package.json')), manifest);
    for (const row of binaries) assert.deepEqual(files.get(`package/native/${row.abi}/index.farm`), row.bytes);
    for (const row of binaries) fs.writeFileSync(path.join(output, 'legal-inputs-' + row.abi + '.json'), row.legal, { flag: 'wx' });
    const receipt = { schemaVersion: 1, name: owner, version, file, sha256: sha256(bytes), size: bytes.length,
      ...identity, native: binaries.map(row => row.receipt), unpackedSize: packed[0].unpackedSize, licenseReviewComplete: false,
      publicationAuthorized: false, published: false };
    fs.writeFileSync(path.join(output, 'receipt.json'), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
    return receipt;
  } finally {
    fs.rmSync(stage, { recursive: true, force: true });
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, input, output] = process.argv.slice(2);
  assert.ok(['native', 'bundle'].includes(mode) && input && output, 'Use native ABI OUTPUT or bundle INPUT OUTPUT');
  const result = mode === 'native' ? buildNative({ abi: input, output: path.resolve(output) })
    : prepareMfBundle({ input: path.resolve(input), output: path.resolve(output) });
  console.log(JSON.stringify(result));
}
