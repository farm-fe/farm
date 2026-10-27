import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import test from 'node:test';
import { root } from './v2-release-inventory.mjs';

test('selected MIT packages retain the existing repository license verbatim', () => {
  const expected = fs.readFileSync(path.join(root, 'LICENSE'));
  for (const directory of ['packages/create-farm-plugin', 'packages/plugin-tools',
    ...['babel', 'electron', 'less', 'postcss', 'react-compiler', 'sass', 'svgr', 'tailwindcss', 'visualizer', 'vuetify'].map(name => `js-plugins/${name}`)]) {
    const manifest = JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json')));
    assert.equal(manifest.license, 'MIT');
    assert.deepEqual(fs.readFileSync(path.join(root, directory, 'LICENSE')), expected);
  }
});
test('ISC and unclassified package declarations are not silently relabeled', () => {
  for (const directory of ['packages/create-farm', 'js-plugins/dts']) assert.equal(JSON.parse(fs.readFileSync(path.join(root, directory, 'package.json'))).license, 'ISC');
  assert.equal(JSON.parse(fs.readFileSync(path.join(root, 'js-plugins/qiankun/package.json'))).license, undefined);
});
