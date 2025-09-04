/**
 * HMR client as a Farm Runtime Plugin
 */
import { HmrClient } from './hmr-client.js';
import { createHotContext } from './hot-module-state.js';

let hmrClient: HmrClient;

export default {
  name: 'farm-runtime-hmr-client-plugin',
  bootstrap(moduleSystem: any) {
    hmrClient = new HmrClient(moduleSystem);
    hmrClient.connect();
  },
  moduleCreated(module: any) {
    // create a hot context for each module
    module.meta.hot = createHotContext(module.id, hmrClient);
  }
};
