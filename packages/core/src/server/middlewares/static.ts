import fs from 'fs';
import path from 'path';
import { Context, Middleware, Next } from 'koa';
import serve from 'koa-static';
import { Server } from '../index.js';

export function staticMiddleware(devServerContext: Server): Middleware {
  const { config } = devServerContext;

  const staticMiddleware = serve(config.distDir, {
    // multiple page maybe receive "about", should auto try extension
    extensions: ['html']
  });

  // Fallback
  const fallbackMiddleware: Middleware = async (ctx: Context, next: Next) => {
    ctx.type = 'html';
    ctx.body = fs.createReadStream(path.join(config.distDir, 'index.html'));
    await next();
  };

  return async (ctx: Context, next: Next) => {
    const requestPath = ctx.request?.path;
    console.log('🤖 == return == requestPath:', requestPath);

    if (requestPath && requestPath.startsWith(config.output.publicPath)) {
      const modifiedPath = requestPath.substring(
        config.output.publicPath.length
      );
      console.log('🤖 == return == modifiedPath:', modifiedPath);

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
