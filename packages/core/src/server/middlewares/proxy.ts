import { Options, createProxyMiddleware } from 'http-proxy-middleware';
import Application, { Middleware, Context, Next } from 'koa';

import { UserConfig } from '../../config/types.js';
import { Logger } from '../../utils/logger.js';
import type { Server } from '../index.js';

export function useProxy(
  options: UserConfig['server']['proxy'],
  app: Application,
  logger: Logger
) {
  for (const path of Object.keys(options)) {
    let opts = options[path] as Options;

    if (typeof opts === 'string') {
      opts = { target: opts, changeOrigin: true };
    }

    const proxyMiddleware = createProxyMiddleware(opts);

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
  const options = config.proxy;
  useProxy(options, devSeverContext.app(), logger);
}
