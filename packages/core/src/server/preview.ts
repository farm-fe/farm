import { existsSync } from 'node:fs';
import { OutgoingHttpHeaders, SecureServerOptions } from 'node:http2';
import type * as net from 'node:net';
import path from 'node:path';
import compression from '@polka/compression';
import connect from 'connect';
import corsMiddleware from 'cors';
import sirv, { RequestHandler } from 'sirv';
import { PreviewServerMiddleware, resolveConfig } from '../config/index.js';
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
import { getValidPublicPath, isObject, version } from '../utils/share.js';
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
  proxy?: Record<string, string | ProxyOptions>;
  middlewares?: PreviewServerMiddleware[];
}

/**
 * Represents a Farm preview server.
 * @class
 */
export class PreviewServer extends httpServer {
  config!: ResolvedUserConfig;
  previewServerOptions!: PreviewServerOptions;
  httpsOptions: SecureServerOptions | undefined;

  publicPath!: string;
  httpServer!: HttpServer;

  middlewares!: connect.Server;
  serve!: RequestHandler;
  closeHttpServerFn!: () => Promise<void>;
  terminateServerFn!: () => Promise<void>;

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
    this.config = await resolveConfig(
      this.inlineConfig,
      'preview',
      'production'
    );

    this.logger = this.config.logger ?? this.logger;

    await this.#resolveOptions();

    this.middlewares = connect();
    this.httpServer = await this.resolveHttpServer(
      this.previewServerOptions,
      this.middlewares,
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
    const { cors, proxy, middlewares } = this.previewServerOptions;
    const { appType, middlewareMode } = this.config.server ?? {};

    if (cors !== false) {
      this.middlewares.use(
        corsMiddleware(typeof cors === 'boolean' ? {} : cors)
      );
    }

    if (proxy) {
      const middlewareServer =
        isObject(middlewareMode) && 'server' in middlewareMode
          ? middlewareMode.server
          : this.httpServer;
      this.middlewares.use(
        proxyMiddleware(this, middlewareServer as HttpServer, proxy)
      );
    }

    this.middlewares.use(compression());

    if (this.publicPath !== '/') {
      this.middlewares.use(publicPathMiddleware(this, false));
    }

    this.middlewares.use(this.serve);

    middlewares?.forEach((middleware) => {
      const mw = middleware(this);
      if (mw) this.middlewares.use(mw);
    });

    if (appType === 'spa' || appType === 'mpa') {
      this.middlewares.use(notFoundMiddleware());
    }
  }

  /**
   * Resolve preview server options
   *
   * @private
   * @returns {Promise<void>}
   */
  async #resolveOptions(): Promise<void> {
    const { server, compilation } = this.config;

    this.publicPath = getValidPublicPath(
      compilation?.output?.publicPath ?? '/'
    );
    const preview = server?.preview;

    const distPath = preview?.distDir || compilation?.output?.path || 'dist';
    const distDir = path.isAbsolute(distPath)
      ? distPath
      : path.resolve(compilation?.root ?? '', distPath);

    if (!existsSync(distDir)) {
      throw new Error(
        `Dist directory "${distDir}" does not exist. Do you mean "farm build"?`
      );
    }

    const headers = (preview?.headers ?? server?.headers) || {};
    this.serve = sirv(distDir, {
      etag: true,
      dev: true,
      single: this.config.server?.appType === 'spa',
      ignores: false,
      setHeaders: (res, pathname) => {
        if (knownJavascriptExtensionRE.test(pathname)) {
          res.setHeader('Content-Type', 'text/javascript');
        }
        if (headers) {
          for (const name in headers) {
            res.setHeader(
              name,
              headers[name] as string | number | readonly string[]
            );
          }
        }
      }
    });

    this.httpsOptions =
      (await this.resolveHttpsConfig(preview?.https ?? server?.https)) ??
      undefined;

    this.previewServerOptions = {
      headers,
      host: typeof preview?.host === 'string' ? preview.host : 'localhost',
      port: preview?.port ?? 1911,
      strictPort: preview?.strictPort ?? false,
      https: this.httpsOptions ?? ({} as Record<string, never>),
      distDir,
      open: preview?.open ?? false,
      cors: preview?.cors ?? false,
      proxy: preview?.proxy ?? server?.proxy,
      middlewares: preview?.middlewares
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
        this.config,
        'preview'
      );

      if (this.config.configFilePath) {
        const shortFile = getShortName(
          this.config.configFilePath,
          this.config.root ?? ''
        );
        this.logger.info(`Using config file at ${bold(green(shortFile))}`);
      }

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
    } catch (error: any) {
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
