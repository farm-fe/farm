import { DevServer } from '../index.js';

export function headersPlugin(context: DevServer) {
  const { app, config } = context._context;
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
