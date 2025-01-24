import type { ModuleSystem, Module } from '../module-system.js';
import type { Resource } from './dynamic-import.js';

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

export interface FarmRuntimePluginContainer {
  // setPlugins
  p(plugins: FarmRuntimePlugin[]): void;
  // hookSerial
  s(hookName: Exclude<keyof FarmRuntimePlugin, "name">, ...args: any[]): void
  // hookBail
  b<T = any>(hookName: Exclude<keyof FarmRuntimePlugin, "name">, ...args: any[]): T | undefined
}

let RUNTIME_PLUGINS: FarmRuntimePlugin[] = [];

function hookPluginsSerial(
  hookName: Exclude<keyof FarmRuntimePlugin, 'name'>,
  ...args: any[]
): // ): Promise<void> {
void {
  for (const plugin of RUNTIME_PLUGINS) {
    const hook = plugin[hookName];

    if (hook) {
      // await hook.apply(plugin, args);
      hook.apply(plugin, args);
    }
  }
}

function hookPluginsBail<T = any>(
  hookName: Exclude<keyof FarmRuntimePlugin, 'name'>,
  ...args: any[]
): // ): Promise<T> {
T | undefined {
  for (const plugin of RUNTIME_PLUGINS) {
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

function setPlugins(plugins: FarmRuntimePlugin[]) {
  RUNTIME_PLUGINS = plugins;
}

const pluginContainer: FarmRuntimePluginContainer = {
  p: setPlugins,
  s: hookPluginsSerial,
  b: hookPluginsBail
}


function bootstrap(): void {
  pluginContainer.s("bootstrap", this);
}


export function initModuleSystem(moduleSystem: ModuleSystem) {
  moduleSystem.p = pluginContainer;
  moduleSystem.b = bootstrap;
}
