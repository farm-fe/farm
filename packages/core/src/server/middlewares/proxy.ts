import proxy, { IKoaProxiesOptions, IBaseKoaProxiesOptions } from 'koa-proxies';
import type { DevServer } from '../index.js';

export type ProxiesOptions = IKoaProxiesOptions;

export function proxyPlugin(devServer: DevServer) {
  const { app, config, logger } = devServer._context;
  if (!config.proxy) {
    return;
  }
  const options = config.proxy;
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
      app.use(proxy(path, opts));
    } catch (err) {
      logger.error(`Error setting proxy for path ${path}: ${err.message}`);
    }
  }
}
