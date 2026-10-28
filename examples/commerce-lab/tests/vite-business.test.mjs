import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { commerceHook, viteBusinessConfig } from '../benchmark/vite-business.mjs';

test('thin Commerce load transport retains query and asset identity and explicitly lowers SVG JSX', async () => {
  let lowered, query, emitted;
  const plugin = commerceHook({ name: 'commerce-test-transport', load: { filters: { resolvedPaths: ['icon\\.svg$'] },
    executor(param, context) { query = param.query; context.emitFile({ resourceType: 'asset', name: 'assets/icon.svg', content: [1, 2] }); return { content: '<svg />', moduleType: 'jsx' }; } } },
    async (code, file, options) => { lowered = { code, file, options }; return { code: 'lowered', map: null }; });
  const result = await plugin.load.call({ emitFile: value => { emitted = value; } }, '/fixture/icon.svg?dom');
  assert.equal(result.code, 'lowered'); assert.equal(lowered.options.loader, 'jsx'); assert.match(lowered.file, /\.svg\?dom\.jsx$/);
  assert.deepEqual(query, [['dom', '']]); assert.equal(emitted.type, 'asset'); assert.equal(emitted.fileName, 'assets/icon.svg');
  assert.deepEqual([...emitted.source], [1, 2]);
  assert.equal(await plugin.load.call({}, '/fixture/not-matched.js'), null);
  assert.throws(() => commerceHook(['native-plugin', {}], () => {}));
});

test('real installed Vite factory preserves original entry and parses actual MDX/DSV/YAML plus JSX SVG', async () => {
  const root = fileURLToPath(new URL('../', import.meta.url));
  assert.equal(path.resolve(process.cwd()), path.resolve(root));
  const config = await viteBusinessConfig(root);
  assert.equal(config.build.target, 'esnext'); assert.equal(config.build.sourcemap, true);
  assert.equal(config.build.minify, 'esbuild'); assert.equal(config.build.rollupOptions.cache, false);
  assert.equal(config.build.rollupOptions.input, path.join(root, 'index.html'));
  assert.equal(config.cacheDir, path.join(root, '.cache/vite-business'));
  assert.equal(config.resolve.alias['@commerce-extension'], path.join(root, 'packages/extensions/business-2048/index.ts'));
  const plugins = config.plugins.flat(Infinity);
  for (const name of ['vite-commerce-js-wasm', 'vite-commerce-js-channel-virtual', 'vite-commerce-js-product-url', 'vite-commerce-js-product-image-dom', 'vite-commerce-js-local-components', 'commerce-vite-svg', 'commerce-vite-gzip']) assert.ok(plugins.some(plugin => plugin.name === name), name);
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'commerce-vite-data-'));
  try {
    const data = plugins.find(plugin => plugin.name === 'commerce-vite-data');
    for (const [file, content, expected] of [['rules.yaml', 'maxDiscount: 20\n', '"maxDiscount":20'], ['tiers.csv', 'minQuantity,discount\n5,10\n', '"discount":"10"'], ['shipping.tsv', 'destination\tcents\nexport\t1500\n', '"cents":"1500"']]) {
      const id = path.join(directory, file); await fs.writeFile(id, content); assert.ok((await data.load(id)).includes(expected));
    }
    const mdx = path.join(directory, 'Guide.mdx'); await fs.writeFile(mdx, '# Guide\n\n| Attribute | Review |\n| --- | --- |\n| Size | Required |\n');
    assert.match(await data.load(mdx), /_components\.table/);
    const svg = plugins.find(plugin => plugin.name === 'commerce-vite-svg');
    const result = await svg.load.call({ addWatchFile() {} }, path.join(root, 'apps/admin/assets/brand.svg'));
    assert.ok(result.code.includes('react')); assert.doesNotMatch(result.code, /return\s*</);
    assert.equal(await svg.load.call({}, path.join(root, 'apps/admin/assets/product-home.svg')), null);
    assert.equal(await svg.load.call({}, path.join(root, 'apps/admin/assets/product-home.svg?dom')), null);
  } finally { await fs.rm(directory, { recursive: true }); }
});
