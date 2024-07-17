import { Options, createProxyMiddleware } from 'http-proxy-middleware';
import { Context, Middleware, Next } from 'koa';

import { UserConfig, UserHmrConfig } from '../../config/types.js';
import { Logger } from '../../utils/logger.js';
import type { Server } from '../index.js';

export function useProxy(
  options: UserConfig['server'],
  devSeverContext: Server,
  logger: Logger
) {
  const proxyOption = options['proxy'];
  for (const path of Object.keys(proxyOption)) {
    let opts = proxyOption[path] as Options;

    if (typeof opts === 'string') {
      opts = { target: opts, changeOrigin: true };
    }

    const proxyMiddleware = createProxyMiddleware(opts);
    const server = devSeverContext.server;
    const hmrOptions = options.hmr as UserHmrConfig;
    if (server) {
      server.on('upgrade', (req, socket: any, head) => {
        for (const path in options.proxy) {
          const opts = proxyOption[path] as Options;
          if (
            opts.ws ||
            opts.target?.toString().startsWith('ws:') ||
            opts.target?.toString().startsWith('wss:')
          ) {
            const proxy = createProxyMiddleware(opts);
            if (opts.pathRewrite) {
              const fromPath = Object.keys(opts.pathRewrite)[0];
              const toPath: string = (
                opts.pathRewrite as { [regexp: string]: string }
              )[fromPath];
              if (req.url === hmrOptions.path) {
                req.url = '';
              }
              req.url = rewritePath(req.url, fromPath, toPath);
            }
            proxy.upgrade(req, socket, head);
            return;
          }
        }
      });
    }

    const errorHandlerMiddleware = async (ctx: Context, next: Next) => {
      try {
        await new Promise<void>((resolve, reject) => {
          proxyMiddleware(ctx.req, ctx.res, (err) => {
            if (err) {
              reject(err);
            } else {
              resolve();
            }
          });
        });
        await next();
      } catch (err) {
        logger.error(`Error in proxy for path ${path}: \n ${err.stack}`);
      }
    };

    try {
      if (path.length > 0) {
        const pathRegex = new RegExp(path);
        const app = devSeverContext.app();
        app.use((ctx, next) => {
          if (pathRegex.test(ctx.path)) {
            return errorHandlerMiddleware(ctx, next);
          }
          return next();
        });
      }
    } catch (err) {
      logger.error(`Error setting proxy for path ${path}: \n ${err.stack}`);
    }
  }
}

export function proxy(devSeverContext: Server): Middleware {
  const { config, logger } = devSeverContext;
  if (!config.proxy) {
    return;
  }

  useProxy(config, devSeverContext, logger);
}

function rewritePath(path: string, fromPath: RegExp | string, toPath: string) {
  if (fromPath instanceof RegExp) {
    return path.replace(fromPath, toPath);
  } else {
    return path.replace(new RegExp(fromPath), toPath);
  }
}
