interface RawLazyCompileResult {
  dynamicResourcesMap: Record<string, any>;
}

interface LazyCompilationQueueItem {
  modulePath: string;
  moduleId: string;
  resolve: (data: any) => void;
  promise: Promise<void>;
}

const FarmModuleSystem: any = 'FARM_MODULE_SYSTEM';
const moduleId = 'MODULE_ID';
const modulePath = 'MODULE_PATH';
const serverUrl = 'FARM_NODE_LAZY_COMPILE_SERVER_URL';

/**
 * If the serverUrl is 'FARM_NODE_LAZY_COMPILE_SERVER_URL', it means the serverUrl is not set and it's not node lazy compile, and we should think it's a browser lazy compile.
 * FARM_NODE_LAZY_COMPILE_SERVER_URL will be replaced by the real server url during the build process when node lazy compile is enabled.
 */
const isNodeLazyCompile =
  serverUrl !== 'FARM_NODE_LAZY' + '_COMPILE_SERVER_URL';

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

if (FarmModuleSystem.lazyCompiling === undefined) {
  FarmModuleSystem.lazyCompiling = false;
}
if (FarmModuleSystem.compilingModules === undefined) {
  FarmModuleSystem.compilingModules = new Map<string, Promise<any>>();
}
if (FarmModuleSystem.lazyCompilingQueue === undefined) {
  FarmModuleSystem.lazyCompilingQueue = [] as LazyCompilationQueueItem[];
}

const compilingModules = FarmModuleSystem.compilingModules;

let promise = Promise.resolve();

function queueLazyCompilation() {
  let resolve: () => void = () => {
    throw new Error('Lazy compiling queue resolve not set');
  };
  promise = new Promise((r) => (resolve = r));
  compilingModules.set(modulePath, promise);
  FarmModuleSystem.lazyCompilingQueue.push({
    modulePath,
    moduleId,
    resolve,
    promise
  } as LazyCompilationQueueItem);
}

if (compilingModules.has(modulePath)) {
  promise = promise.then(() => compilingModules.get(modulePath));
} else {
  if (FarmModuleSystem.lazyCompiling) {
    const queueItem = FarmModuleSystem.lazyCompilingQueue.find(
      (m) => m.modulePath === modulePath
    );

    if (!queueItem) {
      queueLazyCompilation();
    } else {
      promise = queueItem.promise;
    }
  } else {
    const compileModules = () => {
      FarmModuleSystem.lazyCompiling = true;
      const queue = [...FarmModuleSystem.lazyCompilingQueue];
      FarmModuleSystem.lazyCompilingQueue = [];
      const paths = queue.map((item) => item.modulePath);
      const url = `/__lazy_compile?paths=${encodeURIComponent(paths.join(','))}&t=${Date.now()}${
        isNodeLazyCompile ? '&node=true' : ''
      }`;

      const fetchLazyCompileResult = !isNodeLazyCompile
        ? import(url)
        : fetch(url);

      return fetchLazyCompileResult.then((module: any) => {
        const result: RawLazyCompileResult = module.default || module;

        if (result.dynamicResourcesMap) {
          FarmModuleSystem.setDynamicModuleResourcesMap(
            result.dynamicResourcesMap
          );
        }

        const promises: Promise<any>[] = [];

        for (const { modulePath, resolve, moduleId } of queue) {
          compilingModules.delete(modulePath);
          promises.push(
            FarmModuleSystem.loadDynamicResources(moduleId, true).then(resolve)
          );
        }

        return Promise.all(promises).then(() => {
          if (FarmModuleSystem.lazyCompilingQueue.length > 0) {
            return compileModules();
          } else {
            FarmModuleSystem.lazyCompiling = false;
          }
        });
      });
    };

    queueLazyCompilation();
    compileModules();
  }
}

export const __farm_async = true;
export default promise;
