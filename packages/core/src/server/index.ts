import { existsSync, mkdirSync } from 'fs';

import Koa from 'koa';
import serve from 'koa-static';
import { WebSocketServer } from 'ws';

import { Compiler } from '../compiler/index.js';
import {
  UserServerConfig,
  NormalizedServerConfig,
  normalizeDevServerOptions,
} from '../config/index.js';
import { resources } from './middlewares/resources.js';
import { hmr } from './middlewares/hmr.js';
import { HmrEngine } from './hmr-engine.js';

/**
 * Farm Dev Server, responsible of:
 * * parse and normalize dev server options
 * * launch http server based on options
 * * compile the project in dev mode and serve the production
 * * HMR middleware and websocket supported
 */
export class DevServer {
  private _app: Koa;
  private _dist: string;

  ws: WebSocketServer;
  config: NormalizedServerConfig;
  hmrEngine?: HmrEngine;

  constructor(private _compiler: Compiler, options?: UserServerConfig) {
    this.config = normalizeDevServerOptions(options);
    this._app = new Koa();
    this._dist = this._compiler.config.config.output.path as string;

    if (!existsSync(this._dist)) {
      mkdirSync(this._dist, { recursive: true });
    }

    if (this.config.writeToDisk) {
      this._app.use(serve(this._dist));
    } else {
      this._app.use(resources(this._compiler));
    }

    if (this.config.hmr) {
      this.ws = new WebSocketServer({
        port: this.config.hmr.port,
        host: this.config.hmr.host,
      });
      this._app.use(hmr(this));
      this.hmrEngine = new HmrEngine(this._compiler, this);
    }
  }

  async listen(): Promise<void> {
    // compile the project and start the dev server
    await this._compiler.compile();

    if (this.config.writeToDisk) {
      this._compiler.writeResourcesToDisk();
    }

    this._app.listen(this.config.port);

    console.log(`http://localhost:${this.config.port}`);
  }
}
