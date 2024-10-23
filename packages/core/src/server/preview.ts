import { OutgoingHttpHeaders, SecureServerOptions } from 'node:http2';
import connect from 'connect';
import { resolveConfig } from '../config/index.js';
import {
  FarmCliOptions,
  ResolvedUserConfig,
  UserConfig
} from '../config/types.js';
import { CommonServerOptions, httpServer } from './http.js';
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

    this.#resolveOptions();

    const [httpsOptions, publicFiles] = await Promise.all([
      this.resolveHttpsConfig(this.previewServerOptions.https),
      await initPublicFiles(this.resolvedUserConfig)
    ]);
    this.middlewares = connect();
    this.httpServer = this.previewServerOptions.middlewareMode
      ? null
      : await this.resolveHttpServer(
          this.previewServerOptions,
          this.middlewares,
          httpsOptions
        );
    // this.resolveHttpServer();
  }

  #resolveOptions() {
    const headers =
      this.resolvedUserConfig.preview?.headers ||
      this.resolvedUserConfig.server?.headers;
    const host =
      typeof this.resolvedUserConfig.preview.host === 'string'
        ? this.resolvedUserConfig.preview.host
        : 'localhost';
    const port = this.resolvedUserConfig.preview.port || 1911;
    // const middlewareMode = this.resolvedUserConfig.preview.middlewareMode || false;
    const https =
      this.resolvedUserConfig.preview?.https ||
      this.resolvedUserConfig.server?.https;
    this.previewServerOptions = {
      headers,
      host,
      port,
      https,
      middlewareMode: true
    };
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
    } catch (error) {
      throw error;
    }
  }
}
