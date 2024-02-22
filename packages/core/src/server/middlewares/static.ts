import { Middleware, Context, Next } from 'koa';
import { DevServer } from '../index.js';
// import serve from 'koa-static';
// import mount from 'koa-mount';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export function staticMiddleware(devSeverContext: DevServer): Middleware {
  const { config } = devSeverContext;

  return async (ctx: Context) => {
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

    try {
      // const staticMiddleware = mount('/', serve(path.join(__dirname, './buildc')));

      // Fallback route
      const fallbackMiddleware = async (ctx: Context, next: Next) => {
        ctx.type = 'html';
        ctx.body = fs.createReadStream(
          path.join(__dirname, config.output.path)
        );
        await next();
      };

      return async (ctx: Context, next: Next) => {
        await fallbackMiddleware(ctx, next);
      };
    } catch (error) {
      this.logger.error('Static file handling error:', error);
      ctx.status = 500;
    }
  };
}

// export function StaticFilesHandler(distDir: string) {
//   const staticFilesServer = sirv(distDir, {
//     etag: true,
//     single: true
//   });

//   return async (ctx: Context, next: Next) => {
//     await new Promise<void>((resolve) => {
//       staticFilesServer(ctx.req, ctx.res, () => {
//         resolve();
//       });
//     });
//     await next();
//   };
// }
