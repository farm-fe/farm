interface RawLazyCompileResult {
  mutableModules: string;
  immutableModules: string;
  dynamicResourcesMap: Record<string, any>;
}

interface LazyCompileResult {
  modules: Record<
    string,
    (module: any, exports: any, require: (id: string) => any) => void
  >;
}

const FarmModuleSystem: any = 'FARM_MODULE_SYSTEM';
const moduleId = `MODULE_ID`;
const modulePath = `MODULE_PATH`;

if (FarmModuleSystem.lazyCompiling === undefined) {
  FarmModuleSystem.lazyCompiling = false;
}
if (FarmModuleSystem.compilingModules === undefined) {
  FarmModuleSystem.compilingModules = new Map<string, Promise<any>>();
}
if (FarmModuleSystem.lazyCompilingQueue === undefined) {
  FarmModuleSystem.lazyCompilingQueue = [] as [
    string,
    string,
    (val: any) => void,
    Promise<void>
  ][];
}

const compilingModules = FarmModuleSystem.compilingModules;

let promise = Promise.resolve();

if (compilingModules.has(modulePath)) {
  promise = promise.then(() => compilingModules.get(modulePath));
} else {
  if (FarmModuleSystem.lazyCompiling) {
    const queueItem = FarmModuleSystem.lazyCompilingQueue.find(
      (m) => m[0] === modulePath
    );

    if (!queueItem) {
      let resolve: () => void = () => {
        throw new Error('Lazy compiling queue resolve not set');
      };
      promise = new Promise((r) => (resolve = r));
      compilingModules.set(modulePath, promise);
      FarmModuleSystem.lazyCompilingQueue.push([
        modulePath,
        moduleId,
        resolve,
        promise,
      ]);
    } else {
      promise = queueItem[2];
    }
  } else {
    const compileModules = (paths: string[]) => {
      FarmModuleSystem.lazyCompiling = true;
      const queue = [...FarmModuleSystem.lazyCompilingQueue];
      FarmModuleSystem.lazyCompilingQueue = [];

      const url = '/__lazy_compile?paths=' + paths.join(',') + `&t=${Date.now()}`;

      promise = import(url).then((module: any) => {
        const result: RawLazyCompileResult = module.default;
        
        FarmModuleSystem.dynamicModuleResourcesMap = result.dynamicResourcesMap;
        const mutableModules = eval(result.mutableModules);
        const immutableModules = eval(result.immutableModules);
        
        const modules = { ...mutableModules, ...immutableModules };

        for (const moduleId in modules) {
          FarmModuleSystem.update(moduleId, modules[moduleId]);
        }

        FarmModuleSystem.lazyCompiling = false;

        for (const path of paths) {
          compilingModules.delete(path);
          const queueItem = queue.find((item) => item[0] === path);

          if (queueItem) {
            const [, itemId, resolve] = queueItem;
            resolve(FarmModuleSystem.require(itemId));
          }
        }

        if (FarmModuleSystem.lazyCompilingQueue.length > 0) {
          compileModules(FarmModuleSystem.lazyCompilingQueue.map((m) => m[0]));
        }

        // The lazy compiled module should not contains side effects, as it may be executed twice
        const exports = FarmModuleSystem.require(moduleId);

        return exports;
      });

      for (const path of paths) {
        compilingModules.set(path, promise);
      }
    };

    const paths = [modulePath];
    compileModules(paths);
  }
}

export const __farm_async = true;
export default promise;
