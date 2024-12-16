import { ModuleSystem } from './module-system.js';
import { FarmRuntimePlugin } from './plugin.js';
import { __farm_global_this__ } from './resource-loader.js';

__farm_global_this__.__farm_module_system__ = (function () {
  const moduleSystem = new ModuleSystem();

  return function () {
    return moduleSystem;
  };
})()();

export * from './resource-loader.js'

export { ModuleSystem, FarmRuntimePlugin as Plugin };
