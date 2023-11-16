import { ModuleSystem } from './../../runtime/src/module-system';
import type { FarmRuntimePlugin } from '@farmfe/runtime/src/plugin';

export default <FarmRuntimePlugin>{
  name: 'farm-runtime-import-meta',
  _moduleSystem: {} as ModuleSystem,
  bootstrap(system: ModuleSystem) {
    this._moduleSystem = system;
  },
  moduleCreated(module) {
    module.meta.env = {
      ...process.env,
      mode: process.env.NODE_ENV,
      dev: process.env.NODE_ENV === 'development',
      prod: process.env.NODE_ENV === 'production'
    };
    const publicPath = this._moduleSystem.publicPaths[0];
    const url = `${location.host}${publicPath === '/' ? '' : publicPath}/${
      module.resource_pot
    }`;
    module.meta.url = url;
  }
};
