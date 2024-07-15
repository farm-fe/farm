import type * as http from 'node:http';
import type { Server } from 'node:http';
import type { OutgoingHttpHeaders as HttpServerHeaders } from 'node:http';
import { type Http2SecureServer } from 'node:http2';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import path from 'node:path';
import { WatchOptions } from 'chokidar';
import connect from 'connect';
import fse from 'fs-extra';
import { Compiler } from '../compiler/index.js';
import { normalizePublicPath } from '../config/normalize-config/normalize-output.js';
import { NormalizedServerConfig, ResolvedUserConfig } from '../config/types.js';
import { logger } from '../utils/logger.js';
import { initPublicFiles } from '../utils/publicDir.js';
import { FileWatcher } from '../watcher/index.js';
import { HMRChannel } from './hmr.js';
import {
  CommonServerOptions,
  resolveHttpServer,
  resolveHttpsConfig
} from './http.js';
import { WsServer } from './ws.js';
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
export class newServer {
  private compiler: CompilerType;

  ws: WsServer;
  config: ResolvedUserConfig;
  serverConfig: CommonServerOptions & NormalizedServerConfig;
  httpsOptions: HttpsServerOptions;
  publicDir?: string;
  publicPath?: string;
  server?: HttpServer;
  watcher: FileWatcher;

  constructor(compiler: CompilerType, config: ResolvedUserConfig) {
    this.compiler = compiler;
    this.config = config;

    if (!this.compiler) return;

    this.publicPath =
      normalizePublicPath(
        compiler.config.config.output.targetEnv,
        compiler.config.config.output.publicPath,
        logger,
        false
      ) || '/';
  }

  getCompiler(): CompilerType {
    return this.compiler;
  }

  async createServer() {
    const initPublicFilesPromise = initPublicFiles(this.config);
    const { root, server: serverConfig } = this.config;
    this.httpsOptions = await resolveHttpsConfig(serverConfig.https);
    const { middlewareMode } = serverConfig;
    const middlewares = connect() as connect.Server;
    this.server = middlewareMode
      ? null
      : await resolveHttpServer(
          serverConfig as CommonServerOptions,
          middlewares,
          this.httpsOptions
        );

    const publicFiles = await initPublicFilesPromise;
    const { publicDir } = this.config.compilation.assets;
    this.createWebSocketServer();
  }

  public async createWebSocketServer() {
    if (!this.server) {
      throw new Error('Websocket requires a server.');
    }
    const wsServer = new WsServer(
      this.server,
      this.config,
      this.httpsOptions,
      null
    );
  }
}
