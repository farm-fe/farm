import type * as http from 'node:http';
import type { Server } from 'node:http';
import type { OutgoingHttpHeaders as HttpServerHeaders } from 'node:http';
import { type Http2SecureServer } from 'node:http2';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import path from 'node:path';
import { WatchOptions } from 'chokidar';
import connect from 'connect';
import fse from 'fs-extra';
import { WebSocketServer as WebSocketServerRaw_ } from 'ws';
import { Compiler } from '../compiler/index.js';
import { normalizePublicPath } from '../config/normalize-config/normalize-output.js';
import { NormalizedServerConfig, ResolvedUserConfig } from '../config/types.js';
import { logError } from '../server/error.js';
import { logger } from '../utils/logger.js';
import { initPublicFiles } from '../utils/publicDir.js';
import { isObject } from '../utils/share.js';
import { FileWatcher } from '../watcher/index.js';
import { HMRChannel } from './hmr.js';
import {
  CommonServerOptions,
  resolveHttpServer,
  resolveHttpsConfig
} from './http.js';
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

function noop() {
  // noop
}

export class newServer {
  private compiler: CompilerType;

  ws: WsServer;
  config: ResolvedUserConfig;
  serverConfig: CommonServerOptions & NormalizedServerConfig;
  httpsOptions: HttpsServerOptions;
  publicDir?: string;
  publicPath?: string;
  httpServer?: HttpServer;
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
    this.httpServer = middlewareMode
      ? null
      : await resolveHttpServer(
          serverConfig as CommonServerOptions,
          middlewares,
          this.httpsOptions
        );

    const publicFiles = await initPublicFilesPromise;
    const { publicDir } = this.config.compilation.assets;
    this.createWebSocketServer();

    // middleware

    middlewares.use(
      resourceMiddleware(this.httpServer, this.compiler, this.publicPath)
    );

    middlewares.use((req, _res, next) => {
      console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);
      next();
    });

    // 定义一个响应中间件
    // middlewares.use((req, res, next) => {
    //   if (req.url === '/') {
    //     res.setHeader('Content-Type', 'text/plain; charset=utf-8');
    //     res.end('你好，这是 Connect 中间件！');
    //   } else {
    //     next();
    //   }
    // });

    // // 定义一个 404 处理中间件
    // middlewares.use((_req, res) => {
    //   res.statusCode = 404;
    //   res.setHeader('Content-Type', 'text/plain;charset=utf-8');
    //   res.end('404 - 页面未找到');
    // });
  }

  public async createWebSocketServer() {
    // @ts-ignore
    if (!this.httpServer) {
      throw new Error('Websocket requires a server.');
    }

    const wsServer = new WsServer(
      this.httpServer,
      this.config,
      this.httpsOptions,
      this.publicPath,
      null
    );
  }

  public async listen(): Promise<void> {
    if (!this.httpServer) {
      // this.logger.error('HTTP server is not created yet');
      return;
    }
    const { port, open, protocol, hostname } = this.config.server;

    await this.compile();
    // const { createServer } = await import('node:http');

    // this.httpServer = createServer((req, res) => {
    //   if (req.url === '/') {
    //     // res.writeHead(200, { 'Content-Type': 'text/plain' });
    //     // res.end('Hello, World!');
    //   } else if (req.url === '/about') {
    //     res.writeHead(200, { 'Content-Type': 'text/plain' });
    //     res.end('About page');
    //   } else {
    //     res.writeHead(404, { 'Content-Type': 'text/plain' });
    //     res.end('404 Not Found');
    //   }
    // });

    // this.httpServer.on('request', (req, res) => {
    //   // 设置响应头
    //   // res.writeHead(200, { 'Content-Type': 'application/json' });
    //   res.writeHead(200, { 'Content-Type': 'text/plain; charset=utf-8' });

    //   // 创建响应体对象
    //   const responseBody = {
    //     message: "这是使用 on('request') 方法的响应",
    //     timestamp: new Date().toISOString(),
    //     path: req.url
    //   };

    //   // 将对象转换为 JSON 字符串
    //   const jsonResponse = JSON.stringify(responseBody);
    //   // res.setHeader('Content-Type', 'text/plain; charset=utf-8');
    //   // 发送响应
    //   res.end(jsonResponse);
    // });

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
