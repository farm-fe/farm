import { existsSync, mkdirSync } from 'fs';

import Koa from 'koa';
import serve from 'koa-static';

import { Compiler } from '../compiler/index.js';
import { resources } from './middlewares/resources.js';
import {
  DevServerOptions,
  NormalizedDevServerOptions,
  normalizeDevServerOptions,
} from './normalizeDevServerOptions.js';

/**
 * Farm Dev Server, responsible of:
 * * parse and normalize dev server options
 * * launch http server based on options
 * * compile the project in dev mode and serve the production
 * * HMR middleware and websocket supported
 */
export class DevServer {
  private _options: NormalizedDevServerOptions;
  private _app: Koa;
  private _dist: string;

  constructor(private _compiler: Compiler, options?: DevServerOptions) {
    this._options = normalizeDevServerOptions(options);
    this._app = new Koa();
    this._dist = this._compiler.config.config.output.path as string;

    if (!existsSync(this._dist)) {
      mkdirSync(this._dist, { recursive: true });
    }

    if (this._options.writeToDisk) {
      this._app.use(serve(this._dist));
    } else {
      this._app.use(resources(this._compiler));
    }
  }

  async listen(): Promise<void> {
    await this._compiler.compile();

    if (this._options.writeToDisk) {
      this._compiler.writeResourcesToDisk();
    }

    this._app.listen(this._options.port);

    console.log(`http://localhost:${this._options.port}`);
  }
}
