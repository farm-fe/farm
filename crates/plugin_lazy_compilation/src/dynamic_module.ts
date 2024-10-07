interface Resource {
  path: string;
  type: 0 | 1; // 0: script, 1: link
}

interface RawLazyCompileResult {
  dynamicResources: Resource[];
  dynamicModuleResourcesMap: Record<string, number[]>;
}

interface LazyCompilationQueueItem {
  modulePath: string;
  moduleId: string;
  resolve: (data: any) => void;
  promise: Promise<void>;
}

// Inject during compile time
const FarmModuleSystem: any = 'FARM_MODULE_SYSTEM';
const moduleId = 'MODULE_ID';
const modulePath = 'MODULE_PATH';
const serverUrl = 'FARM_LAZY_COMPILE_SERVER_URL';

function getServerUrl() {
  // server url is not defined, return empty string instead
  if (serverUrl === 'FARM_LAZY' + '_COMPILE_SERVER_URL') {
    return '';
  }

  return serverUrl;
}

async function fetch(path: string) {
  const url = `${getServerUrl()}${path}`;
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

function startLazyCompiling(queue: any[]) {
  FarmModuleSystem.lazyCompiling = true;
  // TODO enable following feature in 2.0
  // // @ts-ignore ignore type
  // if (window.__farm_hide_lazy_compile_progress === true) {
  //   // show the compiling progress at the top of the page
  //   const progress = document.getElementById('farm-lazy-compiling') || document.createElement('div');
  //   progress.id = 'farm-lazy-compiling';
  //   progress.style.backgroundColor = '#ff9ff330';
  //   progress.style.color = '#6f1a5f';
  //   progress.style.zIndex = '9999999999';
  //   document.body.prepend(progress);
  //   progress.innerText = 'Compiling ' + queue.map((item) => item.modulePath).join(', ') + '...';
  // }
}

function endLazyCompiling() {
  FarmModuleSystem.lazyCompiling = false;
  // const progress = document.getElementById('farm-lazy-compiling');
  // if (progress) {
  //   document.body.removeChild(progress);
  // }
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
  promise = compilingModules.get(modulePath);
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
      const isNodeLazyCompile = FarmModuleSystem.targetEnv === 'node';
      const queue = [...FarmModuleSystem.lazyCompilingQueue];
      FarmModuleSystem.lazyCompilingQueue = [];
      startLazyCompiling(queue);
      const paths = queue.map((item) => item.modulePath);
      const url = `/__lazy_compile?paths=${encodeURIComponent(paths.join(','))}&t=${Date.now()}${
        isNodeLazyCompile ? '&node=true' : ''
      }`;

      const fetchLazyCompileResult = !isNodeLazyCompile
        ? import(`${getServerUrl()}${url}`)      // Adding full uri to make it work in webview context like vscode extension
        : fetch(url);

      return fetchLazyCompileResult.then((module: any) => {
        const result: RawLazyCompileResult = module.default || module;

        if (result.dynamicResources && result.dynamicModuleResourcesMap) {
          FarmModuleSystem.setDynamicModuleResourcesMap(
            result.dynamicResources,
            result.dynamicModuleResourcesMap
          );
        }
        FarmModuleSystem.reRegisterModules = true;
        const promises: Promise<any>[] = [];

        for (const { modulePath, resolve, moduleId } of queue) {
          compilingModules.delete(modulePath);
          promises.push(
            FarmModuleSystem.loadDynamicResources(moduleId, true).then((mod) => {
              resolve(mod);
              compilingModules.delete(modulePath);
          }));
        }

        return Promise.all(promises).then(() => {
          if (FarmModuleSystem.lazyCompilingQueue.length > 0) {
            return compileModules();
          } else {
            endLazyCompiling();
            FarmModuleSystem.reRegisterModules = false;
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
