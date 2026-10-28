// cspell:words TMQ
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import test from 'node:test';
import { root } from './v2-release-inventory.mjs';
import { verifyLicenseComponents } from './v2-normal-pack-contract.mjs';

test('selected MIT packages retain the existing repository license verbatim', () => {
  const expected = fs.readFileSync(path.join(root, 'LICENSE'));
  for (const directory of ['packages/create-farm-plugin', 'packages/plugin-tools',
    ...['babel', 'electron', 'less', 'postcss', 'react-compiler', 'sass', 'svgr', 'tailwindcss', 'visualizer', 'vuetify'].map(name => `js-plugins/${name}`)]) {
    const manifest = JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json')));
    assert.equal(manifest.license, 'MIT');
    assert.deepEqual(fs.readFileSync(path.join(root, directory, 'LICENSE')), expected);
  }
});
test('corrected Farm MIT declarations retain distinct upstream texts', () => {
  for (const owner of ['packages/create-farm', 'js-plugins/dts', 'js-plugins/qiankun']) {
    const location = path.join(root, owner);
    verifyLicenseComponents(location, JSON.parse(fs.readFileSync(path.join(location, 'package.json'))), root);
  }
  for (const directory of ['packages/create-farm', 'js-plugins/dts']) { assert.equal(JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json'))).license, 'MIT'); assert.deepEqual(fs.readFileSync(path.join(root, directory, 'LICENSE')), fs.readFileSync(path.join(root, 'LICENSE'))); }
  const directory = path.join(root, 'js-plugins/qiankun');
  assert.equal(JSON.parse(fs.readFileSync(path.join(directory, 'package.json'))).license, 'MIT');
  assert.deepEqual(fs.readFileSync(path.join(directory, 'LICENSE')), fs.readFileSync(path.join(root, 'LICENSE')));
  const components = JSON.parse(fs.readFileSync(path.join(directory, 'license-components.json')));
  assert.ok(components.components.every(component => component.status === 'verified'));
  assert.ok(fs.readFileSync(path.join(directory, 'licenses/vite-plugin-qiankun-MIT.txt'), 'utf8').includes('TMQ'));
});
