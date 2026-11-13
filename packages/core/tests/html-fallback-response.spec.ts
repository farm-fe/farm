import http from 'node:http';
import type { AddressInfo } from 'node:net';
import connect from 'connect';
import getEtag from 'etag';
import { expect, test, vi } from 'vitest';
import type { Server } from '../src/server/index.js';
import { htmlFallbackMiddleware } from '../src/server/middlewares/htmlFallback.js';
import { notFoundMiddleware } from '../src/server/middlewares/notFound.js';

vi.mock('../src/utils/index.js', () => ({
  cleanUrl: (url: string) => url.replace(/[?#].*$/, ''),
  removeSlash: (path: string) => path.replace(/^[/\\\\]+/, '')
}));

const html = '<h1>Farm</h1>';
const cases = [
  [
    'explicit HTML',
    '/page.html',
    'GET',
    { 'page.html': html },
    {},
    200,
    html,
    0
  ],
  ['root', '/', 'GET', { 'index.html': html }, {}, 200, html, 0],
  ['extensionless', '/page', 'GET', { 'page.html': html }, {}, 200, html, 0],
  [
    'SPA fallback',
    '/route/nested',
    'GET',
    { 'index.html': html },
    {},
    200,
    html,
    0
  ],
  ['missing index', '/route', 'GET', {}, {}, 404, '', 1],
  ['HEAD', '/page.html', 'HEAD', { 'page.html': html }, {}, 200, '', 0],
  [
    'ETag',
    '/page.html',
    'GET',
    { 'page.html': html },
    { 'if-none-match': getEtag(html, { weak: true }) },
    304,
    '',
    0
  ],
  [
    'non HTML accept',
    '/page.html',
    'GET',
    { 'page.html': html },
    { accept: 'application/json' },
    404,
    '',
    1
  ]
] as const;

for (const runtimeMode of [false, true]) {
  test.each(cases)(
    'HTML response terminates middleware: %s (runtime=' + runtimeMode + ')',
    async (_, url, method, resources, headers, status, body, nextCalls) => {
      const resource = (name: string) =>
        (resources as Record<string, string>)[name];
      const next = vi.fn();
      const app = connect();
      app.use(
        htmlFallbackMiddleware({
          runtimeMode,
          waitForRuntimeReady: async () => {},
          compiler: { resource },
          runtime: { resource },
          config: {},
          serverOptions: { appType: 'spa' }
        } as unknown as Server)
      );
      app.use((req, res, forward) => {
        next();
        forward();
      });
      app.use(notFoundMiddleware());
      const server = http.createServer(app);
      try {
        await new Promise<void>((resolve) =>
          server.listen(0, '127.0.0.1', resolve)
        );
        const result = await new Promise<{ status: number; body: string }>(
          (resolve, reject) => {
            http
              .get(
                {
                  hostname: '127.0.0.1',
                  port: (server.address() as AddressInfo).port,
                  path: url,
                  method,
                  headers
                },
                (res) => {
                  let body = '';
                  res.on('data', (chunk) => {
                    body += chunk;
                  });
                  res.on('end', () =>
                    resolve({ status: res.statusCode ?? 0, body })
                  );
                  res.on('error', reject);
                }
              )
              .on('error', reject);
          }
        );
        expect(result).toEqual({ status, body });
        expect(next).toHaveBeenCalledTimes(nextCalls);
      } finally {
        await new Promise<void>((resolve, reject) =>
          server.close((error) => (error ? reject(error) : resolve()))
        );
      }
    }
  );
}
