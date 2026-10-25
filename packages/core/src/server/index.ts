export type { OwnedRuntimeServer } from './owner.js';
export { RuntimeServerOwner } from './owner.js';
export * from './preview.js';
export type { RuntimeCompiler } from './runtime.js';
export { ServerRuntime } from './runtime.js';

import fs, { PathLike } from 'node:fs';
import type { Server as HttpBaseServer, IncomingMessage } from 'node:http';
import type { Http2SecureServer } from 'node:http2';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import type * as net from 'node:net';
import connect from 'connect';
import corsMiddleware from 'cors';
import { Compiler, createCompiler } from '../compiler/index.js';
import { __FARM_GLOBAL__ } from '../config/_global.js';
import type {
  FarmCliOptions,
  NormalizedServerConfig,
  ResolvedUserConfig,
  UserConfig
} from '../config/types.js';
import { colors, handleLazyCompilation, resolveConfig } from '../index.js';
import { getSortedPluginHooksBindThis } from '../plugin/index.js';
import type {
  CompilerUpdateItem,
  JsUpdateResult,
  PersistentCacheConfig
} from '../types/binding.js';
import { isCacheDirExists } from '../utils/cacheDir.js';
import { createDebugger } from '../utils/debug.js';
import { convertErrorMessage } from '../utils/error.js';
import {
  resolveServerUrls,
  setupSIGTERMListener,
  teardownSIGTERMListener
} from '../utils/http.js';
import { bootstrap, Logger, printServerUrls } from '../utils/logger.js';
import { initPublicFiles } from '../utils/publicDir.js';
import {
  arrayEqual,
  getValidPublicPath,
  isObject,
  normalizePath
} from '../utils/share.js';
import Watcher from '../watcher/index.js';
import { HmrEngine } from './hmr-engine.js';
import type { CommonServerOptions, ResolvedServerUrls } from './http.js';
import { httpServer } from './http.js';
import {
  hmrPingMiddleware,
  htmlFallbackMiddleware,
  lazyCompilationMiddleware,
  notFoundMiddleware,
  outputFilesMiddleware,
  proxyMiddleware,
  publicMiddleware,
  publicPathMiddleware,
  resourceDiskMiddleware,
  resourceMiddleware,
  staticMiddleware
} from './middlewares/index.js';
import type { ManagedProxyMiddleware } from './middlewares/proxy.js';
import { openBrowser } from './open.js';
import { ServerRuntime } from './runtime.js';
import { WsServer } from './ws.js';

export type HttpServer = HttpBaseServer | Http2SecureServer;

type CompilerType = Compiler | undefined;

export type ServerConfig = CommonServerOptions & NormalizedServerConfig;

function transportSignature(server: ResolvedUserConfig['server']): string {
  const transport = { ...server };
  // Resource middleware reads headers from the committed configuration per request.
  delete transport.headers;
  // HTTP proxy routes can swap behind the same request/upgrade dispatchers.
  // HTTPS chooses its HTTP1/HTTP2 server kind based on proxy configuration.
  if (!transport.https) delete transport.proxy;
  return JSON.stringify(transport);
}

export interface ServerCreateOptions {
  /** Use one compiler, synchronous updates and successful resource snapshots. */
  runtime?: boolean;
  /** Allocate independent HMR transport now and bind it during listen(). */
  deferWebSocketListen?: boolean;
  /** Reload trusted configuration outside compilation hooks. */
  reloadConfig?: () => Promise<ResolvedUserConfig>;
  /** Delegate configuration lifecycle to an owner outside this server's update queue. */
  onConfigChange?: (server: Server) => Promise<void>;
  /** An external owner may handle process signals for the complete lifetime. */
  handleSignals?: boolean;
}

export const debugServer = createDebugger('farm:server');

export function shouldAutoSyncHmrPort(
  server: Pick<ServerConfig, 'port' | 'hmr'>
) {
  return (
    isObject(server.hmr) &&
    !server.hmr.server &&
    server.hmr.port === server.port
  );
}

export function syncServerPort(
  config: ResolvedUserConfig,
  serverPort: number,
  shouldSyncHmrPort = config.server
    ? shouldAutoSyncHmrPort(config.server)
    : false
) {
  if (shouldSyncHmrPort && config.server) {
    config.server.hmr.port = serverPort;
    if (config.compilation?.define) {
      config.compilation.define.FARM_HMR_PORT = serverPort.toString();
    }
  }

  if (config.server) {
    config.server.port = serverPort;
  }
}

/**
 * Represents a Farm development server.
 * @class
 */
export class Server extends httpServer {
  ws!: WsServer;
  serverOptions!: ServerConfig;
  httpsOptions: HttpsServerOptions | undefined;
  shouldSyncHmrPort = false;
  publicDir: string | undefined;
  publicPath?: string;
  publicFiles?: Set<string>;
  httpServer!: HttpServer | null;
  watcher!: Watcher;
  hmrEngine?: HmrEngine;
  middlewares!: connect.Server;
  compiler?: CompilerType;
  root!: string;
  config!: ResolvedUserConfig;
  closeHttpServerFn!: () => Promise<void>;
  terminateServerFn!: (_: unknown, exitCode?: number) => Promise<void>;
  postConfigureServerHooks: ((() => void) | void)[] = [];
  logger: Logger;
  readonly runtimeMode: boolean;
  runtime?: ServerRuntime;
  private runtimeReady: Promise<void>;
  private resolveRuntimeReady!: () => void;
  private startupFailure?: unknown;
  private runtimeClosing = false;
  private runtimeUpdates: Promise<void> = Promise.resolve();
  private closing?: Promise<void>;
  private runtimePreparation?: Promise<void>;
  private runtimeListenerSuspended = false;
  private runtimeTransportConfig?: string;
  private runtimeProxy?: ManagedProxyMiddleware;
  private readonly retiredRuntimeProxies = new Set<ManagedProxyMiddleware>();
  private runtimeProxyUpgrade?: (
    request: IncomingMessage,
    socket: net.Socket,
    head: Buffer
  ) => void;

  /**
   * Creates an instance of Server.
   * @param {FarmCliOptions & UserConfig} inlineConfig - The inline configuration options.
   */
  constructor(
    readonly inlineConfig: FarmCliOptions & UserConfig = {},
    readonly createOptions: ServerCreateOptions = {}
  ) {
    super();
    this.logger = new Logger();
    this.runtimeMode = createOptions.runtime === true;
    this.runtimeReady = new Promise((resolve) => {
      this.resolveRuntimeReady = resolve;
    });
  }

  /**
   * Creates a lazy compilation server that enable lazy compilation when targeting node
   */
  static async createAndStartLazyCompilationServer(
    config: ResolvedUserConfig
  ): Promise<Server> {
    const server = new Server();
    server.config = config;

    server.#resolveOptions();

    const httpsOptions = await server.resolveHttpsConfig(
      server.serverOptions.https
    );
    server.httpsOptions = httpsOptions;

    server.middlewares = connect() as connect.Server;
    server.httpServer = await server.resolveHttpServer(
      server.serverOptions as CommonServerOptions,
      server.middlewares,
      server.httpsOptions
    );

    server.middlewares.use(lazyCompilationMiddleware(server));

    const { port, hostname, strictPort } = server.serverOptions;
    const serverPort = await server.httpServerStart({
      port,
      strictPort: strictPort,
      host: hostname.host
    });
    server.#updateServerPort(serverPort, 'WATCH');

    return server;
  }

  /**
   * Creates and initializes the server.
   *
   * This method performs the following operations:
   * Resolves HTTPS configuration
   * Handles public files
   * Creates middleware
   * Creates HTTP server (if not in middleware mode)
   * Initializes HMR engine
   * Creates WebSocket server
   * Sets up Vite invalidation handler
   * Initializes middlewares
   *
   * @returns {Promise<void>} A promise that resolves when the server creation process is complete
   * @throws {Error} If an error occurs during the server creation process
   */
  static async createServer(
    inlineConfig: FarmCliOptions & UserConfig = {},
    resolvedConfig?: ResolvedUserConfig,
    options: ServerCreateOptions = {}
  ): Promise<Server> {
    const server = new Server(inlineConfig, options);
    try {
      server.config =
        resolvedConfig ??
        (await resolveConfig(server.inlineConfig, 'dev', 'development'));
      if (server.runtimeMode && server.config.compilation?.lazyCompilation) {
        throw new Error(
          '[FARM_SERVER_RUNTIME] runtime mode requires lazyCompilation:false'
        );
      }

      server.logger = server.config.logger ?? server.logger;
      server.runtimeTransportConfig = transportSignature(server.config.server);

      server.#resolveOptions();

      const [httpsOptions, publicFiles] = await Promise.all([
        server.resolveHttpsConfig(server.serverOptions.https),
        server.#handlePublicFiles()
      ]);
      server.httpsOptions = httpsOptions;
      server.publicFiles = publicFiles;
      server.middlewares = connect() as connect.Server;
      server.httpServer = server.serverOptions.middlewareMode
        ? null
        : await server.resolveHttpServer(
            server.serverOptions as CommonServerOptions,
            server.middlewares,
            server.httpsOptions
          );

      // close server function prepare promise
      server.closeHttpServerFn = server.closeServer();

      // init hmr engine When actually updating, we need to get the clients of ws for broadcast, 、
      // so we can instantiate hmrEngine by default at the beginning.
      server.createHmrEngine();

      // init websocket server
      await server.createWebSocketServer();

      // invalidate vite handler
      server.#invalidateVite();

      // init watcher
      await server.#createWatcher();

      await server.handleConfigureServer();

      // init middlewares
      server.#initializeMiddlewares();

      server.terminateServerFn = async (_: unknown, exitCode?: number) => {
        try {
          await server.close();
        } finally {
          process.exitCode ??= exitCode ? 128 + exitCode : undefined;
          process.exit();
        }
      };

      if (
        !server.serverOptions.middlewareMode &&
        options.handleSignals !== false
      ) {
        setupSIGTERMListener(server.terminateServerFn);
      }

      if (!server.serverOptions.middlewareMode && server.httpServer) {
        const httpServer = server.httpServer;
        httpServer.once('listening', () => {
          // update actual port since this may be different from initial value
          server.serverOptions.port = (
            httpServer.address() as net.AddressInfo
          ).port;
        });
      }

      return server;
    } catch (error) {
      if (server.runtimeMode) await server.close();
      throw error;
    }
  }

  /**
   * create watcher
   */
  async #createWatcher() {
    this.watcher = new Watcher(this.config);

    await this.watcher.createWatcher(this.runtimeMode ? null : undefined);

    if (this.runtimeMode) {
      const events = {
        add: 'added',
        addDir: 'added',
        change: 'updated',
        unlink: 'removed',
        unlinkDir: 'removed'
      } as const;
      for (const [event, type] of Object.entries(events)) {
        this.watcher.on(event as keyof typeof events, (file) => {
          if (this.runtimeClosing) return;
          if (
            (this.createOptions.reloadConfig ||
              this.createOptions.onConfigChange) &&
            this.watcher.isConfigFilesChanged(file)
          ) {
            void (
              this.createOptions.onConfigChange
                ? this.createOptions.onConfigChange(this)
                : this.reloadRuntimeConfig()
            ).catch((error) => this.reportRuntimeError(error));
            return;
          }
          void this.updateRuntime([{ path: normalizePath(file), type }]).catch(
            (error) => this.reportRuntimeError(error)
          );
        });
      }
      this.watchRuntimeConfig();
      return;
    }

    this.watcher.on('add', async (file: string) => {
      if (!this.compiler) return;
      const currentAddedItem: CompilerUpdateItem = {
        path: normalizePath(file),
        type: 'added'
      };

      await this.compiler.update([currentAddedItem]);
    });

    this.watcher.on('unlink', async (file: string) => {
      if (!this.compiler) return;
      const currentRemovedItem: CompilerUpdateItem = {
        path: normalizePath(file),
        type: 'removed'
      };
      await this.compiler.update([currentRemovedItem]);

      // Fix #2035, skip if the file is irrelevant
      if (this.compiler.hasModule(file)) {
        const parentFiles = this.compiler.getParentFiles(file);
        const normalizedParentFiles: CompilerUpdateItem[] = parentFiles.map(
          (file) => ({
            path: normalizePath(file),
            type: 'updated'
          })
        );
        this.hmrEngine?.hmrUpdate(normalizedParentFiles, true);
      }
    });

    this.watcher.on('change', async (file: string) => {
      file = normalizePath(file);

      if (this.watcher.isConfigFilesChanged(file)) {
        try {
          await this.restartServer();
        } catch (e: any) {
          this.logger.error(`restart server error ${e}`);
        }

        return;
      }

      try {
        const currentRemovedItem: CompilerUpdateItem = {
          path: normalizePath(file),
          type: 'updated'
        };
        this.hmrEngine?.hmrUpdate(currentRemovedItem);
      } catch (error: any) {
        this.logger.error(`Farm Hmr Update Error: ${error}`);
      }
    });

    const handleUpdateFinish = (updateResult: JsUpdateResult) => {
      if (!this.compiler) return;
      const compiler = this.compiler;
      const added = [
        ...updateResult.added,
        ...updateResult.extraWatchResult.add
      ].map((addedModule) => {
        const resolvedPath = compiler.transformModulePath(
          this.root,
          addedModule
        );
        return resolvedPath;
      });
      const filteredAdded = added.filter((file) =>
        this.watcher.filterWatchFile(file, this.root)
      );

      if (filteredAdded.length > 0) {
        this.watcher.add(filteredAdded);
      }
    };

    this.hmrEngine?.onUpdateFinish(handleUpdateFinish);
  }

  #updateServerPort(serverPort: number, command: 'START' | 'WATCH') {
    syncServerPort(this.config, serverPort, this.shouldSyncHmrPort);

    this.serverOptions.port = serverPort;

    if (this.config.compilation?.lazyCompilation) {
      handleLazyCompilation(this.config, command);
    }
  }

  /**
   * Restarts the server.
   * @returns {Promise<void>}
   */
  async restartServer(): Promise<void> {
    if (this.serverOptions.middlewareMode) {
      await this.restart();
      return;
    }
    const { port: prevPort, host: prevHost } = this.serverOptions;
    const prevUrls = this.resolvedUrls;

    const newServer = await this.restart();
    if (!newServer) return;
    const {
      serverOptions: { port, host },
      resolvedUrls
    } = newServer;

    const urlsChanged = prevUrls
      ? this.hasUrlsChanged(
          prevUrls,
          resolvedUrls ?? { local: [], network: [] }
        )
      : true;
    if (port !== prevPort || host !== prevHost || urlsChanged) {
      __FARM_GLOBAL__.__FARM_SHOW_DEV_SERVER_URL__ = true;
    } else {
      __FARM_GLOBAL__.__FARM_SHOW_DEV_SERVER_URL__ = false;
    }

    newServer.printUrls();
  }

  /**
   * Checks if the server URLs have changed.
   * @param {ResolvedServerUrls} oldUrls - The old server URLs.
   * @param {ResolvedServerUrls} newUrls - The new server URLs.
   * @returns {boolean} True if the URLs have changed, false otherwise.
   */
  hasUrlsChanged(oldUrls: ResolvedServerUrls, newUrls: ResolvedServerUrls) {
    return !(
      oldUrls === newUrls ||
      (oldUrls &&
        newUrls &&
        arrayEqual(oldUrls.local, newUrls.local) &&
        arrayEqual(oldUrls.network, newUrls.network))
    );
  }

  /**
   * Restarts the server.
   */
  async restart() {
    let newServer: Server | undefined;
    try {
      await this.close();
      newServer = await Server.createServer(
        this.inlineConfig,
        undefined,
        this.createOptions
      );
    } catch (error: any) {
      this.logger.error(`Failed to restart server :\n ${error}`);
      return;
    }
    await this.watcher.close();
    await newServer.listen();
    return newServer;
  }

  /**
   * Creates and initializes the WebSocket server.
   * @throws {Error} If the HTTP server is not created.
   */
  async createWebSocketServer() {
    if (!this.httpServer) {
      throw new Error(
        'Websocket requires a http server. please check the server is created'
      );
    }

    this.ws = new WsServer(this);
    await this.ws.createWebSocketServer(
      this.runtimeMode && this.createOptions.deferWebSocketListen
    );
  }

  /**
   * Creates and initializes the Hot Module Replacement (HMR) engine.
   * @throws {Error} If the HTTP server is not created.
   */
  createHmrEngine() {
    if (!this.httpServer) {
      throw new Error(
        'HmrEngine requires a http server. please check the server is be created'
      );
    }

    this.hmrEngine = new HmrEngine(this);
  }

  /**
   * Starts the server and begins listening for connections.
   * @returns {Promise<void>}
   * @throws {Error} If there's an error starting the server.
   */
  async listen(): Promise<void> {
    this.assertRuntimeOpen();
    if (!this.httpServer) {
      this.logger.warn('HTTP server is not created yet');
      return;
    }
    const { port, hostname, open, strictPort } = this.serverOptions;

    try {
      if (this.runtimeMode) await this.ws.listen();
      this.assertRuntimeOpen();
      const serverPort = await this.httpServerStart({
        port,
        strictPort: strictPort,
        host: hostname.host
      });
      this.assertRuntimeOpen();
      this.#updateServerPort(serverPort, 'START');

      this.resolvedUrls = await resolveServerUrls(this.httpServer, this.config);
      this.assertRuntimeOpen();

      // compile the project and start the dev server
      if (this.runtimeMode) await this.prepareRuntime();
      else await this.#startCompile();
      this.assertRuntimeOpen();

      // watch extra files after compile
      if (this.runtimeMode) this.watcher.getExtraWatchedFiles(this.compiler);
      this.watcher?.watchExtraFiles?.();

      if (open) {
        this.#openServerBrowser();
      }
    } catch (error: any) {
      if (this.runtimeMode) {
        await this.close();
        // A pending startup may bind after the memoized close has completed.
        if (this.httpServer?.listening) await this.closeHttpServerFn?.();
        throw error;
      }
      this.logger.error(`Start DevServer Error: ${error} \n ${error.stack}`);
      // throw error;
    }
  }

  private assertRuntimeOpen(): void {
    if (this.runtimeMode && this.runtimeClosing) {
      throw new Error('Server runtime is closed');
    }
  }

  /** Compile a fixed-endpoint candidate without binding its HTTP listener. */
  prepareRuntime(): Promise<void> {
    this.assertRuntimeOpen();
    if (!this.runtimeMode)
      return Promise.reject(
        new Error('[FARM_SERVER_RUNTIME] runtime mode required')
      );
    if (!this.serverOptions.port)
      return Promise.reject(
        new Error(
          '[FARM_SERVER_RUNTIME] preparation requires a resolved HTTP port'
        )
      );
    this.runtimePreparation ??= this.#startCompile();
    return this.runtimePreparation;
  }

  /** Stop accepting new HTTP connections while retaining native resources and WS peers. */
  async suspendRuntimeListener(): Promise<void> {
    this.assertRuntimeOpen();
    if (!this.runtimeMode)
      throw new Error('[FARM_SERVER_RUNTIME] runtime mode required');
    await this.waitForRuntimeReady();
    await this.runtimeUpdates;
    this.assertRuntimeOpen();
    if (this.runtimeListenerSuspended) return;
    if (!this.httpServer?.listening)
      throw new Error('[FARM_SERVER_RUNTIME] HTTP listener is not active');
    this.httpServer.close((error) => {
      if (error) this.logger.error(error);
    });
    this.runtimeListenerSuspended = true;
  }

  /** Restore the original endpoint and successful generation after a failed handoff. */
  async resumeRuntimeListener(): Promise<void> {
    this.assertRuntimeOpen();
    if (!this.runtimeMode)
      throw new Error('[FARM_SERVER_RUNTIME] runtime mode required');
    if (!this.runtimeListenerSuspended) return;
    try {
      const port = await this.httpServerStart({
        port: this.serverOptions.port,
        strictPort: true,
        host: this.serverOptions.hostname.host
      });
      this.assertRuntimeOpen();
      this.#updateServerPort(port, 'START');
      this.resolvedUrls = await resolveServerUrls(
        requireDefined(this.httpServer),
        this.config
      );
      this.assertRuntimeOpen();
      this.runtimeListenerSuspended = false;
    } catch (error) {
      if (this.runtimeClosing && this.httpServer?.listening)
        await this.closeHttpServerFn?.();
      throw error;
    }
  }

  /**
   * Get current compiler instance in the server
   * @returns { CompilerType } return current compiler, may be compiler or undefined
   */
  getCompiler(): CompilerType {
    return this.compiler;
  }

  /**
   * Set current compiler instance in the server
   * @param { Compiler } compiler - choose a new compiler instance
   */
  setCompiler(compiler: Compiler) {
    this.compiler = compiler;
  }

  async waitForRuntimeReady(): Promise<void> {
    await this.runtimeReady;
    if (this.startupFailure) throw this.startupFailure;
    if (this.runtimeClosing || !this.runtime)
      throw new Error(
        '[FARM_SERVER_RUNTIME] runtime is closed or not initialized'
      );
  }

  /** All runtime filesystem events, including unknown files and directories. */
  updateRuntime(paths: CompilerUpdateItem[]): Promise<JsUpdateResult> {
    const batch = paths.map((item) => ({ ...item }));
    return this.queueRuntimeUpdate(() => batch);
  }

  private watchRuntimeConfig(): void {
    this.watcher.add([
      ...(this.config.configFilePath ? [this.config.configFilePath] : []),
      ...(this.config.configFileDependencies ?? []),
      ...(this.config.envFiles ?? [])
    ]);
  }

  /** Serialize configuration preparation with ordinary native updates and close. */
  canReloadRuntimeConfig(candidate: ResolvedUserConfig): boolean {
    return (
      candidate.root === this.config.root &&
      transportSignature(candidate.server) === this.runtimeTransportConfig
    );
  }

  reloadRuntimeConfig(prepared?: ResolvedUserConfig): Promise<void> {
    const operation = this.runtimeUpdates.then(async () => {
      await this.waitForRuntimeReady();
      if (!prepared && !this.createOptions.reloadConfig)
        throw new Error('[FARM_SERVER_RUNTIME] no configuration loader');
      const candidate =
        prepared ??
        (await requireDefined(this.createOptions.reloadConfig).call(
          this.createOptions
        ));
      this.assertRuntimeOpen();
      if (!this.canReloadRuntimeConfig(candidate))
        throw new Error(
          '[FARM_SERVER_RUNTIME_TRANSPORT] transport or root changes require a server restart'
        );
      const candidateServer: ServerConfig = {
        ...this.serverOptions,
        headers: candidate.server?.headers ?? {},
        proxy: candidate.server?.proxy ?? {}
      };
      candidate.server = candidateServer;
      requireDefined(candidate.compilation).define = {
        ...requireDefined(candidate.compilation).define,
        ...Object.fromEntries(
          Object.entries(this.config.compilation?.define ?? {}).filter(
            ([key]) => key.startsWith('FARM_HMR_')
          )
        )
      };
      const candidateProxy = proxyMiddleware(
        { config: candidate },
        requireDefined(this.httpServer),
        candidateServer.proxy ?? {},
        false
      );
      let published = false;
      try {
        const compiler = createCompiler(candidate);
        await requireDefined(this.runtime).replaceCompiler(compiler);
        this.assertRuntimeOpen();
        const previousProxy = this.runtimeProxy;
        this.runtimeProxy = candidateProxy;
        this.setCompiler(compiler);
        this.config = candidate;
        this.serverOptions = candidateServer;
        published = true;
        const retirementErrors: unknown[] = [];
        try {
          if (previousProxy) {
            this.retiredRuntimeProxies.add(previousProxy);
            previousProxy.retire(() =>
              this.retiredRuntimeProxies.delete(previousProxy)
            );
          }
        } catch (error) {
          retirementErrors.push(error);
        }
        this.watcher.config = candidate;
        this.watchRuntimeConfig();
        this.watcher.getExtraWatchedFiles(compiler);
        this.watcher.watchExtraFiles();
        this.ws.send({ type: 'full-reload' });
        if (retirementErrors.length)
          throw new AggregateError(
            retirementErrors,
            '[FARM_SERVER_RUNTIME] configuration committed; retired proxy cleanup failed'
          );
      } catch (error) {
        if (!published) {
          try {
            candidateProxy.close();
          } catch (cleanupError) {
            throw new AggregateError(
              [error, cleanupError],
              '[FARM_SERVER_RUNTIME] candidate failed and proxy cleanup failed'
            );
          }
        }
        throw error;
      }
    });
    this.runtimeUpdates = operation.then(
      () => undefined,
      () => undefined
    );
    return operation;
  }

  private queueRuntimeUpdate(
    paths: () => CompilerUpdateItem[]
  ): Promise<JsUpdateResult> {
    const operation = this.runtimeUpdates.then(async () => {
      await this.waitForRuntimeReady();
      const result = await requireDefined(this.runtime).update(paths());
      if (this.runtimeClosing)
        throw new Error('[FARM_SERVER_RUNTIME] runtime is closed');
      this.watcher.getExtraWatchedFiles(this.compiler);
      this.watcher.watchExtraFiles();
      this.hmrEngine?.sendUpdate(result);
      return result;
    });
    this.runtimeUpdates = operation.then(
      () => undefined,
      () => undefined
    );
    return operation;
  }

  private reportRuntimeError(error: unknown): void {
    if (this.runtimeClosing) return;
    const value = error instanceof Error ? error : new Error(String(error));
    this.ws.send({
      type: 'error',
      err: { message: convertErrorMessage(value), stack: value.stack ?? '' }
    });
    this.logger.error(`Update Error: ${convertErrorMessage(value)}`);
  }

  /**
   * Adds additional files to be watched by the compiler.
   * @param {string} root - The root directory.
   * @param {string[]} deps - Array of file paths to be watched.
   */
  addWatchFile(root: string, deps: string[]): void {
    this.getCompiler()?.addExtraWatchFile(root, deps);
  }

  /**
   * Handles the configureServer hook.
   */
  async handleConfigureServer() {
    const reflexServer = new Proxy(this, {
      get: (_, property: keyof Server) =>
        this[property as keyof this] ?? undefined,
      set: (_, property: keyof Server, value: unknown) => {
        this[property as keyof this] = value as this[keyof this];
        return true;
      }
    });
    const { jsPlugins } = this.config;

    for (const hook of getSortedPluginHooksBindThis(
      jsPlugins ?? [],
      'configureServer'
    )) {
      this.postConfigureServerHooks.push(await hook(reflexServer));
    }
  }

  /**
   * resolve and setting server options
   *
   * this method extracts compilation and server options from resolvedUserConfig
   * and set the publicPath and publicDir， and then resolve server options
   * @private
   * @returns { void }
   */
  #resolveOptions(): void {
    const { compilation, root, server } = this.config;
    this.publicPath = getValidPublicPath(compilation?.output?.publicPath);
    this.publicDir = (
      compilation?.assets as { publicDir?: string } | undefined
    )?.publicDir;
    if (server?.origin?.endsWith('/')) {
      server.origin = server.origin.slice(0, -1);
      this.logger.warn(
        `${colors.bold('(!)')} server.origin should not end with "/". Using "${
          server.origin
        }" instead.`
      );
    }
    this.serverOptions = server as CommonServerOptions & NormalizedServerConfig;
    this.shouldSyncHmrPort = shouldAutoSyncHmrPort(this.serverOptions);
    this.root = root ?? '';
  }

  /**
   * Initializes and configures the middleware stack for the server.
   * @private
   */
  #initializeMiddlewares() {
    this.middlewares.use(hmrPingMiddleware());

    const { proxy, middlewareMode, cors, appType } = this.serverOptions;

    if (cors) {
      this.middlewares.use(
        corsMiddleware(typeof cors === 'boolean' ? {} : cors)
      );
    }

    if (this.runtimeMode) {
      this.runtimeProxy = proxyMiddleware(
        this,
        requireDefined(this.httpServer),
        proxy ?? {},
        false
      );
      this.runtimeProxyUpgrade = (request, socket, head) =>
        this.runtimeProxy?.upgrade(request, socket, head);
      this.httpServer?.on('upgrade', this.runtimeProxyUpgrade);
      this.middlewares.use((request, response, next) => {
        if (this.runtimeProxy) this.runtimeProxy(request, response, next);
        else next();
      });
    } else if (proxy) {
      const middlewareServer =
        (isObject(middlewareMode) && 'server' in middlewareMode
          ? middlewareMode.server
          : null) || this.httpServer;

      this.middlewares.use(
        proxyMiddleware(this, middlewareServer as HttpServer, proxy)
      );
    }

    if (this.publicPath !== '/') {
      this.middlewares.use(
        publicPathMiddleware(this, this.serverOptions.middlewareMode)
      );
    }

    if (!this.runtimeMode && fs.existsSync(this.publicDir as PathLike)) {
      this.middlewares.use(publicMiddleware(this));
    }

    if (this.config.compilation?.lazyCompilation) {
      this.middlewares.use(lazyCompilationMiddleware(this));
    }

    if (!this.runtimeMode) this.middlewares.use(staticMiddleware(this));

    // Check dev resource tree in `_output_files` url
    this.middlewares.use(outputFilesMiddleware(this));

    if (!this.runtimeMode && this.config.server?.writeToDisk) {
      this.middlewares.use(resourceDiskMiddleware(this));
    } else {
      this.middlewares.use(resourceMiddleware(this));
    }

    this.postConfigureServerHooks.forEach((fn) => {
      if (typeof fn === 'function') fn();
    });

    this.serverOptions.middlewares?.forEach((middleware) => {
      const mw = middleware(this);
      if (mw) this.middlewares.use(mw);
    });

    if (appType === 'spa' || appType === 'mpa') {
      this.middlewares.use(htmlFallbackMiddleware(this));
      this.middlewares.use(notFoundMiddleware());
    }
  }

  /**
   * Compiles the project.
   * @private
   * @returns {Promise<void>}
   * @throws {Error} If compilation fails.
   */
  async #compile(): Promise<void> {
    if (!this.compiler) return;
    try {
      if (this.runtimeMode) await requireDefined(this.runtime).compile();
      else await this.compiler.compile();

      await (this.config.server?.writeToDisk
        ? this.compiler.writeResourcesToDisk()
        : this.compiler.callWriteResourcesHook());
    } catch (err: any) {
      if (this.runtimeMode) throw err;
      this.logger.error(`Compilation failed: ${convertErrorMessage(err)}`, {
        exit: true
      });
    }
  }

  /**
   * Opens the server URL in the default browser.
   * @private
   */
  async #openServerBrowser() {
    const url =
      this.resolvedUrls?.local?.[0] ?? this.resolvedUrls?.network?.[0] ?? '';
    openBrowser(url);
  }

  /**
   * Starts the compilation process.
   * @private
   */
  async #startCompile() {
    try {
      this.assertRuntimeOpen();
      this.setCompiler(createCompiler(this.config));
      if (!this.compiler) return;
      if (this.runtimeMode) this.runtime = new ServerRuntime(this.compiler);

      for (const hook of getSortedPluginHooksBindThis(
        this.config.jsPlugins ?? [],
        'configureCompiler'
      )) {
        await hook?.(this.compiler);
        this.assertRuntimeOpen();
      }

      // check if cache dir exists
      const { persistentCache } = this.compiler.config.compilation ?? {};
      const cacheDir = (persistentCache as PersistentCacheConfig)?.cacheDir;
      const hasCacheDir = await isCacheDirExists(cacheDir ?? '');
      this.assertRuntimeOpen();
      const start = performance.now();
      await this.#compile();

      const duration = performance.now() - start;
      bootstrap(duration, this.compiler.config, hasCacheDir);
    } catch (error) {
      this.startupFailure = error;
      throw error;
    } finally {
      this.resolveRuntimeReady();
    }
  }

  /**
   * Handles the initialization of public files.
   * @private
   * @returns {Promise<Set<string>>} A promise that resolves to a set of public file paths.
   */
  async #handlePublicFiles(): Promise<Set<string> | undefined> {
    const initPublicFilesPromise = initPublicFiles(this.config);
    return await initPublicFilesPromise;
  }

  /**
   * Sets up the Vite invalidation handler.
   * @private
   */
  #invalidateVite(): void {
    // Note: path should be Farm's id, which is a relative path in dev mode,
    // but in vite, it's a url path like /xxx/xxx.js

    this.ws.wss.on('vite:invalidate', ({ path, message }: any) => {
      // find hmr boundary starting from the parent of the file
      this.logger.info(`HMR invalidate: ${path}. ${message ?? ''} `);
      if (!this.compiler || !this.hmrEngine) return;
      if (this.runtimeMode) {
        void this.queueRuntimeUpdate(() =>
          requireDefined(this.compiler)
            .getParentFiles(path)
            .map((file) => ({
              path: normalizePath(file),
              type: 'updated' as const
            }))
        ).catch((error) => this.reportRuntimeError(error));
        return;
      }
      const parentFiles = this.compiler.getParentFiles(path);
      const normalizeParentFiles = parentFiles.map((file) =>
        normalizePath(file)
      );
      this.hmrEngine.hmrUpdate(
        normalizeParentFiles.map((file) => ({
          path: file,
          type: 'updated'
        })),
        true
      );
    });
  }

  /**
   * Closes the server and sockets.
   * @returns {() => Promise<void>}
   */
  closeServer(): () => Promise<void> {
    if (!this.httpServer) {
      return () => Promise.resolve();
    }
    debugServer?.(`prepare close dev server`);

    let hasListened = false;
    const openSockets = new Set<net.Socket>();
    const httpServer = this.httpServer;

    httpServer.on('connection', (socket) => {
      openSockets.add(socket);
      debugServer?.(`has open server socket ${openSockets}`);

      socket.on('close', () => {
        debugServer?.('close all server socket');
        openSockets.delete(socket);
      });
    });

    httpServer.once('listening', () => {
      hasListened = true;
    });

    return () =>
      new Promise<void>((resolve, reject) => {
        openSockets.forEach((s) => s.destroy());

        if (hasListened) {
          httpServer.close((err) => {
            // A suspended listener has already completed close. Final cleanup
            // still destroys owned sockets, but closing it again is successful.
            if (
              err &&
              (err as NodeJS.ErrnoException).code !== 'ERR_SERVER_NOT_RUNNING'
            ) {
              reject(err);
            } else {
              resolve();
            }
          });
        } else {
          resolve();
        }
      });
  }

  async close() {
    if (this.closing) return this.closing;
    this.runtimeClosing = true;
    this.resolveRuntimeReady();
    this.closing = this.closeRuntimeAndServer();
    return this.closing;
  }

  private async closeRuntimeAndServer() {
    if (this.terminateServerFn && !this.serverOptions?.middlewareMode) {
      teardownSIGTERMListener(this.terminateServerFn);
    }

    const runtimeErrors: unknown[] = [];
    try {
      await this.runtime?.close();
    } catch (error) {
      runtimeErrors.push(error);
    }
    if (this.runtimePreparation)
      await Promise.allSettled([this.runtimePreparation]);
    await this.runtimeUpdates;
    const cleanupResults = await Promise.allSettled([
      Promise.resolve().then(() => {
        if (this.runtimeProxyUpgrade)
          this.httpServer?.off('upgrade', this.runtimeProxyUpgrade);
        const errors: unknown[] = [];
        for (const proxy of [
          this.runtimeProxy,
          ...this.retiredRuntimeProxies
        ]) {
          try {
            proxy?.close();
          } catch (error) {
            errors.push(error);
          }
        }
        this.retiredRuntimeProxies.clear();
        if (errors.length)
          throw new AggregateError(errors, 'runtime proxy cleanup failed');
      }),
      Promise.resolve().then(() => this.watcher?.close()),
      Promise.resolve().then(() => this.ws?.close()),
      Promise.resolve().then(() => this.closeHttpServerFn?.())
    ]);
    this.resolvedUrls = null;
    for (const result of cleanupResults) {
      if (result.status === 'rejected') runtimeErrors.push(result.reason);
    }
    if (runtimeErrors.length)
      throw new AggregateError(runtimeErrors, 'Server cleanup failed');
  }

  printUrls() {
    if (!__FARM_GLOBAL__.__FARM_SHOW_DEV_SERVER_URL__) {
      return;
    }
    if (this.resolvedUrls) {
      printServerUrls(this.resolvedUrls, this.serverOptions.host, this.logger);
    } else if (this.serverOptions.middlewareMode) {
      throw new Error('cannot print server URLs in middleware mode.');
    } else {
      throw new Error(
        'cannot print server URLs before server.listen is called.'
      );
    }
  }
}

function requireDefined<T>(value: T | null | undefined): T {
  if (value === null || value === undefined) {
    throw new Error('Expected initialized server state');
  }
  return value;
}
