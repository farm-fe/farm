import type { ServerResponse } from 'node:http';
import type { DevServerMiddleware, JsPlugin, Server } from '@farmfe/core';
import type Connect from 'connect';
import { createDateSourceMiddleware } from './node/dataSource';

export function records(devServer: Server): ReturnType<DevServerMiddleware> {
  return async (
    req: Connect.IncomingMessage,
    res: ServerResponse,
    next: Connect.NextFunction
  ) => {
    const compiler = devServer.getCompiler();
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
