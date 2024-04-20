interface RawLazyCompileResult {
  mutableModules: string;
  immutableModules: string;
  dynamicResourcesMap: Record<string, any>;
}

const FarmModuleSystem: any = 'FARM_MODULE_SYSTEM';
const moduleId = `MODULE_ID`;
const modulePath = `MODULE_PATH`;
const serverUrl = 'FARM_NODE_LAZY_COMPILE_SERVER_URL';

/**
 * If the serverUrl is 'FARM_NODE_LAZY_COMPILE_SERVER_URL', it means the serverUrl is not set and it's not node lazy compile, and we should think it's a browser lazy compile.
 * FARM_NODE_LAZY_COMPILE_SERVER_URL will be replaced by the real server url during the build process when node lazy compile is enabled.
 */
const isNodeLazyCompile =
  serverUrl !== 'FARM_NODE_LAZY' + '_COMPILE_SERVER_URL';

const debouncePageReload = function (ms: number) {
  let timer;

  return function () {
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (window && window.location && window.location.reload) {
        window.location.reload();
      }
    }, ms);
  };
};
const pageReload = debouncePageReload(500);

async function fetch(path: string) {
  const url = `${serverUrl}${path}`;
  const http = 'http';
  return import(http).then((http) => {
    return new Promise((resolve, reject) => {
      http.get(url, (res: any) => {
        let data = '';
        res.on('data', (chunk: any) => {
          data += chunk;
        });
        res.on('end', () => {
          resolve(JSON.parse(data));
        });
        res.on('error', (err: any) => {
          reject(err);
        });
      });
    });
  });
}

const lazyCompilationRuntimePlugin = {
  name: 'farm-lazy-compilation',
  moduleNotFound: () => {
    // reload the page if the module is not found
    pageReload();
  }
};

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
        promise
      ]);
    } else {
      promise = queueItem[2];
    }
  } else {
    const compileModules = (paths: string[]) => {
      FarmModuleSystem.lazyCompiling = true;
      const queue = [...FarmModuleSystem.lazyCompilingQueue];
      FarmModuleSystem.lazyCompilingQueue = [];

      const url = `/__lazy_compile?paths=${paths.join(',')}&t=${Date.now()}${
        isNodeLazyCompile ? '&node=true' : ''
      }`;

      const fetchLazyCompileResult = !isNodeLazyCompile
        ? import(url)
        : fetch(url);

      promise = fetchLazyCompileResult.then((module: any) => {
        const result: RawLazyCompileResult = module.default || module;

        if (result.dynamicResourcesMap) {
          FarmModuleSystem.dynamicModuleResourcesMap =
            result.dynamicResourcesMap;
        }

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

        // fix #878
        !isNodeLazyCompile &&
          FarmModuleSystem.addPlugin(lazyCompilationRuntimePlugin);
        // The lazy compiled module should not contains side effects, as it may be executed twice
        const exports = FarmModuleSystem.require(moduleId);
        !isNodeLazyCompile &&
          FarmModuleSystem.removePlugin(lazyCompilationRuntimePlugin.name);

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
