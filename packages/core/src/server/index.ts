import http from 'node:http';
import http2 from 'node:http2';
import Koa from 'koa';
import compression from 'koa-compress';

import path from 'node:path';
import { promisify } from 'node:util';
import { Compiler } from '../compiler/index.js';
import { __FARM_GLOBAL__ } from '../config/_global.js';
import {
  DEFAULT_HMR_OPTIONS,
  DevServerMiddleware,
  NormalizedServerConfig,
  UserPreviewServerConfig,
  UserServerConfig,
  normalizePublicDir
} from '../config/index.js';
import {
  getValidPublicPath,
  normalizePublicPath
} from '../config/normalize-config/normalize-output.js';
import { resolveHostname, resolveServerUrls } from '../utils/http.js';
import {
  Logger,
  bootstrap,
  clearScreen,
  normalizeBasePath,
  printServerUrls
} from '../utils/index.js';
import { FileWatcher } from '../watcher/index.js';
import { logError } from './error.js';
import { HmrEngine } from './hmr-engine.js';
import {
  cors,
  headers,
  lazyCompilation,
  proxy,
  records,
  resources,
  staticMiddleware
} from './middlewares/index.js';
import { openBrowser } from './open.js';
import { Server as httpServer } from './type.js';
import WsServer from './ws.js';

/**
 * Farm Dev Server, responsible for:
 * * parse and normalize dev server options
 * * launch http server based on options
 * * compile the project in dev mode and serve the production
 * * HMR middleware and websocket supported
 */
interface ServerUrls {
  local: string[];
  network: string[];
}

type ErrorMap = {
  EACCES: string;
  EADDRNOTAVAIL: string;
};

interface ImplDevServer {
  createServer(options: UserServerConfig): void;
  createDevServer(options: UserServerConfig): void;
  createPreviewServer(options: UserServerConfig): void;
  listen(): Promise<void>;
  close(): Promise<void>;
  getCompiler(): Compiler;
}

export class Server implements ImplDevServer {
  private _app: Koa;
  private restart_promise: Promise<void> | null = null;
  private compiler: Compiler | null;
  public logger: Logger;

  ws: WsServer;
  config: NormalizedServerConfig & UserPreviewServerConfig;
  hmrEngine?: HmrEngine;
  server?: httpServer;
  publicDir?: string;
  publicPath?: string;
  resolvedUrls?: ServerUrls;
  watcher: FileWatcher;

  constructor({
    compiler = null,
    logger
  }: {
    compiler?: Compiler | null;
    logger: Logger;
  }) {
    this.compiler = compiler;
    this.logger = logger ?? new Logger();

    this.initializeKoaServer();

    if (!compiler) return;

    this.publicDir = normalizePublicDir(compiler?.config.config.root);

    this.publicPath =
      normalizePublicPath(
        compiler.config.config.output.targetEnv,
        compiler.config.config.output.publicPath,
        logger,
        false
      ) || '/';
  }

  getCompiler(): Compiler {
    return this.compiler;
  }

  app(): Koa {
    return this._app;
  }

  async listen(): Promise<void> {
    if (!this.server) {
      this.logger.error('HTTP server is not created yet');
      return;
    }
    const { port, open, protocol, hostname } = this.config;

    const start = Date.now();
    // compile the project and start the dev server
    await this.compile();

    // watch extra files after compile
    this.watcher?.watchExtraFiles?.();

    bootstrap(Date.now() - start, this.compiler.config);

    await this.startServer(this.config);

    !__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ &&
      (await this.displayServerUrls());

    if (open) {
      let publicPath = getValidPublicPath(this.publicPath) ?? '/';

      const serverUrl = `${protocol}://${hostname.name}:${port}${publicPath}`;
      openBrowser(serverUrl);
    }
  }

  private async compile(): Promise<void> {
    try {
      await this.compiler.compile();
    } catch (err) {
      throw new Error(logError(err) as unknown as string);
    }

    if (this.config.writeToDisk) {
      this.compiler.writeResourcesToDisk();
    } else {
      this.compiler.callWriteResourcesHook();
    }
  }

  async startServer(serverOptions: UserServerConfig) {
    const { port, hostname } = serverOptions;
    const listen = promisify(this.server.listen).bind(this.server);
    try {
      await listen(port, hostname.host);
    } catch (error) {
      this.handleServerError(error, port, hostname.host);
    }
  }

  handleServerError(
    error: Error & { code?: string },
    port: number,
    host: string | undefined
  ) {
    const errorMap: ErrorMap = {
      EACCES: `Permission denied to use port ${port} `,
      EADDRNOTAVAIL: `The IP address host: ${host} is not available on this machine.`
    };

    const errorMessage =
      errorMap[error.code as keyof ErrorMap] ||
      `An error occurred: ${error.stack} `;
    this.logger.error(errorMessage);
  }

  async close() {
    if (!this.server) {
      this.logger.error('HTTP server is not created yet');
    }
    // the server is already closed
    if (!this.server.listening) {
      return;
    }
    const promises = [];
    if (this.ws) {
      promises.push(this.ws.close());
    }

    if (this.server) {
      promises.push(new Promise((resolve) => this.server.close(resolve)));
    }

    await Promise.all(promises);
  }

  async restart(promise: () => Promise<void>) {
    if (!this.restart_promise) {
      this.restart_promise = promise();
    }
    return this.restart_promise;
  }

  private initializeKoaServer() {
    this._app = new Koa();
  }

  public async createServer(
    options: NormalizedServerConfig & UserPreviewServerConfig
  ) {
    const { https, host } = options;
    const protocol = https ? 'https' : 'http';
    const hostname = await resolveHostname(host);
    const publicPath = getValidPublicPath(
      this.compiler?.config.config.output?.publicPath ??
        options?.output.publicPath
    );
    // TODO refactor previewServer If it's preview server, then you can't use create server. we need to create a new one because hmr is false when you preview.
    const hmrPath = normalizeBasePath(
      path.join(publicPath, options.hmr.path ?? DEFAULT_HMR_OPTIONS.path)
    );

    this.config = {
      ...options,
      port: Number(process.env.FARM_DEV_SERVER_PORT || options.port),
      hmr: {
        ...options.hmr,
        path: hmrPath
      },
      protocol,
      hostname
    };

    if (https) {
      this.server = http2.createSecureServer(
        {
          ...https,
          allowHTTP1: true
        },
        this._app.callback()
      );
    } else {
      this.server = http.createServer(this._app.callback());
    }
  }

  public createWebSocket() {
    if (!this.server) {
      throw new Error('Websocket requires a server.');
    }
    this.ws = new WsServer(this.server, this.config, this.hmrEngine);
  }

  private invalidateVite() {
    // Note: path should be Farm's id, which is a relative path in dev mode,
    // but in vite, it's a url path like /xxx/xxx.js
    this.ws.on('vite:invalidate', ({ path, message }) => {
      // find hmr boundary starting from the parent of the file
      this.logger.info(`HMR invalidate: ${path}. ${message ?? ''} `);
      const parentFiles = this.compiler.getParentFiles(path);
      this.hmrEngine.hmrUpdate(parentFiles, true);
    });
  }

  public async createPreviewServer(options: UserPreviewServerConfig) {
    await this.createServer(options as NormalizedServerConfig);

    this.applyPreviewServerMiddlewares(this.config.middlewares);

    await this.startServer(this.config);

    await this.displayServerUrls(true);
  }

  public async createDevServer(options: NormalizedServerConfig) {
    if (!this.compiler) {
      throw new Error('DevServer requires a compiler for development mode.');
    }

    await this.createServer(options);

    this.hmrEngine = new HmrEngine(this.compiler, this, this.logger);

    this.createWebSocket();

    this.invalidateVite();

    this.applyServerMiddlewares(options.middlewares);
  }

  static async resolvePortConflict(
    normalizedDevConfig: NormalizedServerConfig,
    logger: Logger
  ): Promise<void> {
    let devPort = normalizedDevConfig.port;
    let hmrPort = normalizedDevConfig.hmr.port;

    const { strictPort, host } = normalizedDevConfig;
    const httpServer = http.createServer();
    const isPortAvailable = (portToCheck: number) => {
      return new Promise((resolve, reject) => {
        const onError = async (error: { code: string }) => {
          if (error.code === 'EADDRINUSE') {
            clearScreen();
            if (strictPort) {
              httpServer.removeListener('error', onError);
              reject(new Error(`Port ${devPort} is already in use`));
            } else {
              logger.warn(`Port ${devPort} is in use, trying another one...`);
              httpServer.removeListener('error', onError);
              resolve(false);
            }
          } else {
            logger.error(`Error in httpServer: ${error} `);
            reject(true);
          }
        };
        httpServer.on('error', onError);
        httpServer.on('listening', () => {
          httpServer.close();
          resolve(true);
        });
        httpServer.listen(portToCheck, host as string);
      });
    };

    let isPortAvailableResult = await isPortAvailable(devPort);

    while (isPortAvailableResult === false) {
      if (typeof normalizedDevConfig.hmr === 'object') {
        normalizedDevConfig.hmr.port = ++hmrPort;
      }

      normalizedDevConfig.port = ++devPort;
      isPortAvailableResult = await isPortAvailable(devPort);
    }
  }

  /**
   * Add listening files for root manually
   *
   * > listening file with root must as file.
   *
   * @param root
   * @param deps
   */

  addWatchFile(root: string, deps: string[]): void {
    this.getCompiler().addExtraWatchFile(root, deps);
  }

  applyMiddlewares(internalMiddlewares?: DevServerMiddleware[]) {
    internalMiddlewares.forEach((middleware) => {
      const middlewareImpl = middleware(this);

      if (middlewareImpl) {
        if (Array.isArray(middlewareImpl)) {
          middlewareImpl.forEach((m) => {
            this._app.use(m);
          });
        } else {
          this._app.use(middlewareImpl);
        }
      }
    });
  }

  setCompiler(compiler: Compiler) {
    this.compiler = compiler;
  }

  private applyPreviewServerMiddlewares(
    middlewares?: DevServerMiddleware[]
  ): void {
    const internalMiddlewares = [
      ...(middlewares || []),
      compression,
      proxy,
      staticMiddleware
    ];
    this.applyMiddlewares(internalMiddlewares as DevServerMiddleware[]);
  }

  private applyServerMiddlewares(middlewares?: DevServerMiddleware[]): void {
    const internalMiddlewares = [
      ...(middlewares || []),
      headers,
      lazyCompilation,
      cors,
      resources,
      records,
      proxy
    ];

    this.applyMiddlewares(internalMiddlewares as DevServerMiddleware[]);
  }

  private async displayServerUrls(showPreviewFlag = false) {
    let publicPath = getValidPublicPath(
      this.compiler
        ? this.compiler.config.config.output?.publicPath
        : this.config.output.publicPath
    );

    this.resolvedUrls = await resolveServerUrls(
      this.server,
      this.config,
      publicPath
    );

    if (this.resolvedUrls) {
      printServerUrls(this.resolvedUrls, this.logger, showPreviewFlag);
    } else {
      throw new Error('cannot print server URLs with Server Error.');
    }
  }
}
