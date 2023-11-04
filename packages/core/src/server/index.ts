import http from 'node:http';
import Koa from 'koa';

import { Compiler } from '../compiler/index.js';
import {
  DEFAULT_HMR_OPTIONS,
  DevServerPlugin,
  normalizeDevServerOptions,
  NormalizedServerConfig,
  normalizePublicDir,
  normalizePublicPath,
  urlRegex,
  UserConfig,
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
  corsPlugin,
  headersPlugin,
  hmrPlugin,
  lazyCompilationPlugin,
  proxyPlugin,
  recordsPlugin,
  resourcesPlugin
} from './middlewares/index.js';
import { __FARM_GLOBAL__ } from '../config/_global.js';
import { resolveServerUrls } from '../utils/http.js';
import WsServer from './ws.js';

/**
 * Farm Dev Server, responsible for:
 * * parse and normalize dev server options
 * * launch http server based on options
 * * compile the project in dev mode and serve the production
 * * HMR middleware and websocket supported
 */

interface FarmServerContext {
  config: UserServerConfig;
  app: Koa;
  server: http.Server;
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
  createFarmServer(options: UserServerConfig): void;
  listen(): Promise<void>;
  close(): Promise<void>;
  getCompiler(): Compiler;
}

export class DevServer implements ImplDevServer {
  private _app: Koa;
  public _context: FarmServerContext;

  ws: WsServer;
  config: NormalizedServerConfig;
  hmrEngine?: HmrEngine;
  server?: http.Server;
  publicDir?: string;
  publicPath?: string;
  userConfig?: UserConfig;

  constructor(
    private _compiler: Compiler,
    public logger: Logger,
    options?: UserConfig
  ) {
    this.publicDir = normalizePublicDir(
      _compiler.config.config.root,
      options.publicDir
    );

    this.publicPath =
      normalizePublicPath(
        options?.compilation?.output?.publicPath,
        logger,
        false
      ) || '/';

    this.userConfig = options;
    this.createFarmServer(options.server);
  }

  getCompiler(): Compiler {
    return this._compiler;
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
    this.getNormalizedPublicPath();

    const start = Date.now();
    // compile the project and start the dev server
    await this.compile();

    bootstrap(Date.now() - start);

    await this.startServer(this.config);

    __FARM_GLOBAL__.__FARM_RESTART_DEV_SERVER__ && this.printServerUrls();

    if (open) {
      openBrowser(`${protocol}://${hostname}:${port}${this.publicPath}`);
    }
  }

  private async compile(): Promise<void> {
    if (process.env.FARM_PROFILE) {
      this._compiler.compileSync();
    } else {
      await this._compiler.compile();
    }

    if (this.config.writeToDisk) {
      const base = this.publicPath.match(/^https?:\/\//) ? '' : this.publicPath;
      this._compiler.writeResourcesToDisk(base);
    }
  }

  private getNormalizedPublicPath(): string {
    if (urlRegex.test(this.publicPath)) {
      return '/';
    } else {
      return this.publicPath.startsWith('/')
        ? this.publicPath
        : `/${this.publicPath}`;
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

  async restart() {
    // TODO restart
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

  public createFarmServer(options: UserServerConfig) {
    const { https = false, host = 'localhost', plugins = [] } = options;
    const protocol = https ? 'https' : 'http';
    let hostname;
    if (typeof host !== 'boolean') {
      hostname = host === '0.0.0.0' ? 'localhost' : host;
    } else {
      hostname = 'localhost';
    }
    this.config = normalizeDevServerOptions(
      { ...options, protocol, hostname },
      'development'
    );

    this._app = new Koa();
    this.server = http.createServer(this._app.callback());
    this.ws = new WsServer(this.server, this.config, true);

    this._context = {
      config: this.config,
      app: this._app,
      server: this.server,
      compiler: this._compiler,
      logger: this.logger,
      serverOptions: {}
    };
    this.resolvedFarmServerPlugins(plugins);
  }

  static async resolvePortConflict(
    userConfig: UserConfig,
    logger: Logger
  ): Promise<void> {
    const normalizedDevConfig = normalizeDevServerOptions(
      userConfig.server,
      'development'
    );
    userConfig.server = normalizedDevConfig;

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
      userConfig.server.hmr = { port: ++hmrPort };
      userConfig.server.port = ++devPort;
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

  private resolvedFarmServerPlugins(middlewares?: DevServerPlugin[]): void {
    const resolvedPlugins = [
      ...(middlewares || []),
      headersPlugin,
      lazyCompilationPlugin,
      hmrPlugin,
      corsPlugin,
      resourcesPlugin,
      recordsPlugin,
      proxyPlugin
    ];

    resolvedPlugins.forEach((plugin) => plugin(this));
  }

  private async printServerUrls() {
    this._context.serverOptions.resolvedUrls = await resolveServerUrls(
      this.server,
      this.config,
      this.userConfig
    );
    if (this._context.serverOptions.resolvedUrls) {
      printServerUrls(this._context.serverOptions.resolvedUrls, this.logger);
    } else {
      throw new Error('cannot print server URLs with Server Error.');
    }
  }
}
