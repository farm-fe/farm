const assert = require('node:assert/strict');
const { version } = require('electron/package.json');
const checksums = require('electron/checksums.json');

// Keep the example on its lockfile version and the repository's existing mirror.
assert.equal(version, '30.0.7', 'Review Electron download policy when updating Electron');
const mirror = 'https://npmmirror.com/mirrors/electron/';
const download = {
  // Forge renders configuration strings in place; keep the package data untouched.
  checksums: { ...checksums },
  unsafelyDisableChecksums: false,
  mirrorOptions: {
    mirror,
    // @electron/get otherwise lets npm/ELECTRON variables override the URL.
    resolveAssetURL(details) {
      assert.equal(details.version, `v${version}`, 'Unexpected Electron version');
      assert.equal(details.artifactName, 'electron', 'Unexpected Electron artifact');
      assert.ok(!details.isGeneric && !details.artifactSuffix, 'Unexpected Electron archive');
      const file = `electron-v${version}-${details.platform}-${details.arch}.zip`;
      assert.ok(Object.hasOwn(download.checksums, file), 'Missing official Electron checksum');
      assert.match(download.checksums[file], /^[a-f0-9]{64}$/, 'Invalid official Electron checksum');
      return `${mirror}v${version}/${file}`;
    },
  },
};
module.exports = { electronVersion: version, download };
