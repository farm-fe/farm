import { DevServer } from '../index.js';

export function headersPlugin(devSeverContext: DevServer) {
  const { app, config } = devSeverContext._context;
  if (!config.headers) return;
  app.use(async (ctx, next) => {
    if (config.headers) {
      for (const name in config.headers) {
        ctx.set(name, config.headers[name] as string | string[]);
      }
    }
    await next();
  });
}
