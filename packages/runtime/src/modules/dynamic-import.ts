/**
 * Define dynamic import polyfill of ModuleSystem, including:
 * 1. moduleSystem.dynamicImport
 * 2. ResourceLoader
 */
// using native ability to load resources if target env is node.

import type { ModuleSystem } from '../module-system.js';
import type { ResourceLoadResult } from './plugin.js';

export interface Resource {
  path: string;
  // 0: script, 1: link
  type: 0 | 1;
}

// Injected during compile time
declare const __FARM_RUNTIME_TARGET_ENV__: 'browser' | 'node';
declare const __FARM_ENABLE_RUNTIME_PLUGIN__: boolean;

let dynamicResources: Resource[] = [];
// dynamic module entry and resources map
let dynamicModuleResourcesMap: Record<string, number[]> = {};
const loadedResources: Record<string, boolean> = {};
const loadingResources: Record<string, Promise<void> | null> = {};
// available public paths, when loading resources, we will try each publicPath until it is available, this is so called `resource loading retry`
const publicPaths: string[] = [];

let moduleSystem: ModuleSystem;

// append properties in module system
export function initModuleSystem(ms: ModuleSystem) {
  moduleSystem = ms;
  moduleSystem.d = dynamicImport;
  moduleSystem.sp = setPublicPaths;
  moduleSystem.si = setInitialLoadedResources;
  moduleSystem.sd = setDynamicModuleResourcesMap;
  moduleSystem.l = loadDynamicResourcesOnly;
}

function dynamicImport(id: string): Promise<any> {
  return loadDynamicResources(id);
}

function loadDynamicResources(id: string, force = false): Promise<any> {
  const resources = dynamicModuleResourcesMap[id].map((index) => dynamicResources[index]);

  return loadDynamicResourcesOnly(id, force)
    .then(() => {
      // Do not require the module if all the resources are not js resources
      if (resources.every(resource => resource.type !== 0)) {
        return;
      }

      if (!moduleSystem.m()[id]) {
        throw new Error(
          `Dynamic imported module "${id}" is not registered.`,
        );
      }
      const result = moduleSystem.r(id);
      // if the module is async, return the default export, the default export should be a promise
      if (result.__farm_async) {
        return result.default;
      } else {
        return result;
      }
    })
    .catch((err) => {
      console.error(`[Farm] Error loading dynamic module "${id}"`, err);
      throw err;
    });
}


function loadDynamicResourcesOnly(id: string, force = false): Promise<any> {
  const resources = dynamicModuleResourcesMap[id].map((index) => dynamicResources[index]);

  if (!moduleSystem.m()[id] && (!resources || resources.length === 0)) {
    throw new Error(
      `Dynamic imported module "${id}" does not belong to any resource`,
    );
  }
  // force reload resources
  if (force) {
    moduleSystem.a(id);
  }
  // loading all required resources, and return the exports of the entry module
  return Promise.all(
    resources.map((resource) => {
      if (force) {
        const resourceLoaded = isResourceLoaded(resource.path);
        setLoadedResource(resource.path, false);

        if (resourceLoaded) {
          return load({
            ...resource,
            // force reload the resource
            path: `${resource.path}?t=${Date.now()}`
          });
        }
      }
      return load(resource);
    }),
  )
}

function load(resource: Resource): Promise<void> {
  if (__FARM_RUNTIME_TARGET_ENV__ === 'node') {
    return loadResourceNode(resource); 
  } else {
    if (loadedResources[resource.path]) {
      // Skip inject Promise polyfill for runtime
      return Promise.resolve();
    } else if (loadingResources[resource.path]) {
      return loadingResources[resource.path];
    }
  
    if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
      const result = moduleSystem.p.b(
        'loadResource',
        resource
      );
    
      if (result) {
        return result.then((res: ResourceLoadResult) => {
          if (res.success) {
            setLoadedResource(resource.path);
          } else if (res.retryWithDefaultResourceLoader) {
            return loadResource(resource, 0);
          } else {
            throw new Error(
              `[Farm] Failed to load resource: "${resource.path}, type: ${resource.type}". Original Error: ${res.err}`
            );
          }
        });
      }
    }
    
    return loadResource(resource, 0);
  }
}

function loadResourceNode(resource: Resource) {
  if (__FARM_ENABLE_RUNTIME_PLUGIN__) {
    const result = moduleSystem.p.b(
      'loadResource',
      resource
    );
  
    if (result) {
      return result.then((res: ResourceLoadResult) => {
        if (!res.success && res.retryWithDefaultResourceLoader) {
          if (resource.type === 0) {
            return import(`./${resource.path}`);
          } else if (resource.type === 1) {
            return Promise.resolve();
          }
        } else if (!res.success) {
          throw new Error(
            `[Farm] Failed to load resource: "${resource.path}, type: ${resource.type}". Original Error: ${res.err}`
          );
        }
      });
    }
  }

  if (resource.type === 0) {
    return import(`./${resource.path}`);
  } else if (resource.type === 1) {
    return Promise.resolve();
  }

}

function loadResource(resource: Resource, index: number): Promise<void> {
  const publicPath = publicPaths[index];
  const url = `${
    publicPath.endsWith('/') ? publicPath.slice(0, -1) : publicPath
  }/${resource.path}`;

  let promise = Promise.resolve();

  if (resource.type === 0) {
    promise = loadScript(url);
  } else if (resource.type === 1) {
    promise = loadLink(url);
  }

  loadingResources[resource.path] = promise;

  promise
    .then(() => {
      loadedResources[resource.path] = true;
      loadingResources[resource.path] = null;
    })
    .catch((e) => {
      console.warn(
        `[Farm] Failed to load resource "${url}" using publicPath: ${publicPaths[index]}`
      );
      index++;

      if (index < publicPaths.length) {
        return loadResource(resource, index);
      } else {
        loadingResources[resource.path] = null;
        throw new Error(
          `[Farm] Failed to load resource: "${resource.path}, type: ${resource.type}". ${e}`
        );
      }
    });

  return promise;
}

function loadScript(path: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.src = path;
    document.body.appendChild(script);

    script.onload = () => {
      resolve();
    };
    script.onerror = (e) => {
      reject(e);
    };
  });
}

function loadLink(path: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = path;
    document.head.appendChild(link);

    link.onload = () => {
      resolve();
    };
    link.onerror = (e) => {
      reject(e);
    };
  });
}

function setLoadedResource(path: string, loaded = true) {
  loadedResources[path] = loaded;
}

function isResourceLoaded(path: string) {
  return loadedResources[path];
}

// The public paths are injected during compile time
function setPublicPaths(p: string[]): void {
  for (const key in p) {
    publicPaths[key] = p[key]
  }
}

function setInitialLoadedResources(resources: string[]) {
  resources.forEach(resource => {
    setLoadedResource(resource);
  });
}

  // These two methods are used to support dynamic module loading, the dynamic module info is collected by the compiler and injected during compile time
  // This method can also be called during runtime to add new dynamic modules
function setDynamicModuleResourcesMap(
    dr: Resource[],
    dmp: Record<string, number[]>,
  ): void {
    dynamicResources = dr;
    dynamicModuleResourcesMap = dmp;
  }