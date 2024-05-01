import { ModuleSystem } from './module-system';
import { FarmRuntimePlugin } from './plugin';
import { __farm_global_this__ } from './resource-loader';

__farm_global_this__.__farm_module_system__ = (() => {
  const moduleSystem = new ModuleSystem();

  return () => moduleSystem;
})()();

export { ModuleSystem, FarmRuntimePlugin as Plugin };
