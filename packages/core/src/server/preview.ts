import { OutgoingHttpHeaders, SecureServerOptions } from 'node:http2';
import path from 'node:path';
import connect from 'connect';
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
import { httpServer } from './http.js';

export interface PreviewServerOptions {
  headers: OutgoingHttpHeaders;
  host: string;
  port: number;
  strictPort: boolean;
  https: SecureServerOptions;
  distDir: string;
  open: boolean | string;
  cors: boolean;

  root: string;
}

export class PreviewServer extends httpServer {
  resolvedUserConfig: ResolvedUserConfig;
  previewServerOptions: PreviewServerOptions;
  httpsOptions: SecureServerOptions;

  app: connect.Server;
  serve: RequestHandler;

  constructor(readonly inlineConfig: FarmCliOptions & UserConfig) {
    super();
  }

  async createPreviewServer() {
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

    this.app.use(this.serve);
  }

  async #resolveOptions() {
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

    this.previewServerOptions = {
      headers,
      host: typeof preview.host === 'string' ? preview.host : 'localhost',
      port: preview?.port || 1911,
      strictPort: preview?.strictPort || false,
      https: preview?.https || server?.https,
      distDir,
      open: preview?.open || false,
      cors: preview?.cors || false,
      root
    };

    this.httpsOptions = await this.resolveHttpsConfig(
      this.previewServerOptions.https
    );
  }

  async listen() {
    if (!this.httpServer) {
      this.logger.error(
        'HTTP server is not created yet, this is most likely a farm internal error.'
      );
      return;
    }

    try {
      await this.httpServerStart({
        port: this.previewServerOptions.port,
        strictPort: true,
        host: this.previewServerOptions.host
      });

      this.resolvedUrls = await resolveServerUrls(
        this.httpServer,
        this.resolvedUserConfig
      );

      printServerUrls(
        this.resolvedUrls,
        this.previewServerOptions.host,
        this.logger
      );
    } catch (error) {
      throw error;
    }
  }
}
