import { Module } from './module';
import type { ModuleSystem } from './module-system';

export interface FarmRuntimePlugin {
  // plugin name
  name: string;
  // invoked when the module system is bootstrapped
  bootstrap?: (moduleSystem: ModuleSystem) => void | Promise<void>;
  // invoked after new module instances are created
  moduleCreated?: (module: Module) => void | Promise<void>;
  // invoked after module initialization functions are called
  moduleInitialized?: (module: Module) => void | Promise<void>;
  // invoked after module caches are read, return true to skip cache reading
  readModuleCache?: (module: Module) => boolean | Promise<boolean>;
}

export class FarmRuntimePluginContainer {
  plugins: FarmRuntimePlugin[] = [];

  constructor(plugins: FarmRuntimePlugin[]) {
    this.plugins = plugins;
  }

  async hookSerial(
    hookName: Exclude<keyof FarmRuntimePlugin, 'name'>,
    ...args: any[]
  ): Promise<void> {
    for (const plugin of this.plugins) {
      const hook = plugin[hookName];

      if (hook) {
        await hook.apply(plugin, args);
      }
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  async hookBail(
    hookName: Exclude<keyof FarmRuntimePlugin, 'name'>,
    ...args: any[]
  ): Promise<any> {
    for (const plugin of this.plugins) {
      const hook = plugin[hookName];

      if (hook) {
        const result = await hook.apply(plugin, args);

        if (result) {
          return result;
        }
      }
    }
  }
}
