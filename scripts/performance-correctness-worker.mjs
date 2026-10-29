import fs from 'node:fs/promises';
import path from 'node:path';
import http from 'node:http';
import { pathToFileURL } from 'node:url';
import { verifyBusinessBrowser, BUSINESS_CORRECTNESS_PROTOCOL } from '../examples/commerce-lab/benchmark/business-correctness.mjs';

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
export async function verifyBuiltFarm(directory, browser, verify = verifyBusinessBrowser) {
  const server = await serveBuild(directory);
  let context;
  try {
    context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
    const page = await context.newPage(); page.setDefaultTimeout(15000);
    const result = await verify(page, server.url);
    if (result?.functional !== true) throw new Error('Farm browser correctness failed');
    return { protocol: BUSINESS_CORRECTNESS_PROTOCOL, status: 'complete', measured: false, result };
  } finally { try { await context?.close(); } finally { await server.close(); } }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { chromium } = await import('playwright-chromium');
  const ownerFile = process.env.FARM_PERFORMANCE_BROWSER_OWNER;
  async function owner(state, browserPid = null) {
    if (!ownerFile) return;
    const stat = await fs.lstat(ownerFile);
    if (!stat.isFile() || stat.isSymbolicLink() || stat.size > 4096) throw new Error('Invalid supervisor browser receipt');
    const old = JSON.parse(await fs.readFile(ownerFile, 'utf8'));
    if (old.rootPid !== null && old.rootPid !== process.pid) throw new Error('Browser receipt owner mismatch');
    const temporary = ownerFile + '.tmp';
    await fs.writeFile(temporary, JSON.stringify({ rootPid: process.pid, browserPid, state }), { flag: 'wx' });
    await fs.rename(temporary, ownerFile);
  }
  let server, browser, browserPid;
  await owner('launching');
  try {
    server = await chromium.launchServer({ headless: true, timeout: 30000 });
    browserPid = server.process().pid;
    await owner('open', browserPid);
    browser = await chromium.connect(server.wsEndpoint());
    if (process.argv[2] !== '--farm') throw new Error('Expected --farm output-directory receipt-file');
    await fs.writeFile(process.argv[4], JSON.stringify(await verifyBuiltFarm(process.argv[3], browser)));
  }
  finally {
    try { await browser?.close(); }
    finally {
      await server?.close();
      if (server) {
        const child = server.process();
        if (child.exitCode === null && child.signalCode === null) throw new Error('BrowserServer close did not confirm process exit');
        await owner('closed', browserPid);
      }
    }
  }
}
