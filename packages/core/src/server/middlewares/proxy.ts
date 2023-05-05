import { URL } from 'node:url';

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
      opts = { target: opts };
    }
    opts.logs = (ctx: any, target: string) => {
      logger.info(
        `${ctx.req.method} "${ctx.req.oldPath}" proxy to ${chalk.cyan(
          new URL(ctx.req.url, target)
        )}`
      );
    };
    app.use(proxy(path, opts));
  });
}
