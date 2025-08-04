import type { Resource } from "./modules/dynamic-import.js";
import type { FarmRuntimePluginContainer } from "./modules/plugin.js";

// Injected during compile time, and the if statement will be removed during compile time
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
  __farm_require__?: (moduleId: string) => any,
  __farm_dynamic_require__?: (moduleId: string) => any,
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

function setGlobalRequire(globalThis: any) {
  // polyfill require when running in browser or node with Farm runtime
  const __global_this__: any = typeof globalThis !== 'undefined' ? globalThis : {};
  __global_this__.require = __global_this__.require || farmRequire;
}

// It will be removed if __FARM_RUNTIME_TARGET_ENV__ is not browser when building runtime 
if (__FARM_RUNTIME_TARGET_ENV__ === 'browser') {
  setGlobalRequire(window);  
}
if (__FARM_RUNTIME_TARGET_ENV__ === 'node') {
  setGlobalRequire(global);
}

// all modules registered
const __farm_internal_modules__: Record<string, ModuleInitialization> = {};
// module cache after module initialized
const __farm_internal_cache__: Record<string, Module> = {};

export var __farm_internal_module_system__ = {
  r: farmRequire,
  g: farmRegister,
  m: () => __farm_internal_modules__,
  c: () => __farm_internal_cache__,
} as ModuleSystem;

if (__FARM_ENABLE_EXTERNAL_MODULES__) {
  // externalModules
  __farm_internal_module_system__.em = {}
  // The external modules are injected during compile time.
  __farm_internal_module_system__.se = function setExternalModules(externalModules: Record<string, any>): void {
    for (const key in externalModules) {
      let em = externalModules[key];
      // add a __esModule flag to the module if the module has default export
      if (em && em.default && !em.__esModule) {
        em = Object.assign({}, em, { __esModule: true });  
      }

      __farm_internal_module_system__.em[key]= em;
    }
  }
  // init `window['xxxx] = {}`
  const __farm_global_this__: any = $__farm_global_this__$ = {};
  __farm_global_this__.m = __farm_internal_module_system__;
}

export function farmRequire(id: string): any {
  if (__farm_internal_cache__[id]) {
    if (__FARM_RUNTIME_TARGET_ENV__ === 'library') var cachedModuleResult =__farm_internal_cache__[id].exports;
    else var cachedModuleResult = __farm_internal_cache__[id].initializer || __farm_internal_cache__[id].exports;
    // will be removed as dead code if no plugin enabled when minify enabled
    if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
      const shouldSkip = __farm_internal_module_system__.p.b(
        "readModuleCache",
        __farm_internal_cache__[id],
      );
  
      // console.log(`[Farm] shouldSkip: ${shouldSkip} ${moduleId}`);
      if (!shouldSkip) return cachedModuleResult;
    } else return cachedModuleResult;
  }

  const initializer = __farm_internal_modules__[id];

  if (!initializer) {
    if (__FARM_ENABLE_EXTERNAL_MODULES__) {
      // externalModules
      if (__farm_internal_module_system__.em[id]) {
        return __farm_internal_module_system__.em[id];
      }
    }

    if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
      const res = __farm_internal_module_system__.p.b("moduleNotFound", id);

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
  const module = __farm_internal_cache__[id] = {
    id,
    meta: {
      env: {}
    },
    exports: {},
    require: (moduleId: string) => farmRequire(moduleId)
  } as Module;

  if (__FARM_ENABLE_RUNTIME_PLUGIN__) __farm_internal_module_system__.p.s("moduleCreated", module); // call the module created hook

  __farm_internal_cache__[id] = module;
  
  // initialize the new module
  if (__FARM_RUNTIME_TARGET_ENV__ === 'library') initializer(module, module.exports)
  else if (__FARM_ENABLE_TOP_LEVEL_AWAIT__) {
    const result = initializer(
      module,
      module.exports,
      farmRequire,
      __farm_internal_module_system__.d,
    );
     // it's a async module, return the promise
    if (result && result instanceof Promise) {
      module.initializer = result.then(() => {
        if (__FARM_ENABLE_RUNTIME_PLUGIN__) __farm_internal_module_system__.p.s("moduleInitialized", module); // call the module initialized hook

        module.initializer = undefined;
        // return the exports of the module
        return module.exports;
      });

      return module.initializer;
    }
  } else initializer(
      module,
      module.exports,
      farmRequire,
      __farm_internal_module_system__.d,
    );

  if (__FARM_ENABLE_RUNTIME_PLUGIN__) __farm_internal_module_system__.p.s("moduleInitialized", module);  // call the module initialized hook
  
  // return the exports of the module
  return module.exports;
}

export function farmRegister(id: string, module: ModuleInitialization): () => any {
  if (__FARM_RUNTIME_TARGET_ENV__ !== 'library') if (__farm_internal_modules__[id] && !__farm_internal_module_system__._rg) {
      console.warn(
        `Module "${id}" has registered! It should not be registered twice`,
      );
      return;
    }

  __farm_internal_modules__[id] = module;
  return () => farmRequire(id);
}
