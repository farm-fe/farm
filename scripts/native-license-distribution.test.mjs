import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packages = ['packages/core', 'rust-plugins/sass', 'rust-plugins/svgr', 'rust-plugins/tailwindcss'];

test('main and standalone platform legal files retain original bytes', async () => {
  const farm = await fs.readFile(path.join(root, 'LICENSE'));
  for (const relative of packages) {
    const directory = path.join(root, relative);
    assert.deepEqual(await fs.readFile(path.join(directory, 'LICENSE')), farm);
    const legal = ['LICENSE', 'THIRD_PARTY_NOTICES.md'];
    for (const name of await fs.readdir(path.join(directory, 'licenses'))) legal.push(`licenses/${name}`);
    for (const entry of await fs.readdir(path.join(directory, 'npm'), { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      const platform = path.join(directory, 'npm', entry.name);
      const manifest = JSON.parse(await fs.readFile(path.join(platform, 'package.json'), 'utf8'));
      for (const file of legal) {
        assert.deepEqual(await fs.readFile(path.join(platform, file)), await fs.readFile(path.join(directory, file)), `${relative}/${entry.name}/${file}`);
        assert.ok(manifest.files.includes(file.split('/')[0]), `missing files entry: ${relative}/${entry.name}/${file}`);
      }
    }
  }
});

test('fork license originals are preserved separately from Farm attribution', async () => {
  for (const [original, distributed] of [
    ['rust-ecosystem/grass/LICENSE', 'rust-plugins/sass/licenses/grass-MIT.txt'],
    ['rust-ecosystem/svgr-rs/LICENSE', 'rust-plugins/svgr/licenses/svgr-rs-MIT.txt']
  ]) assert.equal(await fs.readFile(path.join(root, distributed), 'utf8'), (await fs.readFile(path.join(root, original), 'utf8')).replaceAll('\r\n', '\n'));
  const tailwind = await fs.readFile(path.join(root, 'rust-plugins/tailwindcss/licenses/tailwindcss-MIT.txt'), 'utf8');
  assert.ok(tailwind.includes('Copyright (c) Tailwind Labs, Inc.'));
  assert.ok(!tailwind.includes('Farm'));
  const notice = await fs.readFile(path.join(root, 'rust-plugins/tailwindcss/THIRD_PARTY_NOTICES.md'), 'utf8');
  assert.ok(notice.includes('ae96721fc5453919ffa41913262f5691c51a0d46'));
});

test('SVGR main package selects matching v2 platform packages before publication', async () => {
  const directory = path.join(root, 'rust-plugins/svgr');
  const parent = JSON.parse(await fs.readFile(path.join(directory, 'package.json'), 'utf8'));
  assert.equal(parent.version, '2.0.0');
  const platforms = await fs.readdir(path.join(directory, 'npm'));
  assert.equal(platforms.length, 9);
  for (const platform of platforms) {
    const manifest = JSON.parse(await fs.readFile(path.join(directory, 'npm', platform, 'package.json'), 'utf8'));
    assert.equal(manifest.name, `${parent.name}-${platform}`);
    assert.equal(manifest.version, parent.version);
    assert.equal(parent.optionalDependencies[manifest.name], parent.version);
    if (platform.endsWith('-musl')) assert.deepEqual(manifest.libc, ['musl']);
    if (platform.endsWith('-gnu')) assert.deepEqual(manifest.libc, ['glibc']);
  }
});
