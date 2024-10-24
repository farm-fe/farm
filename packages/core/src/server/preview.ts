import { OutgoingHttpHeaders, SecureServerOptions } from 'node:http2';
import path from 'node:path';
import connect from 'connect';
import corsMiddleware from 'cors';
import sirv, { RequestHandler } from 'sirv';
import { resolveConfig } from '../config/index.js';
import {
  FarmCliOptions,
  ResolvedUserConfig,
  UserConfig
} from '../config/types.js';
import { resolveServerUrls } from '../utils/http.js';
import { printServerUrls } from '../utils/logger.js';
import { knownJavascriptExtensionRE } from '../utils/url.js';
import { CorsOptions, httpServer } from './http.js';
import { openBrowser } from './open.js';

export interface PreviewServerOptions {
  headers: OutgoingHttpHeaders;
  host: string;
  port: number;
  strictPort: boolean;
  https: SecureServerOptions;
  distDir: string;
  open: boolean | string;
  cors: boolean | CorsOptions;
}

/**
 * Represents a Farm preview server.
 * @class
 */
export class PreviewServer extends httpServer {
  resolvedUserConfig: ResolvedUserConfig;
  previewServerOptions: PreviewServerOptions;
  httpsOptions: SecureServerOptions;

  app: connect.Server;
  serve: RequestHandler;

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
      'production'
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
  }

  /**
   * Initialize middlewares for the preview server.
   * @private
   */
  #initializeMiddlewares() {
    const cors = this.previewServerOptions.cors;
    if (cors) {
      this.app.use(corsMiddleware(typeof cors === 'boolean' ? {} : cors));
    }
    this.app.use(this.serve);
  }

  /**
   * Resolve preview server options
   *
   * @private
   * @returns {Promise<void>}
   */
  async #resolveOptions(): Promise<void> {
    const {
      preview,
      server,
      compilation: { root, output }
    } = this.resolvedUserConfig;

    const distDir =
      preview?.distDir || path.isAbsolute(output?.path)
        ? output?.path
        : path.resolve(root, output?.path || 'dist');

    const headers = preview?.headers || server?.headers;
    this.serve = sirv(distDir, {
      etag: true,
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
      this.previewServerOptions.https
    );

    this.previewServerOptions = {
      headers,
      host: typeof preview.host === 'string' ? preview.host : 'localhost',
      port: preview?.port || 1911,
      strictPort: preview?.strictPort || false,
      https: this.httpsOptions,
      distDir,
      open: preview?.open || false,
      cors: preview?.cors || false
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
   * Close the preview server.
   *
   * @returns {void}
   */
  close(): void {
    this.httpServer && this.httpServer.close();
  }
}
