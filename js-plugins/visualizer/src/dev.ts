import { JsPlugin, Server } from '@farmfe/core';
import type Connect from 'connect';
import { createDateSourceMiddleware } from './node/dataSource';

export function records(devServer: Server): Connect.NextHandleFunction {
  return async (req, res, next) => {
    const compiler = devServer.getCompiler();
    if (!compiler) return;
    return createDateSourceMiddleware(compiler)(req, res, next);
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
      server.middlewares.use(records(server));
    }
  };
}
