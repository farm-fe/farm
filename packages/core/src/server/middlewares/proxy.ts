import proxy, { IKoaProxiesOptions, IBaseKoaProxiesOptions } from 'koa-proxies';
import type { DevServer } from '../index.js';
import { UserConfig } from '../../config/types.js';
import Application from 'koa';
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
    app.on('error', (err, ctx) => {
      // proxy watcher error
      if (ctx.req.oldPath === path) {
        logger.error(`http proxy error:\n ${err.stack}`);
      }
    });

    try {
      if (path.length > 0) {
        app.use(proxy(path[0] === '^' ? new RegExp(path) : path, opts));
      }
    } catch (err) {
      logger.error(`Error setting proxy for path ${path}: ${err.message}`);
    }
  }
}

export function proxyPlugin(devSeverContext: DevServer) {
  const { app, config, logger } = devSeverContext._context;
  if (!config.proxy) {
    return;
  }
  const options = config.proxy;
  useProxy(options, app, logger);
}
