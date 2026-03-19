import { createServer as createNodeServer } from 'node:http';
import { createSsrRuntime } from '../../packages/ssr/dist/index.js';
import { resolveHostPort, resolveOptionalDevHmrPort } from './server/ports.mjs';
import { createRuntimeConfig } from './server/runtime-config.mjs';
import {
  createPingPayload,
  createSsrServerOptions,
  createStartupMessage
} from './server/ssr-options.mjs';

const runtime = createRuntimeConfig();
const hmrPort = await resolveOptionalDevHmrPort({
  command: runtime.command,
  host: runtime.host,
  explicitHmrPort: runtime.explicitHmrPort
});
const hostPort = await resolveHostPort({
  host: runtime.host,
  explicitHostPort: runtime.explicitHostPort
});

const ssrRuntime = await createSsrRuntime(
  createSsrServerOptions({
    runtime,
    hmrPort
  })
);

function isHtmlRequest(req) {
  const method = req.method?.toUpperCase();
  if (method !== 'GET' && method !== 'HEAD') {
    return false;
  }

  const url = req.url ?? '/';
  if (url.includes('/@') || url.includes('/__')) {
    return false;
  }

  const queryIndex = url.indexOf('?');
  const pathname = queryIndex >= 0 ? url.slice(0, queryIndex) : url;
  if (/\.[a-zA-Z0-9]+$/.test(pathname)) {
    return false;
  }

  const accept = req.headers.accept;
  if (!accept) {
    return true;
  }

  return accept.includes('text/html') || accept.includes('*/*');
}

function shouldHandleSsr(req) {
  if (!isHtmlRequest(req)) {
    return false;
  }

  const url = req.url ?? '/';
  if (url.startsWith('/api/')) {
    return false;
  }

  return true;
}

const hostServer = createNodeServer((req, res) => {
  if (req.url === '/api/ping') {
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify(createPingPayload(runtime)));
    return;
  }

  if (shouldHandleSsr(req)) {
    ssrRuntime
      .render(req.url ?? '/', req, res)
      .then((html) => {
        if (res.writableEnded) {
          return;
        }
        res.statusCode = 200;
        res.setHeader('Content-Type', 'text/html; charset=utf-8');
        res.end(html);
      })
      .catch((error) => {
        const stack = error instanceof Error ? error.stack ?? error.message : String(error);
        res.statusCode = 500;
        res.setHeader('Content-Type', 'text/plain; charset=utf-8');
        res.end(stack);
      });
    return;
  }

  ssrRuntime.middlewares(req, res, (error) => {
    if (error) {
      const stack = error instanceof Error ? error.stack ?? error.message : String(error);
      res.statusCode = 500;
      res.setHeader('Content-Type', 'text/plain; charset=utf-8');
      res.end(stack);
      return;
    }

    if (!res.writableEnded) {
      res.statusCode = 404;
      res.end('Not Found');
    }
  });
});

await new Promise((resolve, reject) => {
  hostServer.once('listening', resolve);
  hostServer.once('error', reject);
  hostServer.listen(hostPort, runtime.host);
});

console.log(createStartupMessage({ runtime, hostPort, hmrPort }));
console.log('try GET /api/ping and open /');

const close = async () => {
  await new Promise((resolve, reject) => {
    hostServer.close((error) => {
      if (error) {
        reject(error);
        return;
      }
      resolve();
    });
  });
  await ssrRuntime.close();
};

process.on('SIGINT', async () => {
  await close();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await close();
  process.exit(0);
});
