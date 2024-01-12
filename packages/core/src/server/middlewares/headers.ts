import { Middleware } from 'koa';
import { DevServer } from '../index.js';

export function headers(devSeverContext: DevServer): Middleware {
  const { config } = devSeverContext;
  if (!config.headers) return;

  return async (ctx, next) => {
    if (config.headers) {
      for (const name in config.headers) {
        ctx.set(name, config.headers[name] as string | string[]);
      }
    }
    await next();
  };
}
