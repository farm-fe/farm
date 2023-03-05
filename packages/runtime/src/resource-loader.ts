// using native ability to load resources if target env is node.

export interface Resource {
  path: string;
  type: 'script' | 'link';
}

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore do not check type here
const targetEnv = (globalThis || global || window || self).__FARM_TARGET_ENV__;

/**
 * Loading resources according to their type and target env.
 */
export class ResourceLoader {
  private _loadedResources: Record<string, boolean> = {};

  publicPaths: string[];

  constructor(publicPaths: string[]) {
    this.publicPaths = publicPaths;
  }

  load(resource: Resource): Promise<void> {
    let index = 0;
    while (index < this.publicPaths.length) {
      const publicPath = this.publicPaths[index];
      const url = `${publicPath === '/' ? '' : publicPath}/${resource.path}`;

      if (this._loadedResources[resource.path]) {
        return;
      }
      let promise = Promise.resolve();
      try {
        if (resource.type === 'script') {
          promise = this._loadScript(url);
        } else if (resource.type === 'link') {
          promise = this._loadLink(url);
        }

        promise.then(() => {
          this._loadedResources[resource.path] = true;
        });
        return promise;
      } catch (e) {
        console.error(`[Farm] Failed to load resource "${url}"`, e);
        index++;
      }
    }
  }

  setLoadedResource(path: string) {
    this._loadedResources[path] = true;
  }

  private _loadScript(path: string): Promise<void> {
    if (targetEnv === 'node') {
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
    if (targetEnv === 'node') {
      return Promise.reject(new Error('Not support loading css in SSR'));
      // await import(path);
      // TODO investigate how to load css in SSR
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
