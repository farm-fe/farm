import Koa from 'koa';

import { Compiler } from '../compiler';
import { resources } from './middlewares/resources';
import {
  DevServerOptions,
  NormalizedDevServerOptions,
  normalizeDevServerOptions,
} from './normalizeDevServerOptions';

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

  constructor(private _compiler: Compiler, options?: DevServerOptions) {
    this._options = normalizeDevServerOptions(options);
    this._app = new Koa();

    this._app.use(resources(this._compiler));
  }

  async listen(): Promise<void> {
    await this._compiler.compile();

    this._app.listen(this._options.port);

    console.log(`http://localhost:${this._options.port}`);
  }
}
