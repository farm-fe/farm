/* eslint-disable @typescript-eslint/no-explicit-any */
interface Module {
  id: string;
  exports: any;
  meta: Record<string, any>;
  initialized: boolean;
}

interface Resource {
  id: string;
  path: string;
  loaded: boolean;
}

type ModuleInitialization = (
  module: Module,
  exports: any,
  __farm_require__: (moduleId: string) => any
  // polyfill for `import()`, TODO
  // __farm_load_resource__: (resourceId: string) => Promise<any>
) => void | Promise<void>;

// These global variables will be injected during compilation, and module system plugin may use these global variables.
// globalThis.__farm_module_system_resources__;
// globalThis.__farm_module_system_public_paths__;

globalThis.__acquire_farm_module_system__ =
  (function (/*resources, publicPaths*/) {
    class ModuleSystem {
      // all modules registered
      modules: Record<string, ModuleInitialization>;
      // module cache after module initialized
      cache: Record<string, Module>;
      // // resources generated during compilation
      // resources: Record<string, Resource>;
      // // available public paths, when loading resources, we will try each publicPath until it is available, this is so called `resource loading retry`
      // publicPaths: string[];

      constructor(/*resources: Record<string, Resource>, publicPaths: string[]*/) {
        this.modules = {};
        this.cache = {};
        // this.resources = resources;
        // this.publicPaths = publicPaths;
      }

      // require should be async as we support `top level await`
      async require(moduleId: string): Promise<any> {
        if (this.cache[moduleId]) {
          return this.cache[moduleId];
        }

        const initializer = this.modules[moduleId];

        if (!initializer) {
          throw new Error(`Module "${moduleId}" is not registered`);
        }

        const module: Module = {
          id: moduleId,
          exports: {},
          meta: {},
          initialized: false,
        };

        await initializer(module, module.exports, this.require.bind(this));

        module.initialized = true;
        this.cache[moduleId] = module;

        return module.exports;
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
        Promise.resolve().then(() => console.log(123));
        // update
      }

      // delete(moduleId: string): boolean {
      //   // delete
      // }

      // clearCache(moduleId: string): boolean {
      //   // clear cache
      // }
    }

    const moduleSystem = new ModuleSystem(/*resources, publicPaths*/);

    return function () {
      return moduleSystem;
    };
  })(/*
  globalThis.__farm_module_system_resources__,
  globalThis.__farm_module_system_public_paths__
*/);
