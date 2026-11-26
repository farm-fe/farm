// This script runs inside the clean installed candidate, never the workspace.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import { once } from 'node:events';
import { build } from '@farmfe/core';
import federation from '@farmfe/plugin-module-federation';
import { chromium } from 'playwright-chromium';

const root = process.cwd();
const dist = path.join(root, 'dist');
const server = http.createServer(async (request, response) => {
  try {
    const url = new URL(request.url, 'http://127.0.0.1');
    const file = path.resolve(dist, '.' + decodeURIComponent(url.pathname));
    assert.ok(file.startsWith(dist + path.sep));
    const content = await fs.readFile(file);
    response.writeHead(200, { 'content-type': file.endsWith('.html') ? 'text/html' : 'text/javascript' });
    response.end(content);
  } catch {
    response.writeHead(404);
    response.end();
  }
});
server.listen(0, '127.0.0.1');
await once(server, 'listening');
const origin = 'http://127.0.0.1:' + server.address().port;
const makePlugin = options => {
  const plugin = federation(options);
  assert.equal(path.resolve(plugin[0]), path.resolve(process.argv[2]), 'Did not use the exact embedded binary');
  return plugin;
};
let browser;
try {
  for (const key of ['host', 'remote']) await fs.mkdir(key);
  await fs.writeFile('host/index.html', '<!doctype html><link rel="icon" href="data:,"><main id="value"></main><script type="module" src="./main.js"></script>');
  await fs.writeFile('host/main.js', "import { value } from 'sample/Value'; document.querySelector('#value').textContent = value;");
  await fs.writeFile('remote/value.js', "export const value = 'exact bundled native federation';");
  const config = (key, input, options) => ({
    root: path.join(root, key), mode: 'production', configFile: false, plugins: [makePlugin(options)],
    compilation: { input, output: { path: path.join(dist, key), publicPath: origin + '/' + key + '/', targetEnv: 'browser-esnext' },
      persistentCache: false, lazyCompilation: false },
  });
  await build(config('remote', { remoteEntry: 'farm-mf-native:container' },
    { name: 'sampleProvider', globalName: 'sampleGlobal', format: 'global', publicPath: origin + '/remote/',
      exposes: { './Value': './value.js' } }));
  assert.ok((await fs.stat('dist/remote/remoteEntry.js')).isFile());
  await build(config('host', { index: './index.html' },
    { name: 'sampleHost', format: 'global', remotes: { sample: { name: 'sampleProvider', globalName: 'sampleGlobal',
      format: 'global', entry: origin + '/remote/remoteEntry.js' } } }));
  browser = await chromium.launch();
  const page = await browser.newPage();
  const errors = [], requests = [];
  page.on('pageerror', error => errors.push(String(error)));
  page.on('requestfailed', request => errors.push(request.url() + ': ' + request.failure()?.errorText));
  page.on('response', response => {
    if (response.status() >= 400) errors.push(response.status() + ' ' + response.url());
    if (response.ok()) requests.push(response.url());
  });
  await page.goto(origin + '/host/index.html');
  await page.waitForFunction(() => document.querySelector('#value')?.textContent === 'exact bundled native federation', undefined, { timeout: 20000 });
  assert.ok(requests.includes(origin + '/remote/remoteEntry.js'), 'Remote entry did not load in browser');
  assert.deepEqual(errors, []);
} finally {
  try { await browser?.close(); }
  finally { await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve())); }
}
