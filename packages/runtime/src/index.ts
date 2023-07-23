import { ModuleSystem } from './module-system';
import { __farm_global_this__ } from './resource-loader';

__farm_global_this__.__farm_module_system__ = (function () {
  const moduleSystem = new ModuleSystem();

  return function () {
    return moduleSystem;
  };
})()();
