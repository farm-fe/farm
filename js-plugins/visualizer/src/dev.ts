import { DevServerMiddleware, JsPlugin, Server } from '@farmfe/core';
import { Context, Middleware } from 'koa';
import { createDateSourceMiddleware } from './node/dataSource';

export function records(devServer: Server): Middleware {
  const compiler = devServer.getCompiler();
  return async (ctx: Context, next: () => Promise<any>) => {
    return createDateSourceMiddleware(compiler)(ctx.req, ctx.res, next);
  };
}

export default function farmRecorderPlugin(): JsPlugin {
  return {
    name: 'record-viewer-dev-data-source',
    config(config) {
      config.compilation ??= {};
      config.compilation = { ...config.compilation, record: true };
      return config;
    },
    configureServer(server) {
      const middlewares = [records] as DevServerMiddleware[];
      server.applyMiddlewares(middlewares);
    }
  };
}
