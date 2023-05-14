/**
 * Serve resources that stored in memory. This middleware will be enabled when server.writeToDisk is false.
 */

import { extname } from 'node:path';
import { Context, Next } from 'koa';
import { Compiler } from '../../compiler/index.js';
import { DevServer } from '../index.js';

export function resources(compiler: Compiler) {
  return async (ctx: Context, next: Next) => {
    await next();

    if (ctx.method !== 'HEAD' && ctx.method !== 'GET') return;
    // the response is already handled
    if (ctx.body || ctx.status !== 404) return;

    // if compiler is compiling, wait for it to finish
    if (compiler.compiling) {
      await new Promise((resolve) => {
        compiler.onUpdateFinish(() => resolve(undefined));
      });
    }

    const resourcePath = ctx.path.slice(1) || 'index.html'; // remove leading slash
    ctx.type = extname(resourcePath);
    const resource = compiler.resources()[resourcePath];

    if (!resource) return;

    ctx.body = Buffer.from(resource);
  };
}

export function resourcesPlugin(distance: DevServer) {
  distance._context.app.use(resources(distance._context.compiler));
}
