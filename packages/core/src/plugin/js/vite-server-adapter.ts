import { Logger } from '../../utils/logger.js';
import { CompilationContext, ViteModule } from '../type.js';
import { throwIncompatibleError } from './utils.js';

export class ViteDevServerAdapter {
  private _server: any;
  private _config: any;
  private _moduleGraphAdapter: any;
  [key: string]: any;
  [key: symbol]: any;
  constructor(pluginName: string, config: any, server: any, logger: Logger) {
    this._server = server;
    this._config = config;
    this._moduleGraphAdapter = createViteModuleGraphAdapter(pluginName, logger);

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
  logger: Logger;

  constructor(pluginName: string, logger: Logger) {
    // context will be set in buildStart hook
    this.context = undefined;
    this.pluginName = pluginName;
    this.logger = logger;
  }

  getModulesByFile(file: string): ViteModule[] {
    const raw = this.context.viteGetModulesByFile(file);

    return raw.map((item) => {
      return proxyViteModuleNode(
        item,
        this.pluginName,
        this.context,
        this.logger
      );
    });
  }

  getModuleById(id: string): ViteModule {
    const raw = this.context.viteGetModuleById(id);

    if (raw) {
      return proxyViteModuleNode(
        raw,
        this.pluginName,
        this.context,
        this.logger
      );
    }
  }

  async getModuleByUrl(url: string): Promise<ViteModule | undefined> {
    if (url.startsWith('/')) {
      url = url.slice(1);
      const raw = this.context.viteGetModuleById(url);

      if (raw) {
        return proxyViteModuleNode(
          raw,
          this.pluginName,
          this.context,
          this.logger
        );
      }
    }
  }

  invalidateModule() {
    /** does thing for now, only for compatibility */
  }
}

export function createModuleGraph(pluginName: string, logger: Logger) {
  return new ViteModuleGraphAdapter(pluginName, logger);
}

export function proxyViteModuleNode(
  node: ViteModule,
  pluginName: string,
  context: CompilationContext,
  logger: Logger
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

      throwIncompatibleError(
        logger,
        pluginName,
        'viteModuleNode',
        allowedKeys,
        key
      );
    }
  });

  return proxy;
}

export function createViteDevServerAdapter(
  pluginName: string,
  config: any,
  server: any,
  logger: Logger
) {
  const proxy = new Proxy(
    new ViteDevServerAdapter(pluginName, config, server, logger),
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

        throwIncompatibleError(
          logger,
          pluginName,
          'viteDevServer',
          allowedKeys,
          key
        );
      }
    }
  );

  return proxy;
}

export function createViteModuleGraphAdapter(
  pluginName: string,
  logger: Logger
) {
  const proxy = new Proxy(new ViteModuleGraphAdapter(pluginName, logger), {
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

      throwIncompatibleError(
        logger,
        pluginName,
        'viteModuleGraph',
        allowedKeys,
        key
      );
    },
    set(target, p, newValue, _receiver) {
      if (p === 'context') {
        target.context = newValue;
        return true;
      }

      throwIncompatibleError(
        logger,
        pluginName,
        'viteModuleGraph',
        ['context'],
        p
      );
    }
  });

  return proxy;
}
