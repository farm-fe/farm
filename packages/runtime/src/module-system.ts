import { Module } from './module';
import { FarmRuntimePlugin, FarmRuntimePluginContainer } from './plugin';
import { Resource, ResourceLoader, targetEnv } from './resource-loader';

/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-ignore swc helpers does not have type definition
import interopRequireDefault from '@swc/helpers/lib/_interop_require_default.js';
// @ts-ignore swc helpers does not have type definition
import interopRequireWildcard from '@swc/helpers/lib/_interop_require_wildcard.js';
// @ts-ignore swc helpers does not have type definition
import exportStar from '@swc/helpers/lib/_export_star.js';

const INTERNAL_MODULE_MAP: Record<string, any> = {
  '@swc/helpers/lib/_interop_require_default.js': interopRequireDefault,
  '@swc/helpers/lib/_interop_require_wildcard.js': interopRequireWildcard,
  '@swc/helpers/lib/_export_star.js': exportStar,
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
    console.log(moduleId, INTERNAL_MODULE_MAP[moduleId], INTERNAL_MODULE_MAP);

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

    // if running on node, using native require to load node built-in modules
    if (targetEnv === 'node') {
      const { __farmNodeRequire, __farmNodeBuiltinModules } =
        // TODO: polyfill globalThis
        globalThis as unknown as {
          __farmNodeRequire: (id: string) => any;
          __farmNodeBuiltinModules: string[];
        };

      if (moduleId.startsWith('node:')) {
        const nodeModuleId = moduleId.slice(5);
        return __farmNodeRequire(nodeModuleId);
      } else if (__farmNodeBuiltinModules.includes(moduleId)) {
        return __farmNodeRequire(moduleId);
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

  dynamicRequire(moduleId: string): Promise<any> {
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
    ).then(() => {
      const result = this.require(moduleId);
      // if the module is async, return the default export, the default export should be a promise
      if (result.__farm_async) {
        return result.default;
      } else {
        return result;
      }
    });
  }

  register(moduleId: string, initializer: ModuleInitialization): void {
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
