import { ModuleSystem } from './module-system';

type FarmGlobalThis = (Window &
  typeof globalThis &
  typeof global &
  typeof self) & {
  noop: () => void;
  __farm_module_system__: ModuleSystem;
};

const __farm_global_this__ = (globalThis ||
  window ||
  global ||
  self) as FarmGlobalThis;

__farm_global_this__.noop = function () {
  /* do nothing */
};

__farm_global_this__.__farm_module_system__ = (function () {
  const moduleSystem = new ModuleSystem();

  return function () {
    return moduleSystem;
  };
})()();
