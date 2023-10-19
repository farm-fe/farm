import type { PluginContext } from 'rollup';
import type { UserConfig } from '../../config/types.js';
import type { CompilationContext } from '../type.js';
import { relative } from 'path';
import { DefaultLogger } from '../../utils/logger.js';

const contextCache = new Map<string, PluginContext>();

export function farmContextToViteContext(
  farmContext: CompilationContext,
  currentHandlingFile?: string,
  pluginName?: string,
  hookName?: string,
  config?: UserConfig
): PluginContext {
  const cacheKey = pluginName + hookName;
  if (contextCache.has(cacheKey)) {
    return contextCache.get(cacheKey) as PluginContext;
  }
  const logger = new DefaultLogger();

  const log = (message: any) => {
    if (typeof message === 'function') {
      message = message();
    }

    console.log(message);
  };

  const cacheError = () => {
    throw new Error(
      `Vite plugin ${pluginName} is not compatible with Farm for now. Because cache(called by hook ${pluginName}.${hookName}) is not supported in Farm`
    );
  };

  const viteContext: PluginContext = {
    addWatchFile: (id) => {
      if (!currentHandlingFile) {
        throw new Error(
          `Vite plugin ${pluginName} is not compatible with Farm for now. Because addWatchFile(called by hook ${pluginName}.${hookName}) can only be called in load hook or transform hook in Farm.`
        );
      }
      farmContext.addWatchFile(currentHandlingFile, id);
    },
    debug: log,
    emitFile: (params) => {
      if (params.type === 'asset') {
        let content: number[] = [];

        if (typeof params.source === 'string') {
          content = [...Buffer.from(params.source)];
        } else {
          content = [...params.source];
        }

        farmContext.emitFile({
          resolvedPath: currentHandlingFile ?? 'vite-plugin-adapter',
          name: params.fileName ?? params.name,
          content,
          resourceType: 'asset'
        });

        return 'vite-plugin-adapter-unsupported-reference-id';
      } else {
        throw new Error(
          `Vite plugin ${pluginName} is not compatible with Farm for now. Because emitFile(called by hook ${pluginName}.${hookName}) can only emit asset in Farm.`
        );
      }
    },
    error: (message): never => {
      if (typeof message === 'object') {
        farmContext.error(JSON.stringify(message));
      } else {
        farmContext.error(message);
      }

      return undefined as unknown as never;
    },
    getFileName: () => {
      throw new Error(
        `Vite plugin ${pluginName} is not compatible with Farm for now. Because getFileName(called by hook ${pluginName}.${hookName}) is not supported in Farm`
      );
    },
    getModuleIds: () => {
      throw new Error(
        `Vite plugin ${pluginName} is not compatible with Farm for now. Because getModuleIds(called by hook ${pluginName}.${hookName}) is not supported in Farm`
      );
    },
    getModuleInfo: () => {
      throw new Error(
        `Vite plugin ${pluginName} is not compatible with Farm for now. Because getModuleInfo(called by hook ${pluginName}.${hookName}) is not supported in Farm`
      );
    },
    getWatchFiles: () => {
      return farmContext.getWatchFiles();
    },
    info: log,
    load: (_) => {
      throw new Error(
        `Vite plugin ${pluginName} is not compatible with Farm for now. Because load(called by hook ${pluginName}.${hookName}) is not supported in Farm`
      );
    },
    meta: {
      rollupVersion: '3.29.4',
      watchMode: config.compilation.mode !== 'production'
    },
    parse: (_) => {
      throw new Error(
        `Vite plugin ${pluginName} is not compatible with Farm for now. Because parse(called by hook ${pluginName}.${hookName}) is not supported in Farm`
      );
    },
    resolve: async (source, importer, options) => {
      if (options.custom.caller === `${pluginName}.${hookName}`) {
        return null;
      }

      const farmResolveResult = await farmContext.resolve(
        {
          source,
          importer: {
            relativePath: relative(config.compilation.root, importer),
            queryString: importer.split('?')[1] ?? ''
          },
          kind: options.isEntry ? 'entry' : 'import'
        },
        {
          meta: {},
          caller: `${pluginName}.${hookName}`
        }
      );

      if (farmResolveResult) {
        return {
          id: farmResolveResult.resolvedPath,
          external: farmResolveResult.external,
          resolvedBy: 'vite-plugin-adapter-farm-resolve',
          moduleSideEffects: farmResolveResult.sideEffects,
          meta: {
            ...farmResolveResult.meta,
            caller: `${pluginName}.${hookName}`
          },
          // TODO these 2 options are not supported in farm
          assertions: {},
          syntheticNamedExports: false
        };
      }

      return null;
    },
    setAssetSource(assetReferenceId, source) {
      this.emitFile({
        type: 'asset',
        source,
        name: assetReferenceId
      });
    },
    warn: (message) => {
      if (typeof message === 'object') {
        farmContext.warn(JSON.stringify(message));
      } else if (typeof message === 'function') {
        farmContext.warn(JSON.stringify(message()));
      } else {
        farmContext.warn(message);
      }
    },
    cache: {
      set: cacheError,
      get: cacheError,
      delete: cacheError,
      has: cacheError
    },
    moduleIds: new Set<string>()[Symbol.iterator](),
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore Vite specific property
    getCombinedSourcemap() {
      logger.warn(
        '`vite-plugin-adapter`: getCombinedSourcemap is not supported in Farm for now. It will always return undefined.'
      );
      return undefined;
    }
  };

  contextCache.set(cacheKey, viteContext);

  return viteContext;
}
