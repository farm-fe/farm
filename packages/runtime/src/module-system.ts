import { Module } from "./module";
import { type FarmRuntimePlugin, FarmRuntimePluginContainer } from "./plugin";
import {
  type Resource,
  ResourceLoader,
  isBrowser,
  targetEnv,
} from "./resource-loader";

declare const nodeRequire: (id: string) => any;

/* eslint-disable @typescript-eslint/no-explicit-any */
type ModuleInitializationFunction = (
  module: Module,
  exports: any,
  __farm_require__: (moduleId: string) => any,
  __farm_dynamic_require__: (moduleId: string) => any,
) => void | Promise<void>;

/* eslint-disable @typescript-eslint/no-explicit-any */
export type ModuleInitialization = ModuleInitializationFunction & {
  __farm_resource_pot__?: string;
};

export class ModuleSystem {
  // all modules registered
  private modules: Record<string, ModuleInitialization>;
  // module cache after module initialized
  private cache: Record<string, Module>;
  // external modules injected during compile
  private externalModules: Record<string, any>;
  private reRegisterModules: boolean;
  // available public paths, when loading resources, we will try each publicPath until it is available, this is so called `resource loading retry`
  publicPaths: string[];
  dynamicResources: Resource[];
  // dynamic module entry and resources map
  dynamicModuleResourcesMap: Record<string, number[]>;
  // resources loader
  resourceLoader: ResourceLoader;
  // runtime plugin container
  pluginContainer: FarmRuntimePluginContainer;
  targetEnv: "browser" | "node";

  constructor() {
    this.modules = {};
    this.cache = {};
    this.publicPaths = [];
    this.dynamicModuleResourcesMap = {};
    this.resourceLoader = new ResourceLoader(this, this.publicPaths);
    this.pluginContainer = new FarmRuntimePluginContainer([]);
    this.targetEnv = targetEnv;
    this.externalModules = {};
    this.reRegisterModules = false;
  }

  require(moduleId: string, isCJS = false): any {
    // return the cached exports if cache exists
    // console.log(`[Farm] require module "${moduleId}" from cache`);
    if (this.cache[moduleId]) {
      const shouldSkip = this.pluginContainer.hookBail(
        "readModuleCache",
        this.cache[moduleId],
      );

      // console.log(`[Farm] shouldSkip: ${shouldSkip} ${moduleId}`);
      if (!shouldSkip) {
        return this.cache[moduleId].exports;
      }
    }

    const initializer = this.modules[moduleId];

    if (!initializer) {
      if (this.externalModules[moduleId]) {
        const exports = this.externalModules[moduleId];

        // fix `const assert = require('assert');` when assert is external. This leads to `assert is not a function` error caused by default export different from esm and cjs
        if (isCJS) {
          return exports.default || exports;
        }

        return exports;
      }
      // try node require if target Env is node
      if ((this.targetEnv === "node" || !isBrowser) && nodeRequire) {
        const externalModule = nodeRequire(moduleId);
        return externalModule;
      }
      this.pluginContainer.hookSerial("moduleNotFound", moduleId);
      // return a empty module if the module is not registered
      console.debug(`[Farm] Module "${moduleId}" is not registered`);
      return {};
      // throw new Error(`Module "${moduleId}" is not registered`);
    }

    // create a full new module instance and store it in cache to avoid cyclic initializing
    const module = new Module(moduleId, this.require.bind(this));
    module.resource_pot = initializer.__farm_resource_pot__;
    // call the module created hook
    this.pluginContainer.hookSerial("moduleCreated", module);

    this.cache[moduleId] = module;

    if (!(globalThis || global || window || {}).require) {
      (globalThis || global || window || { require: undefined }).require =
        this.require.bind(this);
    }
    // initialize the new module
    const result = initializer(
      module,
      module.exports,
      this.require.bind(this),
      this.farmDynamicRequire.bind(this),
    );

    // it's a async module, return the promise
    if (result && result instanceof Promise) {
      return result.then(() => {
        // call the module initialized hook
        this.pluginContainer.hookSerial("moduleInitialized", module);
        // return the exports of the module
        return module.exports;
      });
    } else {
      // call the module initialized hook
      this.pluginContainer.hookSerial("moduleInitialized", module);
      // return the exports of the module
      return module.exports;
    }
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

    return this.loadDynamicResources(moduleId);
  }

  loadDynamicResources(moduleId: string, force = false): Promise<any> {
    const resources = this.dynamicModuleResourcesMap[moduleId].map((index) => this.dynamicResources[index]);

    if (!resources || resources.length === 0) {
      throw new Error(
        `Dynamic imported module "${moduleId}" does not belong to any resource`,
      );
    }
    // force reload resources
    if (force) {
      this.reRegisterModules = true;
      this.clearCache(moduleId);
    }
    // loading all required resources, and return the exports of the entry module
    return Promise.all(
      resources.map((resource) => {
        if (force) {
          this.resourceLoader.setLoadedResource(resource.path, false);
        }
        return this.resourceLoader.load(resource);
      }),
    )
      .then(() => {
        if (!this.modules[moduleId]) {
          throw new Error(
            `Dynamic imported module "${moduleId}" is not registered.`,
          );
        }
        this.reRegisterModules = false;
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

  register(moduleId: string, initializer: ModuleInitialization): void {
    // console.log(`[Farm] register module "${moduleId}"`, console.trace());
    if (this.modules[moduleId] && !this.reRegisterModules) {
      // throw new Error(
      //   `Module "${moduleId}" has registered! It should not be registered twice`
      // );
      console.warn(
        `Module "${moduleId}" has registered! It should not be registered twice`,
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
      this.clearCache(moduleId);
      delete this.modules[moduleId];
      return true;
    } else {
      return false;
    }
  }

  getModuleUrl(moduleId: string): string {
    const publicPath = this.publicPaths[0] ?? "";

    if (isBrowser) {
      const url = `${window.location.protocol}//${window.location.host}${
        publicPath.endsWith("/") ? publicPath.slice(0, -1) : publicPath
      }/${this.modules[moduleId].__farm_resource_pot__}`;
      return url;
    } else {
      return this.modules[moduleId].__farm_resource_pot__;
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
    dynamicResources: Resource[],
    dynamicModuleResourcesMap: Record<string, number[]>,
  ): void {
    this.dynamicResources = dynamicResources;
    this.dynamicModuleResourcesMap = dynamicModuleResourcesMap;
  }

  // The public paths are injected during compile time
  setPublicPaths(publicPaths: string[]): void {
    this.publicPaths = publicPaths;
    this.resourceLoader.publicPaths = this.publicPaths;
  }

  // The plugins are injected during compile time.
  setPlugins(plugins: FarmRuntimePlugin[]): void {
    this.pluginContainer.plugins = plugins;
  }
  // This method can be called during runtime to add new plugins
  addPlugin(plugin: FarmRuntimePlugin): void {
    if (this.pluginContainer.plugins.every((p) => p.name !== plugin.name)) {
      this.pluginContainer.plugins.push(plugin);
    }
  }
  // This method can be called during runtime to remove plugins
  removePlugin(pluginName: string): void {
    this.pluginContainer.plugins = this.pluginContainer.plugins.filter(
      (p) => p.name !== pluginName,
    );
  }

  // The external modules are injected during compile time.
  setExternalModules(externalModules: Record<string, any>): void {
    Object.assign(this.externalModules, externalModules || {});
  }

  // bootstrap should be called after all three methods above are called, and the bootstrap call is also injected during compile time
  // This method should only be called once
  bootstrap(): void {
    this.pluginContainer.hookSerial("bootstrap", this);
  }
}
