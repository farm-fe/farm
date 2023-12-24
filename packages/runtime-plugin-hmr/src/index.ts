/**
 * HMR client as a Farm Runtime Plugin
 */
import type { FarmRuntimePlugin } from '@farmfe/runtime';
import { createHotContext } from './hot-module-state';
import { HmrClient } from './hmr-client';

let hmrClient: HmrClient;

export default <FarmRuntimePlugin>{
  name: 'farm-runtime-hmr-client-plugin',
  bootstrap(moduleSystem) {
    hmrClient = new HmrClient(moduleSystem);
    hmrClient.connect();
  },
  moduleCreated(module) {
    // create a hot context for each module
    module.meta.hot = createHotContext(module.id, hmrClient);
  }
};
