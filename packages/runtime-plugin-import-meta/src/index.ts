import type { ModuleSystem } from "@farmfe/runtime";

let _moduleSystem = {} as ModuleSystem;

function assignObj(target: any, source: any) {
  for (const field in source) {
    target[field] = source[field];
  }
}

export default {
  name: "farm-runtime-import-meta",
  bootstrap: (system: ModuleSystem) => {
    _moduleSystem = system;
  },
  moduleCreated: (module: any) => {
    const publicPath = _moduleSystem.pp?.[0] || "";
    const isSSR = _moduleSystem.te === "node";

    module.meta.env = FARM_PROCESS_ENV ?? {};
    assignObj(module.meta.env, {
      dev: process.env.NODE_ENV === "development",
      prod: process.env.NODE_ENV === "production",
      BASE_URL: publicPath,
      SSR: isSSR,
    });

    module.meta.url = _moduleSystem.m()[module.id].url;
  },
};
