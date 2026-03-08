import { EventEmitter } from 'node:events';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import type { FarmCliOptions, UserConfig } from '@farmfe/core';
import { describe, expect, it, vi } from 'vitest';
import {
  buildSsrAppWithFactories,
  createSsrPreviewServerWithFactories,
  importModuleWithCssInterop,
  resolveSsrPreviewMetadataWithFactories,
  type SsrBuildPreviewFactories,
  type SsrPreviewClientServerLike,
  startSsrPreviewServerWithFactories
} from '../src/build-preview.js';
import type {
  SsrDevHostServerLike,
  SsrMiddleware,
  SsrMiddlewareServer
} from '../src/dev-server.js';

class FakeHostServer extends EventEmitter implements SsrDevHostServerLike {
  listening = false;
  listenCalls: Array<{ port: number; hostname?: string }> = [];
  closeCalls = 0;

  listen(port: number, hostname?: string) {
    this.listenCalls.push({ port, hostname });
    this.listening = true;
    queueMicrotask(() => this.emit('listening'));
    return this;
  }

  close(callback?: (error?: Error) => void) {
    this.closeCalls++;
    this.listening = false;
    callback?.();
    return this;
  }
}

class FailingHostServer extends EventEmitter implements SsrDevHostServerLike {
  listening = false;
  closeCalls = 0;

  listen(_port: number, _hostname?: string) {
    queueMicrotask(() =>
      this.emit('error', new Error('preview listen failed'))
    );
    return this;
  }

  close(callback?: (error?: Error) => void) {
    this.closeCalls++;
    callback?.();
    return this;
  }
}

function createMiddlewareServer(...middlewares: SsrMiddleware[]) {
  const stack = [...middlewares];
  const middlewareServer = ((req, res, next) => {
    let index = 0;

    const run = (error?: unknown) => {
      if (error) {
        next?.(error);
        return;
      }

      const middleware = stack[index++];
      if (!middleware) {
        next?.();
        return;
      }
      middleware(req, res, run);
    };

    run();
  }) as SsrMiddlewareServer;

  middlewareServer.use = (middleware: SsrMiddleware) => {
    stack.push(middleware);
  };

  return middlewareServer;
}

function createFactories(params: {
  hostServer: SsrDevHostServerLike;
  clientPreviewServer?: SsrPreviewClientServerLike;
  readFile?: (filePath: string) => Promise<string>;
  importModule?: (filePath: string) => Promise<Record<string, unknown>>;
}) {
  const calls: Array<FarmCliOptions & UserConfig> = [];
  const clientPreviewServer =
    params.clientPreviewServer ??
    ({
      middlewares: createMiddlewareServer((_req, _res, next) => next()),
      close: vi.fn(async () => undefined)
    } satisfies SsrPreviewClientServerLike);

  const factories: SsrBuildPreviewFactories = {
    runBuild: vi.fn(async (config) => {
      calls.push(config);
    }),
    resolveBuildOutput: vi.fn(async (config) => ({
      root: config.root ?? '/project',
      outputPath:
        config.configFile === 'client'
          ? '/project/dist/client'
          : '/project/dist/server'
    })),
    createClientPreviewServer: vi.fn(async () => clientPreviewServer),
    createHostServer: vi.fn(() => params.hostServer),
    readFile:
      params.readFile ??
      vi.fn(async () => '<html><body><!--app-html--></body></html>'),
    importModule:
      params.importModule ??
      vi.fn(async () => ({
        default(url: string) {
          return `<div>preview:${url}</div>`;
        }
      }))
  };

  return {
    factories,
    calls,
    clientPreviewServer
  };
}

describe('farm ssr build/preview api', () => {
  it('buildSsrApp runs client build then server build', async () => {
    const hostServer = new FakeHostServer();
    const { factories, calls } = createFactories({ hostServer });
    const clientConfig = { configFile: 'client' };
    const serverConfig = { configFile: 'server' };

    await buildSsrAppWithFactories(
      {
        client: clientConfig,
        server: serverConfig
      },
      factories
    );

    expect(calls).toEqual([clientConfig, serverConfig]);
  });

  it('renders preview html through built server entry and default template', async () => {
    const trace: string[] = [];
    const hostServer = new FakeHostServer();
    const { factories } = createFactories({
      hostServer,
      clientPreviewServer: {
        middlewares: createMiddlewareServer((_req, _res, next) => {
          trace.push('client');
          next();
        }),
        close: vi.fn(async () => undefined)
      }
    });

    const previewServer = await createSsrPreviewServerWithFactories(
      {
        client: { configFile: 'client' },
        server: { configFile: 'server' },
        ssr: {
          entry: 'entry-server.js'
        }
      },
      factories
    );

    const req = {
      method: 'GET',
      url: '/preview',
      headers: { accept: 'text/html' }
    };
    const headers: Record<string, string> = {};
    const body = await new Promise<string>((resolve) => {
      const res = {
        setHeader(name: string, value: string) {
          headers[name] = value;
        },
        end(html: string) {
          resolve(html);
        }
      };

      previewServer.middlewares(req as never, res as never, () =>
        resolve('next')
      );
    });

    expect(body).toContain('<div>preview:/preview</div>');
    expect(headers['Content-Type']).toContain('text/html');
    expect(trace).toEqual([]);
    expect((factories.readFile as any).mock.calls[0]?.[0]).toBe(
      '/project/dist/client/index.html'
    );
    expect((factories.importModule as any).mock.calls[0]?.[0]).toBe(
      '/project/dist/server/entry-server.js'
    );

    await previewServer.close();
  });

  it('supports preview template.transform with built html template', async () => {
    const hostServer = new FakeHostServer();
    const builtTemplate =
      '<html><body><div id=root></div><script src=/index.abc.js data-farm-resource=true></script></body></html>';
    const { factories } = createFactories({
      hostServer,
      readFile: vi.fn(async () => builtTemplate)
    });

    const previewServer = await createSsrPreviewServerWithFactories(
      {
        client: { configFile: 'client' },
        server: { configFile: 'server' },
        ssr: {
          entry: 'entry-server.js',
          template: {
            transform({ template, appHtml }) {
              return template.replace(
                '<div id=root></div>',
                `<div id=root>${appHtml}</div>`
              );
            }
          }
        }
      },
      factories
    );

    const body = await new Promise<string>((resolve) => {
      previewServer.middlewares(
        {
          method: 'GET',
          url: '/preview-transform',
          headers: { accept: 'text/html' }
        } as never,
        {
          setHeader() {},
          end(html: string) {
            resolve(html);
          }
        } as never,
        () => resolve('next')
      );
    });

    expect(body).toContain(
      '<div id=root><div>preview:/preview-transform</div></div>'
    );
    expect(body).toContain('data-farm-resource=true');
    expect(body).not.toContain('/src/main.ts');
    await previewServer.close();
  });

  it('prefers built output template when preview template.file is relative', async () => {
    const hostServer = new FakeHostServer();
    const readFileMock = vi.fn(async (filePath: string) => {
      if (filePath === '/project/dist/client/index.html') {
        return '<html><body><div id=root><!--app-html--></div><script src=/index.built.js data-farm-resource=true></script></body></html>';
      }

      if (filePath === '/project/index.html') {
        return '<html><body><div id=root><!--app-html--></div><script type="module" src="/src/main.ts"></script></body></html>';
      }

      throw Object.assign(new Error(`unexpected file: ${filePath}`), {
        code: 'ENOENT'
      });
    });
    const { factories } = createFactories({
      hostServer,
      readFile: readFileMock
    });

    const previewServer = await createSsrPreviewServerWithFactories(
      {
        client: { configFile: 'client' },
        server: { configFile: 'server' },
        ssr: {
          entry: 'entry-server.js',
          template: {
            file: 'index.html'
          }
        }
      },
      factories
    );

    const body = await new Promise<string>((resolve) => {
      previewServer.middlewares(
        {
          method: 'GET',
          url: '/about',
          headers: { accept: 'text/html' }
        } as never,
        {
          setHeader() {},
          end(html: string) {
            resolve(html);
          }
        } as never,
        () => resolve('next')
      );
    });

    expect(body).toContain('data-farm-resource=true');
    expect(body).not.toContain('/src/main.ts');
    expect(readFileMock.mock.calls[0]?.[0]).toBe(
      '/project/dist/client/index.html'
    );
    await previewServer.close();
  });

  it('falls back to root template file when relative preview template is absent in build output', async () => {
    const hostServer = new FakeHostServer();
    const readFileMock = vi.fn(async (filePath: string) => {
      if (filePath === '/project/dist/client/custom.html') {
        throw Object.assign(new Error('missing output template'), {
          code: 'ENOENT'
        });
      }

      if (filePath === '/project/custom.html') {
        return '<html><body><div id=root><!--app-html--></div><script src=/custom.js data-farm-resource=true></script></body></html>';
      }

      throw Object.assign(new Error(`unexpected file: ${filePath}`), {
        code: 'ENOENT'
      });
    });
    const { factories } = createFactories({
      hostServer,
      readFile: readFileMock
    });

    const previewServer = await createSsrPreviewServerWithFactories(
      {
        client: { configFile: 'client' },
        server: { configFile: 'server' },
        ssr: {
          entry: 'entry-server.js',
          template: {
            file: 'custom.html'
          }
        }
      },
      factories
    );

    const body = await new Promise<string>((resolve) => {
      previewServer.middlewares(
        {
          method: 'GET',
          url: '/fallback-template',
          headers: { accept: 'text/html' }
        } as never,
        {
          setHeader() {},
          end(html: string) {
            resolve(html);
          }
        } as never,
        () => resolve('next')
      );
    });

    expect(body).toContain('data-farm-resource=true');
    expect(readFileMock.mock.calls.map((args) => args[0])).toEqual([
      '/project/dist/client/custom.html',
      '/project/custom.html'
    ]);
    await previewServer.close();
  });

  it('uses default preview entry when ssr.entry is omitted', async () => {
    const hostServer = new FakeHostServer();
    const { factories } = createFactories({ hostServer });

    const previewServer = await createSsrPreviewServerWithFactories(
      {
        client: { configFile: 'client' },
        server: { configFile: 'server' },
        ssr: {}
      },
      factories
    );

    await new Promise<void>((resolve) => {
      previewServer.middlewares(
        {
          method: 'GET',
          url: '/default-entry',
          headers: { accept: 'text/html' }
        } as never,
        {
          setHeader() {},
          end() {
            resolve();
          }
        } as never,
        () => resolve()
      );
    });

    expect((factories.importModule as any).mock.calls[0]?.[0]).toBe(
      '/project/dist/server/index.js'
    );

    await previewServer.close();
  });

  it('resolves preview metadata with entry/template defaults', async () => {
    const hostServer = new FakeHostServer();
    const { factories } = createFactories({ hostServer });

    const metadata = await resolveSsrPreviewMetadataWithFactories(
      {
        client: { configFile: 'client' },
        server: { configFile: 'server' },
        ssr: {}
      },
      factories
    );

    expect(metadata.clientBuildOutput).toEqual({
      root: '/project',
      outputPath: '/project/dist/client'
    });
    expect(metadata.serverBuildOutput).toEqual({
      root: '/project',
      outputPath: '/project/dist/server'
    });
    expect(metadata.templateFilePath).toBe('/project/dist/client/index.html');
    expect(metadata.serverEntryFilePath).toBe('/project/dist/server/index.js');
    expect(metadata.manifestFileCandidates).toEqual([
      '/project/dist/client/manifest.json',
      '/project/dist/client/ssr-manifest.json'
    ]);
  });

  it('falls through to preview static middlewares for non-html requests', async () => {
    const trace: string[] = [];
    const hostServer = new FakeHostServer();
    const { factories } = createFactories({
      hostServer,
      clientPreviewServer: {
        middlewares: createMiddlewareServer((_req, _res, next) => {
          trace.push('client');
          next();
        }),
        close: vi.fn(async () => undefined)
      }
    });

    const previewServer = await createSsrPreviewServerWithFactories(
      {
        client: { configFile: 'client' },
        server: { configFile: 'server' },
        ssr: {
          entry: 'entry-server.js'
        }
      },
      factories
    );

    const req = {
      method: 'GET',
      url: '/main.js',
      headers: { accept: 'text/javascript' }
    };
    await new Promise<void>((resolve, reject) => {
      const res = {};
      previewServer.middlewares(
        req as never,
        res as never,
        (error?: unknown) => {
          if (error) {
            reject(error);
            return;
          }
          resolve();
        }
      );
    });

    expect(trace).toEqual(['client']);
    expect((factories.importModule as any).mock.calls.length).toBe(0);
    await previewServer.close();
  });

  it('cleans up preview resources when start fails during listen', async () => {
    const hostServer = new FailingHostServer();
    const clientPreviewServer = {
      middlewares: createMiddlewareServer((_req, _res, next) => next()),
      close: vi.fn(async () => undefined)
    } satisfies SsrPreviewClientServerLike;
    const { factories } = createFactories({
      hostServer,
      clientPreviewServer
    });

    await expect(
      startSsrPreviewServerWithFactories(
        {
          client: { configFile: 'client' },
          server: { configFile: 'server' },
          ssr: { entry: 'entry-server.js' }
        },
        factories
      )
    ).rejects.toThrow('preview listen failed');

    expect((clientPreviewServer.close as any).mock.calls.length).toBe(1);
    expect(hostServer.closeCalls).toBe(0);
  });

  it('uses fallback entry name in export error message', async () => {
    const hostServer = new FakeHostServer();
    const { factories } = createFactories({
      hostServer,
      importModule: vi.fn(async () => ({
        notDefault() {
          return '<div>no-default</div>';
        }
      }))
    });
    const previewServer = await createSsrPreviewServerWithFactories(
      {
        client: { configFile: 'client' },
        server: { configFile: 'server' },
        ssr: {}
      },
      factories
    );

    const body = await new Promise<string>((resolve) => {
      previewServer.middlewares(
        {
          method: 'GET',
          url: '/missing-export',
          headers: { accept: 'text/html' }
        } as never,
        {
          setHeader() {},
          end(content: string) {
            resolve(content);
          }
        } as never,
        () => resolve('next')
      );
    });

    expect(body).toContain('from "index.js"');
    await previewServer.close();
  });

  it('imports server module with css side-effect imports via fallback interop', async () => {
    const tmpRoot = await mkdtemp(path.join(tmpdir(), 'farm-ssr-css-interop-'));
    const entryPath = path.join(tmpRoot, 'entry.mjs');
    const depPath = path.join(tmpRoot, 'dep.mjs');
    const cssPath = path.join(tmpRoot, 'style.css');

    try {
      await writeFile(depPath, "export const value = 'ok';\n", 'utf-8');
      await writeFile(cssPath, '.page { color: red; }\n', 'utf-8');
      await writeFile(
        entryPath,
        "import './style.css';\nimport { value } from './dep.mjs';\nexport default () => `interop:${value}`;\n",
        'utf-8'
      );

      const mod = await importModuleWithCssInterop(entryPath, readFile);
      expect(typeof mod.default).toBe('function');
      expect((mod.default as () => string)()).toBe('interop:ok');
    } finally {
      await rm(tmpRoot, { recursive: true, force: true });
    }
  });

  it('keeps createRequire usable when module is loaded through css fallback interop', async () => {
    const tmpRoot = await mkdtemp(path.join(tmpdir(), 'farm-ssr-css-require-'));
    const entryPath = path.join(tmpRoot, 'entry.mjs');
    const cssPath = path.join(tmpRoot, 'style.css');

    try {
      await writeFile(cssPath, '.entry { color: blue; }\n', 'utf-8');
      await writeFile(
        entryPath,
        "import { createRequire } from 'module';\nimport './style.css';\nconst require = createRequire(import.meta.url);\nexport default () => typeof require;\n",
        'utf-8'
      );

      const mod = await importModuleWithCssInterop(entryPath, readFile);
      expect((mod.default as () => string)()).toBe('function');
    } finally {
      await rm(tmpRoot, { recursive: true, force: true });
    }
  });
});
