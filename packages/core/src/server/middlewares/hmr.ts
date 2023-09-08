/**
 * HMR middleware waits for HMR request from client and return the corresponding updated modules.
 *
 * When a file changed, the dev server will first update the modified file and its dependencies and send signals to the browser client via websocket,
 * and store the updated result with a unique id in a cache, the client will send a `/__hmr?id=xxx` import() request to fetch the updated modules and execute it.
 */

// import { Context } from 'koa';
import { WebSocketServer } from 'ws';
import { HmrEngine } from '../hmr-engine.js';
import { DevServer } from '../index.js';

// /**
//  * @deprecated HMR result is now served by websocket, send to client via websocket and get the result by `eval` the websocket message.
//  * @param context DevServer
//  */
// export function hmr(server: DevServer) {
//   return async (ctx: Context, next: () => Promise<void>) => {
//     if (ctx.path === '/__hmr') {
//       const result = server.hmrEngine?.getUpdateResult?.(
//         ctx.query.id as string
//       );

//       if (result) {
//         ctx.status = 200;
//         ctx.type = 'application/javascript';
//         ctx.body = result;
//       } else {
//         throw new Error(`HMR update result not found for id ${ctx.query.id}`);
//       }
//     } else {
//       await next();
//     }
//   };
// }

export function hmrPlugin(context: DevServer) {
  const { config, _context, logger } = context;
  if (config.hmr) {
    if (config.hmr.host === config.host && config.hmr.port === config.port) {
      context.ws = new WebSocketServer({
        noServer: true
      });
      context.server.on('upgrade', (request, socket, head) => {
        if (
          request.url === config.hmr.path &&
          request.headers['sec-websocket-protocol'] === 'farm_hmr'
        ) {
          context.ws.handleUpgrade(request, socket, head, (ws) => {
            context.ws.emit('connection', ws, request);
          });
        }
      });
    } else {
      context.ws = new WebSocketServer({
        port: config.hmr.port,
        host: config.hmr.host,
        path: config.hmr.path
      });
    }

    // _context.app.use(hmr(context));
    context.hmrEngine = new HmrEngine(_context.compiler, context, logger);
  }
}
