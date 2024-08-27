import fs, { PathLike } from 'node:fs';
import { WatchOptions } from 'chokidar';
import connect from 'connect';
import corsMiddleware from 'cors';

import { Compiler } from '../compiler/index.js';
import { createCompiler } from '../index.js';
import { FileWatcher } from '../watcher/index.js';
import { HmrEngine } from './hmr-engine.js';
import { CommonServerOptions, httpServer } from './http.js';
import { openBrowser } from './open.js';
import { WsServer } from './ws.js';

import { __FARM_GLOBAL__ } from '../config/_global.js';
import { getCacheDir, isCacheDirExists } from '../utils/cacheDir.js';
import { Logger, bootstrap, logger } from '../utils/logger.js';
import { initPublicFiles } from '../utils/publicDir.js';
import { isObject } from '../utils/share.js';

import {
  adaptorViteMiddleware,
  hmrPingMiddleware,
  htmlFallbackMiddleware,
  lazyCompilationMiddleware,
  notFoundMiddleware,
  proxyMiddleware,
  publicMiddleware,
  publicPathMiddleware,
  resourceMiddleware
} from './middlewares/index.js';

import type * as http from 'node:http';
import type { ServerOptions as HttpsServerOptions, Server } from 'node:http';
import type { Http2SecureServer } from 'node:http2';
import type { HMRChannel } from './hmr.js';

import { normalizePublicPath } from '../config/normalize-config/normalize-output.js';
import type {
  NormalizedServerConfig,
  ResolvedUserConfig
} from '../config/types.js';

export type HttpServer = Server | Http2SecureServer;

type CompilerType = Compiler | undefined;

export interface HmrOptions {
  protocol?: string;
  host?: string;
  port?: number;
  clientPort?: number;
  path?: string;
  timeout?: number;
  overlay?: boolean;
  server?: Server;
  /** @internal */
  channels?: HMRChannel[];
}

export interface ServerOptions extends CommonServerOptions {
  /**
   * Configure HMR-specific options (port, host, path & protocol)
   */
  hmr?: HmrOptions | boolean;
  /**
   * Do not start the websocket connection.
   * @experimental
   */
  ws?: false;
  /**
   * chokidar watch options or null to disable FS watching
   * https://github.com/paulmillr/chokidar#api
   */
  watchOptions?: WatchOptions | undefined;
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
  origin?: string;
}

export function noop() {
  // noop
}

type ServerConfig = CommonServerOptions & NormalizedServerConfig;

// TODO 改 newServer 的 name and PascalCase
export class NewServer extends httpServer {
  ws: WsServer;
  serverOptions: ServerConfig;
  httpsOptions: HttpsServerOptions;
  publicDir: string | boolean | undefined;
  publicPath?: string;
  publicFiles?: Set<string>;
  httpServer: HttpServer;
  watcher: FileWatcher;
  hmrEngine?: HmrEngine;
  middlewares: connect.Server;
  compiler: CompilerType;
  constructor(
    readonly resolvedUserConfig: ResolvedUserConfig,
    logger: Logger
  ) {
    super(logger);
    this.#resolveOptions();
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
      const { https, middlewareMode } = this.serverOptions;

      this.httpsOptions = await this.resolveHttpsConfig(https);
      this.publicFiles = await this.#handlePublicFiles();

      this.middlewares = connect() as connect.Server;
      this.httpServer = middlewareMode
        ? null
        : await this.resolveHttpServer(
            this.serverOptions as CommonServerOptions,
            this.middlewares,
            this.httpsOptions
          );

      // init hmr engine When actually updating, we need to get the clients of ws for broadcast, 、
      // so we can instantiate hmrEngine by default at the beginning.
      this.createHmrEngine();

      // init websocket server
      this.createWebSocketServer();

      // invalidate vite handler
      this.#invalidateVite();

      // init middlewares
      this.#initializeMiddlewares();
    } catch (error) {
      this.logger.error(`Failed to create farm server: ${error}`);
      throw error;
    }
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
    // TODO open browser when server is ready && open config is true

    const { port, hostname, open, strictPort } = this.serverOptions;

    try {
      const serverPort = await this.httpServerStart({
        port,
        strictPort: strictPort,
        host: hostname.host
      });

      // 这块要重新设计 restart 还有 端口冲突的问题
      // this.resolvedUserConfig
      this.resolvedUserConfig.compilation.define.FARM_HMR_PORT =
        serverPort.toString();

      this.compiler = await createCompiler(this.resolvedUserConfig, logger);

      // compile the project and start the dev server
      await this.#startCompilation();

      // watch extra files after compile
      this.watcher?.watchExtraFiles?.();
      !__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ &&
        (await this.displayServerUrls(this.serverOptions, this.publicPath));

      if (open) {
        this.#openServerBrowser();
      }
    } catch (error) {
      this.logger.error(`start farm dev server error: ${error}`);
      throw error;
    }
  }

  /**
   * Starts the HTTP server.
   * @protected
   * @param {Object} serverOptions - The server options.
   * @returns {Promise<number>} The port the server is listening on.
   * @throws {Error} If the server fails to start.
   */
  protected async httpServerStart(serverOptions: {
    port: number;
    strictPort: boolean | undefined;
    host: string | undefined;
  }): Promise<number> {
    if (!this.httpServer) {
      throw new Error('httpServer is not initialized');
    }

    let { port, strictPort, host } = serverOptions;

    return new Promise((resolve, reject) => {
      const onError = (e: Error & { code?: string }) => {
        if (e.code === 'EADDRINUSE') {
          if (strictPort) {
            this.httpServer.removeListener('error', onError);
            reject(new Error(`Port ${port} is already in use`));
          } else {
            console.info(`Port ${port} is in use, trying another one...`);
            this.httpServer.listen(++port, host);
          }
        } else {
          this.httpServer.removeListener('error', onError);
          reject(e);
        }
      };

      this.httpServer.on('error', onError);

      this.httpServer.listen(port, host, () => {
        this.httpServer.removeListener('error', onError);
        resolve(port);
      });
    });
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
   * resolve and setting server options
   *
   * this method extracts compilation and server options from resolvedUserConfig
   * and set the publicPath and publicDir， and then resolve server options
   * @private
   * @returns { void }
   */
  #resolveOptions() {
    const { compilation, server } = this.resolvedUserConfig;
    this.publicPath = compilation.output.publicPath;

    this.publicDir = compilation.assets.publicDir;

    this.serverOptions = server as CommonServerOptions & NormalizedServerConfig;
  }

  /**
   * Initializes and configures the middleware stack for the server.
   * @private
   */
  #initializeMiddlewares() {
    this.middlewares.use(hmrPingMiddleware());

    const { proxy, middlewareMode, cors } = this.serverOptions;

    if (cors) {
      this.middlewares.use(
        corsMiddleware(typeof cors === 'boolean' ? {} : cors)
      );
    }

    if (proxy) {
      const middlewareServer =
        isObject(middlewareMode) && 'server' in middlewareMode
          ? middlewareMode.server
          : this.httpServer;

      this.middlewares.use(proxyMiddleware(this, middlewareServer));
    }

    if (this.publicPath !== '/') {
      this.middlewares.use(publicPathMiddleware(this));
    }

    if (fs.existsSync(this.publicDir as PathLike)) {
      this.middlewares.use(publicMiddleware(this));
    }

    if (this.resolvedUserConfig.compilation.lazyCompilation) {
      this.middlewares.use(lazyCompilationMiddleware(this));
    }

    if (this.resolvedUserConfig.vitePlugins.length) {
      this.middlewares.use(adaptorViteMiddleware(this));
    }

    // TODO todo add appType 这块要判断 单页面还是 多页面 多 html 处理不一样
    this.middlewares.use(htmlFallbackMiddleware(this));

    this.middlewares.use(resourceMiddleware(this));

    this.middlewares.use(notFoundMiddleware());
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
      await (this.resolvedUserConfig.server.writeToDisk
        ? this.compiler.writeResourcesToDisk()
        : this.compiler.callWriteResourcesHook());
    } catch (err) {
      this.logger.error('Compilation failed:', err);
      throw err;
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
  async #startCompilation() {
    // check if cache dir exists
    const { root, persistentCache } = this.compiler.config.config;
    const hasCacheDir = await isCacheDirExists(
      getCacheDir(root, persistentCache)
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
  async #handlePublicFiles() {
    const initPublicFilesPromise = initPublicFiles(this.resolvedUserConfig);
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
      const parentFiles = this.compiler.getParentFiles(path);
      this.hmrEngine.hmrUpdate(parentFiles, true);
    });
  }
}
