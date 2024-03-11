import {
  default as koaProxy,
  IKoaProxiesOptions,
  IBaseKoaProxiesOptions
} from 'koa-proxies';
import type { Server } from '../index.js';
import { UserConfig } from '../../config/types.js';
import Application, { Middleware, Context, Next } from 'koa';
import { Logger } from '../../utils/logger.js';

export type ProxiesOptions = IKoaProxiesOptions;

export function useProxy(
  options: UserConfig['server']['proxy'],
  app: Application,
  logger: Logger
) {
  for (const path of Object.keys(options)) {
    let opts = options[path] as IBaseKoaProxiesOptions;

    if (typeof opts === 'string') {
      opts = { target: opts, changeOrigin: true } as IBaseKoaProxiesOptions;
    }
    const proxyMiddleware = koaProxy(
      path[0] === '^' ? new RegExp(path) : path,
      opts
    );

    const errorHandlerMiddleware = async (ctx: Context, next: Next) => {
      try {
        await proxyMiddleware(ctx, next);
      } catch (err) {
        logger.error(`Error in proxy for path ${path}: \n ${err.stack}`);
      }
    };

    try {
      if (path.length > 0) {
        app.use(errorHandlerMiddleware);
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
