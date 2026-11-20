// Credential-free candidate install. Nothing is published or granted.
// cspell:words USERCONFIG GLOBALCONFIG
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { hostNativeTarget } from '../rust-plugins/module-federation/scripts/native-platform.mjs';
import { owner, version, readMfBundleForTest } from './prepare-mf-single-package.mjs';
import { resolveNpmCli } from './v2-npm-cli.mjs';
import { sha256 } from './release-archive.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const artifacts = path.resolve(process.argv[2]);
const receipt = readMfBundleForTest(root, artifacts);
const { abi } = hostNativeTarget();
const proofPath = process.argv[3] && path.resolve(process.argv[3]);
const sameRunArtifactId = process.env.MF_BUNDLE_ARTIFACT_ID ?? null;
if (process.env.GITHUB_ACTIONS === 'true') assert.match(sameRunArtifactId, /^[1-9][0-9]*$/);
const member = receipt.native.find(row => row.abi === abi);
assert.ok(member, 'Missing exact native candidate platform');
const scratch = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'farm-mf-bundle-consumer-')));
try {
  const npm = resolveNpmCli();
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) =>
    !/TOKEN|NPM_CONFIG|FARM_LIB_CORE_PATH|NAPI_RS_NATIVE_LIBRARY_PATH|NODE_PATH|NODE_OPTIONS/i.test(key)));
  env.PLAYWRIGHT_BROWSERS_PATH = path.join(scratch, '.browser');
  env.NPM_CONFIG_USERCONFIG = path.join(scratch, 'empty-user.npmrc');
  env.NPM_CONFIG_GLOBALCONFIG = path.join(scratch, 'empty-global.npmrc');
  for (const name of ['empty-user.npmrc', 'empty-global.npmrc']) fs.writeFileSync(path.join(scratch, name), '');
  fs.writeFileSync(path.join(scratch, 'package.json'), JSON.stringify({ name: 'mf-bundle-consumer', private: true, type: 'module' }));
  execFileSync(process.execPath, [npm, 'install', '--ignore-scripts', '--no-audit', '--no-fund',
    '--registry=https://registry.npmjs.org', '--cache', path.join(scratch, '.cache'),
    '@farmfe/core@2.0.2', 'playwright-chromium@1.55.0', path.join(artifacts, receipt.file)], { cwd: scratch, env, stdio: 'inherit', timeout: 180000 });
  const lock = JSON.parse(fs.readFileSync(path.join(scratch, 'package-lock.json')));
  assert.equal(lock.packages['node_modules/@farmfe/core']?.version, '2.0.2');
  assert.equal(lock.packages['node_modules/' + owner]?.version, version);
  assert.ok(Object.values(lock.packages).every(pkg => !pkg.link), 'Linked packages are not independent evidence');
  const binary = path.join(scratch, 'node_modules', owner, 'native', abi, 'index.farm');
  assert.equal(sha256(fs.readFileSync(binary)), member.binarySha256);
  const coreOrigin = [];
  for (const name of ['@farmfe/core', '@farmfe/core-' + abi]) {
    const url = 'https://registry.npmjs.org/' + encodeURIComponent(name) + '/2.0.2';
    const response = await fetch(url, { redirect: 'error', signal: AbortSignal.timeout(15000) });
    assert.equal(response.status, 200, 'Missing public Core version: ' + name);
    const publicVersion = await response.json();
    assert.equal(publicVersion.name, name);
    assert.equal(publicVersion.version, '2.0.2');
    const installed = lock.packages['node_modules/' + name];
    assert.equal(installed?.version, '2.0.2');
    assert.match(publicVersion.dist.integrity, /^sha512-/);
    assert.equal(installed.integrity, publicVersion.dist.integrity);
    assert.equal(installed.resolved, publicVersion.dist.tarball);
    assert.equal(new URL(installed.resolved).origin, 'https://registry.npmjs.org');
    coreOrigin.push({ name, version: installed.version, integrity: installed.integrity, tarball: installed.resolved });
  }
  execFileSync(process.execPath, ['node_modules/playwright-chromium/cli.js', 'install', 'chromium'],
    { cwd: scratch, env, stdio: 'inherit', timeout: 180000 });
  fs.copyFileSync(path.join(root, 'scripts/mf-single-browser-probe.mjs'), path.join(scratch, 'probe.mjs'), fs.constants.COPYFILE_EXCL);
  execFileSync(process.execPath, ['probe.mjs', binary], { cwd: scratch, env, stdio: 'inherit', timeout: 180000 });
  const proof = { status: 'passed', abi, version, coreOrigin, archiveArtifactId: sameRunArtifactId,
    sourceCommit: receipt.sourceCommit, sourceTree: receipt.sourceTree, archiveSha256: receipt.sha256,
    browser: 'chromium: real host imported remote value', published: false };
  if (proofPath) fs.writeFileSync(proofPath, JSON.stringify(proof, null, 2) + '\n', { flag: 'wx' });
  console.log(JSON.stringify(proof));
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
}
