/**
 * HMR middleware waits for HMR request from client and return the corresponding updated modules.
 *
 * When a file changed, the dev server will first update the modified file and its dependencies and send signals to the browser client via websocket,
 * and store the updated result with a unique id in a cache, the client will send a `/__hmr?id=xxx` import() request to fetch the updated modules and execute it.
 */

import { HmrEngine } from '../hmr-engine.js';
import { DevServer } from '../index.js';

// /**
//  * @deprecated HMR result is now served by websocket, send to client via websocket and get the result by `eval` the websocket message.
//  * @param context DevServer
//  */

export function hmrPlugin(devSeverContext: DevServer) {
  const { config, _context, logger } = devSeverContext;
  if (config.hmr) {
    // TODO use diff hmr port with dev server
    devSeverContext.hmrEngine = new HmrEngine(
      _context.compiler,
      devSeverContext,
      logger
    );
  }
}
