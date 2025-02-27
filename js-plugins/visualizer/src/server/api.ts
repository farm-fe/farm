import http from 'node:http';
import { createServer } from 'vite-bundle-analyzer';
import type { Middleware } from 'vite-bundle-analyzer';
import {
  VisualizerModule,
  evaludateModuleGraph,
  evaludatePluginLifecycle
} from './analyze-module';

export function createInternalServices(mod: VisualizerModule) {
  const server = createServer();

  const middlewares: Record<string, Middleware> = {
    resouce: (c, next) => {
      const analysisModule = evaludateModuleGraph(mod.c, mod.workspaceRoot);
      c.res.writeHead(200, {
        'Content-Type': 'json/application',
        'Cache-Control': 'no-cache'
      });
      c.res.end(JSON.stringify(analysisModule, null, 2));
      next();
    },
    modules: (c, next) => {
      next();
    },
    env_info: (c, next) => {
      next();
    },
    stats: (c, next) => {
      const stats = evaludatePluginLifecycle(mod.c, false);
      c.res.writeHead(200, {
        'Content-Type': 'json/application',
        'Cache-Control': 'no-cache'
      });
      c.res.end(JSON.stringify(stats, null, 2));
      next();
    }
  };

  function handler(
    req: http.IncomingMessage,
    res: http.ServerResponse,
    next: Function
  ) {
    const url = new URL(req.url || '', 'http://localhost');
    const middleware = middlewares[url.pathname.replace('/__visualizer/', '')];
    if (middleware) {
      return middleware(
        { req, res, query: {}, params: {} },
        next as () => void
      );
    }
    next();
  }

  return {
    handler,
    server,
    middlewares
  };
}
