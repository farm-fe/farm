export * from './preview.js';

import fs, { PathLike } from 'node:fs';
import connect from 'connect';
import corsMiddleware from 'cors';

import { Compiler } from '../compiler/index.js';
import { colors, handleLazyCompilation, resolveConfig } from '../index.js';
import Watcher from '../watcher/index.js';
import { HmrEngine } from './hmr-engine.js';
import { httpServer } from './http.js';
import { openBrowser } from './open.js';
import { WsServer } from './ws.js';

import { __FARM_GLOBAL__ } from '../config/_global.js';
import { getSortedPluginHooksBindThis } from '../plugin/index.js';
import { isCacheDirExists } from '../utils/cacheDir.js';
import { createDebugger } from '../utils/debug.js';
import {
  resolveServerUrls,
  setupSIGTERMListener,
  teardownSIGTERMListener
} from '../utils/http.js';
import { Logger, bootstrap, printServerUrls } from '../utils/logger.js';
import { initPublicFiles } from '../utils/publicDir.js';
import {
  arrayEqual,
  getValidPublicPath,
  isObject,
  normalizePath
} from '../utils/share.js';

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

import type {
  Server as HttpBaseServer,
  ServerOptions as HttpsServerOptions
} from 'node:http';
import type { Http2SecureServer } from 'node:http2';
import type * as net from 'node:net';

import { createCompiler } from '../compiler/index.js';
import type {
  FarmCliOptions,
  NormalizedServerConfig,
  ResolvedUserConfig,
  UserConfig
} from '../config/types.js';
import type {
  JsUpdateResult,
  PersistentCacheConfig
} from '../types/binding.js';
import { convertErrorMessage } from '../utils/error.js';
import type { CommonServerOptions, ResolvedServerUrls } from './http.js';

export type HttpServer = HttpBaseServer | Http2SecureServer;

type CompilerType = Compiler | undefined;

export type ServerConfig = CommonServerOptions & NormalizedServerConfig;

export const debugServer = createDebugger('farm:server');

/**
 * Represents a Farm development server.
 * @class
 */
export class Server extends httpServer {
  ws: WsServer;
  serverOptions: ServerConfig;
  httpsOptions: HttpsServerOptions;
  publicDir: string | undefined;
  publicPath?: string;
  publicFiles?: Set<string>;
  httpServer: HttpServer;
  watcher: Watcher;
  hmrEngine?: HmrEngine;
  middlewares: connect.Server;
  compiler?: CompilerType;
  root: string;
  config: ResolvedUserConfig;
  closeHttpServerFn: () => Promise<void>;
  terminateServerFn: (_: unknown, exitCode?: number) => Promise<void>;
  postConfigureServerHooks: ((() => void) | void)[] = [];
  logger: Logger;

  /**
   * Creates an instance of Server.
   * @param {FarmCliOptions & UserConfig} inlineConfig - The inline configuration options.
   */
  constructor(readonly inlineConfig: FarmCliOptions & UserConfig = {}) {
    super();
    this.logger = new Logger();
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
    inlineConfig: FarmCliOptions & UserConfig = {}
  ): Promise<Server> {
    const server = new Server(inlineConfig);
    server.config = await resolveConfig(
      server.inlineConfig,
      'dev',
      'development'
    );

    server.logger = server.config.logger;

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

    if (!server.serverOptions.middlewareMode) {
      setupSIGTERMListener(server.terminateServerFn);
    }

    if (!server.serverOptions.middlewareMode && server.httpServer) {
      server.httpServer.once('listening', () => {
        // update actual port since this may be different from initial value
        server.serverOptions.port = (
          server.httpServer.address() as net.AddressInfo
        ).port;
      });
    }

    return server;
  }

  /**
   * create watcher
   */
  async #createWatcher() {
    this.watcher = new Watcher(this.config);

    await this.watcher.createWatcher();

    this.watcher.on('add', async (file: string) => {
      // TODO pluginContainer hooks
    });

    this.watcher.on('unlink', async (file: string) => {
      // Fix #2035, skip if the file is irrelevant
      if (!this.compiler.hasModule(file)) return;

      const parentFiles = this.compiler.getParentFiles(file);
      const normalizeParentFiles = parentFiles.map((file) =>
        normalizePath(file)
      );
      this.hmrEngine.hmrUpdate(normalizeParentFiles, true);
    });

    this.watcher.on('change', async (file: string) => {
      file = normalizePath(file);

      if (this.watcher.isConfigFilesChanged(file)) {
        try {
          await this.restartServer();
        } catch (e) {
          this.config.logger.error(`restart server error ${e}`);
        }

        return;
      }

      try {
        this.hmrEngine.hmrUpdate(file);
      } catch (error) {
        this.config.logger.error(`Farm Hmr Update Error: ${error}`);
      }
    });

    const handleUpdateFinish = (updateResult: JsUpdateResult) => {
      const added = [
        ...updateResult.added,
        ...updateResult.extraWatchResult.add
      ].map((addedModule) => {
        const resolvedPath = this.compiler.transformModulePath(
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
    this.config.compilation.define.FARM_HMR_PORT = serverPort.toString();

    if (this.config.server.hmr?.port === this.config.server?.port) {
      this.config.server.hmr ??= {};
      this.config.server.hmr.port = serverPort;
    }
    this.config.server.port = serverPort;

    this.serverOptions.port = serverPort;

    if (this.config.compilation.lazyCompilation) {
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
    const {
      serverOptions: { port, host },
      resolvedUrls
    } = newServer;

    if (
      port !== prevPort ||
      host !== prevHost ||
      this.hasUrlsChanged(prevUrls, resolvedUrls)
    ) {
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
    let newServer: Server = null;
    try {
      await this.close();
      newServer = await Server.createServer(this.inlineConfig);
    } catch (error) {
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
    await this.ws.createWebSocketServer();
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
    if (!this.httpServer) {
      this.logger.warn('HTTP server is not created yet');
      return;
    }
    const { port, hostname, open, strictPort } = this.serverOptions;

    try {
      const serverPort = await this.httpServerStart({
        port,
        strictPort: strictPort,
        host: hostname.host
      });
      this.#updateServerPort(serverPort, 'START');

      this.resolvedUrls = await resolveServerUrls(this.httpServer, this.config);

      // compile the project and start the dev server
      await this.#startCompile();

      // watch extra files after compile
      this.watcher?.watchExtraFiles?.();

      if (open) {
        this.#openServerBrowser();
      }
    } catch (error) {
      this.config.logger.error(
        `Start DevServer Error: ${error} \n ${error.stack}`
      );
      // throw error;
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

  /**
   * Adds additional files to be watched by the compiler.
   * @param {string} root - The root directory.
   * @param {string[]} deps - Array of file paths to be watched.
   */
  addWatchFile(root: string, deps: string[]): void {
    this.getCompiler().addExtraWatchFile(root, deps);
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
      jsPlugins,
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
    const {
      compilation: {
        output: { publicPath },
        assets: { publicDir }
      },
      root,
      server
    } = this.config;
    this.publicPath = getValidPublicPath(publicPath);
    this.publicDir = publicDir;
    if (server.origin?.endsWith('/')) {
      server.origin = server.origin.slice(0, -1);
      this.config.logger.warn(
        `${colors.bold('(!)')} server.origin should not end with "/". Using "${
          server.origin
        }" instead.`
      );
    }
    this.serverOptions = server as CommonServerOptions & NormalizedServerConfig;
    this.root = root;
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

    if (proxy) {
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

    if (fs.existsSync(this.publicDir as PathLike)) {
      this.middlewares.use(publicMiddleware(this));
    }

    if (this.config.compilation.lazyCompilation) {
      this.middlewares.use(lazyCompilationMiddleware(this));
    }

    this.middlewares.use(staticMiddleware(this));

    // Check dev resource tree in `_output_files` url
    this.middlewares.use(outputFilesMiddleware(this));

    if (this.config.server.writeToDisk) {
      this.middlewares.use(resourceDiskMiddleware(this));
    } else {
      this.middlewares.use(resourceMiddleware(this));
    }

    this.postConfigureServerHooks.reduce((_, fn) => {
      if (typeof fn === 'function') fn();
    }, null);

    this.serverOptions.middlewares?.forEach((middleware) =>
      this.middlewares.use(middleware(this))
    );

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
    try {
      await this.compiler.compile();

      await (this.config.server.writeToDisk
        ? this.compiler.writeResourcesToDisk()
        : this.compiler.callWriteResourcesHook());
    } catch (err) {
      this.config.logger.error(
        `Compilation failed: ${convertErrorMessage(err)}`,
        {
          exit: true
        }
      );
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
    this.setCompiler(createCompiler(this.config));

    for (const hook of getSortedPluginHooksBindThis(
      this.config.jsPlugins,
      'configureCompiler'
    )) {
      await hook?.(this.compiler);
    }

    // check if cache dir exists
    const { persistentCache } = this.compiler.config.compilation;
    const hasCacheDir = await isCacheDirExists(
      (persistentCache as PersistentCacheConfig).cacheDir
    );
    const start = performance.now();
    await this.#compile();

    const duration = performance.now() - start;
    bootstrap(duration, this.compiler.config, hasCacheDir);
  }

  /**
   * Handles the initialization of public files.
   * @private
   * @returns {Promise<Set<string>>} A promise that resolves to a set of public file paths.
   */
  async #handlePublicFiles(): Promise<Set<string>> {
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
      this.config.logger.info(`HMR invalidate: ${path}. ${message ?? ''} `);
      const parentFiles = this.compiler.getParentFiles(path);
      const normalizeParentFiles = parentFiles.map((file) =>
        normalizePath(file)
      );
      this.hmrEngine.hmrUpdate(normalizeParentFiles, true);
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

    this.httpServer.on('connection', (socket) => {
      openSockets.add(socket);
      debugServer?.(`has open server socket ${openSockets}`);

      socket.on('close', () => {
        debugServer?.('close all server socket');
        openSockets.delete(socket);
      });
    });

    this.httpServer.once('listening', () => {
      hasListened = true;
    });

    return () =>
      new Promise<void>((resolve, reject) => {
        openSockets.forEach((s) => s.destroy());

        if (hasListened) {
          this.httpServer.close((err) => {
            if (err) {
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
    if (!this.serverOptions.middlewareMode) {
      teardownSIGTERMListener(this.terminateServerFn);
    }

    await Promise.allSettled([
      this.watcher.close(),
      this.ws.wss.close(),
      this.closeHttpServerFn()
    ]);
    this.resolvedUrls = null;
  }

  printUrls() {
    if (!__FARM_GLOBAL__.__FARM_SHOW_DEV_SERVER_URL__) {
      return;
    }
    if (this.resolvedUrls) {
      printServerUrls(
        this.resolvedUrls,
        this.serverOptions.host,
        this.config.logger
      );
    } else if (this.serverOptions.middlewareMode) {
      throw new Error('cannot print server URLs in middleware mode.');
    } else {
      throw new Error(
        'cannot print server URLs before server.listen is called.'
      );
    }
  }
}
