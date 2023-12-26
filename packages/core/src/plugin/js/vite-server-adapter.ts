// import { watch } from 'chokidar';
import { DevServer } from '../../index.js';
import WsServer from '../../server/ws.js';
import { CompilationContext, ViteModule } from '../type.js';
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

    return proxyViteModuleNode(raw, this.pluginName, this.context);
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
  const proxy = new Proxy(new ViteModuleGraphAdapter(pluginName), {
    get(target, key) {
      const allowedKeys = ['getModulesByFile', 'getModuleById'];
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
