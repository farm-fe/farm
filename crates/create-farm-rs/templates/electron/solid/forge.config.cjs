const { MakerSquirrel } = require('@electron-forge/maker-squirrel');
const { MakerZIP } = require('@electron-forge/maker-zip');
const { MakerDeb } = require('@electron-forge/maker-deb');
const { MakerRpm } = require('@electron-forge/maker-rpm');
const fs = require('fs-extra');
const path = require('path');

/** @type {import('@electron-forge/shared-types').ForgeConfig} */
const config = {
  packagerConfig: {
    // Ignore all files, including `node_modules`.
    ignore: /.*/,
    beforeCopy: [(
      buildPath,
      electronVersion,
      platform,
      arch,
      callback,
    ) => {
      // Copy some necessary files for running the Electron App.
      const items = [
        'dist',
        'dist-electron',
        'package.json',
      ]
      for (const item of items) {
        fs.copySync(path.join(__dirname, item), path.join(buildPath, item))
      }
      callback()
    }],
  },
  rebuildConfig: {},
  makers: [
    new MakerSquirrel({}),
    new MakerZIP({}, ['darwin']),
    new MakerRpm({}),
    new MakerDeb({}),
  ],
  plugins: [],
};

module.exports = config;
