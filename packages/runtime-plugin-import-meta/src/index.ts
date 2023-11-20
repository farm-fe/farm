import type { FarmRuntimePlugin, ModuleSystem } from '@farmfe/runtime';

const __global_this__ = globalThis || window;

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

    if (__global_this__.location) {
      const url = `${__global_this__.location.protocol}//${
        __global_this__.location.host
      }${publicPath.endsWith('/') ? publicPath.slice(0, -1) : publicPath}/${
        module.resource_pot
      }`;
      module.meta.url = url;
    } else {
      module.meta.url = module.resource_pot;
    }
  }
};
