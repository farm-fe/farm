import { existsSync } from 'node:fs';
import { OutgoingHttpHeaders, SecureServerOptions } from 'node:http2';
import type * as net from 'node:net';
import path from 'node:path';
import compression from '@polka/compression';
import connect from 'connect';
import corsMiddleware from 'cors';
import sirv, { RequestHandler } from 'sirv';
import { resolveConfig } from '../config/index.js';
import {
  FarmCliOptions,
  ResolvedUserConfig,
  UserConfig
} from '../config/types.js';
import { bold, brandColor, green } from '../utils/color.js';
import {
  resolveServerUrls,
  setupSIGTERMListener,
  teardownSIGTERMListener
} from '../utils/http.js';
import { printServerUrls } from '../utils/logger.js';
import { getShortName } from '../utils/path.js';
import { isObject, version } from '../utils/share.js';
import { knownJavascriptExtensionRE } from '../utils/url.js';
import {
  type CommonServerOptions,
  type CorsOptions,
  httpServer
} from './http.js';
import type { HttpServer } from './index.js';
import { notFoundMiddleware } from './middlewares/notFound.js';
import { type ProxyOptions, proxyMiddleware } from './middlewares/proxy.js';
import { publicPathMiddleware } from './middlewares/publicPath.js';
import { openBrowser } from './open.js';

export interface PreviewServerOptions extends CommonServerOptions {
  headers: OutgoingHttpHeaders;
  host: string;
  port: number;
  strictPort: boolean;
  https: SecureServerOptions;
  distDir: string;
  open: boolean | string;
  cors: boolean | CorsOptions;
  proxy: Record<string, string | ProxyOptions>;
}

/**
 * Represents a Farm preview server.
 * @class
 */
export class PreviewServer extends httpServer {
  resolvedUserConfig: ResolvedUserConfig;
  previewServerOptions: PreviewServerOptions;
  httpsOptions: SecureServerOptions;

  publicPath: string;
  httpServer: HttpServer;

  app: connect.Server;
  serve: RequestHandler;
  closeHttpServerFn: () => Promise<void>;
  terminateServerFn: () => Promise<void>;

  /**
   * Creates an instance of PreviewServer.
   * @param {FarmCliOptions & UserConfig} inlineConfig - The inline configuration options.
   */
  constructor(readonly inlineConfig: FarmCliOptions & UserConfig) {
    super();
  }

  /**
   * Creates and initializes the preview server.
   *
   * @returns {Promise<void>} A promise that resolves when the server is ready.
   * @throws {Error} If the server cannot be started.
   */
  async createPreviewServer(): Promise<void> {
    this.resolvedUserConfig = await resolveConfig(
      this.inlineConfig,
      'preview',
      'production',
      'production',
      true
    );

    this.logger = this.resolvedUserConfig.logger;

    await this.#resolveOptions();

    this.app = connect();
    this.httpServer = await this.resolveHttpServer(
      this.previewServerOptions,
      this.app,
      this.httpsOptions
    );

    this.#initializeMiddlewares();

    this.closeHttpServerFn = this.closeHttpServer();
    this.terminateServerFn = async () => {
      try {
        await this.closeHttpServerFn();
      } finally {
        process.exit(0);
      }
    };
    setupSIGTERMListener(this.terminateServerFn);
  }

  /**
   * Initialize middlewares for the preview server.
   * @private
   */
  #initializeMiddlewares() {
    const { cors, proxy } = this.previewServerOptions;
    const { appType, middlewareMode } = this.resolvedUserConfig.server;

    if (cors !== false) {
      this.app.use(corsMiddleware(typeof cors === 'boolean' ? {} : cors));
    }

    if (proxy) {
      const middlewareServer =
        isObject(middlewareMode) && 'server' in middlewareMode
          ? middlewareMode.server
          : this.httpServer;
      this.app.use(
        proxyMiddleware(this, middlewareServer as HttpServer, proxy)
      );
    }

    this.app.use(compression());

    if (this.publicPath !== '/') {
      this.app.use(publicPathMiddleware(this, false));
    }

    this.app.use(this.serve);

    if (appType === 'spa' || appType === 'mpa') {
      this.app.use(notFoundMiddleware());
    }
  }

  /**
   * Resolve preview server options
   *
   * @private
   * @returns {Promise<void>}
   */
  async #resolveOptions(): Promise<void> {
    const {
      server,
      compilation: { root, output }
    } = this.resolvedUserConfig;

    this.publicPath = output.publicPath ?? '/';
    const preview = server?.preview;

    const distPath = preview?.distDir || output?.path || 'dist';
    const distDir = path.isAbsolute(distPath)
      ? distPath
      : path.resolve(root, distPath);

    if (!existsSync(distDir)) {
      throw new Error(
        `Dist directory "${distDir}" does not exist. Do you mean "farm build"?`
      );
    }

    const headers = (preview?.headers ?? server?.headers) || {};
    this.serve = sirv(distDir, {
      etag: true,
      dev: true,
      single: this.resolvedUserConfig.server.appType === 'spa',
      ignores: false,
      setHeaders: (res, pathname) => {
        if (knownJavascriptExtensionRE.test(pathname)) {
          res.setHeader('Content-Type', 'text/javascript');
        }
        if (headers) {
          for (const name in headers) {
            res.setHeader(name, headers[name]);
          }
        }
      }
    });

    this.httpsOptions = await this.resolveHttpsConfig(
      preview?.https ?? server?.https
    );

    this.previewServerOptions = {
      headers,
      host: typeof preview.host === 'string' ? preview.host : 'localhost',
      port: preview?.port ?? 1911,
      strictPort: preview?.strictPort ?? false,
      https: this.httpsOptions,
      distDir,
      open: preview?.open ?? false,
      cors: preview?.cors ?? false,
      proxy: preview?.proxy ?? server?.proxy
    };
  }

  /**
   * Start the preview server.
   *
   * @returns {Promise<void>}
   * @throws {Error} If there's an error starting the server.
   */
  async listen(): Promise<void> {
    if (!this.httpServer) {
      this.logger.error(
        'HTTP server is not created yet, this is most likely a farm internal error.'
      );
      return;
    }

    try {
      await this.httpServerStart(this.previewServerOptions);

      this.resolvedUrls = await resolveServerUrls(
        this.httpServer,
        this.resolvedUserConfig,
        'preview'
      );

      const shortFile = getShortName(
        this.resolvedUserConfig.configFilePath,
        this.resolvedUserConfig.root
      );
      this.logger.info(`Using config file at ${bold(green(shortFile))}`);

      console.log('\n', bold(brandColor(`${'ϟ'}  Farm  v${version}`)), '\n');

      printServerUrls(
        this.resolvedUrls,
        this.previewServerOptions.host,
        this.logger
      );

      if (this.previewServerOptions.open) {
        const url =
          this.resolvedUrls?.local?.[0] ??
          this.resolvedUrls?.network?.[0] ??
          '';
        openBrowser(url);
      }
    } catch (error) {
      throw error;
    }
  }

  /**
   * Close the HTTP server gracefully.
   *
   * @returns {() => Promise<void>} A function that can be called to close the server.
   */
  closeHttpServer(): () => Promise<void> {
    if (!this.httpServer) {
      return () => Promise.resolve();
    }

    let hasListened = false;
    const openSockets = new Set<net.Socket>();

    this.httpServer.on('connection', (socket) => {
      openSockets.add(socket);
      socket.on('close', () => {
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

  /**
   * Close the preview server.
   *
   * @returns {Promise<void>}
   */
  async close(): Promise<void> {
    teardownSIGTERMListener(this.terminateServerFn);
    await this.closeHttpServerFn();
  }
}
