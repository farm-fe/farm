import type { DevServerMiddleware, JsPlugin, Server } from '@farmfe/core';
import type { Context, Middleware } from 'koa';
import { getFarmEnvInfo } from './node/utils/envinfo';

export function records(devServer: Server): Middleware {
  const compiler = devServer.getCompiler();
  return async (ctx: Context, next: () => Promise<any>) => {
    if (ctx.path === '/__record/farm_env_info') {
      const info = await getFarmEnvInfo();
      ctx.body = info;
      await next();
    } else if (ctx.path === '/__record/resources_map') {
      const resource_map = compiler.resourcesMap();
      ctx.body = resource_map;
      await next();
    } else if (ctx.path === '/__record/resource') {
      const id = ctx.query.id as string;
      const resource = compiler.resource(id);
      ctx.body = resource;
      await next();
    } else {
      await next();
    }
  };
}

export default function farmRecorderPlugin(): JsPlugin {
  return {
    name: 'record-viewer-dev-data-source',
    config(config) {
      config.compilation = { ...config.compilation, record: true };
      return config;
    },
    configureDevServer(server) {
      const middlewares = [records] as DevServerMiddleware[];
      server.applyMiddlewares(middlewares);
    }
  };
}
