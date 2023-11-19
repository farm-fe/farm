import type { FarmRuntimePlugin, ModuleSystem } from '@farmfe/runtime';

export default <FarmRuntimePlugin>{
  name: 'farm-runtime-import-meta',
  _moduleSystem: {} as ModuleSystem,
  bootstrap(system: ModuleSystem) {
    this._moduleSystem = system;
  },
  moduleCreated(module) {
    module.meta.env = {
      mode: process.env.NODE_ENV,
      dev: process.env.NODE_ENV === 'development',
      prod: process.env.NODE_ENV === 'production'
    };
    const publicPath = this._moduleSystem.publicPaths[0];

    if (this._moduleSystem.targetEnv === 'node') {
      module.meta.url = module.resource_pot;
    } else {
      const url = `${location.host}${publicPath === '/' ? '' : publicPath}/${
        module.resource_pot
      }`;
      module.meta.url = url;
    }
  }
};
