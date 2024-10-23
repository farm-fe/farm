import { OutgoingHttpHeaders, SecureServerOptions } from 'node:http2';
import connect from 'connect';
import { resolveConfig } from '../config/index.js';
import {
  FarmCliOptions,
  ResolvedUserConfig,
  UserConfig
} from '../config/types.js';
import { resolveServerUrls } from '../utils/http.js';
import { printServerUrls } from '../utils/logger.js';
import { CommonServerOptions, httpServer } from './http.js';
import { htmlFallbackMiddleware } from './middlewares/htmlFallback.js';
import { publicMiddleware } from './middlewares/publicResource.js';
import { resourceMiddleware } from './middlewares/resource.js';
import { initPublicFiles } from './publicDir.js';

export interface PreviewServerOptions extends CommonServerOptions {
  headers: OutgoingHttpHeaders;
  host: string;
  port: number;
  https: SecureServerOptions;
  middlewareMode: boolean;
}

export class PreviewServer extends httpServer {
  resolvedUserConfig: ResolvedUserConfig;
  previewServerOptions: PreviewServerOptions;
  httpsOptions: SecureServerOptions;

  publicDir: string;
  publicPath: string;
  publicFiles: Set<string>;

  middlewares: connect.Server;

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

    this.middlewares = connect();
    this.httpServer = this.previewServerOptions.middlewareMode
      ? null
      : await this.resolveHttpServer(
          this.previewServerOptions,
          this.middlewares,
          this.httpsOptions
        );

    this.#initializeMiddlewares();
  }

  async #resolveOptions() {
    const {
      compilation: {
        output: { publicPath },
        assets: { publicDir }
      },
      preview,
      server
    } = this.resolvedUserConfig;
    this.publicPath = publicPath;
    // this.publicDir = publicDir;
    this.publicDir =
      '/home/fu050409/Desktop/Workspace/farm/examples/refactor-react/dist/';

    const headers = preview?.headers || server?.headers;
    const host = typeof preview.host === 'string' ? preview.host : 'localhost';
    const port = preview?.port || 1911;
    // const middlewareMode = this.resolvedUserConfig.preview.middlewareMode || false;
    const https = preview?.https || server?.https;
    this.previewServerOptions = {
      headers,
      host,
      port,
      https,
      middlewareMode: false
    };

    [this.httpsOptions, this.publicFiles] = await Promise.all([
      this.resolveHttpsConfig(this.previewServerOptions.https),
      await initPublicFiles(this.resolvedUserConfig)
    ]);
  }

  #initializeMiddlewares() {
    // if ()
    this.middlewares.use(publicMiddleware(this));
    console.log(this.publicPath);

    this.middlewares.use(htmlFallbackMiddleware(this));
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
        // TODO
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
