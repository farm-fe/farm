import type { ModuleSystem, Plugin } from '@farmfe/runtime';

const __global_this__ = typeof globalThis !== 'undefined' ? globalThis : window;

export default (<Plugin>{
  name: 'farm-runtime-import-meta',
  _moduleSystem: {} as ModuleSystem,
  bootstrap(system: ModuleSystem) {
    this._moduleSystem = system;
  },
  moduleCreated(module) {
    module.meta.env = {
      ...((FARM_PROCESS_ENV) ?? {}),
      mode: process.env.NODE_ENV,
      dev: process.env.NODE_ENV === 'development',
      prod: process.env.NODE_ENV === 'production'
    };
    const publicPath = this._moduleSystem.publicPaths?.[0] || '';

    const { location } = __global_this__;
    const url = location
      ? `${location.protocol}//${location.host}${publicPath.replace(
        /\/$/,
        ''
      )}/${module.id}?t=${Date.now()}`
      : module.resource_pot;
    module.meta.url = url;
  }
});
