import { Module } from './module';
import type { ModuleSystem } from './module-system';

export interface FarmRuntimePlugin {
  // plugin name
  name: string;
  // invoked when the module system is bootstrapped
  bootstrap: (moduleSystem: ModuleSystem) => void | Promise<void>;
  // invoked when new module instances are created
  createModule: (module: Module) => void | Promise<void>;
  // invoked when module initialization functions are called
  initializeModule: (module: Module) => void | Promise<void>;
}
