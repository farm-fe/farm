/**
 * Real Firefox regression checks for #2266, without building Farm's Rust core.
 * Requires installed repository dependencies plus playwright-firefox and ws.
 * These extra packages can live outside the repository, for example:
 *
 *   $browserDeps = Join-Path $env:TEMP 'farm-hmr-firefox'
 *   npm install --prefix $browserDeps playwright-firefox@1.55.0 ws
 *   $env:FARM_FIREFOX_PACKAGE_PATH = Join-Path $browserDeps 'node_modules/playwright-firefox'
 *   node packages/runtime-plugin-hmr/tests/firefox-navigation.cjs
 *
 * Set FARM_FIREFOX_PACKAGE_PATH in the shell environment. The browser package's
 * install script downloads Firefox; its CLI can also run `install firefox`.
 * Optional arguments: --root <checkout>, --baseline <git-ref>, --output <json>.
 * This harness serves the actual transpiled HMR client with an overlay stub.
 * It uses real HTTP navigation and WebSockets; no native compiler is involved.
 */
const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');
const fs = require('node:fs');
const http = require('node:http');
const { createRequire } = require('node:module');
const path = require('node:path');

function argument(name, fallback) {
  const index = process.argv.indexOf(name);
  return index === -1 ? fallback : process.argv[index + 1];
}

const root = path.resolve(
  argument('--root', path.resolve(__dirname, '../../..'))
);
const baselineRef = argument('--baseline', '2000ef8a');
const outputPath = argument('--output', undefined);
const repositoryRequire = createRequire(path.join(root, 'package.json'));
const browserPackage = process.env.FARM_FIREFOX_PACKAGE_PATH;
const browserRequire = browserPackage
  ? createRequire(path.join(path.resolve(browserPackage), 'package.json'))
  : repositoryRequire;
const { firefox } = browserPackage
  ? require(path.resolve(browserPackage))
  : browserRequire('playwright-firefox');
const { WebSocketServer } = browserRequire('ws');
const ts = repositoryRequire('typescript');
const clientPath = 'packages/runtime-plugin-hmr/src/hmr-client.ts';
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function clientJavaScript(source) {
  // TypeScript erases the type-only imports. The two remaining imports are
  // replaced with an inert overlay and logger to isolate socket recovery.
  return ts
    .transpileModule(source, {
      compilerOptions: {
        target: ts.ScriptTarget.ES2020,
        module: ts.ModuleKind.ES2020
      }
    })
    .outputText.replace(/^import .*?;\s*$/gm, '');
}

async function fixture(browser, source) {
  const state = {
    sourceLoads: 0,
    pings: 0,
    terminateNext: false,
    replacementCloses: 0
  };
  const trace = [];
  const sockets = new Set();
  const server = http.createServer((request, response) => {
    const url = new URL(request.url, 'http://localhost');
    response.setHeader('Content-Type', 'text/html');
    if (url.pathname === '/trace') {
      trace.push(url.searchParams.get('event'));
      response.end();
    } else if (url.pathname === '/__hmr') {
      state.pings++;
      response.writeHead(400);
      response.end();
    } else if (url.pathname === '/target') {
      setTimeout(
        () => response.end('<!doctype html><h1>Target reached</h1>'),
        700
      );
    } else if (url.pathname === '/client.js') {
      response.setHeader('Content-Type', 'text/javascript');
      response.end(
        'const logger={debug(){},warn(){},error(){}};' +
          'const ErrorOverlay=class {};const overlayId="overlay";' +
          clientJavaScript(source) +
          '\nwindow.client=new HmrClient({});window.client.connect();'
      );
    } else if (url.pathname === '/') {
      state.sourceLoads++;
      response.end(`<!doctype html>
        <button id="navigate" onclick="location.assign('/target')">Navigate</button>
        <button id="interact">Continue working</button>
        <script>
          window.FARM_HMR_HOST=true;
          window.FARM_HMR_PROTOCOL='';
          window.FARM_HMR_PATH='/__hmr';
          const record=(event)=>navigator.sendBeacon('/trace?event='+encodeURIComponent(event));
          for(const event of ['pageshow','pagehide','beforeunload']) {
            window.addEventListener(event,()=>record(event));
          }
          window.addEventListener('beforeunload',()=>setTimeout(()=>{
            record('next-task-socket-state:'+window.client.socket.readyState);
          },0));
          const NativeWebSocket=WebSocket;
          window.WebSocket=class extends NativeWebSocket {
            constructor(...args) {
              super(...args);
              this.addEventListener('close',event=>record('socket-close:'+event.code));
            }
          };
        </script>
        <script type="module" src="/client.js"></script>`);
    } else {
      response.writeHead(404);
      response.end();
    }
  });
  const socketServer = new WebSocketServer({ server, path: '/__hmr' });
  socketServer.on('connection', (socket) => {
    sockets.add(socket);
    socket.on('close', () => sockets.delete(socket));
    if (state.terminateNext) {
      state.terminateNext = false;
      setTimeout(() => {
        state.replacementCloses++;
        socket.terminate();
      }, 25);
    }
  });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const url = `http://127.0.0.1:${server.address().port}/`;
  const page = await browser.newPage();
  page.setDefaultTimeout(5000);
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  const test = {
    page,
    url,
    state,
    trace,
    errors,
    terminate() {
      for (const socket of sockets) socket.terminate();
    },
    sendUpdate() {
      for (const socket of sockets) {
        socket.send(
          JSON.stringify({ type: 'custom', event: 'test-update', data: {} })
        );
      }
    },
    async close() {
      await page.close();
      for (const socket of sockets) socket.terminate();
      await new Promise((resolve) => socketServer.close(resolve));
      server.closeIdleConnections();
      await new Promise((resolve) => server.close(resolve));
    }
  };
  try {
    await page.goto(url);
    await page.waitForFunction(() => window.client?.socket.readyState === 1);
    return test;
  } catch (error) {
    await test.close();
    throw error;
  }
}

async function dismissNavigation(page) {
  await page.evaluate(() => {
    window.onbeforeunload = (event) => {
      event.preventDefault();
      event.returnValue = '';
    };
  });
  const dismissed = page
    .waitForEvent('dialog', { timeout: 5000 })
    .then((dialog) => dialog.dismiss());
  await Promise.all([
    page.click('#navigate', { noWaitAfter: true }),
    dismissed
  ]);
  await pause(100);
  await page.evaluate(() => {
    window.onbeforeunload = null;
  });
}

async function disconnectDuringRecovery(test) {
  // Hold the ping unresolved so navigation starts with a genuinely CLOSED
  // socket and outstanding recovery, rather than after a completed reload.
  await test.page.evaluate(() => {
    window.originalFetch = window.fetch;
    window.fetch = () => new Promise(() => {});
  });
  test.terminate();
  await test.page.waitForFunction(() => window.client.socket.readyState === 3);
}

async function main() {
  const baseline = execFileSync(
    'git',
    ['show', `${baselineRef}:${clientPath}`],
    {
      cwd: root,
      encoding: 'utf8'
    }
  );
  const fixed = fs.readFileSync(path.join(root, clientPath), 'utf8');
  const browser = await firefox.launch({ headless: true });
  const result = {
    issue: 2266,
    baseline: baselineRef,
    firefox: browser.version(),
    executedAt: new Date().toISOString(),
    success: false,
    cases: []
  };
  async function run(version, source, name, check) {
    const test = await fixture(browser, source);
    try {
      const details = await check(test);
      assert.deepEqual(test.errors, []);
      result.cases.push({ version, name, passed: true, ...details });
    } finally {
      await test.close();
    }
  }
  try {
    for (const [version, source] of [
      ['baseline', baseline],
      ['fixed', fixed]
    ]) {
      await run(version, source, 'delayed-location-assign', async (test) => {
        const reached = test.page
          .waitForURL('**/target', { timeout: 3500 })
          .then(
            () => true,
            () => false
          );
        await test.page.click('#navigate', { noWaitAfter: true });
        const targetReached = await reached;
        await pause(100);
        assert.equal(targetReached, version === 'fixed');
        if (version === 'fixed') assert.equal(test.state.pings, 0);
        return { targetReached, pings: test.state.pings, trace: test.trace };
      });
      await run(
        version,
        source,
        'canceled-navigation-and-server-restart',
        async (test) => {
          await dismissNavigation(test.page);
          assert.equal(test.page.url(), test.url);
          assert.equal(
            await test.page.evaluate(() => window.client.socket.readyState),
            1
          );
          const loads = test.state.sourceLoads;
          test.terminate();
          await test.page.waitForFunction(
            () => window.client?.socket.readyState === 1
          );
          await pause(500);
          assert.equal(test.state.sourceLoads - loads, 1);
          return {
            reloadsAfterServerDisconnect: test.state.sourceLoads - loads
          };
        }
      );
    }
    await run(
      'fixed',
      fixed,
      'closed-socket-canceled-navigation',
      async (test) => {
        await disconnectDuringRecovery(test);
        await dismissNavigation(test.page);
        await test.page.waitForFunction(
          () => window.client.socket.readyState === 1
        );
        await test.page.evaluate(() => {
          window.client.customListenersMap.set('test-update', [
            () => {
              window.receivedUpdate = true;
            }
          ]);
        });
        test.sendUpdate();
        await test.page.waitForFunction(() => window.receivedUpdate === true);
        // A failed speculative reconnect must not reload; actual input can then
        // resume recovery in the canceled document.
        test.terminate();
        await test.page.waitForFunction(
          () => window.client.socket.readyState === 3
        );
        await test.page.click('#interact');
        await test.page.waitForFunction(
          () => window.client.socket.readyState === 1
        );
        await test.page.evaluate(() => {
          window.fetch = window.originalFetch;
        });
        const loads = test.state.sourceLoads;
        test.terminate();
        await pause(500);
        assert.equal(test.state.sourceLoads - loads, 1);
        return { updateReceived: true, recoveryAfterTrustedInput: true };
      }
    );
    await run(
      'fixed',
      fixed,
      'closed-socket-delayed-navigation-replacement-fails',
      async (test) => {
        await disconnectDuringRecovery(test);
        await test.page.evaluate(() => {
          window.fetch = window.originalFetch;
        });
        test.state.terminateNext = true;
        const loads = test.state.sourceLoads;
        const pings = test.state.pings;
        const reached = test.page.waitForURL('**/target', { timeout: 3500 });
        await test.page.click('#navigate', { noWaitAfter: true });
        await reached;
        assert.equal(test.state.replacementCloses, 1);
        assert.equal(test.state.sourceLoads, loads);
        assert.equal(test.state.pings, pings);
        return {
          targetReached: true,
          replacementCloses: 1,
          extraReloads: 0,
          extraPings: 0
        };
      }
    );
    result.success = true;
  } catch (error) {
    result.error = error.stack;
    process.exitCode = 1;
  } finally {
    await browser.close();
    const json = `${JSON.stringify(result, null, 2)}\n`;
    if (outputPath) {
      fs.mkdirSync(path.dirname(path.resolve(outputPath)), { recursive: true });
      fs.writeFileSync(outputPath, json);
    }
    process.stdout.write(json);
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
