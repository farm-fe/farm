import { createServer } from 'node:http';
import type { AddressInfo } from 'node:net';
import { describe, expect, it, vi } from 'vitest';
import { resourceMiddleware } from '../src/server/middlewares/resource.js';
import { ServerRuntime } from '../src/server/runtime.js';

describe('runtime HTTP resources', () => {
  it('serves the committed buffer instead of the live compiler during an update', async () => {
    let live = Buffer.from('complete initial page');
    const compiler = {
      compile: vi.fn(async () => {}),
      update: vi.fn(async () => ({})),
      waitForCompileFinish: vi.fn(async () => {}),
      flushCache: vi.fn(async () => {}),
      resources: () => ({ 'index.html': live }),
      resource: () => live,
      compiling: false
    };
    const runtime = new ServerRuntime(compiler as any);
    await runtime.compile();
    live = Buffer.from('partial native output');
    const middleware = resourceMiddleware({
      compiler,
      runtime,
      runtimeMode: true,
      waitForRuntimeReady: async () => {},
      config: {},
      publicPath: '/'
    } as any);
    const http = createServer((request, response) => {
      void middleware(request, response, () => {
        response.writeHead(404).end();
      });
    });
    try {
      await new Promise<void>((resolve) =>
        http.listen(0, '127.0.0.1', resolve)
      );
      const response = await fetch(
        `http://127.0.0.1:${(http.address() as AddressInfo).port}/index.html`
      );
      expect(await response.text()).toBe('complete initial page');
    } finally {
      http.closeAllConnections();
      await new Promise<void>((resolve, reject) =>
        http.close((error) => (error ? reject(error) : resolve()))
      );
      await runtime.close();
    }
  });
});
