import chalk from 'chalk';
import proxy, { IKoaProxiesOptions, IBaseKoaProxiesOptions } from 'koa-proxies';
import type { ServerOptions as HttpProxyServerOptions } from 'http-proxy';
import { DevServer } from '../index.js';

export type ProxiesOptions = IKoaProxiesOptions & HttpProxyServerOptions;

export function proxyPlugin(context: DevServer) {
  const { app, config, logger } = context._context;
  if (!config.proxy) {
    return;
  }
  const options = config.proxy;
  Object.keys(options).forEach((path) => {
    let opts = options[path] as IBaseKoaProxiesOptions;

    if (typeof opts === 'string') {
      opts = { target: opts, changeOrigin: true } as IBaseKoaProxiesOptions;
    }
    app.on('error', (err, ctx) => {
      // proxy watcher error
      if (ctx.req.oldPath === path) {
        logger.info(`${chalk.red(`http proxy error:`)}\n${err.stack}`);
      }
    });

    app.use(proxy(path, opts));
  });
}
