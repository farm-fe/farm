import { ModuleSystem } from './module-system';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let __farm_globalThis: any;

if (globalThis) {
  __farm_globalThis = globalThis;
} else if (window) {
  __farm_globalThis = window;
} else if (global) {
  __farm_globalThis = global;
}

__farm_globalThis.noop = function () {
  /* do nothing */
};

__farm_globalThis.__acquire_farm_module_system__ = (function () {
  const moduleSystem = new ModuleSystem();

  return function () {
    return moduleSystem;
  };
})();
