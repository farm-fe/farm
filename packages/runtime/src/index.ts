import { ModuleSystem } from './module-system';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const __farm_global_this__: any = globalThis || window || global || self;

__farm_global_this__.noop = function () {
  /* do nothing */
};

__farm_global_this__.__farm_module_system__ = (function () {
  const moduleSystem = new ModuleSystem();

  return function () {
    return moduleSystem;
  };
})()();
