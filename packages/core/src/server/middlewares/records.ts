/**
 * record middleware.
 */

import { Context } from 'koa';

import { DevServer } from '../index.js';

export function records(server: DevServer) {
  const compiler = server.getCompiler();

  return async (ctx: Context, next: () => Promise<any>) => {
    if (ctx.path === '/__record/modules') {
      ctx.body = compiler.modules();
      await next();
    } else if (ctx.path === '/__record/resolve') {
      const id = ctx.query.id as string;
      ctx.body = compiler.getResolveRecords(id);
      await next();
    } else if (ctx.path === '/__record/transform') {
      const id = ctx.query.id as string;
      ctx.body = compiler.getTransformRecords(id);
      await next();
    } else if (ctx.path === '/__record/process') {
      const id = ctx.query.id as string;
      ctx.body = compiler.getProcessRecords(id);
      await next();
    } else if (ctx.path === '/__record/analyze_deps') {
      const id = ctx.query.id as string;
      ctx.body = compiler.getAnalyzeDepsRecords(id);
      await next();
    } else if (ctx.path === '/__record/resource_pot') {
      const id = ctx.query.id as string;
      ctx.body = compiler.getResourcePotRecordsById(id);
      await next();
    } else {
      await next();
    }
  };
}

export function recordsPlugin(distance: DevServer) {
  distance._context.app.use(records(distance));
}
