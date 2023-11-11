/**
 * HMR client as a Farm Runtime Plugin
 */
import type { FarmRuntimePlugin } from '@farmfe/runtime/src/plugin';
import { applyHotUpdates, createHotContext } from './hot-module-state';
import { RawHmrUpdateResult } from './types';
declare const FARM_HMR_PORT: string | undefined;
declare const FARM_HMR_HOST: string | undefined;
declare const FARM_HMR_PATH: string | undefined;

const port = Number(FARM_HMR_PORT || 9000);
// TODO use import.meta to get hostname
const host =
  typeof FARM_HMR_HOST === 'boolean'
    ? window.location.hostname || 'localhost'
    : FARM_HMR_HOST || 'localhost';

const path = FARM_HMR_PATH || '/__hmr';

export default <FarmRuntimePlugin>{
  name: 'farm-runtime-hmr-client-plugin',
  bootstrap(moduleSystem) {
    console.log('[Farm HMR] connecting to the server...');

    function connect() {
      // setup websocket connection
      const socket = new WebSocket(`ws://${host}:${port}${path}`, 'farm_hmr');
      // listen for the message from the server
      // when the user save the file, the server will recompile the file(and its dependencies as long as its dependencies are changed)
      // after the file is recompiled, the server will generated a update resource and send its id to the client
      // the client will use the id to fetch the update resource and apply the update
      socket.addEventListener('message', (event) => {
        // const data = JSON.parse(event.data) as HmrUpdatePacket;
        const result: RawHmrUpdateResult = eval(`(${event.data})`);
        const immutableModules = eval(result.immutableModules);
        const mutableModules = eval(result.mutableModules);
        const modules = { ...immutableModules, ...mutableModules };
        applyHotUpdates(
          {
            added: result.added,
            changed: result.changed,
            removed: result.removed,
            boundaries: result.boundaries,
            modules,
            dynamicResourcesMap: result.dynamicResourcesMap
          },
          moduleSystem
        );
        // import(`/__hmr?id=${data.id}`).then(
        //   (result: { default: HmrUpdateResult }) => {
        //     applyHotUpdates(result.default, moduleSystem);
        //   }
        // );
      });

      socket.addEventListener('open', () => {
        console.log('[Farm HMR] connected to the server');
      });
      // TODO use ping/pong to detect the connection is closed, and if the server is online again, reload the page
      // socket.addEventListener('close', () => setTimeout(connect, 3000));

      return socket;
    }

    connect();
  },
  moduleCreated(module) {
    // create a hot context for each module
    module.meta.hot = createHotContext(module.id);
  }
};
