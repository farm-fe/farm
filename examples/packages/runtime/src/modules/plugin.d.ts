import type { ModuleSystem, Module } from '../module-system.js';
import type { Resource } from './dynamic-import.js';
export interface ResourceLoadResult {
    success: boolean;
    retryWithDefaultResourceLoader?: boolean;
    err?: Error;
}
export interface FarmRuntimePlugin {
    name: string;
    bootstrap?: (moduleSystem: ModuleSystem) => void | Promise<void>;
    moduleCreated?: (module: Module) => void | Promise<void>;
    moduleInitialized?: (module: Module) => void | Promise<void>;
    readModuleCache?: (module: Module) => boolean | Promise<boolean>;
    moduleNotFound?: (moduleId: string) => void | Promise<void>;
    loadResource?: (resource: Resource, targetEnv: 'browser' | 'node') => Promise<ResourceLoadResult>;
}
export interface FarmRuntimePluginContainer {
    p(plugins: FarmRuntimePlugin[]): void;
    s(hookName: Exclude<keyof FarmRuntimePlugin, "name">, ...args: any[]): void;
    b<T = any>(hookName: Exclude<keyof FarmRuntimePlugin, "name">, ...args: any[]): T | undefined;
}
declare let RUNTIME_PLUGINS: FarmRuntimePlugin[];
declare function hookPluginsSerial(hookName: Exclude<keyof FarmRuntimePlugin, 'name'>, ...args: any[]): void;
declare function hookPluginsBail<T = any>(hookName: Exclude<keyof FarmRuntimePlugin, 'name'>, ...args: any[]): T | undefined;
declare function setPlugins(plugins: FarmRuntimePlugin[]);
declare const pluginContainer: FarmRuntimePluginContainer;
declare function bootstrap(): void;
export declare function initModuleSystem(moduleSystem: ModuleSystem);
