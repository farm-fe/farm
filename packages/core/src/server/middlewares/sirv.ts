import { Middleware, Context } from 'koa';
import { DevServer } from '../index.js';
import sirv from 'sirv';

export function sirvMiddleware(devSeverContext: DevServer): Middleware {
  const { previewConfig } = devSeverContext._context;

  return async (ctx: any, next: any) => {
    const requestPath = ctx.request?.path;

    if (
      requestPath &&
      requestPath.startsWith(previewConfig.output.publicPath)
    ) {
      const modifiedPath = requestPath.substring(
        previewConfig.output.publicPath.length
      );

      if (modifiedPath.startsWith('/')) {
        ctx.request.path = modifiedPath;
      } else {
        ctx.request.path = `/${modifiedPath}`;
      }
    }

    const handleStatic = StaticFilesHandler(previewConfig.output.path);
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
