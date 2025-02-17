import http from 'node:http';
import { createServer } from 'vite-bundle-analyzer';
import type { C, Middleware } from 'vite-bundle-analyzer';

export function createInternalServices() {
  const server = createServer();

  const middlewares: Record<string, Middleware> = {
    resouce: (c, next) => {
      c.res.writeHead(200, {
        'Content-Type': 'json/application',
        'Cache-Control': 'no-cache'
      });
      c.res.end(JSON.stringify({ a: 1 }));
      next();
    },
    modules: (c, next) => {
      next();
    },
    env_info: (c, next) => {
      next();
    },
    stats: (c, next) => {
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
