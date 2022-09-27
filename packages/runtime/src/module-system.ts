import { Module } from './module';
import { Resource } from './resource-loader';

/* eslint-disable @typescript-eslint/no-explicit-any */
type ModuleInitialization = (
  module: Module,
  exports: any,
  __farm_require__: (moduleId: string) => any,
  __farm_dynamic_require__: (moduleId: string) => any
) => void | Promise<void>;

export class ModuleSystem {
  // all modules registered
  modules: Record<string, ModuleInitialization>;
  // module cache after module initialized
  cache: Record<string, Module>;
  // available public paths, when loading resources, we will try each publicPath until it is available, this is so called `resource loading retry`
  publicPaths: string[];
  // dynamic module entry and resources map
  dynamic_module_resources_map: Record<string, Resource[]>;

  constructor() {
    this.modules = {};
    this.cache = {};
    this.publicPaths = [];
    this.dynamic_module_resources_map = {};
  }

  // require should be async as we support `top level await`
  // This feature requires Node 16 and higher
  async require(moduleId: string): Promise<any> {
    // return the cached exports if cache exists
    if (this.cache[moduleId]) {
      return this.cache[moduleId].exports;
    }

    const initializer = this.modules[moduleId];

    if (!initializer) {
      throw new Error(`Module "${moduleId}" is not registered`);
    }

    // create a full new module instance and store it in cache to avoid cyclic initializing
    const module = new Module(moduleId);
    this.cache[moduleId] = module;
    // initialize the new module
    await initializer(
      module,
      module.exports,
      this.require.bind(this),
      this.dynamicRequire.bind(this)
    );
    module.initialized = true;

    // return the exports of the module
    return module.exports;
  }

  async dynamicRequire(moduleId: string): Promise<any> {
    const resources = this.dynamic_module_resources_map[moduleId];

    if (!resources || resources.length === 0) {
      throw new Error(
        `Dynamic imported module "${module}" does not belong to any resource`
      );
    }
  }

  register(moduleId: string, initializer: ModuleInitialization): void {
    if (this.modules[moduleId]) {
      throw new Error(
        `Module "${moduleId}" has registered! It should not be registered twice`
      );
    }

    this.modules[moduleId] = initializer;
  }

  // update(moduleId: string, init: ModuleInitialization): void {
  //   Promise.resolve().then(() => console.log(123));
  //   // update
  // }

  // delete(moduleId: string): boolean {
  //   // delete
  // }

  // clearCache(moduleId: string): boolean {
  //   // clear cache
  // }

  setDynamicModuleResourcesMap(
    dynamic_module_resources_map: Record<string, Resource[]>
  ): void {
    this.dynamic_module_resources_map = dynamic_module_resources_map;
  }

  setPublicPaths(publicPaths: string[]): void {
    this.publicPaths = publicPaths;
  }
}
