import http from 'node:http';
import path from 'node:path';
import fs from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {chromium} from 'playwright-chromium';

const base = path.dirname(fileURLToPath(import.meta.url));
const servers = [];
const evidence = {serve:process.env.MF_SERVE || 'static', mode:process.env.MF_MODE || 'production', requests:[], checks:[], errors:[]};
const types = {'.html':'text/html','.js':'text/javascript','.css':'text/css','.json':'application/json','.map':'application/json'};
let browser;
try {
  if (process.env.MF_SERVE !== 'dev') for (const [dir,port,prefix] of [['host',4100,'/'],['remote-a',4101,'/deploy/a/'],['remote-b',4102,'/deploy/b/']]) {
    const root = path.join(base,'dist',dir);
    const server = http.createServer(async (req,res) => {
      const pathname = new URL(req.url,'http://localhost').pathname;
      const relative = decodeURIComponent(pathname.slice(prefix.length)) || 'index.html';
      const file = path.resolve(root,relative);
      res.setHeader('Access-Control-Allow-Origin','*');
      res.setHeader('Cache-Control','no-store');
      if (!pathname.startsWith(prefix) || !file.startsWith(root + path.sep)) {res.writeHead(404);res.end('Not found');return;}
      try {const content = await fs.readFile(file);res.setHeader('Content-Type',types[path.extname(file)] || 'application/octet-stream');res.end(content);}
      catch {res.writeHead(404);res.end('Not found');}
    });
    await new Promise((resolve,reject) => {server.once('error',reject);server.listen(port,'127.0.0.1',resolve);});
    servers.push(server);
  }
  browser = await chromium.launch({headless:true});
  const page = await browser.newPage();
  page.on('response',r => evidence.requests.push({url:r.url(),status:r.status()}));
  page.on('pageerror',e => { evidence.errors.push(e.message); console.error(e.stack); });
  const consoleErrors = [];
  const requestFailures = [];
  page.on('requestfailed',request => requestFailures.push({url:request.url(),error:request.failure()?.errorText}));
  page.on('console',msg => {if(msg.type() === 'error') consoleErrors.push(msg.text());});
  await page.goto('http://127.0.0.1:4100');
  await page.waitForFunction(() => window.probe?.ready,{},{timeout:30000});
  assert.equal(await page.locator('#button-a').evaluate(e => getComputedStyle(e).color),'rgb(17, 85, 153)');
  assert.equal(await page.locator('#button-b').evaluate(e => getComputedStyle(e).color),'rgb(153, 51, 17)');
  evidence.checks.push('two remotes and computed CSS');
  await page.locator('#button-a').click();
  await page.waitForFunction(() => document.querySelector('#button-a').textContent === 'lazy-a:1');
  await page.locator('#button-b').click();
  await page.waitForFunction(() => document.querySelector('#button-b').textContent === 'lazy-b:1');
  await page.locator('#button-a').click();
  await page.waitForFunction(() => document.querySelector('#button-a').textContent === 'lazy-a:2');
  assert.equal(await page.locator('#button-b').textContent(),'lazy-b:1');
  evidence.checks.push('independent lazy chunks and module state');
  const identity = await page.evaluate(async () => ({
    repeat:await window.probe.repeat(),
    hostUnchanged:window.probe.hostUnchanged,
    namespaces:['farm_mf_host','farm_mf_remoteA','farm_mf_remoteB'].map(n => !!window[n]?.m),
    distinct:window.farm_mf_host?.m !== window.farm_mf_remoteA?.m && window.farm_mf_remoteA?.m !== window.farm_mf_remoteB?.m,
    containers:[typeof window.remoteA?.get,typeof window.remoteA?.init,typeof window.remoteB?.get],
    factories:await Promise.all([window.remoteA.get('./Button'),window.remoteA.get('./Button')]).then(([a,b]) => a() === b())
  }));
  assert.equal(identity.hostUnchanged,true);assert.equal(identity.repeat,true);assert.deepEqual(identity.namespaces,[true,true,true]);assert.equal(identity.distinct,true);
  assert.deepEqual(identity.containers,['function','function','function']);assert.equal(identity.factories,true);
  evidence.identity = identity; evidence.checks.push('namespace isolation, get/init, concurrent get and module identity');
  assert.deepEqual(evidence.errors,[]);
  assert.deepEqual(consoleErrors,[]);
  assert.deepEqual(requestFailures,[]);
  for (const [port,prefix] of [[4101,'/deploy/a/'],[4102,'/deploy/b/']]) {
    const requests = evidence.requests.filter(r => r.url.startsWith(`http://127.0.0.1:${port}`));
    assert.ok(requests.some(r => r.url.endsWith('.css')));
    assert.ok(requests.filter(r => r.url.endsWith('.js')).length >= 3);
    assert.ok(requests.every(r => r.status === 200 && new URL(r.url).pathname.startsWith(prefix)));
  }
  evidence.checks.push('remote-origin JS/CSS deployment subpaths');
  const failures = await page.evaluate(async () => {
    const capture = async fn => {try {await fn();return null;}catch(e){return e.message;}};
    return {expose:await capture(window.probe.missingExpose),entry:await capture(window.probe.missingEntry)};
  });
  assert.match(failures.expose,/remote\/Unknown/);assert.match(failures.expose,/Unknown expose/);
  assert.match(failures.entry,/missing\/Button/);assert.match(failures.entry,/4101\/missing\/remoteEntry.js/);
  evidence.failures = failures; evidence.checks.push('missing expose and entry diagnostics');
  // New contexts avoid a previous successful chunk/CSS load masking failure behavior.
  const lazyFailure = await browser.newPage();
  await lazyFailure.goto('http://127.0.0.1:4100');await lazyFailure.waitForFunction(() => window.probe?.ready);
  const lazyUrls = evidence.requests.filter(r => r.url.includes(':4101/') && r.url.endsWith('.js') && !r.url.endsWith('remoteEntry.js'));
  const lazyUrl = lazyUrls.at(-1).url;
  await lazyFailure.route(lazyUrl,route => route.abort('failed'));
  await lazyFailure.locator('#button-a').click();
  await lazyFailure.waitForFunction(() => window.lastLazyError,{}, {timeout:15000});
  evidence.lazyFailure = await lazyFailure.evaluate(() => window.lastLazyError);
  assert.ok(evidence.lazyFailure.length); evidence.checks.push('missing lazy chunk rejects');
  await lazyFailure.close();
  const cssFailure = await browser.newPage();
  await cssFailure.route('http://127.0.0.1:4101/**/*.css',route => route.abort('failed'));
  const cssErrors = [];
  cssFailure.on('pageerror',e => cssErrors.push(e.message));
  const cssErrorEvent = cssFailure.waitForEvent('pageerror', {timeout:15000});
  await cssFailure.goto('http://127.0.0.1:4100');
  await cssErrorEvent;
  assert.ok(cssErrors.some(e => /remote\/Button/.test(e)),JSON.stringify(cssErrors));
  evidence.cssFailure = cssErrors; evidence.checks.push('missing CSS rejects remote load');
  await cssFailure.close();
  await page.screenshot({path:path.join(base,'evidence.png')});
  evidence.consoleErrorsIncludingExpectedFailures = consoleErrors;
  evidence.requestFailuresIncludingExpectedFailures = requestFailures;
  evidence.passed = true;
} catch (error) {
  evidence.passed = false;evidence.failure = error.stack;process.exitCode = 1;
} finally {
  if (browser) await browser.close();
  await Promise.all(servers.map(server => new Promise(resolve => server.close(resolve))));
  await fs.writeFile(path.join(base,`evidence-${evidence.mode}-${evidence.serve}.json`),JSON.stringify(evidence,null,2));
  console.log(JSON.stringify(evidence,null,2));
}
