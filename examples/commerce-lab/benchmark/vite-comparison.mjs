import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';
import { performance } from 'node:perf_hooks';

export const VITE_COMPARISON_VERSION = '5.2.8';
export const COMPARISON_PROTOCOL = 'commerce-catalog-quote-subset-v1';
const here = path.dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

export async function assertViteVersion() {
  const packagePath = require.resolve('vite/package.json');
  const { version } = JSON.parse(await fs.readFile(packagePath, 'utf8'));
  if (version !== VITE_COMPARISON_VERSION) throw new Error(`Vite oracle version mismatch: expected ${VITE_COMPARISON_VERSION}, received ${version}`);
  return { version, packagePath };
}

/** Both builders receive byte-identical input, including the real Commerce quote function. */
export async function prepareComparisonFixture(root) {
  root = path.resolve(root);
  await fs.mkdir(root, { recursive: true });
  const sources = new Map();
  for (const name of (await fs.readdir(path.join(here, 'comparison-fixture'))).sort()) {
    sources.set(name, await fs.readFile(path.join(here, 'comparison-fixture', name)));
  }
  sources.set('business-quote.mjs', await fs.readFile(path.join(here, 'business-quote.mjs')));
  const products = Array.from({ length: 240 }, (_, index) => ({ id: `product-${index}`, name: `Commerce product ${String(index).padStart(3, '0')}`, category: ['home', 'office', 'travel'][index % 3], price: 1000 + index * 25, stock: 100 + index }));
  sources.set('products.js', Buffer.from(`export const products = ${JSON.stringify(products)};\n`));
  const hash = createHash('sha256');
  for (const [name, bytes] of [...sources].sort(([a], [b]) => a.localeCompare(b))) {
    hash.update(name).update('\0').update(bytes).update('\0');
    await fs.writeFile(path.join(root, name), bytes);
  }
  return { protocol: COMPARISON_PROTOCOL, fingerprint: hash.digest('hex'), productCount: products.length, root, files: [...sources.keys()].sort() };
}

export function comparisonConfigs(root, { port = 4321 } = {}) {
  root = path.resolve(root);
  return {
    farm: { root, configFile: false, compilation: { input: { index: path.join(root, 'index.html') }, output: { path: path.join(root, 'dist-farm'), targetEnv: 'browser-es2017' }, persistentCache: false, lazyCompilation: false, sourcemap: false, minify: true, presetEnv: false, progress: false }, server: { host: '127.0.0.1', port, open: false } },
    vite: { root, configFile: false, clearScreen: false, logLevel: 'error', build: { outDir: path.join(root, 'dist-vite'), emptyOutDir: true, target: 'es2017', sourcemap: false, minify: 'esbuild' }, server: { host: '127.0.0.1', port, strictPort: true, open: false } }
  };
}

export async function collectComparisonArtifacts(directory) {
  const files = [];
  async function walk(relative = '') {
    for (const entry of (await fs.readdir(path.join(directory, relative), { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
      const name = path.join(relative, entry.name);
      if (entry.isDirectory()) await walk(name);
      else if (entry.isFile()) {
        const bytes = await fs.readFile(path.join(directory, name));
        files.push({ name: name.replaceAll('\\', '/'), rawBytes: bytes.length, gzipBytes: gzipSync(bytes, { level: 9 }).length, sha256: createHash('sha256').update(bytes).digest('hex') });
      }
    }
  }
  await walk();
  if (!files.some(file => file.name.endsWith('.html')) || !files.some(file => file.name.endsWith('.js'))) throw new Error('Comparison build has no HTML/JS output');
  return { files, rawBytes: files.reduce((total, file) => total + file.rawBytes, 0), gzipBytes: files.reduce((total, file) => total + file.gzipBytes, 0), gzipMethod: 'each emitted file, node:zlib gzip level 9; excludes HTTP headers' };
}

export async function runComparisonBuild(tool, root) {
  if (!['farm', 'vite'].includes(tool)) throw new Error(`Unknown comparison tool: ${tool}`);
  const oracle = await assertViteVersion();
  const configs = comparisonConfigs(root);
  const builder = tool === 'vite' ? await import('vite') : await import('@farmfe/core');
  const started = performance.now();
  await builder.build(configs[tool]);
  const buildMs = performance.now() - started;
  return { protocol: COMPARISON_PROTOCOL, tool, viteVersion: oracle.version, buildMs, artifacts: await collectComparisonArtifacts(path.join(root, `dist-${tool}`)) };
}

export async function startViteComparison(root, options = {}) {
  await assertViteVersion();
  const { createServer } = await import('vite');
  const server = await createServer(comparisonConfigs(root, options).vite);
  const started = performance.now();
  try {
    await server.listen();
    const url = server.resolvedUrls.local[0];
    const readiness = await verifyComparisonHttp(url);
    return { server, url, readyMs: performance.now() - started, readiness, close: () => server.close() };
  } catch (error) {
    await server.close();
    throw error;
  }
}

export async function verifyComparisonHttp(url) {
  const response = await fetch(url);
  const html = await response.text();
  if (!response.ok || !html.includes('Commerce catalog and quote')) throw new Error(`Comparison page unavailable: ${response.status}`);
  const scripts = [...html.matchAll(/<script\b[^>]*\bsrc=["']([^"']+)["']/g)].map(match => match[1]);
  if (!scripts.length) throw new Error('Comparison HTML has no entry script');
  for (const source of scripts) {
    const script = await fetch(new URL(source, url));
    if (!script.ok || !(await script.text()).trim()) throw new Error(`Comparison entry unavailable: ${source}`);
  }
  return { status: response.status, scripts, functional: false };
}

/** Playwright-compatible page. Run for each dev server AND each built preview. */
export async function verifyComparisonBrowser(page, url) {
  const errors = [];
  const onError = error => errors.push(String(error));
  const onResponse = response => { if (response.status() >= 400) errors.push(`HTTP ${response.status()}: ${response.url()}`); };
  const onRequestFailed = request => errors.push(`Request failed: ${request.url()}`);
  const onConsole = message => { if (message.type() === 'error') errors.push(`Console error: ${message.text()}`); };
  page.on('pageerror', onError);
  page.on('response', onResponse);
  page.on('requestfailed', onRequestFailed);
  page.on('console', onConsole);
  try {
    await page.goto(url);
    await page.locator('#count').filter({ hasText: '240 products' }).waitFor();
    await page.locator('#category').selectOption('office');
    await page.locator('#count').filter({ hasText: '80 products' }).waitFor();
    await page.locator('#category').selectOption('all');
    await page.locator('#search').fill('product 000');
    await page.locator('#count').filter({ hasText: '1 products' }).waitFor();
    await page.locator('[data-product="product-0"]').click();
    await page.locator('#quote').click();
    await page.locator('#total').filter({ hasText: '2400' }).waitFor();
    await page.locator('#quantity').fill('3');
    await page.locator('#quote').click();
    await page.locator('#total').filter({ hasText: '3200' }).waitFor();
    await page.locator('#quantity').fill('100');
    await page.locator('#quote').click();
    await page.locator('#total').filter({ hasText: 'Insufficient stock after reservation' }).waitFor();
    if (errors.length) throw new Error(errors.join('\n'));
    return { protocol: COMPARISON_PROTOCOL, functional: true, checks: ['catalog-240', 'category-80', 'search-1', 'lazy-detail', 'quote-2400', 'tier-3200', 'stock-rejection'], pageErrors: errors };
  } finally {
    page.off('pageerror', onError);
    page.off('response', onResponse);
    page.off('requestfailed', onRequestFailed);
    page.off('console', onConsole);
  }
}
