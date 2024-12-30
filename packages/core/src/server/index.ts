export * from './preview.js';

import fs, { PathLike } from 'node:fs';
import connect from 'connect';
import corsMiddleware from 'cors';

import { Compiler } from '../compiler/index.js';
import { bold, colors, getShortName, green, resolveConfig } from '../index.js';
import Watcher from '../watcher/index.js';
import { HmrEngine } from './hmr-engine.js';
import { httpServer } from './http.js';
import { openBrowser } from './open.js';
import { WsServer } from './ws.js';

import { __FARM_GLOBAL__ } from '../config/_global.js';
import {
  getPluginHooks,
  getSortedPluginHooksBindThis
} from '../plugin/index.js';
import { isCacheDirExists } from '../utils/cacheDir.js';
import { createDebugger } from '../utils/debug.js';
import {
  resolveServerUrls,
  setupSIGTERMListener,
  teardownSIGTERMListener
} from '../utils/http.js';
import { Logger, bootstrap, printServerUrls } from '../utils/logger.js';
import { initPublicFiles } from '../utils/publicDir.js';
import { arrayEqual, isObject, normalizePath } from '../utils/share.js';

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

import type * as http from 'node:http';
import type {
  Server as HttpBaseServer,
  ServerOptions as HttpsServerOptions
} from 'node:http';
import type { Http2SecureServer } from 'node:http2';
import type * as net from 'node:net';

import { createCompiler } from '../compiler/index.js';
import type {
  FarmCliOptions,
  HmrOptions,
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

export interface ServerOptions extends CommonServerOptions {
  /**
   * Configure HMR-specific options (port, host, path & protocol)
   */
  hmr?: HmrOptions | boolean;
  /**
   * Do not start the websocket connection.
   */
  ws?: false;
  /**
   * Create dev server to be used as a middleware in an existing server
   * @default false
   */
  middlewareMode?:
    | boolean
    | {
        /**
         * Parent server instance to attach to
         *
         * This is needed to proxy WebSocket connections to the parent server.
         */
        server: http.Server;
      };
  appType?: 'spa' | 'mpa' | 'custom';
  /**
   * Origin for the generated asset URLs.
   */
  origin?: string;
}

type ServerConfig = CommonServerOptions & NormalizedServerConfig;

export const debugServer = createDebugger('farm:server');

export function noop() {
  // noop
}

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
  compiler: CompilerType;
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
  constructor(readonly inlineConfig: FarmCliOptions & UserConfig) {
    super();
    this.logger = new Logger();
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
  async createServer(): Promise<void> {
    try {
      this.config = await resolveConfig(
        this.inlineConfig,
        'start',
        'development',
        'development'
      );

      this.logger = this.config.logger;

      this.#resolveOptions();

      this.compiler = createCompiler(this.config);

      for (const hook of getPluginHooks(
        this.config.jsPlugins,
        'configureCompiler'
      )) {
        await hook?.(this.compiler);
      }

      const [httpsOptions, publicFiles] = await Promise.all([
        this.resolveHttpsConfig(this.serverOptions.https),
        this.#handlePublicFiles()
      ]);
      this.httpsOptions = httpsOptions;
      this.publicFiles = publicFiles;
      this.middlewares = connect() as connect.Server;
      this.httpServer = this.serverOptions.middlewareMode
        ? null
        : await this.resolveHttpServer(
            this.serverOptions as CommonServerOptions,
            this.middlewares,
            this.httpsOptions
          );

      // close server function prepare promise
      this.closeHttpServerFn = this.closeServer();

      // init hmr engine When actually updating, we need to get the clients of ws for broadcast, 、
      // so we can instantiate hmrEngine by default at the beginning.
      this.createHmrEngine();

      // init websocket server
      this.createWebSocketServer();

      // invalidate vite handler
      this.#invalidateVite();

      // init watcher
      await this.#createWatcher();

      await this.handleConfigureServer();

      // init middlewares
      this.#initializeMiddlewares();

      this.terminateServerFn = async (_: unknown, exitCode?: number) => {
        try {
          await this.close();
        } finally {
          process.exitCode ??= exitCode ? 128 + exitCode : undefined;
          process.exit();
        }
      };

      if (!this.serverOptions.middlewareMode) {
        setupSIGTERMListener(this.terminateServerFn);
      }

      if (!this.serverOptions.middlewareMode && this.httpServer) {
        this.httpServer.once('listening', () => {
          // update actual port since this may be different from initial value
          this.serverOptions.port = (
            this.httpServer.address() as net.AddressInfo
          ).port;
        });
      }
    } catch (error) {
      // this.logger.error(
      //   `Failed to create farm server: ${error}`
      // );
      throw error;
    }
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
      const parentFiles = this.compiler.getParentFiles(file);
      const normalizeParentFiles = parentFiles.map((file) =>
        normalizePath(file)
      );
      this.hmrEngine.hmrUpdate(normalizeParentFiles, true);
    });

    this.watcher.on('change', async (file: string) => {
      file = normalizePath(file);
      const shortFile = getShortName(file, this.config.root);
      const isConfigFile = this.config.configFilePath === file;
      const isConfigDependencyFile = this.config.configFileDependencies.some(
        (name) => file === name
      );
      const isEnvFile = this.config.envFiles.some((name) => file === name);
      if (isConfigFile || isConfigDependencyFile || isEnvFile) {
        __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ = true;
        this.logger.info(
          `${bold(green(shortFile))} changed, Server is being restarted`,
          true
        );
        debugServer?.(`[config change] ${colors.dim(file)}`);
        try {
          await this.restartServer();
        } catch (e) {
          this.config.logger.error(`restart server error ${e}`);
        }
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
      newServer = new Server(this.inlineConfig);
      await newServer.createServer();
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
        'Websocket requires a http server. please check the server is be created'
      );
    }

    this.ws = new WsServer(this);
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

      this.config.compilation.define.FARM_HMR_PORT = serverPort.toString();

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
    this.publicPath = publicPath;
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
        `Compilation failed: ${convertErrorMessage(err)}`
      );
      // throw err;
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
