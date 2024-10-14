import type { ModuleSystem } from '@farmfe/runtime';

const __global_this__ = typeof globalThis !== 'undefined' ? globalThis : window;

export default {
  name: 'farm-runtime-import-meta',
  _moduleSystem: {} as ModuleSystem,
  bootstrap(system: ModuleSystem) {
    this._moduleSystem = system;
  },
  moduleCreated(module: any) {
    const publicPath = this._moduleSystem.publicPaths?.[0] || "";
    const isSSR = this._moduleSystem.targetEnv === "node";
    const { location } = __global_this__;

    let baseUrl;
    try {
      baseUrl = (
        location
          ? new URL(
              publicPath,
              `${location.protocol}//${location.host}`,
            )
          : new URL(module.resource_pot)
      ).pathname;
    } catch (_) {
      baseUrl = '/';
    }

    module.meta.env = {
      ...(FARM_PROCESS_ENV ?? {}),
      dev: process.env.NODE_ENV === 'development',
      prod: process.env.NODE_ENV === 'production',
      BASE_URL: baseUrl,
      SSR: isSSR,
    };

    const url = location
      ? `${location.protocol}//${location.host}${publicPath.replace(
        /\/$/,
        ''
      )}/${module.id}?t=${Date.now()}`
      : module.resource_pot;
    module.meta.url = url;
  }
};
