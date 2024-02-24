import { Middleware, Context, Next } from 'koa';
import { Server } from '../index.js';
import serve from 'koa-static';
import path from 'path';
import fs from 'fs';

export function staticMiddleware(devServerContext: Server): Middleware {
  const { config } = devServerContext;

  const staticMiddleware = serve(path.join(process.cwd(), config.output.path));

  // Fallback
  const fallbackMiddleware: Middleware = async (ctx: Context, next: Next) => {
    ctx.type = 'html';
    ctx.body = fs.createReadStream(
      path.join(process.cwd(), config.output.path, 'index.html')
    );
    await next();
  };

  return async (ctx: Context, next: Next) => {
    const requestPath = ctx.request?.path;

    if (requestPath && requestPath.startsWith(config.output.publicPath)) {
      const modifiedPath = requestPath.substring(
        config.output.publicPath.length
      );

      ctx.request.path = `/${modifiedPath}`;

      try {
        // Serve middleware for static files
        await staticMiddleware(ctx, async () => {
          // If staticMiddleware doesn't find the file or refresh current page router, execute fallbackMiddleware
          await fallbackMiddleware(ctx, next);
        });
      } catch (error) {
        this.logger.error('Static file handling error:', error);
        ctx.status = 500;
      }
    } else {
      await next();
    }
  };
}
