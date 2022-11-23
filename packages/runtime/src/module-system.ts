import { Module } from './module';
import { FarmRuntimePlugin, FarmRuntimePluginContainer } from './plugin';
import { Resource, ResourceLoader } from './resource-loader';

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
  dynamicModuleResourcesMap: Record<string, Resource[]>;
  // resources loader
  resourceLoader: ResourceLoader;
  // runtime plugin container
  pluginContainer: FarmRuntimePluginContainer;

  constructor() {
    this.modules = {};
    this.cache = {};
    this.publicPaths = [];
    this.dynamicModuleResourcesMap = {};
    this.resourceLoader = new ResourceLoader(this.publicPaths);
    this.pluginContainer = new FarmRuntimePluginContainer([]);
  }

  // TODO require should be async as we support `top level await`, This feature requires Node 16 and higher
  require(moduleId: string): any {
    // return the cached exports if cache exists
    console.log(`[Farm] require module "${moduleId}" from cache`);
    if (this.cache[moduleId]) {
      const shouldSkip = this.pluginContainer.hookBail(
        'readModuleCache',
        this.cache[moduleId]
      );

      console.log(`[Farm] shouldSkip: ${shouldSkip} ${moduleId}`);
      if (!shouldSkip) {
        return this.cache[moduleId].exports;
      }
    }

    const initializer = this.modules[moduleId];

    if (!initializer) {
      throw new Error(`Module "${moduleId}" is not registered`);
    }

    // create a full new module instance and store it in cache to avoid cyclic initializing
    const module = new Module(moduleId);
    // call the module created hook
    this.pluginContainer.hookSerial('moduleCreated', module);

    this.cache[moduleId] = module;
    // initialize the new module
    initializer(
      module,
      module.exports,
      this.require.bind(this),
      this.dynamicRequire.bind(this)
    );
    // call the module initialized hook
    this.pluginContainer.hookSerial('moduleInitialized', module);
    // return the exports of the module
    return module.exports;
  }

  async dynamicRequire(moduleId: string): Promise<any> {
    const resources = this.dynamicModuleResourcesMap[moduleId];

    if (!resources || resources.length === 0) {
      throw new Error(
        `Dynamic imported module "${module}" does not belong to any resource`
      );
    }

    // loading all required resources, and return the exports of the entry module
    await Promise.all(
      resources.map((resource) => this.resourceLoader.load(resource))
    );
    return this.require(moduleId);
  }

  register(moduleId: string, initializer: ModuleInitialization): void {
    if (this.modules[moduleId]) {
      throw new Error(
        `Module "${moduleId}" has registered! It should not be registered twice`
      );
    }

    this.modules[moduleId] = initializer;
  }

  update(moduleId: string, init: ModuleInitialization): void {
    this.modules[moduleId] = init;
  }

  delete(moduleId: string): boolean {
    if (this.modules[moduleId]) {
      delete this.modules[moduleId];
      return true;
    } else {
      return false;
    }
  }

  clearCache(moduleId: string): boolean {
    if (this.modules[moduleId]) {
      delete this.cache[moduleId];
      return true;
    } else {
      return false;
    }
  }

  // These two methods are used to support dynamic module loading, the dynamic module info is collected by the compiler and injected during compile time
  setDynamicModuleResourcesMap(
    dynamicModuleResourcesMap: Record<string, Resource[]>
  ): void {
    this.dynamicModuleResourcesMap = dynamicModuleResourcesMap;
  }

  setPublicPaths(publicPaths: string[]): void {
    this.publicPaths = publicPaths;
    this.resourceLoader.publicPaths = this.publicPaths;
  }

  // The plugins are injected during compile time.
  setPlugins(plugins: FarmRuntimePlugin[]): void {
    this.pluginContainer.plugins = plugins;
  }

  // bootstrap should be called after all three methods above are called
  bootstrap(): void {
    this.pluginContainer.hookSerial('bootstrap', this);
  }
}
