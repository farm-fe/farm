import type { Resource } from "./modules/dynamic-import.js";
import type { FarmRuntimePluginContainer } from "./modules/plugin.js";
declare const __FARM_RUNTIME_TARGET_ENV__: 'browser' | 'node' | 'library';
declare const __FARM_ENABLE_RUNTIME_PLUGIN__: boolean;
declare const __FARM_ENABLE_TOP_LEVEL_AWAIT__: boolean;
declare const __FARM_ENABLE_EXTERNAL_MODULES__: boolean;
declare let window: any;
export interface Module {
    id: string;
    exports?: any;
    initializer?: Promise<any> | undefined;
    resource_pot?: string;
    meta?: Record<string, any>;
    require?: (id: string) => any;
}
type ModuleInitializationFunction = (module: Module, exports: any, __farm_require__: (moduleId: string) => any, __farm_dynamic_require__: (moduleId: string) => any) => void | Promise<void>;
export type ModuleInitialization = ModuleInitializationFunction;
export interface ModuleSystem {
    _rg: boolean;
    p: FarmRuntimePluginContainer;
    em: Record<string, any>;
    r(id: string): any;
    g(id: string, module: ModuleInitialization): () => any;
    d(id: string): Promise<void>;
    m(): Record<string, ModuleInitialization>;
    c(): Record<string, Module>;
    u(moduleId: string, init: ModuleInitialization): void;
    e(moduleId: string): boolean;
    a(moduleId: string): boolean;
    l(moduleId: string, force?: boolean): Promise<any>;
    se(externalModules: Record<string, any>): void;
    si(resources: string[]): void;
    sd(dynamicResources: Resource[], dynamicModuleResourcesMap: Record<string, number[]>): void;
    sp(publicPaths: string[]): void;
    b(): void;
}
declare const __farm_global_this__: any;
declare const modules: Record<string, ModuleInitialization>;
declare const cache: Record<string, Module>;
export declare const moduleSystem: ModuleSystem;
export declare function farmRequire(id: string): any;
export declare function farmRegister(id: string, module: ModuleInitialization): () => any;
