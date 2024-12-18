import type { Module } from "./module.js";

// Injected during compile time
declare const __FARM_ENABLE_RUNTIME_PLUGIN__: boolean;

/* eslint-disable @typescript-eslint/no-explicit-any */
type ModuleInitializationFunction = (
  module: Module,
  exports: any,
  __farm_require__: (moduleId: string) => any,
  __farm_dynamic_require__: (moduleId: string) => any,
) => void | Promise<void>;

export type ModuleInitialization = ModuleInitializationFunction & {
  __farm_resource_pot__?: string;
};

export interface ModuleSystem {
  // all modules registered
  modules: Record<string, ModuleInitialization>;
  // module cache after module initialized
  cache: Record<string, Module>;

  reRegisterModules?: boolean;
}

export const moduleSystem: ModuleSystem = {
  modules: {},
  cache: {}
};

export function require(id: string): any {
  if (moduleSystem.cache[id]) {
    const cachedModuleResult = moduleSystem.cache[id].initializer || moduleSystem.cache[id].exports;
    // will be removed as dead code if no plugin enabled
    if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
      const shouldSkip = pluginContainer.hookBail(
        "readModuleCache",
        moduleSystem.cache[id],
      );
  
      // console.log(`[Farm] shouldSkip: ${shouldSkip} ${moduleId}`);
      if (!shouldSkip) {
        return cachedModuleResult;
      }
    } else {
      return cachedModuleResult;
    }
  }

  const initializer = moduleSystem.modules[id];

  const module = moduleSystem.cache[id] = {
    id,
    require: (moduleId: string) => require(moduleId)
  } as Module;

  const moduleInitialization = moduleSystem.modules[id];
  const moduleExports = module.exports;

  return moduleExports;
}

export function register(id: string, module: ModuleInitialization): () => any {
  moduleSystem.modules[id] = module;
  return () => require(id);
}