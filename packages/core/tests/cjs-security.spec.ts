// cspell:words maxFragments maxBufferedChunks sensitiveHeaders
import { readFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { createRequire } from 'node:module';
import { PassThrough } from 'node:stream';
import { describe, expect, test } from 'vitest';

const bundle = readFileSync(
  new URL('../dist/cjs/index.cjs', import.meta.url),
  'utf8'
);
const { Server } = createRequire(import.meta.url)('../dist/cjs/index.cjs');

// npm audit sees installed dependencies, not these inlined implementations.
// Check the actual require() artifact as well as the build lockfile.
describe('Core CJS bundled dependency security', () => {
  test('the bundled HMR server rejects a missing Upgrade header without throwing', async () => {
    const server = new Server();
    server.config = { server: { port: 0 } };
    server.httpServer = createServer();
    const socket = new PassThrough();
    const errors: Error[] = [];
    try {
      await server.createWebSocketServer();
      server.ws.wss.on('wsClientError', (error: Error) => errors.push(error));
      expect(() =>
        server.ws.wss.handleUpgrade(
          { method: 'GET', headers: {} },
          socket,
          Buffer.alloc(0),
          () => {
            throw new Error('Invalid upgrade was accepted');
          }
        )
      ).not.toThrow();
      expect(errors.map((error) => error.message)).toEqual([
        'Invalid Upgrade header'
      ]);
      expect(server.ws.wss.options.maxFragments).toBeGreaterThan(0);
      expect(server.ws.wss.options.maxBufferedChunks).toBeGreaterThan(0);
      expect(Number.isSafeInteger(server.ws.wss.options.maxFragments)).toBe(
        true
      );
      expect(
        Number.isSafeInteger(server.ws.wss.options.maxBufferedChunks)
      ).toBe(true);
    } finally {
      socket.destroy();
      await server.ws?.close();
    }
  });

  test('follow-redirects bundles configurable sensitive header filtering', () => {
    expect(
      bundle.includes('sensitiveHeaders.concat(options.sensitiveHeaders)')
    ).toBe(true);
    expect(
      bundle.includes(
        'removeMatchingHeaders(this._headerFilter, this._options.headers)'
      )
    ).toBe(true);
  });

  test('cross-spawn uses the bounded backslash escaping expression', () => {
    expect(bundle.includes(String.raw`/(?=(\\+?)?)\1"/g`)).toBe(true);
    expect(bundle.includes(String.raw`/(?=(\\+?)?)\1$/`)).toBe(true);
    expect(bundle.includes(String.raw`/(\\*)"/g`)).toBe(false);
    expect(bundle.includes(String.raw`/(\\*)$/`)).toBe(false);
  });
});
