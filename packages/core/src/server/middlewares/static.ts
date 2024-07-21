import fs from 'fs';
import path, { relative } from 'path';
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
    await next();

    // If staticMiddleware doesn't find the file, try to serve index.html
    if (ctx.status === 404 && !ctx.body) {
      ctx.type = 'html';
      ctx.body = fs.createReadStream(path.join(config.distDir, 'index.html'));
    }
  };

  return async (ctx: Context, next: Next) => {
    if (ctx.status !== 404 || ctx.body) {
      await next();
      return;
    }

    const requestPath = ctx.request?.path;
    let modifiedPath = requestPath;

    if (requestPath) {
      if (config.output.publicPath.startsWith('/')) {
        modifiedPath = requestPath.substring(config.output.publicPath.length);
      } else {
        const publicPath = relative(
          path.join(config.distDir, config.output.publicPath),
          config.distDir
        );
        modifiedPath = requestPath.substring(publicPath.length + 1);
      }
    }

    ctx.request.path = `/${modifiedPath}`;

    try {
      // Serve middleware for static files
      await staticMiddleware(ctx, async () => {
        // If staticMiddleware doesn't find the file or refresh current page router, execute fallbackMiddleware
        await fallbackMiddleware(ctx, next);
      });
    } catch (error) {
      devServerContext.logger.error('Static file handling error:', error);
      ctx.status = 500;
    }
  };
}
