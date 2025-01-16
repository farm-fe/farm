import type { FarmRuntimePluginContainer } from "./modules/plugin.js";

// Injected during compile time
declare const __FARM_RUNTIME_TARGET_ENV__: 'browser' | 'node' | 'library';
declare const __FARM_ENABLE_RUNTIME_PLUGIN__: boolean;
declare const __FARM_ENABLE_TOP_LEVEL_AWAIT__: boolean;
declare const __FARM_ENABLE_EXTERNAL_MODULES__: boolean;

export interface Module {
  id: string;
  exports?: any;
  // initialize promise if this module is a async module
  initializer?: Promise<any> | undefined;
  resource_pot?: string;
  meta?: Record<string, any>;
  require?: (id: string) => any;
}

type ModuleInitializationFunction = (
  module: Module,
  exports: any,
  __farm_require__: (moduleId: string) => any,
  __farm_dynamic_require__: (moduleId: string) => any,
) => void | Promise<void>;

export type ModuleInitialization = ModuleInitializationFunction;

export interface ModuleSystem {
  // require
  r(id: string): any;
  // register
  g(id: string, module: ModuleInitialization): () => any
  // dynamicImport
  d(id: string): Promise<void>,
  // getModules
  m(): Record<string, ModuleInitialization>,
  // getCache
  c(): Record<string, Module>,
  // updateModule
  u(moduleId: string, init: ModuleInitialization): void
  // deleteModule
  e(moduleId: string): boolean
  // clearCache
  a(moduleId: string): boolean
}

const __farm_global_this__: any = '<@__farm_global_this__@>';

// It will be removed if __FARM_RUNTIME_TARGET_ENV__ is not browser when building runtime 
if (__FARM_RUNTIME_TARGET_ENV__ === 'browser') {
  // polyfill require when running in browser
  const __global_this__: any = typeof window !== 'undefined' ? window : {};
  __global_this__.require ||= require;
}

const pluginContainer: FarmRuntimePluginContainer = __farm_global_this__.p;
const dynamicImport: (id: string) => Promise<any> = __farm_global_this__.d;

// all modules registered
const modules: Record<string, ModuleInitialization> = {};
// module cache after module initialized
const cache: Record<string, Module> = {};

if (__FARM_ENABLE_EXTERNAL_MODULES__) {
  // externalModules
  __farm_global_this__.e = {}
  // The external modules are injected during compile time.
  __farm_global_this__.se = function setExternalModules(externalModules: Record<string, any>): void {
    Object.assign(this.externalModules, externalModules || {});
  }
}

__farm_global_this__.m = {
  r: farmRequire,
  g: farmRegister,
  d: dynamicImport,
  m: () => modules,
  c: () => cache,
} as ModuleSystem;

export function farmRequire(id: string): any {
  if (cache[id]) {
    const cachedModuleResult = cache[id].initializer || cache[id].exports;
    // will be removed as dead code if no plugin enabled when minify enabled
    if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
      const shouldSkip = pluginContainer.b(
        "readModuleCache",
        cache[id],
      );
  
      // console.log(`[Farm] shouldSkip: ${shouldSkip} ${moduleId}`);
      if (!shouldSkip) {
        return cachedModuleResult;
      }
    } else {
      return cachedModuleResult;
    }
  }

  const initializer = modules[id];

  if (!initializer) {
    if (__FARM_ENABLE_EXTERNAL_MODULES__) {
      // externalModules
      if (__farm_global_this__.e[id]) {
        return __farm_global_this__.e[id];
      }
    }
   
    console.debug(`[Farm] Module "${id}" is not registered`);

    if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
      const res = pluginContainer.b("moduleNotFound", id);

      if (res) {
        return res
      }
    }

    // return a empty module if the module is not registered
    return {};
  }

  // create a full new module instance and store it in cache to avoid cyclic initializing
  const module = cache[id] = {
    id,
    require: (moduleId: string) => require(moduleId)
  } as Module;

  if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
    // call the module created hook
    pluginContainer.s("moduleCreated", module);
  }

  cache[id] = module;

  // initialize the new module
  const result = initializer(
    module,
    module.exports,
    require,
    dynamicImport,
  );

  if (__FARM_ENABLE_TOP_LEVEL_AWAIT__) {
     // it's a async module, return the promise
    if (result && result instanceof Promise) {
      module.initializer = result.then(() => {
        if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
          // call the module initialized hook
          pluginContainer.s("moduleInitialized", module);
        }
        module.initializer = undefined;
        // return the exports of the module
        return module.exports;
      });

      return module.initializer;
    }
  }

  if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
    // call the module initialized hook
    pluginContainer.s("moduleInitialized", module);
  }
  // return the exports of the module
  return module.exports;
}

// TODO set reRegisterModules when calling register 
export function farmRegister(id: string, module: ModuleInitialization, reRegisterModules: boolean): () => any {
  if (modules[id] && !reRegisterModules) {
    console.warn(
      `Module "${id}" has registered! It should not be registered twice`,
    );
    return;
  }

  modules[id] = module;
  return () => require(id);
}
