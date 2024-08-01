import type * as http from 'node:http';
import type { Server } from 'node:http';
import type { OutgoingHttpHeaders as HttpServerHeaders } from 'node:http';
import { type Http2SecureServer } from 'node:http2';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import path from 'node:path';
import { WatchOptions } from 'chokidar';
import compression from 'compression';
import connect from 'connect';
import fse from 'fs-extra';
import { WebSocketServer as WebSocketServerRaw_ } from 'ws';
import { Compiler } from '../compiler/index.js';
import { normalizePublicPath } from '../config/normalize-config/normalize-output.js';
import { NormalizedServerConfig, ResolvedUserConfig } from '../config/types.js';
import { logError } from '../server/error.js';
import { Logger, bootstrap, logger } from '../utils/logger.js';
import { initPublicFiles } from '../utils/publicDir.js';
import { isObject } from '../utils/share.js';
import { FileWatcher } from '../watcher/index.js';
import { HmrEngine } from './hmr-engine.js';
import { HMRChannel } from './hmr.js';
import {
  CommonServerOptions,
  resolveHttpServer,
  resolveHttpsConfig
} from './http.js';
import { htmlFallbackMiddleware } from './middlewares/htmlFallback.js';
import { publicMiddleware } from './middlewares/public.js';
import { resourceMiddleware } from './middlewares/resource.js';
import { WebSocketClient, WebSocketServer, WsServer } from './ws.js';
export type HttpServer = http.Server | Http2SecureServer;

type CompilerType = Compiler | null;

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
  watchOptions?: WatchOptions | null;
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

export class newServer {
  ws: any;
  serverOptions: CommonServerOptions & NormalizedServerConfig;
  httpsOptions: HttpsServerOptions;
  // public assets directory
  publicDir?: string | boolean;
  // base path of server
  publicPath?: string;
  // publicFile
  publicFiles?: string[];
  httpServer?: HttpServer;
  watcher: FileWatcher;
  hmrEngine?: HmrEngine;

  constructor(
    private readonly compiler: CompilerType,
    private readonly config: ResolvedUserConfig,
    private readonly logger: Logger
  ) {
    if (!this.compiler) {
      this.logger.error(
        'Compiler is not provided, server will not work, please provide a compiler e.q. `new Compiler(config)`'
      );
      return;
    }
    this.resolveOptions(config);
  }

  public getCompiler(): CompilerType {
    return this.compiler;
  }

  private resolveOptions(config: ResolvedUserConfig) {
    const { targetEnv, publicPath } = config.compilation.output;
    this.publicDir = config.compilation.assets.publicDir;
    this.publicPath =
      normalizePublicPath(targetEnv, publicPath, this.logger, false) || '/';

    this.serverOptions = config.server as CommonServerOptions &
      NormalizedServerConfig;
  }

  public async createServer() {
    this.httpsOptions = await resolveHttpsConfig(this.serverOptions.https);
    const publicFiles = await this.handlePublicFiles();
    const { middlewareMode } = this.serverOptions;
    const middlewares = connect() as connect.Server;
    this.httpServer = middlewareMode
      ? null
      : await resolveHttpServer(
          this.serverOptions as CommonServerOptions,
          middlewares,
          this.httpsOptions
        );

    this.createWebSocketServer();
    this.hmrEngine = new HmrEngine(
      this.compiler,
      this.httpServer,
      this.config,
      this.ws,
      this.logger
    );

    // middleware
    // middlewares.use(compression());

    if (this.publicDir) {
      middlewares.use(publicMiddleware(this.logger, this.config, publicFiles));
    }
    // TODO todo add appType
    middlewares.use(
      htmlFallbackMiddleware(
        this.httpServer,
        this.compiler,
        this.publicPath,
        this.config
      )
    );

    middlewares.use(
      resourceMiddleware(
        this.httpServer,
        this.compiler,
        this.publicPath,
        this.config
      )
    );
  }

  private async handlePublicFiles() {
    const initPublicFilesPromise = initPublicFiles(this.config);
    return await initPublicFilesPromise;
  }

  public async createWebSocketServer() {
    if (!this.httpServer) {
      throw new Error(
        'Websocket requires a http server. please check the server is be created'
      );
    }

    this.ws = new WsServer(
      this.httpServer,
      this.config,
      this.httpsOptions,
      this.publicPath,
      null
    );
  }

  public async listen(): Promise<void> {
    if (!this.httpServer) {
      this.logger.warn('HTTP server is not created yet');
      return;
    }
    const { port, open, protocol, hostname } = this.config.server;

    const start = Date.now();
    await this.compile();
    bootstrap(Date.now() - start, this.compiler.config);

    this.httpServer.listen(port, hostname.name, () => {
      console.log(`Server running at ${protocol}://${hostname.name}:${port}/`);
    });
  }

  private async compile(): Promise<void> {
    try {
      await this.compiler.compile();
    } catch (err) {
      throw new Error(logError(err) as unknown as string);
    }

    if (this.config.server.writeToDisk) {
      this.compiler.writeResourcesToDisk();
    } else {
      this.compiler.callWriteResourcesHook();
    }
  }
}
