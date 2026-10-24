import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { describe, expect, it, vi } from 'vitest';
import { resolveConfig } from '../src/config/index.js';
import { Server } from '../src/server/index.js';

describe('public Server runtime mode', () => {
  for (const boundary of ['closed', 'before-bind', 'after-bind'] as const) {
    it(`rejects startup at the ${boundary} close boundary`, async () => {
      const root = await fs.mkdtemp(
        path.join(os.tmpdir(), 'farm-runtime-close-')
      );
      let server: Server | undefined;
      let release = () => {};
      try {
        await fs.writeFile(
          path.join(root, 'index.html'),
          '<html><head></head><body>Close</body></html>'
        );
        const inline = {
          root,
          configFile: false as const,
          compilation: { lazyCompilation: false, persistentCache: false },
          server: { port: 0, host: '127.0.0.1', hmr: { port: 0 } }
        };
        server = await Server.createServer(
          inline,
          await resolveConfig(inline, 'dev', 'development'),
          { runtime: true }
        );
        const compile = vi.spyOn(server, 'setCompiler');
        if (boundary === 'closed') {
          await server.close();
          await expect(server.listen()).rejects.toThrow(/closed/i);
        } else {
          const original = (server as any).httpServerStart.bind(server);
          let reached = () => {};
          const ready = new Promise<void>((resolve) => {
            reached = resolve;
          });
          const gate = new Promise<void>((resolve) => {
            release = resolve;
          });
          vi.spyOn(server as any, 'httpServerStart').mockImplementation(
            async (...args: any[]) => {
              const result =
                boundary === 'after-bind' ? await original(...args) : undefined;
              reached();
              await gate;
              return boundary === 'before-bind' ? original(...args) : result;
            }
          );
          const outcome = server.listen().then(
            () => null,
            (error) => error
          );
          await ready;
          await server.close();
          release();
          expect(await outcome).toBeInstanceOf(Error);
        }
        expect(compile).not.toHaveBeenCalled();
        expect(server.httpServer?.listening).toBe(false);
      } finally {
        release();
        await server?.close();
        if (server?.httpServer?.listening) {
          await new Promise<void>((resolve) =>
            requireDefined(requireDefined(server).httpServer).close(() =>
              resolve()
            )
          );
        }
        await fs.rm(root, { recursive: true, force: true });
      }
    });
  }

  it('selects runtime ownership without requiring a compilation plugin', async () => {
    const root = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-server-runtime-')
    );
    let server: Server | undefined;
    try {
      await fs.writeFile(
        path.join(root, 'index.html'),
        '<html><head></head><body>Runtime</body></html>'
      );
      const inline = {
        root,
        configFile: false as const,
        compilation: {
          input: {},
          lazyCompilation: false,
          persistentCache: false
        },
        server: { port: 0, host: '127.0.0.1', hmr: { port: 0 } }
      };
      const resolved = await resolveConfig(inline, 'dev', 'development');
      server = await Server.createServer(inline, resolved, {
        runtime: true
      } as any);
      expect((server as any).runtimeMode).toBe(true);
    } finally {
      await server?.close();
      await fs.rm(root, { recursive: true, force: true });
    }
  });

  it('compiles once and registers external inputs before the first file edit', async () => {
    const owner = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-server-runtime-external-')
    );
    const root = path.join(owner, 'site');
    const external = path.join(owner, 'external.js');
    let server: Server | undefined;
    try {
      await fs.mkdir(root);
      await fs.writeFile(external, 'export default "external initial";');
      await fs.writeFile(
        path.join(root, 'entry.js'),
        `import message from ${JSON.stringify(external.replaceAll('\\', '/'))}; console.log(message);`
      );
      await fs.writeFile(
        path.join(root, 'index.html'),
        '<!doctype html><html><head></head><body>Runtime initial<script type="module" src="./entry.js"></script></body></html>'
      );
      const inline = {
        root,
        configFile: false as const,
        compilation: {
          input: { index: './index.html' },
          lazyCompilation: false,
          persistentCache: false
        },
        server: { port: 0, host: '127.0.0.1', hmr: { port: 0 } }
      };
      const resolved = await resolveConfig(inline, 'dev', 'development');
      server = await Server.createServer(inline, resolved, { runtime: true });
      await server.listen();
      const address = requireDefined(server.httpServer).address();
      expect(typeof address).toBe('object');
      expect(server.config.server.port).toBe(
        (address as { port: number }).port
      );
      expect(server.config.compilation.define?.FARM_HMR_PORT).toBe(
        String((address as { port: number }).port)
      );
      expect(server.runtime?.compiler).toBe(server.compiler);
      expect(server.runtime?.resource('index.html')?.toString()).toContain(
        'Runtime initial'
      );
      expect(
        server.watcher.extraWatchedFiles.some(
          (file) => path.resolve(file) === external
        )
      ).toBe(true);
      const update = vi.spyOn(requireDefined(server.compiler), 'update');
      const broadcast = vi.spyOn(
        requireDefined(server.hmrEngine),
        'sendUpdate'
      );
      await vi.waitFor(() =>
        expect(
          Object.values(requireDefined(server).watcher.getWatched()).flat()
        ).toContain('external.js')
      );
      await fs.writeFile(external, 'export default "external updated";');
      await vi.waitFor(() => expect(broadcast).toHaveBeenCalled(), {
        timeout: 3000
      });
      expect(update.mock.calls.some((call) => call[1] === true)).toBe(true);
      expect(
        Object.values(requireDefined(server.runtime).resources()).some(
          (bytes) => bytes.toString().includes('external updated')
        )
      ).toBe(true);
      const directory = path.join(root, 'new-directory');
      await fs.mkdir(directory);
      await vi.waitFor(
        () =>
          expect(
            update.mock.calls
              .flatMap((call) => call[0])
              .some(
                (item) =>
                  item.path === directory.replaceAll('\\', '/') &&
                  item.type === 'added'
              )
          ).toBe(true),
        { timeout: 3000 }
      );
      await server.updateRuntime([]);
      const defaultHmr = vi
        .spyOn(requireDefined(server.hmrEngine), 'hmrUpdate')
        .mockResolvedValue(undefined);
      server.ws.wss.emit('vite:invalidate', {
        path: 'entry.js',
        message: 'runtime boundary retry'
      });
      expect(defaultHmr).not.toHaveBeenCalled();
      await server.updateRuntime([]);
      expect(update.mock.calls.every((call) => call[1] === true)).toBe(true);
    } finally {
      await server?.close();
      await fs.rm(owner, { recursive: true, force: true });
    }
  });

  it('rejects initial compilation failure and closes the listener and watcher', async () => {
    const root = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-server-runtime-failure-')
    );
    let server: Server | undefined;
    try {
      await fs.writeFile(
        path.join(root, 'index.html'),
        '<html><head></head><body><script type="module" src="./entry.js"></script></body></html>'
      );
      await fs.writeFile(path.join(root, 'entry.js'), 'export function {');
      const inline = {
        root,
        configFile: false as const,
        compilation: { lazyCompilation: false, persistentCache: false },
        server: { port: 0, host: '127.0.0.1', hmr: { port: 0 } }
      };
      server = await Server.createServer(
        inline,
        await resolveConfig(inline, 'dev', 'development'),
        { runtime: true }
      );
      await expect(server.listen()).rejects.toThrow(/Parse/);
      expect(server.httpServer?.listening).toBe(false);
      expect(server.watcher.getInternalWatcher()).toBeNull();
    } finally {
      await server?.close();
      await fs.rm(root, { recursive: true, force: true });
    }
  });

  it('cleans up when server initialization fails before listen', async () => {
    const root = await fs.mkdtemp(
      path.join(os.tmpdir(), 'farm-server-runtime-initialize-')
    );
    let captured: Server | undefined;
    try {
      await fs.writeFile(
        path.join(root, 'index.html'),
        '<html><head></head><body>Runtime</body></html>'
      );
      const inline = {
        root,
        configFile: false as const,
        compilation: { lazyCompilation: false, persistentCache: false },
        server: {
          port: 0,
          host: '127.0.0.1',
          hmr: { port: 0 },
          middlewares: [
            (server: Server) => {
              captured = server;
              throw new Error('runtime middleware initialization failure');
            }
          ]
        }
      };
      await expect(
        Server.createServer(
          inline,
          await resolveConfig(inline, 'dev', 'development'),
          { runtime: true }
        )
      ).rejects.toThrow('runtime middleware initialization failure');
      expect(captured?.watcher.getInternalWatcher()).toBeNull();
    } finally {
      await captured?.close();
      await fs.rm(root, { recursive: true, force: true });
    }
  });
});

function requireDefined<T>(value: T | null | undefined): T {
  if (value === null || value === undefined) {
    throw new Error('Expected initialized server state');
  }
  return value;
}
