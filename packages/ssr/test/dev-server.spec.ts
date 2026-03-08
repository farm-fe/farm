import { EventEmitter } from 'node:events';
import type { FarmModuleRunner } from '@farmfe/core';
import { describe, expect, it, vi } from 'vitest';
import {
  createSsrDevServerWithFactories,
  type SsrDevCompilerLike,
  type SsrDevFarmServerLike,
  type SsrDevHostServerLike,
  type SsrDevServerFactories,
  type SsrDevWatcherLike,
  type SsrMiddleware,
  type SsrMiddlewareServer,
  startSsrDevServerWithFactories
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
    queueMicrotask(() => this.emit('error', new Error('listen failed')));
    return this;
  }

  close(callback?: (error?: Error) => void) {
    this.closeCalls++;
    callback?.();
    return this;
  }
}

class FakeWatcher implements SsrDevWatcherLike {
  private readonly listeners = {
    add: [] as Array<(path: string) => Promise<void> | void>,
    change: [] as Array<(path: string) => Promise<void> | void>,
    unlink: [] as Array<(path: string) => Promise<void> | void>
  };

  addedPaths: string[] = [];

  on(
    event: 'add' | 'change' | 'unlink',
    listener: (path: string) => Promise<void> | void
  ) {
    this.listeners[event].push(listener);
    return this;
  }

  add(paths: string | string[]) {
    this.addedPaths.push(...(Array.isArray(paths) ? paths : [paths]));
    return this;
  }

  filterWatchFile(_file: string, _root: string) {
    return true;
  }

  async emit(event: 'add' | 'change' | 'unlink', file: string) {
    for (const listener of this.listeners[event]) {
      await listener(file);
    }
  }
}

class FakeTemplateCompiler {
  constructor(private readonly templates: Record<string, string>) {}

  resource(name: string) {
    return this.templates[name] ?? this.templates[name.replace(/^\//, '')];
  }
}

class FakeServerCompiler implements SsrDevCompilerLike {
  compiling = false;
  value = 1;
  updateCalls = 0;
  moduleId = '/src/entry-server.ts';

  async compile() {}

  async update(
    _paths: Array<{ path: string; type: 'added' | 'updated' | 'removed' }>
  ) {
    this.updateCalls++;
    this.value += 1;
    return {
      added: [],
      changed: [this.moduleId],
      removed: [],
      extraWatchResult: {
        add: []
      }
    };
  }

  hasModule(path: string) {
    return path === this.moduleId;
  }

  transformModulePath(_root: string, path: string) {
    return path;
  }

  async waitForCompileFinish() {}

  fetchModule(id: string) {
    if (id !== this.moduleId) {
      return null;
    }

    return {
      id,
      url: id,
      file: id,
      invalidate: false,
      code: `const value = ${this.value}; __farm_ssr_export_name__("value", () => value);`,
      map: null
    };
  }

  resource(_name: string) {
    return null;
  }

  resources() {
    return {};
  }
}

class FakeServerCompilerWithoutHasModule extends FakeServerCompiler {
  hasModule(_path: string) {
    return false;
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

function createFarmServerMock(
  withWatcher = false,
  templateHtml = '<html><body><!--app-html--></body></html>'
) {
  const farmMiddlewares = createMiddlewareServer();
  const trace: string[] = [];
  farmMiddlewares.use((_req, _res, next) => {
    trace.push('farm');
    next();
  });

  const watcher = withWatcher ? new FakeWatcher() : undefined;
  const templateCompiler = new FakeTemplateCompiler({
    'index.html': templateHtml
  });
  const runner = {
    import: vi.fn(async () => ({
      default(url: string) {
        return `<div>render:${url}</div>`;
      }
    })),
    close: vi.fn(async () => undefined)
  } as unknown as FarmModuleRunner;

  const farmServer: SsrDevFarmServerLike = {
    middlewares: farmMiddlewares,
    root: '/project',
    getCompiler: () => templateCompiler,
    watcher,
    createModuleRunner: vi.fn(async () => runner),
    close: vi.fn(async () => undefined)
  };

  return { farmServer, runner, watcher, trace };
}

function createFactories(
  farmServer: SsrDevFarmServerLike,
  hostServer: SsrDevHostServerLike,
  serverCompiler?: FakeServerCompiler
): SsrDevServerFactories {
  return {
    createFarmServer: vi.fn(async () => farmServer),
    createHostServer: vi.fn(() => hostServer),
    createServerCompiler: vi.fn(async () => {
      if (!serverCompiler) {
        throw new Error('server compiler is not configured in test factory');
      }

      return {
        root: '/project',
        publicPath: '/',
        compiler: serverCompiler
      };
    })
  };
}

describe('farm ssr dev server api shell', () => {
  it('renders html with ssr template resource', async () => {
    const { farmServer } = createFarmServerMock();
    const hostServer = new FakeHostServer();

    const devServer = await createSsrDevServerWithFactories(
      {
        client: {},
        ssr: {
          entry: '/src/entry-server.ts',
          template: {
            resource: 'index.html',
            placeholder: '<!--app-html-->'
          }
        }
      },
      createFactories(farmServer, hostServer)
    );

    const req = {
      method: 'GET',
      url: '/hello',
      headers: { accept: 'text/html' }
    };
    const headers: Record<string, string> = {};
    const body = await new Promise<string>((resolve) => {
      const res = {
        statusCode: 0,
        setHeader(name: string, value: string) {
          headers[name] = value;
        },
        end(html: string) {
          resolve(html);
        }
      };

      devServer.middlewares(req as never, res as never, () => resolve('next'));
    });

    expect(body).toContain('<div>render:/hello</div>');
    expect(headers['Content-Type']).toContain('text/html');
    await devServer.close();
  });

  it('falls back to root container replacement when default placeholder is missing', async () => {
    const { farmServer } = createFarmServerMock(
      false,
      '<html><body><div id="root"></div></body></html>'
    );
    const hostServer = new FakeHostServer();

    const devServer = await createSsrDevServerWithFactories(
      {
        client: {},
        ssr: {
          entry: '/src/entry-server.ts',
          template: {
            resource: 'index.html'
          }
        }
      },
      createFactories(farmServer, hostServer)
    );

    const body = await new Promise<string>((resolve) => {
      const res = {
        statusCode: 0,
        setHeader() {},
        end(html: string) {
          resolve(html);
        }
      };

      devServer.middlewares(
        {
          method: 'GET',
          url: '/root-fallback',
          headers: { accept: 'text/html' }
        } as never,
        res as never,
        () => resolve('next')
      );
    });

    expect(body).toContain(
      '<div id="root"><div>render:/root-fallback</div></div>'
    );
    await devServer.close();
  });

  it('supports custom template load/transform for legacy html engines', async () => {
    const { farmServer } = createFarmServerMock();
    const hostServer = new FakeHostServer();

    const devServer = await createSsrDevServerWithFactories(
      {
        client: {},
        ssr: {
          entry: '/src/entry-server.ts',
          template: {
            async load({ url }) {
              return `<html><body><%- body %>|<%- url %>|${url}</body></html>`;
            },
            transform({ template, appHtml, url }) {
              return template
                .replace('<%- body %>', appHtml)
                .replace('<%- url %>', url);
            }
          }
        }
      },
      createFactories(farmServer, hostServer)
    );

    const req = {
      method: 'GET',
      url: '/legacy',
      headers: { accept: 'text/html' }
    };
    const body = await new Promise<string>((resolve) => {
      const res = {
        statusCode: 0,
        setHeader() {},
        end(html: string) {
          resolve(html);
        }
      };

      devServer.middlewares(req as never, res as never, () => resolve('next'));
    });

    expect(body).toContain('<div>render:/legacy</div>');
    expect(body).toContain('/legacy');
    await devServer.close();
  });

  it('composes middleware order and returns contract', async () => {
    const { farmServer, runner, trace } = createFarmServerMock();
    const hostServer = new FakeHostServer();

    const devServer = await createSsrDevServerWithFactories(
      {
        client: {},
        ssrMiddleware: (_req, _res, next) => {
          trace.push('ssr');
          next();
        }
      },
      createFactories(farmServer, hostServer)
    );

    const req = { method: 'GET', url: '/', headers: {} };
    const res = {
      setHeader: vi.fn(),
      getHeader: vi.fn(),
      removeHeader: vi.fn(),
      end: vi.fn()
    };

    await new Promise<void>((resolve, reject) => {
      devServer.middlewares(req as never, res as never, (error?: unknown) => {
        if (error) {
          reject(error);
          return;
        }
        resolve();
      });
    });

    expect(trace).toEqual(['ssr', 'farm']);
    expect(devServer.runner).toBe(runner);
    expect(typeof devServer.listen).toBe('function');
    expect(typeof devServer.close).toBe('function');
  });

  it('handles listen idempotency and close idempotency', async () => {
    const { farmServer, runner } = createFarmServerMock();
    const hostServer = new FakeHostServer();

    const devServer = await createSsrDevServerWithFactories(
      {
        client: {},
        host: {
          port: 3201,
          hostname: '127.0.0.1'
        }
      },
      createFactories(farmServer, hostServer)
    );

    await devServer.listen();
    await devServer.listen();
    await devServer.close();
    await devServer.close();

    expect(hostServer.listenCalls).toEqual([
      { port: 3201, hostname: '127.0.0.1' }
    ]);
    expect(hostServer.closeCalls).toBe(1);
    expect((runner.close as any).mock.calls.length).toBe(1);
    expect((farmServer.close as any).mock.calls.length).toBe(1);
  });

  it('retries farm dev server creation with next hmr port when current port is in use', async () => {
    const { farmServer } = createFarmServerMock();
    const hostServer = new FakeHostServer();
    let call = 0;
    const createFarmServer = vi.fn(async (config: any) => {
      call++;
      if (call === 1) {
        throw new Error(
          'WebSocket server error: Port is already in use. Please set a different port using the `server.hmr.port` option.'
        );
      }
      return farmServer;
    });

    const factories: SsrDevServerFactories = {
      createFarmServer,
      createHostServer: vi.fn(() => hostServer),
      createServerCompiler: vi.fn(async () => {
        throw new Error('server compiler should not be created in this case');
      })
    };

    const devServer = await createSsrDevServerWithFactories(
      {
        client: {
          server: {
            hmr: {
              port: 9811
            }
          }
        }
      },
      factories
    );

    expect(createFarmServer).toHaveBeenCalledTimes(2);
    expect(createFarmServer.mock.calls[0]?.[0]?.server?.hmr?.port).toBe(9811);
    expect(createFarmServer.mock.calls[1]?.[0]?.server?.hmr?.port).toBe(9812);

    await devServer.close();
  });

  it('cleans up resources when start fails during listen', async () => {
    const { farmServer, runner } = createFarmServerMock();
    const hostServer = new FailingHostServer();

    await expect(
      startSsrDevServerWithFactories(
        {
          client: {}
        },
        createFactories(farmServer, hostServer)
      )
    ).rejects.toThrow('listen failed');

    expect((runner.close as any).mock.calls.length).toBe(1);
    expect((farmServer.close as any).mock.calls.length).toBe(1);
    expect(hostServer.closeCalls).toBe(0);
  });

  it('wires server compiler updates through a single watcher into runner hmr', async () => {
    const { farmServer, watcher } = createFarmServerMock(true);
    const hostServer = new FakeHostServer();
    const serverCompiler = new FakeServerCompiler();

    const devServer = await createSsrDevServerWithFactories(
      {
        client: {},
        server: {},
        runner: {
          hmr: true
        }
      },
      createFactories(farmServer, hostServer, serverCompiler)
    );

    const first = await devServer.runner.import<{ value: number }>(
      '/src/entry-server.ts'
    );
    expect(first.value).toBe(1);

    await watcher?.emit('change', '/src/entry-server.ts');

    const second = await devServer.runner.import<{ value: number }>(
      '/src/entry-server.ts'
    );
    expect(second.value).toBe(2);
    expect(serverCompiler.updateCalls).toBe(1);

    await devServer.close();
  });

  it('does not refresh runner cache when update path is outside server module graph', async () => {
    const { farmServer, watcher } = createFarmServerMock(true);
    const hostServer = new FakeHostServer();
    const serverCompiler = new FakeServerCompilerWithoutHasModule();

    const devServer = await createSsrDevServerWithFactories(
      {
        client: {},
        server: {},
        runner: {
          hmr: true
        }
      },
      createFactories(farmServer, hostServer, serverCompiler)
    );

    const first = await devServer.runner.import<{ value: number }>(
      '/src/entry-server.ts'
    );
    expect(first.value).toBe(1);

    await watcher?.emit('change', '/src/entry-server.ts');

    const second = await devServer.runner.import<{ value: number }>(
      '/src/entry-server.ts'
    );
    expect(second.value).toBe(1);
    expect(serverCompiler.updateCalls).toBe(1);

    await devServer.close();
  });
});
