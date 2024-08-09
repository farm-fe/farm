// using native ability to load resources if target env is node.

import type { ModuleSystem } from './module-system';
import type { ResourceLoadResult } from './plugin';

export interface Resource {
  path: string;
  type: 0 | 1; // 0: script, 1: link
}

// Injected during build
export const __farm_global_this__: any = '<@__farm_global_this__@>';

export const targetEnv = __farm_global_this__.__FARM_TARGET_ENV__ || 'node';
export const isBrowser =
  targetEnv === 'browser' && (globalThis || window).document;

/**
 * Loading resources according to their type and target env.
 */
export class ResourceLoader {
  private _loadedResources: Record<string, boolean> = {};
  private _loadingResources: Record<string, Promise<void>> = {};

  publicPaths: string[];

  constructor(private moduleSystem: ModuleSystem, publicPaths: string[]) {
    this.publicPaths = publicPaths;
  }

  load(resource: Resource, index = 0): Promise<void> {
    // it's not running in browser
    if (!isBrowser) {
      const result = this.moduleSystem.pluginContainer.hookBail(
        'loadResource',
        resource
      );

      if (result) {
        return result.then((res: ResourceLoadResult) => {
          if (!res.success && res.retryWithDefaultResourceLoader) {
            if (resource.type === 0) {
              return this._loadScript(`./${resource.path}`);
            } else if (resource.type === 1) {
              return this._loadLink(`./${resource.path}`);
            }
          } else if (!res.success) {
            throw new Error(
              `[Farm] Failed to load resource: "${resource.path}, type: ${resource.type}". Original Error: ${res.err}`
            );
          }
        });
      } else {
        if (resource.type === 0) {
          return this._loadScript(`./${resource.path}`);
        } else if (resource.type === 1) {
          return this._loadLink(`./${resource.path}`);
        }
      }
    }

    const publicPath = this.publicPaths[index];
    const url = `${
      publicPath.endsWith('/') ? publicPath.slice(0, -1) : publicPath
    }/${resource.path}`;

    if (this._loadedResources[resource.path]) {
      return;
    } else if (this._loadingResources[resource.path]) {
      return this._loadingResources[resource.path];
    }

    const result = this.moduleSystem.pluginContainer.hookBail(
      'loadResource',
      resource
    );

    if (result) {
      return result.then((res: ResourceLoadResult) => {
        if (res.success) {
          this.setLoadedResource(resource.path);
        } else if (res.retryWithDefaultResourceLoader) {
          return this._load(url, resource, index);
        } else {
          throw new Error(
            `[Farm] Failed to load resource: "${resource.path}, type: ${resource.type}". Original Error: ${res.err}`
          );
        }
      });
    } else {
      return this._load(url, resource, index);
    }
  }

  setLoadedResource(path: string, loaded = true) {
    this._loadedResources[path] = loaded;
  }

  isResourceLoaded(path: string) {
    return this._loadedResources[path];
  }

  private _load(url: string, resource: Resource, index: number): Promise<void> {
    let promise = Promise.resolve();

    if (resource.type === 0) {
      promise = this._loadScript(url);
    } else if (resource.type === 1) {
      promise = this._loadLink(url);
    }

    this._loadingResources[resource.path] = promise;

    promise
      .then(() => {
        this._loadedResources[resource.path] = true;
        this._loadingResources[resource.path] = null;
      })
      .catch((e) => {
        console.warn(
          `[Farm] Failed to load resource "${url}" using publicPath: ${this.publicPaths[index]}`
        );
        index++;

        if (index < this.publicPaths.length) {
          return this._load(url, resource, index);
        } else {
          this._loadingResources[resource.path] = null;
          throw new Error(
            `[Farm] Failed to load resource: "${resource.path}, type: ${resource.type}". ${e}`
          );
        }
      });
    return promise;
  }

  private _loadScript(path: string): Promise<void> {
    if (!isBrowser) {
      return import(path);
    } else {
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
  }

  private _loadLink(path: string): Promise<void> {
    if (!isBrowser) {
      // return Promise.reject(new Error('Not support loading css in SSR'));
      // ignore css loading in SSR
      return Promise.resolve();
    } else {
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
  }
}
