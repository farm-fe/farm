import { ServerOptions } from 'node:https';
import path from 'node:path';
import connect from 'connect';
import { resolveConfig } from '../config/index.js';
import { mergeConfig } from '../config/mergeConfig.js';
import {
  FarmCliOptions,
  UserConfig,
  UserPreviewServerConfig
} from '../config/types.js';
import { resolveServerUrls } from '../utils/http.js';
import { Logger, printServerUrls } from '../utils/logger.js';
import { CommonServerOptions, httpServer } from './http.js';

class PreviewServer extends httpServer {
  previewOptions: UserPreviewServerConfig;
  httpsOptions: ServerOptions;
  publicPath?: string;
  publicDir?: string;
  resolvedUserConfig: UserConfig;
  logger: Logger;
  middlewares: connect.Server;

  // TODO: add annotations
  constructor(readonly inlineConfig: FarmCliOptions & UserConfig) {
    super();
  }

  // TODO: move this method to base class,
  // this function is duplicated with [Server].
  protected async httpServerStart(serverOptions: {
    port: number;
    strictPort: boolean | undefined;
    host: string | undefined;
  }): Promise<number> {
    if (!this.httpServer) {
      throw new Error('httpServer is not initialized');
    }

    let { port, strictPort, host } = serverOptions;

    return new Promise((resolve, reject) => {
      const onError = (e: Error & { code?: string }) => {
        if (e.code === 'EADDRINUSE') {
          if (strictPort) {
            this.httpServer.removeListener('error', onError);
            reject(new Error(`Port ${port} is already in use`));
          } else {
            this.logger.info(`Port ${port} is in use, trying another one...`);
            this.httpServer.listen(++port, host);
          }
        } else {
          this.httpServer.removeListener('error', onError);
          reject(e);
        }
      };

      this.httpServer.on('error', onError);

      this.httpServer.listen(port, host, () => {
        this.httpServer.removeListener('error', onError);
        resolve(port);
      });
    });
  }
  async createServer() {
    this.resolvedUserConfig = await resolveConfig(
      this.inlineConfig,
      'preview',
      'production',
      'production'
    );
    this.logger = this.resolvedUserConfig.logger;

    this.#resolvePreviewOptions();

    this.httpsOptions = await this.resolveHttpsConfig(
      this.previewOptions.https
    );
    // this.publicFiles
    this.middlewares = connect();
    this.httpServer = this.previewOptions.middlewareMode
      ? null
      : await this.resolveHttpServer(
          this.previewOptions as CommonServerOptions,
          this.middlewares,
          this.httpsOptions
        );
  }

  async listen(): Promise<void> {
    if (!this.httpServer) {
      throw new Error('Server not created');
    }
    const { host, port, open, strictPort } = this.previewOptions;

    try {
      await this.httpServerStart({
        port,
        strictPort,
        // TODO: Check hosts (Why is it a boolean?)
        host: typeof host === 'string' ? host : undefined
      });

      await this.displayServerUrls();

      if (open) {
        // Run openBrowser function
        // await this.openBrowser(port);
      }
    } catch (e) {
      // this.logger.error(e);
      // process.exit(1);
    }
  }

  #resolvePreviewOptions() {
    this.previewOptions = mergeConfig(
      this.resolvedUserConfig.server,
      this.resolvedUserConfig.preview
    );

    const { root, compilation, server } = this.resolvedUserConfig;

    const relativePath =
      this.previewOptions.output.path || compilation.output.path;
    const distDir =
      this.previewOptions.distDir || path.resolve(root, relativePath);
    this.publicPath =
      this.previewOptions.output?.publicPath || compilation.output.publicPath;
    this.publicDir =
      this.previewOptions.distDir || compilation.assets.publicDir;
  }

  // TODO: maybe migrate to utils? This method is
  // duplicated with [Server] too.
  async displayServerUrls() {
    this.resolvedUrls = await resolveServerUrls(
      this.httpServer,
      this.previewOptions,
      this.publicPath
    );

    if (this.resolvedUrls) {
      printServerUrls(
        this.resolvedUrls,
        this.previewOptions.host,
        this.resolvedUserConfig.logger
      );
    } else {
      throw new Error(
        'cannot print server URLs before server.listen is called.'
      );
    }
  }
}

export { PreviewServer };
