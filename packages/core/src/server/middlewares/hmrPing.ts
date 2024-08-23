import { Context, Next } from 'koa';
export function hmrPing() {
  return async (ctx: Context, next: Next) => {
    if (ctx.get('accept') === 'text/x-farm-ping') {
      ctx.status = 204;
    } else {
      await next();
    }
  };
}
