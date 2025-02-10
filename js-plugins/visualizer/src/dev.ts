import { JsPlugin, Server } from '@farmfe/core';
import { NextHandleFunction } from 'connect';
import { createDateSourceMiddleware } from './node/dataSource';

export function records(devServer: Server): NextHandleFunction {
  const compiler = devServer.getCompiler();
  return createDateSourceMiddleware(compiler);
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
      const middlewares = [records(server)];
      for (const middleware of middlewares) {
        server.middlewares.use(middleware);
      }
    }
  };
}
