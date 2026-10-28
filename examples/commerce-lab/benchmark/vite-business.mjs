import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';
import { compile } from '@mdx-js/mdx';
import Papa from 'papaparse';
import { parse } from 'yaml';
import autoImport from 'unplugin-auto-import/vite';
import { channelVirtual } from '../adapters/channel-virtual.mjs';
import { productUrl } from '../adapters/product-url.mjs';
import { productImageDom } from '../adapters/product-image.mjs';
import { publicationStrip } from '../adapters/publication-strip.mjs';
import { modularUi, modularUiJsxPrelude } from '../adapters/modular-ui.mjs';
import { reactComponents } from '../adapters/react-components.mjs';
import { wasm } from '../adapters/wasm.mjs';
import { assertViteVersion, BUSINESS_COMPARISON_PROTOCOL } from './vite-comparison.mjs';
import { generatedCoverage } from './graph-summary.mjs';

export { BUSINESS_COMPARISON_PROTOCOL };
const svgRequire = createRequire(new URL('../../../js-plugins/svgr/package.json', import.meta.url));
const markdownRequire = createRequire(new URL('../../../website/package.json', import.meta.url));
const matches = (filters, value) => filters?.some(pattern => new RegExp(pattern).test(value));
const clean = id => id.split('?')[0].replaceAll('\\', '/');

/** Only the Commerce reference hooks above are accepted, not a general Farm adapter. */
export function commerceHook(plugin, transformWithEsbuild) {
  assert.ok(plugin && !Array.isArray(plugin) && plugin.name?.startsWith('commerce-'));
  return {
    name: `vite-${plugin.name}`, enforce: 'pre',
    async resolveId(source, importer) {
      if (!matches(plugin.resolve?.filters?.sources, source)) return null;
      const result = await plugin.resolve.executor({ source, importer });
      return result ? { id: result.resolvedPath, moduleSideEffects: result.sideEffects } : null;
    },
    async load(id) {
      const [resolvedPath, query = ''] = id.split('?');
      if (!matches(plugin.load?.filters?.resolvedPaths, resolvedPath)) return null;
      const result = await plugin.load.executor({ resolvedPath, moduleId: id, query: [...new URLSearchParams(query)] }, {
        emitFile: resource => {
          assert.equal(resource.resourceType, 'asset');
          return this.emitFile({ type: 'asset', fileName: resource.name, source: Buffer.from(resource.content) });
        },
        addWatchFile: (_module, file) => this.addWatchFile(file),
      });
      if (!result) return null;
      if (result.moduleType === 'jsx') return transformWithEsbuild(result.content, `${id}.jsx`, { loader: 'jsx', jsx: 'automatic', sourcemap: true });
      assert.equal(result.moduleType, 'js', 'Unexpected Commerce load type');
      return { code: result.content, map: null };
    },
    async transform(content, id) {
      if (!matches(plugin.transform?.filters?.resolvedPaths, clean(id))) return null;
      const result = await plugin.transform.executor({ content, resolvedPath: clean(id), moduleId: id,
        moduleType: /\.tsx$/.test(clean(id)) ? 'tsx' : /\.jsx$/.test(clean(id)) ? 'jsx' : 'js' });
      return result ? { code: result.content, map: result.sourceMap ? JSON.parse(result.sourceMap) : null } : null;
    },
  };
}

/** The original app/HTML/fixtures stay intact. Differences are build-tool transport only. */
export async function viteBusinessConfig(root, { outDir, graph } = {}) {
  root = path.resolve(root);
  assert.equal(path.resolve(process.cwd()), root, 'Run the isolated Vite worker with the original lab as cwd');
  await assertViteVersion();
  const { transformWithEsbuild } = await import('vite');
  const { default: tailwind } = await import(new URL('../../vite-adapter-vue-css/performance-tailwind.mjs', import.meta.url).href);
  const { transform: svgTransform } = await import(pathToFileURL(svgRequire.resolve('@svgr/core')).href);
  const { default: svgJsx } = await import(pathToFileURL(svgRequire.resolve('@svgr/plugin-jsx')).href);
  const { default: gfm } = await import(pathToFileURL(markdownRequire.resolve('remark-gfm')).href);
  const generatedRoot = path.join(root, 'packages/extensions/business-2048');
  const loaded = new Set(), transformed = new Set(), emitted = new Set(), workers = [];
  const graphHook = scope => ({
    name: `commerce-vite-graph-${scope}`,
    load(id) { loaded.add(id); return null; },
    transform(_code, id) { transformed.add(id); return null; },
    generateBundle(_options, bundle) {
      const modules = Object.values(bundle).filter(item => item.type === 'chunk').flatMap(chunk => Object.keys(chunk.modules));
      if (scope === 'worker') workers.push(...modules);
      else for (const id of modules) emitted.add(id);
    },
  });
  const data = {
    name: 'commerce-vite-data', enforce: 'pre',
    async load(id) {
      const file = clean(id);
      if (!/\.(csv|tsv|yaml|yml|mdx)$/.test(file) || id.includes('?')) return null;
      const content = await fs.readFile(file, 'utf8');
      if (file.endsWith('.mdx')) return String(await compile(content, { development: false, jsx: false, remarkPlugins: [gfm] }));
      if (/\.(csv|tsv)$/.test(file)) {
        const result = Papa.parse(content, { header: true, skipEmptyLines: true, delimiter: file.endsWith('.tsv') ? '\t' : ',' });
        assert.deepEqual(result.errors, []); return `export default ${JSON.stringify(result.data)};`;
      }
      return `export default ${JSON.stringify(parse(content))};`;
    },
  };
  const svg = {
    name: 'commerce-vite-svg', enforce: 'pre',
    resolveId(source) { if (/^~icons\/commerce\/(inventory|launch)$/.test(source)) return '\0' + source; },
    async load(id) {
      const icon = id.match(/^\0~icons\/commerce\/(inventory|launch)$/);
      const file = icon ? path.join(root, `apps/admin/assets/icons/${icon[1]}.svg`) : clean(id);
      if (!icon && (!file.endsWith('.svg') || id.includes('?') || /product-(home|outdoor|studio|travel)\.svg$/.test(file))) return null;
      this.addWatchFile(file);
      const jsx = await svgTransform(await fs.readFile(file, 'utf8'), { jsxRuntime: 'classic', plugins: [svgJsx] });
      return transformWithEsbuild(jsx, file + '.jsx', { loader: 'jsx', jsx: 'automatic', sourcemap: true });
    },
  };
  const bridges = () => [channelVirtual('js'), productUrl('js'), productImageDom('js'),
    publicationStrip('js', 'production'), modularUiJsxPrelude({ catalog: true }), modularUi('js'), reactComponents('js'), wasm()]
    .map(plugin => commerceHook(plugin, transformWithEsbuild));
  const compress = {
    name: 'commerce-vite-gzip',
    generateBundle(_options, bundle) {
      for (const item of Object.values(bundle)) {
        if (!/\.(js|mjs|json|css|html)$/.test(item.fileName)) continue;
        const bytes = Buffer.from(item.type === 'chunk' ? item.code : item.source);
        if (bytes.length >= 1024) this.emitFile({ type: 'asset', fileName: item.fileName + '.gz', source: gzipSync(bytes, { level: 6 }) });
      }
    },
  };
  return {
    root, configFile: false, clearScreen: false, logLevel: 'info', base: '/', cacheDir: path.join(root, '.cache/vite-business'),
    resolve: { alias: { '@commerce-extension': path.join(generatedRoot, 'index.ts') } },
    define: { __COMMERCE_GROUP__: JSON.stringify('vite') }, // The original app labels its real builder honestly.
    esbuild: { jsx: 'automatic' },
    build: { outDir: outDir ?? path.join(root, 'dist/vite-business'), emptyOutDir: true, target: 'esnext', sourcemap: true, minify: 'esbuild', rollupOptions: { cache: false, input: path.join(root, 'index.html') } },
    plugins: [graphHook('app'), ...bridges(), data, svg, autoImport({ imports: ['react'], dts: false }), tailwind(), compress,
      { name: 'commerce-vite-coverage', async closeBundle() {
        if (!graph) return;
        const generated = [];
        async function walk(directory) { for (const entry of await fs.readdir(directory, { withFileTypes: true })) { const file = path.join(directory, entry.name); if (entry.isDirectory()) await walk(file); else generated.push(clean(file)); } }
        await walk(generatedRoot);
        assert.equal(generated.length, 28673, 'Complete generated workload required');
        // Some asset loaders run before Rollup's default load. Module graph identities
        // supplement observed hooks, without claiming every asset is a JS chunk module.
        const graphIds = new Set(this.getModuleIds());
        const coverage = generatedCoverage(generated.map(file => path.relative(generatedRoot, file)), { loaded: [...loaded], transformed: [...transformed], graph: [...graphIds], emitted: [...emitted], workers }, generatedRoot, root);
        await fs.writeFile(graph, JSON.stringify({ protocol: BUSINESS_COMPARISON_PROTOCOL, status: coverage.missing.length ? 'invalid' : 'complete', ...coverage,
          loaded: [...loaded].sort(), transformed: [...transformed].sort(), emittedChunkModules: [...emitted].sort(), workerModules: [...new Set(workers)].sort(),
          semantics: 'Actual hook/graph identities; CSS and assets need not remain JS chunk modules. Bundler internal module totals are not equated.' }));
        assert.deepEqual(coverage.missing, [], 'Generated sources missing from actual Vite graph');
      } }],
    worker: { format: 'es', plugins: () => [graphHook('worker'), data, commerceHook(wasm(), transformWithEsbuild)] },
  };
}
