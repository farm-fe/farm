import { Middleware, Context } from 'koa';
import { DevServer } from '../index.js';
import sirv from 'sirv';

export function sirvMiddleware(devSeverContext: DevServer): Middleware {
  const { config } = devSeverContext._context;

  return async (ctx: Context, next) => {
    const requestPath = ctx.request?.path;

    if (requestPath && requestPath.startsWith(config.output.publicPath)) {
      const modifiedPath = requestPath.substring(
        config.output.publicPath.length
      );

      if (modifiedPath.startsWith('/')) {
        ctx.request.path = modifiedPath;
      } else {
        ctx.request.path = `/${modifiedPath}`;
      }
    }

    const handleStatic = StaticFilesHandler(config.output.path);
    await handleStatic(ctx, next);
  };
}

export function StaticFilesHandler(distDir: string) {
  const staticFilesServer = sirv(distDir, {
    etag: true,
    single: true
  });

  return async (ctx: Context, next: () => Promise<any>) => {
    await new Promise<void>((resolve) => {
      staticFilesServer(ctx.req, ctx.res, () => {
        resolve();
      });
    });
    await next();
  };
}
