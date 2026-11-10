import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';
import test from 'node:test';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';
const require = createRequire(new URL('../examples/electron/package.json', import.meta.url));
const config = require('./forge.config.cjs').packagerConfig;
const forgeRequire = createRequire(require.resolve('@electron-forge/cli/package.json'));
const coreRequire = createRequire(forgeRequire.resolve('@electron-forge/core'));
const packagerRequire = createRequire(coreRequire.resolve('@electron/packager'));
const { createDownloadOpts, downloadElectronZip } = packagerRequire('./download.js');
const details = () => createDownloadOpts(config, 'darwin', 'arm64');
const policySource = fs.readFileSync(new URL('../examples/electron/electron-download.cjs', import.meta.url), 'utf8');
function fixture(version, checksums) {
  const module = { exports: {} };
  vm.runInNewContext(policySource, { module, require(name) {
    if (name === 'electron/package.json') return { version };
    if (name === 'electron/checksums.json') return checksums;
    return require(name);
  } });
  return module.exports;
}
test('Forge forwards official checksums and exact locked version to its installed packager', () => {
  assert.equal(config.electronVersion, '30.0.7');
  assert.equal(details().checksums, config.download.checksums);
  assert.equal(details().unsafelyDisableChecksums, false);
  for (const [platform, arch, digest] of [
    ['darwin', 'arm64', '9fd8ba7e42da3f321fa09f5b5769c8b2d1bfb20c8f642e0963fe829eb840e47a'],
    ['linux', 'x64', '9df2417f925e970256ab71e4cafecbd4fc217c15fcdec199639cc34cc1a2162b'],
    ['win32', 'x64', '7233ddc5e5e3f182d5b66dc4096ced8092bfdefe2a5f6e2d3f59dea15fc81711'],
  ]) assert.equal(config.download.checksums[`electron-v30.0.7-${platform}-${arch}.zip`], digest);
});
test('policy rejects changed installed version, missing and malformed official checksum', () => {
  assert.throws(() => fixture('30.0.8', {}), /Review Electron/);
  const request = { ...details(), version: 'v30.0.7' };
  assert.throws(() => fixture('30.0.7', {}).download.mirrorOptions.resolveAssetURL(request), /Missing official/);
  assert.throws(() => fixture('30.0.7', { 'electron-v30.0.7-darwin-arm64.zip': 'bad' }).download.mirrorOptions.resolveAssetURL(request), /Invalid official/);
  for (const change of [{ version: 'v30.0.8' }, { artifactName: 'other-artifact' }, { isGeneric: true }, { artifactSuffix: 'symbols' }, { arch: '../other' }]) {
    assert.throws(() => config.download.mirrorOptions.resolveAssetURL({ ...request, ...change }));
  }
});
test('installed production download path ignores mirror overrides and rejects tampered bytes', async () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-electron-download-'));
  const keys = ['npm_config_electron_mirror', 'ELECTRON_MIRROR', 'ELECTRON_CUSTOM_DIR', 'ELECTRON_CUSTOM_FILENAME', 'electron_use_remote_checksums'];
  keys.push('ELECTRON_CUSTOM_VERSION');
  const previous = keys.map(key => process.env[key]);
  let downloads = 0;
  try {
    for (const key of keys.filter(key => key !== 'ELECTRON_CUSTOM_VERSION')) process.env[key] = 'https://invalid.example/';
    delete process.env.ELECTRON_CUSTOM_VERSION;
    await assert.rejects(downloadElectronZip({ ...details(), cacheRoot: temporary, force: true,
      downloader: { async download(url, destination) {
        downloads++;
        assert.equal(url, 'https://npmmirror.com/mirrors/electron/v30.0.7/electron-v30.0.7-darwin-arm64.zip');
        fs.writeFileSync(destination, 'tampered');
      } },
    }), /checksum/i);
    assert.equal(downloads, 1, 'No mirror-provided checksum may be fetched');
    process.env.ELECTRON_CUSTOM_VERSION = '30.0.8';
    await assert.rejects(downloadElectronZip({ ...details(), cacheRoot: temporary, force: true,
      downloader: { async download() { assert.fail('wrong version reached transport'); } },
    }), /Unexpected Electron version/);
  } finally {
    delete process.env.ELECTRON_CUSTOM_VERSION;
    keys.forEach((key, i) => previous[i] === undefined ? delete process.env[key] : process.env[key] = previous[i]);
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
test('installed checksum engine accepts matching bytes with an explicitly synthetic test transport', async () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'farm-electron-valid-'));
  const bytes = Buffer.from('synthetic checksum protocol fixture, not an Electron binary');
  const synthetic = fixture('30.0.7', { 'electron-v30.0.7-darwin-arm64.zip': createHash('sha256').update(bytes).digest('hex') });
  try {
    const archive = await downloadElectronZip({ ...createDownloadOpts(synthetic, 'darwin', 'arm64'), cacheRoot: temporary,
      downloader: { async download(url, destination) { fs.writeFileSync(destination, bytes); } },
    });
    assert.deepEqual(fs.readFileSync(archive), bytes);
  } finally { fs.rmSync(temporary, { recursive: true, force: true }); }
});

test('real Forge config loader renders the checksum copy without requiring the Electron binary', async () => {
  const load = coreRequire('../util/forge-config.js').default;
  const resolved = await load(fileURLToPath(new URL('../examples/electron', import.meta.url)));
  const official = require('electron/checksums.json');
  assert.deepEqual(resolved.packagerConfig.download.checksums, official);
  assert.notEqual(resolved.packagerConfig.download.checksums, official);
  assert.equal(require.cache[require.resolve('electron')], undefined);
});
