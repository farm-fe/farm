/**
 * HMR client as a Farm Runtime Plugin
 */
import type { Plugin } from '@farmfe/runtime';
import { HmrClient } from './hmr-client.js';
import { createHotContext } from './hot-module-state.js';

let hmrClient: HmrClient;

export default (<Plugin>{
  name: 'farm-runtime-hmr-client-plugin',
  bootstrap(moduleSystem) {
    hmrClient = new HmrClient(moduleSystem);
    hmrClient.connect();
  },
  moduleCreated(module) {
    // create a hot context for each module
    module.meta.hot = createHotContext(module.id, hmrClient);
  }
});
