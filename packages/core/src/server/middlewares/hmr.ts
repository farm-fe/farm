/**
 * HMR middleware waits for HMR request from client and return the corresponding updated modules.
 *
 * When a file changed, the dev server will first update the modified file and its dependencies and send signals to the browser client via websocket,
 * and store the updated result with a unique id in a cache, the client will send a `/__hmr?id=xxx` import() request to fetch the updated modules and execute it.
 */

import { Context } from 'koa';
import { DevServer } from '../index.js';

export function hmr(server: DevServer) {
  console.log('middleware hmr');
  return async (ctx: Context, next: () => Promise<any>) => {
    await next();

    if (ctx.path === '/__hmr') {
      console.log('hmr request', ctx.query.id);
      const result = server.hmrEngine?.getUpdateResult?.(
        ctx.query.id as string
      );

      if (result) {
        ctx.type = 'application/javascript';
        ctx.body = result;
      } else {
        throw new Error(`HMR update result not found for id ${ctx.query.id}`);
      }
    }
  };
}
