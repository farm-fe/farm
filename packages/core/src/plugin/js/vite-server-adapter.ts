import { CompilationContext } from '../type.js';

export class ViteDevServerAdapter {
  moduleGraph: ViteModuleGraphAdapter;
  config: any;
  pluginName: string;

  constructor(pluginName: string, config: any) {
    this.moduleGraph = createViteModuleGraphAdapter(pluginName);
    this.config = config;
    this.pluginName = pluginName;
  }
}

export class ViteModuleGraphAdapter {
  context: CompilationContext;

  constructor() {
    this.context = undefined;
  }

  getModulesByFile(
    file: string
  ): ReturnType<CompilationContext['viteGetModulesByFile']> {
    const raw = this.context.viteGetModulesByFile(file);
    const _content = this.context;

    return raw.map((item) => {
      const proxy = new Proxy(item, {
        get(target, key) {
          if (key === 'importers') {
            return _content.viteGetImporters(target.id);
          }

          const allowedKeys = ['url', 'id', 'file', 'type'];

          if (allowedKeys.includes(String(key))) {
            return target[key as keyof typeof target];
          } else {
            throw new Error(
              `Vite plugin '${
                this.pluginName
              }' is not compatible with Farm for now. Because it uses viteModuleNode['${String(
                key
              )}'] which is not supported by Farm. \n Current supported keys are: ${allowedKeys.join(
                ','
              )}`
            );
          }
        }
      });

      return proxy;
    });
  }
}

export function createViteDevServerAdapter(pluginName: string, config: any) {
  const proxy = new Proxy(new ViteDevServerAdapter(pluginName, config), {
    get(target, key) {
      const allowedKeys = ['moduleGraph', 'config'];
      if (allowedKeys.includes(String(key))) {
        return target[key as keyof typeof target];
      } else {
        throw new Error(
          `Vite plugin '${pluginName}' is not compatible with Farm for now. Because it uses viteModuleGraph['${String(
            key
          )}'] which is not supported by Farm. \n Current supported keys are: ${allowedKeys.join(
            ','
          )}`
        );
      }
    }
  });

  return proxy;
}

export function createViteModuleGraphAdapter(pluginName: string) {
  const proxy = new Proxy(new ViteModuleGraphAdapter(), {
    get(target, key) {
      const allowedKeys = ['getModulesByFile', 'context'];

      if (allowedKeys.includes(String(key))) {
        return target[key as keyof typeof target];
      }

      throw new Error(
        `Vite plugin '${pluginName}' is not compatible with Farm for now. Because it uses viteModuleGraph['${String(
          key
        )}'] which is not supported by Farm. \n Current supported keys are: ${allowedKeys.join(
          ','
        )}`
      );
    },
    set(target, p, newValue, _receiver) {
      if (p === 'context') {
        target.context = newValue;
        return true;
      }

      throw new Error(
        `Vite plugin '${pluginName}' is not compatible with Farm for now. Because it uses viteModuleGraph['${String(
          p
        )}'] which is not supported by Farm`
      );
    }
  });

  return proxy;
}
