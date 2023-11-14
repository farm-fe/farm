import { Module } from './module';
import { FarmRuntimePlugin, FarmRuntimePluginContainer } from './plugin';
import { Resource, ResourceLoader, targetEnv } from './resource-loader';

declare const __farmNodeRequire: (moduleId: string) => any;

/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-ignore swc helpers does not have type definition
import { _interop_require_default } from '@swc/helpers/_/_interop_require_default';
// @ts-ignore swc helpers does not have type definition
import { _interop_require_wildcard } from '@swc/helpers/_/_interop_require_wildcard';
// @ts-ignore swc helpers does not have type definition
import { _export_star } from '@swc/helpers/_/_export_star';

const INTERNAL_MODULE_MAP: Record<string, any> = {
  '@swc/helpers/_/_interop_require_default': {
    default: _interop_require_default,
    _: _interop_require_default
  },
  '@swc/helpers/_/_interop_require_wildcard': {
    default: _interop_require_wildcard,
    _: _interop_require_wildcard
  },
  '@swc/helpers/_/_export_star': {
    default: _export_star,
    _: _export_star
  }
};

/* eslint-disable @typescript-eslint/no-explicit-any */
export type ModuleInitialization = (
  module: Module,
  exports: any,
  __farm_require__: (moduleId: string) => any,
  __farm_dynamic_require__: (moduleId: string) => any
) => void | Promise<void>;

export class ModuleSystem {
  // all modules registered
  private modules: Record<string, ModuleInitialization>;
  // module cache after module initialized
  private cache: Record<string, Module>;
  // all resources
  private resources: Map<string, string[]>;
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
    this.resources = new Map();
  }

  // TODO require should be async as we support `top level await`, This feature requires Node 16 and higher
  require(moduleId: string): any {
    if (INTERNAL_MODULE_MAP[moduleId]) {
      return INTERNAL_MODULE_MAP[moduleId];
    }

    // return the cached exports if cache exists
    // console.log(`[Farm] require module "${moduleId}" from cache`);
    if (this.cache[moduleId]) {
      const shouldSkip = this.pluginContainer.hookBail(
        'readModuleCache',
        this.cache[moduleId]
      );

      // console.log(`[Farm] shouldSkip: ${shouldSkip} ${moduleId}`);
      if (!shouldSkip) {
        return this.cache[moduleId].exports;
      }
    }

    const initializer = this.modules[moduleId];

    if (!initializer) {
      // if running on node, using native require to load node built-in modules
      if (targetEnv === 'node') {
        return __farmNodeRequire(moduleId);
      }
      // return a empty module if the module is not registered
      return {};
      // throw new Error(`Module "${moduleId}" is not registered`);
    }

    // create a full new module instance and store it in cache to avoid cyclic initializing
    const module = new Module(moduleId, this.require.bind(this));
    // call the module created hook
    this.pluginContainer.hookSerial('moduleCreated', module);

    this.cache[moduleId] = module;

    if (!(globalThis || global || window || {}).require) {
      (globalThis || global || window || { require: undefined }).require =
        this.require.bind(this);
    }
    // initialize the new module
    initializer(
      module,
      module.exports,
      this.require.bind(this),
      this.farmDynamicRequire.bind(this)
    );
    // call the module initialized hook
    this.pluginContainer.hookSerial('moduleInitialized', module);
    // return the exports of the module
    return module.exports;
  }

  farmDynamicRequire(moduleId: string): Promise<any> {
    if (this.modules[moduleId]) {
      const exports = this.require(moduleId);

      if (exports.__farm_async) {
        return exports.default;
      } else {
        return Promise.resolve(exports);
      }
    }

    const resources = this.dynamicModuleResourcesMap[moduleId];

    if (!resources || resources.length === 0) {
      throw new Error(
        `Dynamic imported module "${moduleId}" does not belong to any resource`
      );
    }

    // loading all required resources, and return the exports of the entry module
    return Promise.all(
      resources.map((resource) => this.resourceLoader.load(resource))
    )
      .then(() => {
        const result = this.require(moduleId);
        // if the module is async, return the default export, the default export should be a promise
        if (result.__farm_async) {
          return result.default;
        } else {
          return result;
        }
      })
      .catch((err) => {
        console.error(`[Farm] Error loading dynamic module "${moduleId}"`, err);
        throw err;
      });
  }

  register(
    moduleId: string,
    initializer: ModuleInitialization,
    resourcePotName: string
  ): void {
    // console.log(`[Farm] register module "${moduleId}"`, console.trace());
    if (this.modules[moduleId]) {
      // throw new Error(
      //   `Module "${moduleId}" has registered! It should not be registered twice`
      // );
      console.warn(
        `Module "${moduleId}" has registered! It should not be registered twice`
      );
      return;
    }

    this.modules[moduleId] = initializer;
    if (this.resources.has(resourcePotName)) {
      const modules = this.resources.get(resourcePotName);
      modules.push(moduleId);
      this.resources.set(resourcePotName, modules);
    } else {
      this.resources.set(resourcePotName, [moduleId]);
    }
  }

  update(moduleId: string, init: ModuleInitialization): void {
    this.modules[moduleId] = init;
    this.clearCache(moduleId);
  }

  delete(moduleId: string): boolean {
    if (this.modules[moduleId]) {
      this.cache[moduleId] && this.cache[moduleId].dispose?.();

      this.clearCache(moduleId);
      delete this.modules[moduleId];
      return true;
    } else {
      return false;
    }
  }

  getModuleUrl(moduleId: string): string {
    for (const [resource, modules] of this.resources) {
      if (modules.includes(moduleId)) {
        const publicPath = this.publicPaths[0];
        // @ts-ignore
        const url = `${window.FARM_HOST}:${window.FARM_PORT}${
          publicPath === '/' ? '' : publicPath
        }/${resource}`;
        return url;
      }
    }
  }

  getCache(moduleId: string): Module | undefined {
    return this.cache[moduleId];
  }

  clearCache(moduleId: string): boolean {
    if (this.cache[moduleId]) {
      delete this.cache[moduleId];
      return true;
    } else {
      return false;
    }
  }

  setInitialLoadedResources(resources: string[]) {
    for (const resource of resources) {
      this.resourceLoader.setLoadedResource(resource);
    }
  }

  // These two methods are used to support dynamic module loading, the dynamic module info is collected by the compiler and injected during compile time
  // This method can also be called during runtime to add new dynamic modules
  setDynamicModuleResourcesMap(
    dynamicModuleResourcesMap: Record<string, Resource[]>
  ): void {
    this.dynamicModuleResourcesMap = dynamicModuleResourcesMap;
  }

  // The public paths are injected during compile time
  // This method can also be called during runtime to add new public paths
  setPublicPaths(publicPaths: string[]): void {
    this.publicPaths = publicPaths;
    this.resourceLoader.publicPaths = this.publicPaths;
  }

  // The plugins are injected during compile time.
  // This method can also be called during runtime to add new plugins
  setPlugins(plugins: FarmRuntimePlugin[]): void {
    this.pluginContainer.plugins = plugins;
  }

  // bootstrap should be called after all three methods above are called, and the bootstrap call is also injected during compile time
  // This method should only be called once
  bootstrap(): void {
    this.pluginContainer.hookSerial('bootstrap', this);
  }
}
