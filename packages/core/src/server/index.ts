import { readFileSync } from 'node:fs';
import http from 'node:http';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import Koa from 'koa';
import { WebSocketServer } from 'ws';
import chalk from 'chalk';
import boxen from 'boxen';
import figlet from 'figlet';
import { Compiler } from '../compiler/index.js';
import {
  UserServerConfig,
  NormalizedServerConfig,
  normalizeDevServerOptions,
  normalizePublicDir,
  DevServerPlugin,
  UserConfig,
  DEFAULT_HMR_OPTIONS
} from '../config/index.js';
import { HmrEngine } from './hmr-engine.js';
import { brandColor, Logger } from '../utils/index.js';
import { lazyCompilationPlugin } from './middlewares/lazy-compilation.js';
import { resourcesPlugin } from './middlewares/resources.js';
import { hmrPlugin } from './middlewares/hmr.js';
import { proxyPlugin } from './middlewares/proxy.js';
import { corsPlugin } from './middlewares/cors.js';
import { openBrowser } from './openBrowser.js';
import { recordsPlugin } from './middlewares/records.js';

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
}

interface ImplDevServer {
  createFarmServer(options: UserServerConfig): void;
  listen(): Promise<void>;
  close(): Promise<void>;
  getCompiler(): Compiler;
}

export class DevServer implements ImplDevServer {
  private _app: Koa;
  public _context: FarmServerContext;

  ws: WebSocketServer;
  config: NormalizedServerConfig;
  hmrEngine?: HmrEngine;
  server?: http.Server;
  publicPath?: string;

  constructor(
    private _compiler: Compiler,
    public logger: Logger,
    options?: UserServerConfig,
    publicPath?: string
  ) {
    this.publicPath = normalizePublicDir(
      _compiler.config.config.root,
      publicPath
    );
    this.createFarmServer(options);
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
    }
    const { port, open, protocol, hostname } = this.config;
    const start = Date.now();
    // compile the project and start the dev server
    if (process.env.FARM_PROFILE) {
      this._compiler.compileSync();
    } else {
      await this._compiler.compile();
    }
    const end = Date.now();
    this.server.listen(port);
    this.startDevLogger(start, end);
    if (open) {
      openBrowser(`${protocol}://${hostname}:${port}`);
    }
  }

  async close() {
    if (!this.server) {
      this.logger.error('HTTP server is not created yet');
    }
    this.closeFarmServer();
  }

  async restart() {
    // TODO restart
  }

  closeFarmServer() {
    this.server?.close(() => {
      this.logger.info('HTTP server is closed');
    });
  }

  createFarmServer(options: UserServerConfig) {
    const { https = false, host = 'localhost', plugins = [] } = options;
    const protocol = https ? 'https' : 'http';
    const hostname = host === '0.0.0.0' ? 'localhost' : host;
    this.config = normalizeDevServerOptions(
      { ...options, protocol, hostname },
      'development'
    );
    this._app = new Koa();
    this.server = http.createServer(this._app.callback());
    this._context = {
      config: this.config,
      app: this._app,
      server: this.server,
      compiler: this._compiler,
      logger: this.logger
    };
    this.resolvedFarmServerPlugins(plugins);
  }

  /**
   *
   * Add listening files for root manually
   *
   * > listening file with root must as file.
   *
   * @param root
   * @param deps
   */
  addWatchFile(root: string, deps: string[]) {
    this.getCompiler().addExtraWatchFile(root, deps);
  }

  private resolvedFarmServerPlugins(middlewares?: DevServerPlugin[]) {
    const resolvedPlugins = [
      ...middlewares,
      lazyCompilationPlugin,
      hmrPlugin,
      corsPlugin,
      resourcesPlugin,
      recordsPlugin,
      proxyPlugin
    ];
    // this._app.use(serve(this._dist));
    resolvedPlugins.forEach((plugin) => plugin(this));
  }

  private startDevLogger(start: number, end: number) {
    const { port, protocol, hostname } = this.config;
    const version = JSON.parse(
      readFileSync(
        join(fileURLToPath(import.meta.url), '../../../package.json'),
        'utf-8'
      )
    ).version;
    this.logger.info(
      boxen(
        `${brandColor(
          figlet.textSync('FARM', {
            width: 40
          })
        )}
  Version ${chalk.green.bold(version)}

  🔥 Ready on ${chalk.green.bold(
    `${protocol}://${hostname}:${port}`
  )} in ${chalk.green.bold(`${end - start}ms`)}.
    `,
        {
          padding: 1,
          margin: 1,
          align: 'center',
          borderColor: 'cyan',
          borderStyle: 'round'
        }
      ),
      false
    );
  }
}

export async function resolvePortConflict(
  config: UserConfig,
  logger: Logger
): Promise<{ updatedConfig: UserConfig }> {
  let hmrPort = DEFAULT_HMR_OPTIONS.port;
  const normalizedDevConfig = normalizeDevServerOptions(
    config.server,
    'development'
  );
  let devPort = normalizedDevConfig.port;

  const server = http.createServer();

  return new Promise((resolve, reject) => {
    // attach listener to the server to listen for port conflict
    const onError = (e: Error & { code?: string }) => {
      if (e.code === 'EADDRINUSE') {
        // TODO: if strictPort, throw Error(`Port ${port} is already in use`))
        logger.info(`Port ${devPort} is in use, trying another one...`);
        // update hmrPort and devPort
        config.server.hmr = { port: ++hmrPort };
        config.server.port = ++devPort;
        server.listen(devPort);
      } else {
        server.removeListener('error', onError);
        reject(e);
      }
    };

    server.on('error', onError);
    server.listen(devPort, () => {
      // logger.info(`Port Available at: http://localhost:${devPort}`);
      server.close();
      const result = { updatedConfig: config };
      resolve(result);
    });
  });
}
