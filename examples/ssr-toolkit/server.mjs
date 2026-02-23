import { createServer as createNodeServer } from 'node:http';
import { createSsrServer } from '../../packages/ssr/dist/index.js';
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

const ssrServer = await createSsrServer(
  createSsrServerOptions({
    runtime,
    hmrPort
  })
);

const hostServer = createNodeServer((req, res) => {
  if (req.url === '/api/ping') {
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify(createPingPayload(runtime)));
    return;
  }

  ssrServer.middlewares(req, res, (error) => {
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
  await ssrServer.close();
};

process.on('SIGINT', async () => {
  await close();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await close();
  process.exit(0);
});
