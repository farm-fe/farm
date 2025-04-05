import type { Resource } from "./modules/dynamic-import.js";
import type { FarmRuntimePluginContainer } from "./modules/plugin.js";

// Injected during compile time
declare const __FARM_RUNTIME_TARGET_ENV__: 'browser' | 'node' | 'library';
declare const __FARM_ENABLE_RUNTIME_PLUGIN__: boolean;
declare const __FARM_ENABLE_TOP_LEVEL_AWAIT__: boolean;
declare const __FARM_ENABLE_EXTERNAL_MODULES__: boolean; // always true if target env is not library

declare let $__farm_global_this__$: any;

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
  // reRegisterModules
  _rg: boolean;
  // pluginContainer
  p: FarmRuntimePluginContainer;
  // externalModules
  em: Record<string, any>;
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
  // loadDynamicResourcesOnly
  l(moduleId: string, force?: boolean): Promise<any>
  // setExternalModules
  se(externalModules: Record<string, any>): void
  // setInitialLoadedResources
  si(resources: string[]): void
  // setDynamicModuleResourcesMap
  // These two methods are used to support dynamic module loading, the dynamic module info is collected by the compiler and injected during compile time
  // This method can also be called during runtime to add new dynamic modules
  sd(dynamicResources: Resource[], dynamicModuleResourcesMap: Record<string, number[]>): void
  // setPublicPaths
  // The public paths are injected during compile time
  sp(publicPaths: string[]): void
  // bootstrap
  // bootstrap should be called after all three methods above are called, and the bootstrap call is also injected during compile time
  // This method should only be called once
  b(): void
}

// init `window['xxxx] = {}`
const __farm_global_this__: any = $__farm_global_this__$ = {};

// It will be removed if __FARM_RUNTIME_TARGET_ENV__ is not browser when building runtime 
if (__FARM_RUNTIME_TARGET_ENV__ === 'browser') {
  // polyfill require when running in browser
  const __global_this__: any = typeof window !== 'undefined' ? window : {};
  __global_this__.require = __global_this__.require || farmRequire;
}
if (__FARM_RUNTIME_TARGET_ENV__ === 'node') {
  const __global_this__: any = typeof global !== 'undefined' ? global : {};
  __global_this__.require = __global_this__.require || farmRequire;
}

// all modules registered
const modules: Record<string, ModuleInitialization> = {};
// module cache after module initialized
const cache: Record<string, Module> = {};

export const moduleSystem = {
  r: farmRequire,
  g: farmRegister,
  m: () => modules,
  c: () => cache,
} as ModuleSystem;

if (__FARM_ENABLE_EXTERNAL_MODULES__) {
  // externalModules
  moduleSystem.em = {}
  // The external modules are injected during compile time.
  moduleSystem.se = function setExternalModules(externalModules: Record<string, any>): void {
    for (const key in externalModules) {
      let em = externalModules[key];
      // add a __esModule flag to the module if the module has default export
      if (em && em.default && !em.__esModule) {
        em = Object.assign({}, em, { __esModule: true });  
      }

      moduleSystem.em[key]= em;
    }
  }
  __farm_global_this__.m = moduleSystem;
}

export function farmRequire(id: string): any {
  if (cache[id]) {
    const cachedModuleResult = cache[id].initializer || cache[id].exports;
    // will be removed as dead code if no plugin enabled when minify enabled
    if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
      const shouldSkip = moduleSystem.p.b(
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
      if (moduleSystem.em[id]) {
        return moduleSystem.em[id];
      }
    }

    if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
      const res = moduleSystem.p.b("moduleNotFound", id);

      if (res) {
        return res
      }
    }

    // fallback to require if target env is node
    if (__FARM_RUNTIME_TARGET_ENV__ === 'node') {
      try {
        return require(id);
      } catch (e) {}
    }

    console.debug(`[Farm] Module "${id}" is not registered`);

    // return a empty module if the module is not registered
    return {};
  }

  // create a full new module instance and store it in cache to avoid cyclic initializing
  const module = cache[id] = {
    id,
    meta: {
      env: {}
    },
    exports: {},
    require: (moduleId: string) => farmRequire(moduleId)
  } as Module;

  if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
    // call the module created hook
    moduleSystem.p.s("moduleCreated", module);
  }

  cache[id] = module;

  // initialize the new module
  const result = initializer(
    module,
    module.exports,
    farmRequire,
    moduleSystem.d,
  );

  if (__FARM_ENABLE_TOP_LEVEL_AWAIT__) {
     // it's a async module, return the promise
    if (result && result instanceof Promise) {
      module.initializer = result.then(() => {
        if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
          // call the module initialized hook
          moduleSystem.p.s("moduleInitialized", module);
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
    moduleSystem.p.s("moduleInitialized", module);
  }
  // return the exports of the module
  return module.exports;
}

export function farmRegister(id: string, module: ModuleInitialization): () => any {
  if (modules[id] && !moduleSystem._rg) {
    console.warn(
      `Module "${id}" has registered! It should not be registered twice`,
    );
    return;
  }

  modules[id] = module;
  return () => farmRequire(id);
}
