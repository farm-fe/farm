import { CompilationContext, ViteModule } from '../type';
import { throwIncompatibleError } from './utils';

// TODO type error refactor vite adaptor
export class ViteDevServerAdapter {
  private _server: any;
  private _config: any;
  private _moduleGraphAdapter: any;
  [key: string]: any;
  [key: symbol]: any;
  constructor(pluginName: string, config: any, server: any) {
    this._server = server;
    this._config = config;
    this._moduleGraphAdapter = createViteModuleGraphAdapter(pluginName);

    return new Proxy(this, {
      get: (_target, prop) => {
        switch (prop) {
          case 'moduleGraph':
            return this._moduleGraphAdapter;
          case 'watcher':
            return this._server.watcher.getInternalWatcher();
          case 'middlewares':
            return this._server.middlewares;
          case 'config':
            return this._config;
          default: {
            const value = this._server[prop];
            return typeof value === 'function'
              ? value.bind(this._server)
              : value;
          }
        }
      },
      set: (_target, prop, value) => {
        this._server[prop] = value;
        return true;
      }
    });
  }
}

export class ViteModuleGraphAdapter {
  context: CompilationContext;
  pluginName: string;

  constructor(pluginName: string) {
    // context will be set in buildStart hook
    this.context = undefined;
    this.pluginName = pluginName;
  }

  getModulesByFile(file: string): ViteModule[] {
    const raw = this.context.viteGetModulesByFile(file);

    return raw.map((item) => {
      return proxyViteModuleNode(item, this.pluginName, this.context);
    });
  }

  getModuleById(id: string): ViteModule {
    const raw = this.context.viteGetModuleById(id);

    if (raw) {
      return proxyViteModuleNode(raw, this.pluginName, this.context);
    }
  }

  async getModuleByUrl(url: string): Promise<ViteModule | undefined> {
    if (url.startsWith('/')) {
      url = url.slice(1);
      const raw = this.context.viteGetModuleById(url);

      if (raw) {
        return proxyViteModuleNode(raw, this.pluginName, this.context);
      }
    }
  }

  invalidateModule() {
    /** does thing for now, only for compatibility */
  }
}

function proxyViteModuleNode(
  node: ViteModule,
  pluginName: string,
  context: CompilationContext
) {
  const proxy = new Proxy(node, {
    get(target, key) {
      if (key === 'importers') {
        return context.viteGetImporters(target.id);
      }

      const allowedKeys = ['url', 'id', 'file', 'type'];

      if (allowedKeys.includes(String(key))) {
        return target[key as keyof typeof target];
      }

      throwIncompatibleError(pluginName, 'viteModuleNode', allowedKeys, key);
    }
  });

  return proxy;
}

export function createViteDevServerAdapter(
  pluginName: string,
  config: any,
  server: any
) {
  const proxy = new Proxy(
    new ViteDevServerAdapter(pluginName, config, server),
    {
      get(target, key) {
        const objectKeys = [
          'constructor',
          'Symbol(Symbol.toStringTag)',
          'prototype'
        ];
        const allowedKeys = [
          'serverOptions',
          'resolvedUrls',
          'printUrls',
          'moduleGraph',
          'config',
          'watcher',
          'middlewares',
          'middlewareCallbacks',
          'ws',
          'httpServer'
        ];
        if (
          objectKeys.includes(String(key)) ||
          allowedKeys.includes(String(key))
        ) {
          return target[key as keyof typeof target];
        }

        throwIncompatibleError(pluginName, 'viteDevServer', allowedKeys, key);
      }
    }
  );

  return proxy;
}

export function createViteModuleGraphAdapter(pluginName: string) {
  const proxy = new Proxy(new ViteModuleGraphAdapter(pluginName), {
    get(target, key) {
      const allowedKeys = [
        'getModulesByFile',
        'getModuleById',
        'getModuleByUrl',
        'invalidateModule'
      ];
      const ownkeys = Reflect.ownKeys(target);

      if (allowedKeys.includes(String(key)) || ownkeys.includes(key)) {
        return target[key as keyof typeof target];
      }

      throwIncompatibleError(pluginName, 'viteModuleGraph', allowedKeys, key);
    },
    set(target, p, newValue, _receiver) {
      if (p === 'context') {
        target.context = newValue;
        return true;
      }

      throwIncompatibleError(pluginName, 'viteModuleGraph', ['context'], p);
    }
  });

  return proxy;
}
