// import { watch } from 'chokidar';
import { DevServer } from '../../index.js';
import WsServer from '../../server/ws.js';
import { CompilationContext } from '../type.js';
import { throwIncompatibleError } from './utils.js';

export class ViteDevServerAdapter {
  moduleGraph: ViteModuleGraphAdapter;
  config: any;
  pluginName: string;
  watcher: any;
  middlewares: any;
  middlewareCallbacks: any[];
  ws: WsServer;

  constructor(pluginName: string, config: any, server: DevServer) {
    this.moduleGraph = createViteModuleGraphAdapter(pluginName);
    this.config = config;
    this.pluginName = pluginName;
    // watcher is not used in Farm vite plugin for now
    // it's only for compatibility
    // this.watcher = watch(config.root);
    this.watcher = {
      add: () => {
        // do nothing
      },
      on: () => {
        // do nothing
      },
      close: () => {
        // do nothing
      }
    };
    this.middlewareCallbacks = [];
    this.middlewares = new Proxy(
      {
        use: (cb: (...args: any[]) => any) => {
          this.middlewareCallbacks.push(cb);
        }
      },
      {
        get(target, key) {
          if (key === 'use') {
            return target[key as keyof typeof target];
          }

          throwIncompatibleError(
            pluginName,
            'viteDevServer.middlewares',
            ['use'],
            key
          );
        }
      }
    );

    this.ws = server.ws;
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
    const _context = this.context;

    return raw.map((item) => {
      const proxy = new Proxy(item, {
        get(target, key) {
          if (key === 'importers') {
            return _context.viteGetImporters(target.id);
          }

          const allowedKeys = ['url', 'id', 'file', 'type'];

          if (allowedKeys.includes(String(key))) {
            return target[key as keyof typeof target];
          }

          throwIncompatibleError(
            this.pluginName,
            'viteModuleNode',
            allowedKeys,
            key
          );
        }
      });

      return proxy;
    });
  }
}

export function createViteDevServerAdapter(
  pluginName: string,
  config: any,
  server: DevServer
) {
  const proxy = new Proxy(
    new ViteDevServerAdapter(pluginName, config, server),
    {
      get(target, key) {
        const allowedKeys = [
          'moduleGraph',
          'config',
          'watcher',
          'middlewares',
          'middlewareCallbacks',
          'ws'
        ];
        if (allowedKeys.includes(String(key))) {
          return target[key as keyof typeof target];
        }

        throwIncompatibleError(pluginName, 'viteDevServer', allowedKeys, key);
      }
    }
  );

  return proxy;
}

export function createViteModuleGraphAdapter(pluginName: string) {
  const proxy = new Proxy(new ViteModuleGraphAdapter(), {
    get(target, key) {
      const allowedKeys = ['getModulesByFile', 'context'];

      if (allowedKeys.includes(String(key))) {
        return target[key as keyof typeof target];
      }

      throwIncompatibleError(pluginName, 'viteModuleGraph', allowedKeys, key);
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
