import http from 'node:http';
import http2 from 'node:http2';
import Koa from 'koa';
import compression from 'koa-compress';

import { Compiler } from '../compiler/index.js';
import {
  DEFAULT_HMR_OPTIONS,
  DevServerMiddleware,
  NormalizedServerConfig,
  normalizePublicDir,
  normalizePublicPath,
  UserServerConfig
} from '../config/index.js';
import { HmrEngine } from './hmr-engine.js';
import { openBrowser } from './openBrowser.js';
import {
  bootstrap,
  clearScreen,
  Logger,
  printServerUrls
} from '../utils/index.js';
import {
  cors,
  headers,
  lazyCompilation,
  proxy,
  records,
  resources,
  sirvMiddleware
} from './middlewares/index.js';
import { __FARM_GLOBAL__ } from '../config/_global.js';
import { resolveServerUrls } from '../utils/http.js';
import WsServer from './ws.js';
import { Server } from './type.js';

/**
 * Farm Dev Server, responsible for:
 * * parse and normalize dev server options
 * * launch http server based on options
 * * compile the project in dev mode and serve the production
 * * HMR middleware and websocket supported
 */

interface FarmServerContext {
  config: UserServerConfig;
  previewConfig: any;
  app: Koa;
  server: Server;
  compiler: Compiler;
  logger: Logger;
  serverOptions?: {
    resolvedUrls?: ServerUrls;
  };
}
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
  createPreviewServer(options: UserServerConfig): void;
  listen(): Promise<void>;
  close(): Promise<void>;
  getCompiler(): Compiler;
}

export class DevServer implements ImplDevServer {
  private _app: Koa;
  private restart_promise: Promise<void> | null = null;
  public _context: FarmServerContext;

  ws: WsServer;
  config: NormalizedServerConfig;
  previewServerConfig: NormalizedServerConfig | any;
  hmrEngine?: HmrEngine;
  server?: Server;
  publicDir?: string;
  publicPath?: string;

  constructor(private compiler: Compiler | null, public logger: Logger) {
    this.initApp();
    if (!compiler) return;
    this.publicDir = normalizePublicDir(compiler?.config.config.root);

    this.publicPath =
      normalizePublicPath(
        compiler.config.config.output?.publicPath,
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

    bootstrap(Date.now() - start, this.compiler.config);

    await this.startServer(this.config);

    !__FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ &&
      (await this.printServerUrls());

    if (open) {
      const publicPath =
        this.publicPath === '/' ? this.publicPath : `/${this.publicPath}`;
      const serverUrl = `${protocol}://${hostname}:${port}${publicPath}`;
      openBrowser(serverUrl);
    }
  }

  private async compile(): Promise<void> {
    await this.compiler.compile();

    if (this.config.writeToDisk) {
      const base = this.publicPath.match(/^https?:\/\//) ? '' : this.publicPath;
      this.compiler.writeResourcesToDisk(base);
    }
  }

  async startServer(serverOptions: UserServerConfig) {
    const { port, host } = serverOptions;
    try {
      await new Promise((resolve) => {
        this.server.listen(port, host as string, () => {
          resolve(port);
        });
      });
    } catch (error) {
      this.handleServerError(error, port, host);
    }
  }

  handleServerError(
    error: Error & { code?: string },
    port: number,
    ip: string | boolean
  ) {
    const errorMap: ErrorMap = {
      EACCES: `Permission denied to use port ${port}`,
      EADDRNOTAVAIL: `The IP address ${ip} is not available on this machine.`
    };

    const errorMessage =
      errorMap[error.code as keyof ErrorMap] || `An error occurred: ${error}`;
    this.logger.error(errorMessage);
    this.close();
  }

  async close() {
    if (!this.server) {
      this.logger.error('HTTP server is not created yet');
    }
    await this.closeFarmServer();
    process.exit(0);
  }

  async restart(promise: () => Promise<void>) {
    if (!this.restart_promise) {
      this.restart_promise = promise();
    }
    return this.restart_promise;
  }

  async closeFarmServer() {
    const promises = [];

    if (this.ws) {
      promises.push(this.ws.close());
    }

    if (this.server) {
      promises.push(new Promise((resolve) => this.server.close(resolve)));
    }

    await Promise.all(promises);
  }

  public initApp() {
    this._app = new Koa();
  }

  public async createPreviewServer(options: any) {
    const { https, port, host = true, middlewares = [] } = options;
    const protocol = https ? 'https' : 'http';
    let hostname;
    if (typeof host !== 'boolean') {
      hostname = host === '0.0.0.0' ? 'localhost' : host;
    } else {
      hostname = 'localhost';
    }
    this.previewServerConfig = {
      ...options,
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

    this._context = {
      config: this.config,
      previewConfig: this.previewServerConfig,
      app: this._app,
      server: this.server,
      compiler: this.compiler,
      logger: this.logger,
      serverOptions: {}
    };
    this.resolvedPreviewServerMiddleware(middlewares);

    await this.startServer({ port, host });

    await this.printServerUrls(true);
  }

  public createServer(options: NormalizedServerConfig) {
    const { https, host = 'localhost', middlewares = [] } = options;
    const protocol = https ? 'https' : 'http';
    let hostname;
    if (typeof host !== 'boolean') {
      hostname = host === '0.0.0.0' ? 'localhost' : host;
    } else {
      hostname = 'localhost';
    }
    this.config = {
      ...options,
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

    this.hmrEngine = new HmrEngine(this.compiler, this, this.logger);
    this.ws = new WsServer(this.server, this.config, this.hmrEngine);

    // Note: path should be Farm's id, which is a relative path in dev mode,
    // but in vite, it's a url path like /xxx/xxx.js
    this.ws.on('vite:invalidate', ({ path, message }) => {
      // find hmr boundary starting from the parent of the file
      this.logger.info(`HMR invalidate: ${path}. ${message ?? ''}`);
      const parentFiles = this.compiler.getParentFiles(path);
      this.hmrEngine.hmrUpdate(parentFiles);
    });

    this._context = {
      config: this.config,
      previewConfig: this.previewServerConfig,
      app: this._app,
      server: this.server,
      compiler: this.compiler,
      logger: this.logger,
      serverOptions: {}
    };
    this.resolvedServerMiddleware(middlewares);
  }

  static async resolvePortConflict(
    normalizedDevConfig: NormalizedServerConfig,
    logger: Logger
  ): Promise<void> {
    let devPort = normalizedDevConfig.port;
    let hmrPort = DEFAULT_HMR_OPTIONS.port;
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
      normalizedDevConfig.hmr.port = ++hmrPort;
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

  _applyMiddlewares(internalMiddlewares?: any[]) {
    internalMiddlewares.forEach((middleware) => {
      const middlewareImpl = middleware(this);
      console.log(middlewareImpl);

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

  private resolvedPreviewServerMiddleware(
    middlewares?: DevServerMiddleware[]
  ): void {
    const internalMiddlewares = [
      ...(middlewares || []),
      compression,
      proxy,
      sirvMiddleware
    ];
    this._applyMiddlewares(internalMiddlewares);
  }

  private resolvedServerMiddleware(middlewares?: DevServerMiddleware[]): void {
    const internalMiddlewares = [
      ...(middlewares || []),
      headers,
      lazyCompilation,
      cors,
      resources,
      records,
      proxy
    ];
    this._applyMiddlewares(internalMiddlewares);
  }

  private async printServerUrls(showPreviewFlag = false) {
    const publicPath = this.compiler
      ? this.publicPath
      : this.previewServerConfig.output.publicPath;
    const config = this.compiler ? this.config : this.previewServerConfig;
    this._context.serverOptions.resolvedUrls = await resolveServerUrls(
      this.server,
      config,
      publicPath
    );
    if (this._context.serverOptions.resolvedUrls) {
      printServerUrls(
        this._context.serverOptions.resolvedUrls,
        this.logger,
        showPreviewFlag
      );
    } else {
      throw new Error('cannot print server URLs with Server Error.');
    }
  }
}
