import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';

const fixtureState = vi.hoisted(() => ({
  config: {} as any,
  watchers: [] as any[]
}));
vi.mock('../binding/index.js', () => ({
  Compiler: class {},
  Resolver: class {},
  JsFileWatcher: class {},
  bindingPath: ''
}));
vi.mock('../src/index.js', () => ({
  resolveConfig: async () => fixtureState.config,
  handleLazyCompilation: async () => {},
  colors: { bold: (value: string) => value }
}));
vi.mock('../src/watcher/index.js', () => ({
  default: class {
    handlers = new Map<string, (file: string) => Promise<void>>();
    constructor() {
      fixtureState.watchers.push(this);
    }
    async createWatcher() {}
    on(event: string, callback: (file: string) => Promise<void>) {
      this.handlers.set(event, callback);
      return this;
    }
    filterWatchFile() {
      return false;
    }
    isConfigFilesChanged() {
      return false;
    }
    add() {}
    async close() {}
  }
}));
vi.mock('../src/utils/publicDir.js', () => ({
  initPublicFiles: async () => new Set()
}));
vi.mock('../src/server/middlewares/index.js', () => {
  const middleware =
    () => (_request: unknown, _response: unknown, next: () => void) =>
      next();
  return Object.fromEntries(
    [
      'hmrPingMiddleware',
      'htmlFallbackMiddleware',
      'lazyCompilationMiddleware',
      'notFoundMiddleware',
      'outputFilesMiddleware',
      'proxyMiddleware',
      'publicMiddleware',
      'publicPathMiddleware',
      'resourceDiskMiddleware',
      'resourceMiddleware',
      'staticMiddleware'
    ].map((name) => [name, middleware])
  );
});

import { HmrEngine } from '../src/server/hmr-engine.js';
import { Server } from '../src/server/index.js';

const roots: string[] = [];
afterEach(async () => {
  vi.restoreAllMocks();
  fixtureState.watchers.length = 0;
  await Promise.all(
    roots.splice(0).map((root) => fs.rm(root, { recursive: true, force: true }))
  );
});

async function fixture() {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'farm-hmr-watcher-'));
  roots.push(root);
  const source = path.join(root, 'route.js');
  const importer = path.join(root, 'index.js');
  await fs.writeFile(source, "export default 'route';");
  await fs.writeFile(importer, "export default 'importer';");
  const send = vi.fn();
  fixtureState.config = {
    root,
    server: { middlewareMode: true, hmr: { port: 0 } },
    jsPlugins: [],
    compilation: { output: { path: 'dist' } }
  };
  vi.spyOn(Server.prototype, 'closeServer').mockReturnValue(async () => {});
  vi.spyOn(Server.prototype, 'createHmrEngine').mockImplementation(function (
    this: Server
  ) {
    this.hmrEngine = new HmrEngine(this);
  });
  vi.spyOn(Server.prototype, 'createWebSocketServer').mockImplementation(
    async function (this: Server) {
      this.ws = { wss: { clients: new Set([{ send }]), on: vi.fn() } } as never;
    }
  );
  const server = await Server.createServer();
  let sourcePresent = true;
  const result = {
    added: [],
    changed: ['index.js'],
    removed: ['route.js'],
    immutableModules: '{}',
    mutableModules: '{}',
    boundaries: {},
    extraWatchResult: { add: [], remove: [] }
  };
  const compiler = {
    config: { root },
    compiling: false,
    waitForCompileFinish: vi.fn(async () => {}),
    hasModule: vi.fn((file: string) => file === importer || sourcePresent),
    getParentFiles: vi.fn(() => [importer]),
    transformModulePath: (_root: string, value: string) =>
      path.join(root, value),
    onUpdateFinish: vi.fn(),
    update: vi.fn(async (_paths: unknown) => {
      sourcePresent = false;
      return result;
    }),
    writeResourcesToDisk: vi.fn()
  };
  server.compiler = compiler as never;
  server.logger.info = vi.fn();
  const watcher = fixtureState.watchers[0];
  return { server, source, importer, compiler, watcher, send };
}

it('watcher additions reach HMR and send the compiler result', async () => {
  const { source, compiler, watcher, send } = await fixture();
  compiler.hasModule.mockReturnValue(false);
  await watcher.handlers.get('add')(source);
  expect(compiler.update).toHaveBeenCalledTimes(1);
  expect(send).toHaveBeenCalledTimes(1);
  expect(send.mock.calls[0][0]).toContain("type: 'farm-update'");
});

it('watcher removals capture importers before the compiler removes their source', async () => {
  const { source, importer, compiler, watcher, send } = await fixture();
  await fs.rm(source);
  await watcher.handlers.get('unlink')(source);
  expect(compiler.getParentFiles).toHaveBeenCalledWith(source);
  expect(compiler.update).toHaveBeenCalledTimes(1);
  expect(compiler.update.mock.calls[0][0]).toEqual(
    expect.arrayContaining([
      { path: source.replaceAll('\\', '/'), type: 'removed' },
      { path: importer.replaceAll('\\', '/'), type: 'updated' }
    ])
  );
  expect(send).toHaveBeenCalledTimes(1);
  expect(send.mock.calls[0][0]).toContain("removed: ['route.js']");
});

it('watcher events retain the compiler-only fallback when HMR is disabled', async () => {
  const { server, source, compiler, watcher } = await fixture();
  server.hmrEngine = undefined;
  await watcher.handlers.get('add')(source);
  expect(compiler.update).toHaveBeenCalledTimes(1);
});
