/**
 * Serve resources that stored in memory. This middleware will be enabled when server.writeToDisk is false.
 */

import { Context, Next } from 'koa';
import { extname } from 'path';
import { Compiler } from '../../compiler';

export function resources(compiler: Compiler) {
  return async (ctx: Context, next: Next) => {
    await next();
    console.log(ctx.path, ctx.method, ctx.body, ctx.status);

    if (ctx.method !== 'HEAD' && ctx.method !== 'GET') return;
    // the response is already handled
    if (ctx.body || ctx.status !== 404) return;

    ctx.type = extname(ctx.path);
    const resource = compiler.resources()[ctx.path.slice(1)];
    ctx.body = Buffer.from(resource);
  };
}
