/**
 * record middleware.
 */

import { Context } from 'koa';

import { DevServer } from '../index.js';

export function records(server: DevServer) {
  const compiler = server.getCompiler();

  return async (ctx: Context, next: () => Promise<any>) => {
    if (ctx.path === '/_resolve_records') {
      ctx.body = compiler.getResolveRecords();
      await next();
    } else if (ctx.path === '/_transform_records') {
      const id = ctx.query.id as string;
      ctx.body = compiler.getTransformRecords(id);
      await next();
    } else if (ctx.path === '/_process_records') {
      const id = ctx.query.id as string;
      ctx.body = compiler.getProcessRecords(id);
      await next();
    } else if (ctx.path === '/_analyze_deps_records') {
      const id = ctx.query.id as string;
      ctx.body = compiler.getAnalyzeDepsRecords(id);
      await next();
    } else {
      await next();
    }
  };
}

export function recordsPlugin(distance: DevServer) {
  distance._context.app.use(records(distance));
}
