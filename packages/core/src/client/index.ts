/**
 * HMR client as a Farm Runtime Plugin
 */
import type { FarmRuntimePlugin } from '@farmfe/runtime/src/plugin.js';
import { applyHotUpdates, createHotContext } from './hot-module-state.js';
import { HmrUpdatePacket, HmrUpdateResult } from './types.js';

// TODO using host and port from the config, default to use location.host
const port = 9801;
const host = 'localhost';

export default <FarmRuntimePlugin>{
  name: 'farm-runtime-hmr-client-plugin',
  bootstrap(moduleSystem) {
    console.log('[Farm HMR] connecting to the server...');

    // setup websocket connection
    const socket = new WebSocket(`ws://${host}:${port}`);
    // listen for the message from the server
    // when the user save the file, the server will recompile the file(and its dependencies as long as its dependencies are changed)
    // after the file is recompiled, the server will generated a update resource and send its id to the client
    // the client will use the id to fetch the update resource and apply the update
    socket.addEventListener('message', (event) => {
      const data = JSON.parse(event.data) as HmrUpdatePacket;

      import(`/__hmr?id=${data.id}`).then(
        (result: { default: HmrUpdateResult }) => {
          applyHotUpdates(result.default, moduleSystem);
        }
      );
    });

    socket.addEventListener('open', () => {
      console.log('[Farm HMR] connected to the server');
    });
  },
  moduleCreated(module) {
    // create a hot context for each module
    module.meta.hot = createHotContext(module.id);
  },
};
