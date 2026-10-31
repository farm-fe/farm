import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import { createHash } from 'node:crypto';
// cspell:ignore Marsel Shaikhin Madinah

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packages = ['packages/core', 'rust-plugins/sass', 'rust-plugins/svgr', 'rust-plugins/tailwindcss'];

test('Docs and MDX platform archives retain every existing owner notice verbatim', async () => {
  for (const name of ['docs', 'docs-seo', 'docs-snapshot', 'docs-snippets', 'mdx', 'syntax-highlight']) {
    const owner = path.join(root, 'rust-plugins', name);
    const legal = ['LICENSE', 'THIRD_PARTY_NOTICES.md', ...(await fs.readdir(path.join(owner, 'licenses'))).map(file => `licenses/${file}`)];
    for (const platform of await fs.readdir(path.join(owner, 'npm'))) {
      const directory = path.join(owner, 'npm', platform);
      const manifest = JSON.parse(await fs.readFile(path.join(directory, 'package.json'), 'utf8'));
      for (const file of legal) {
        assert.deepEqual(await fs.readFile(path.join(directory, file)), await fs.readFile(path.join(owner, file)), `${name}/${platform}/${file}`);
        assert.ok(manifest.files.includes(file.split('/')[0]));
      }
    }
  }
});

test('all Commerce native plugins retain their own MIT text in standalone platform packages', async () => {
  const names = ['replace-dirname', 'react', 'worker', 'vue', 'vue-jsx', 'wasm', 'virtual', 'url', 'image', 'icons', 'strip', 'modular-import', 'react-components', 'auto-import', 'svgr', 'yaml', 'dsv', 'compress', 'sass', 'mdx', 'tailwindcss', 'dts'];
  for (const name of names) {
    const directory = path.join(root, 'rust-plugins', name);
    const license = (await fs.readFile(path.join(directory, 'LICENSE'), 'utf8')).replaceAll('\r\n', '\n');
    const platforms = await fs.readdir(path.join(directory, 'npm'));
    assert.equal(platforms.length, 9);
    if (name === 'vue-jsx') assert.ok(license.includes('Pig Fang'));
    if (name === 'virtual') assert.ok(license.includes('Madinah'));
    for (const folder of [directory, ...platforms.map(platform => path.join(directory, 'npm', platform))]) {
      assert.equal((await fs.readFile(path.join(folder, 'LICENSE'), 'utf8')).replaceAll('\r\n', '\n'), license, folder);
      const manifest = JSON.parse(await fs.readFile(path.join(folder, 'package.json'), 'utf8'));
      assert.ok(manifest.files.includes('LICENSE'), folder);
    }
  }
});

test('icons distribute linked Fervid and SVGR terms without relabeling upstream work', async () => {
  const directory = path.join(root, 'rust-plugins/icons');
  const platforms = await fs.readdir(path.join(directory, 'npm'));
  for (const folder of [directory, ...platforms.map(platform => path.join(directory, 'npm', platform))]) {
    const fervid = await fs.readFile(path.join(folder, 'licenses/fervid-Apache-2.0.txt'));
    assert.equal(createHash('sha256').update(fervid).digest('hex'), 'c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4');
    assert.equal(await fs.readFile(path.join(folder, 'licenses/svgr-rs-MIT.txt'), 'utf8'), (await fs.readFile(path.join(root, 'rust-ecosystem/svgr-rs/LICENSE'), 'utf8')).replaceAll('\r\n', '\n'));
    const notice = await fs.readFile(path.join(folder, 'THIRD_PARTY_NOTICES.md'), 'utf8');
    assert.ok(notice.includes('4f26ab48f81eb9688fb84e9fa14541373d5bca7d'));
    assert.ok(notice.includes('patches/fervid_parser/README.farm.md'));
    const manifest = JSON.parse(await fs.readFile(path.join(folder, 'package.json'), 'utf8'));
    for (const entry of ['LICENSE', 'licenses', 'THIRD_PARTY_NOTICES.md']) assert.ok(manifest.files.includes(entry));
  }
});

test('Vue standalone distributions retain pinned Fervid Apache terms and existing MIT attribution', async () => {
  const directory = path.join(root, 'rust-plugins/vue');
  const revision = '4f26ab48f81eb9688fb84e9fa14541373d5bca7d';
  assert.ok((await fs.readFile(path.join(directory, 'Cargo.toml'), 'utf8')).includes(revision));
  const platforms = await fs.readdir(path.join(directory, 'npm'));
  assert.equal(platforms.length, 9);
  for (const folder of [directory, ...platforms.map(platform => path.join(directory, 'npm', platform))]) {
    const original = await fs.readFile(path.join(folder, 'licenses/fervid-Apache-2.0.txt'));
    assert.equal(createHash('sha256').update(original).digest('hex'), 'c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4');
    const notice = await fs.readFile(path.join(folder, 'THIRD_PARTY_NOTICES.md'), 'utf8');
    assert.ok(notice.includes(revision));
    assert.ok(notice.includes('Marsel Shaikhin'));
    const mit = await fs.readFile(path.join(folder, 'LICENSE'), 'utf8');
    assert.ok(mit.includes('Copyright (c) 2021-present Evan You'));
    assert.ok(mit.includes('Copyright (c) 2025 farm-fe'));
    const manifest = JSON.parse(await fs.readFile(path.join(folder, 'package.json'), 'utf8'));
    for (const name of ['LICENSE', 'THIRD_PARTY_NOTICES.md', 'licenses']) assert.ok(manifest.files.includes(name));
  }
});

test('main and standalone platform legal files retain original bytes', async () => {
  const farm = await fs.readFile(path.join(root, 'LICENSE'));
  for (const relative of packages) {
    const directory = path.join(root, relative);
    const packageLicense = await fs.readFile(path.join(directory, 'LICENSE'));
    if (relative === 'rust-plugins/tailwindcss') {
      const original = await fs.readFile(path.join(root, 'website/licenses/ffd8a1e5aadc4dbb4b38346fea235a18bfbda64e44abec8ab1f9d34384ea2439.txt'), 'utf8');
      assert.equal(packageLicense.toString('utf8').replaceAll('\r\n', '\n').trimEnd(), original.replaceAll('\r\n', '\n').trimEnd());
      assert.ok(packageLicense.includes('Copyright (c) 2023 Tailwind Labs Inc.'));
      assert.ok(packageLicense.includes('Copyright (c) 2025 farm-fe'));
    } else assert.deepEqual(packageLicense, farm);
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
