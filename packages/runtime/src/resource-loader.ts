// using native ability to load resources if target env is node.
declare const __FARM_TARGET_ENV__: 'node' | 'browser';

export interface Resource {
  path: string;
  type: 'script' | 'link';
}

/**
 * Loading resources according to their type and target env.
 */
export class ResourceLoader {
  publicPaths: string[];

  constructor(publicPaths: string[]) {
    this.publicPaths = publicPaths;
  }

  async load(resource: Resource): Promise<void> {
    let index = 0;

    while (index < this.publicPaths.length) {
      const publicPath = this.publicPaths[index];
      const url = `${publicPath}/${resource.path}`;

      try {
        if (resource.type === 'script') {
          await this._loadScript(url);
        } else if (resource.type === 'link') {
          await this._loadLink(url);
        }

        return;
      } catch (e) {
        index++;
      }
    }
  }

  private async _loadScript(path: string): Promise<void> {
    if (__FARM_TARGET_ENV__ === 'node') {
      await import(path);
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

  private async _loadLink(path: string): Promise<void> {
    if (__FARM_TARGET_ENV__ === 'node') {
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
