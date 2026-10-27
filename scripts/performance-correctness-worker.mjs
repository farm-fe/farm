import fs from 'node:fs/promises';
import path from 'node:path';
import http from 'node:http';
import { pathToFileURL } from 'node:url';
import { verifyComparisonBrowser, COMPARISON_PROTOCOL } from '../examples/commerce-lab/benchmark/vite-comparison.mjs';

export async function serveBuild(directory) {
  const root = await fs.realpath(directory);
  const server = http.createServer(async (request, response) => {
    try {
      const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
      const file = await fs.realpath(path.resolve(root, `.${pathname === '/' ? '/index.html' : pathname}`));
      const relative = path.relative(root, file);
      if (relative.startsWith('..') || path.isAbsolute(relative)) throw new Error('Outside preview root');
      const bytes = await fs.readFile(file);
      const type = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.svg': 'image/svg+xml' }[path.extname(file)] ?? 'application/octet-stream';
      response.writeHead(200, { 'Content-Type': type }); response.end(bytes);
    } catch { response.writeHead(404); response.end('Not found'); }
  });
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  return { url: `http://127.0.0.1:${server.address().port}`, close: () => new Promise(resolve => { server.close(resolve); server.closeAllConnections(); }) };
}
export async function verifyBuiltComparison(root, browser) {
  const results = {};
  for (const tool of ['farm', 'vite']) {
    const server = await serveBuild(path.join(root, `dist-${tool}`));
    let context;
    try {
      context = await browser.newContext();
      const page = await context.newPage(); page.setDefaultTimeout(15000);
      results[tool] = await verifyComparisonBrowser(page, server.url);
    } finally { await context?.close(); await server.close(); }
  }
  return { protocol: COMPARISON_PROTOCOL, status: 'complete', measured: false, results };
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { chromium } = await import('playwright-chromium');
  const browser = await chromium.launch({ headless: true });
  try { await fs.writeFile(process.argv[3], JSON.stringify(await verifyBuiltComparison(process.argv[2], browser))); }
  finally { await browser.close(); }
}
