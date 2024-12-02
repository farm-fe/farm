import { Module } from './module.js';
import type { ModuleSystem } from './module-system.js';
import type { Resource } from './resource-loader.js';

export interface ResourceLoadResult {
  success: boolean;
  retryWithDefaultResourceLoader?: boolean;
  err?: Error;
}

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
  // called when module is not found
  moduleNotFound?: (moduleId: string) => void | Promise<void>;
  // called when loading resources, custom your resource loading in this hook.
  // return { success: true } to indicate that this resources have been loaded successfully.
  // return { success: false, retryWithDefaultResourceLoader: true } to indicate that this resources have not been loaded successfully and should be retried with the default resource loader.
  loadResource?: (
    resource: Resource,
    targetEnv: 'browser' | 'node'
  ) => Promise<ResourceLoadResult>;
}

/* eslint-disable @typescript-eslint/no-explicit-any */
export class FarmRuntimePluginContainer {
  plugins: FarmRuntimePlugin[] = [];

  constructor(plugins: FarmRuntimePlugin[]) {
    this.plugins = plugins;
  }

  // TODO support async later
  // async hookSerial(
  hookSerial(
    hookName: Exclude<keyof FarmRuntimePlugin, 'name'>,
    ...args: any[]
  ): // ): Promise<void> {
  void {
    for (const plugin of this.plugins) {
      const hook = plugin[hookName];

      if (hook) {
        // await hook.apply(plugin, args);
        hook.apply(plugin, args);
      }
    }
  }

  // TODO support async later
  // async hookBail<T = any>(
  hookBail<T = any>(
    hookName: Exclude<keyof FarmRuntimePlugin, 'name'>,
    ...args: any[]
  ): // ): Promise<T> {
  T | undefined {
    for (const plugin of this.plugins) {
      const hook = plugin[hookName];

      if (hook) {
        // const result = await hook.apply(plugin, args);
        const result = hook.apply(plugin, args);

        if (result) {
          return result as T;
        }
      }
    }
    return undefined;
  }
}
