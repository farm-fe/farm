import { JsPlugin, CompilationContext } from '../type.js';
import {
  convertEnforceToPriority,
  customParseQueryString,
  formatId,
  formatLoadModuleType,
  formatTransformModuleType,
  getContentValue,
  isObject,
  isString,
  encodeStr,
  decodeStr,
  deleteUndefinedPropertyDeeply
} from './utils.js';
import { DevServer, UserConfig } from '../../index.js';

// only use types from vite and we do not install vite as a dependency
import type {
  Plugin,
  UserConfig as ViteUserConfig,
  HmrContext,
  ViteDevServer,
  ModuleNode
} from 'vite';
import type { PluginContext, ResolveIdResult } from 'rollup';
import path, { relative } from 'path';
import {
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult
} from '../../../binding/index.js';
import merge from 'lodash.merge';
import { readFile } from 'fs/promises';
import {
  ViteDevServerAdapter,
  createViteDevServerAdapter
} from './vite-server-adapter.js';

/// turn a vite plugin to farm js plugin
export class VitePluginAdapter implements JsPlugin {
  name = 'to-be-override';
  priority = 0;

  private _rawPlugin: Plugin;
  private _farmConfig: UserConfig;
  private _viteConfig: ViteUserConfig;
  private _viteDevServer: ViteDevServerAdapter;

  buildStart: JsPlugin['buildStart'];
  resolve: JsPlugin['resolve'];
  load: JsPlugin['load'];
  transform: JsPlugin['transform'];
  buildEnd: JsPlugin['buildEnd'];
  finish: JsPlugin['finish'];
  updateModules: JsPlugin['updateModules'];
  // filter for js plugin to improve performance
  filters: string[];

  constructor(rawPlugin: Plugin, farmConfig: UserConfig, filters: string[]) {
    this.name = rawPlugin.name;
    this.priority = convertEnforceToPriority(rawPlugin.enforce);
    this._rawPlugin = rawPlugin;
    this._farmConfig = farmConfig;
    this._viteConfig = farmConfigToViteConfig(farmConfig);

    this.filters = filters;

    // convert hooks
    this.buildStart = this.viteBuildStartToFarmBuildStart();
    this.resolve = this.viteResolveIdToFarmResolve();
    this.load = this.viteLoadToFarmLoad();
    this.transform = this.viteTransformToFarmTransform();
    this.buildEnd = this.viteBuildEndToFarmBuildEnd();
    this.finish = this.viteCloseBundleToFarmFinish();
    this.updateModules = this.viteHandleHotUpdateToFarmUpdateModules();
  }

  // call both config and configResolved
  config(config: UserConfig['compilation']) {
    this._farmConfig.compilation = config;
    this._viteConfig = farmConfigToViteConfig(this._farmConfig);

    const configHook = this.wrap_raw_plugin_hook(
      'config',
      this._rawPlugin.config
    );

    if (configHook) {
      this._viteConfig = merge(this._viteConfig, configHook(this._viteConfig));
      this._farmConfig = viteConfigToFarmConfig(
        this._viteConfig,
        this._farmConfig,
        this.name
      );
    }

    const configResolvedHook = this.wrap_raw_plugin_hook(
      'configResolved',
      this._rawPlugin.configResolved
    );

    if (configResolvedHook) {
      configResolvedHook(this._viteConfig);
    }

    return this._farmConfig.compilation;
  }

  configDevServer(_: DevServer) {
    const hook = this.wrap_raw_plugin_hook(
      'configureServer',
      this._rawPlugin.configureServer
    );

    this._viteDevServer = createViteDevServerAdapter(
      this.name,
      this._viteConfig
    );

    if (hook) {
      hook(this._viteDevServer);
    }
  }

  private should_execute_plugin() {
    // TODO add command config
    const command =
      this._farmConfig.compilation?.mode === 'production' ? 'build' : 'serve';

    if (typeof this._rawPlugin.apply === 'function') {
      return this._rawPlugin.apply(this._viteConfig, {
        mode: this._farmConfig.compilation.mode,
        command,
        ssrBuild: this._farmConfig.compilation.output.targetEnv === 'node'
      });
    } else if (this._rawPlugin.apply === undefined) {
      return true;
    }

    return this._rawPlugin.apply === command;
  }

  private wrap_executor(executor: (...args: any[]) => any) {
    return async (...args: any[]) => {
      if (this.should_execute_plugin()) {
        return await executor(...args);
      }
    };
  }

  private wrap_raw_plugin_hook(
    hookName: string,
    hook: object | undefined | ((...args: any[]) => any),
    farmContext?: CompilationContext,
    currentHandlingFile?: string
  ) {
    if (hook === undefined) {
      return undefined;
    }

    if (typeof hook !== 'function') {
      throw new Error(
        `${hookName} hook of vite plugin ${this.name} is not a function. Farm only supports vite plugin with function hooks. This Plugin is not compatible with farm.`
      );
    }

    if (farmContext) {
      const pluginContext = farmContextToViteContext(
        farmContext,
        currentHandlingFile,
        this.name,
        hookName,
        this._farmConfig
      );
      return hook.bind(pluginContext);
    } else {
      return hook;
    }
  }

  private viteBuildStartToFarmBuildStart(): JsPlugin['buildStart'] {
    return {
      executor: this.wrap_executor((_, context) => {
        const hook = this.wrap_raw_plugin_hook(
          'buildStart',
          this._rawPlugin.buildStart,
          context
        );
        hook?.();
      })
    };
  }

  private viteResolveIdToFarmResolve(): JsPlugin['resolve'] {
    return {
      filters: { sources: ['.*'], importers: this.filters },
      executor: this.wrap_executor(
        async (
          params: PluginResolveHookParam,
          context: CompilationContext
        ): Promise<PluginResolveHookResult> => {
          const hook = this.wrap_raw_plugin_hook(
            'resolveId',
            this._rawPlugin.resolveId,
            context
          );
          const absImporterPath = path.resolve(
            process.cwd(),
            params.importer?.relativePath ?? ''
          );
          const resolveIdResult: ResolveIdResult = await hook?.(
            decodeStr(params.source),
            absImporterPath,
            { isEntry: params.kind === 'entry' }
          );
          const removeQuery = (path: string) => {
            const queryIndex = path.indexOf('?');
            if (queryIndex !== -1) {
              return path.slice(0, queryIndex);
            }
            return path.concat('');
          };
          if (isString(resolveIdResult)) {
            return {
              resolvedPath: removeQuery(encodeStr(resolveIdResult)),
              query: customParseQueryString(resolveIdResult),
              sideEffects: false,
              external: false,
              // TODO support meta
              meta: {}
            };
          } else if (isObject(resolveIdResult)) {
            return {
              resolvedPath: removeQuery(encodeStr(resolveIdResult?.id)),
              query: customParseQueryString(resolveIdResult!.id),
              sideEffects: Boolean(resolveIdResult?.moduleSideEffects),
              // TODO support relative and absolute external
              external: Boolean(resolveIdResult?.external),
              meta: {}
            };
          }
          return null;
        }
      )
    };
  }

  private viteLoadToFarmLoad(): JsPlugin['load'] {
    return {
      filters: {
        // TODO support internal filter optimization for common plugins like @vitejs/plugin-vue
        resolvedPaths: this.filters
      },
      executor: this.wrap_executor(
        async (
          params: PluginLoadHookParam,
          context: CompilationContext
        ): Promise<PluginLoadHookResult> => {
          const hook = this.wrap_raw_plugin_hook(
            'load',
            this._rawPlugin.load,
            context
          );

          const resolvedPath = decodeStr(params.resolvedPath);

          // append query
          const id = formatId(resolvedPath, params.query);
          const result = await hook?.(id);

          if (result) {
            return {
              content: getContentValue(result),
              // only support css as first class citizen for vite plugins
              moduleType: formatLoadModuleType(id)
              // TODO support meta, sourcemap and sideEffects
            };
          }
        }
      )
    };
  }

  private viteTransformToFarmTransform(): JsPlugin['transform'] {
    return {
      filters: {
        resolvedPaths: this.filters
      },
      executor: this.wrap_executor(
        async (
          params: PluginTransformHookParam,
          context: CompilationContext
        ): Promise<PluginTransformHookResult> => {
          const hook = this.wrap_raw_plugin_hook(
            'transform',
            this._rawPlugin.transform,
            context
          );
          const resolvedPath = decodeStr(params.resolvedPath);
          // append query
          const id = formatId(resolvedPath, params.query);
          const result = await hook?.(params.content, id, {
            ssr: this._farmConfig.compilation.output.targetEnv === 'node'
          });

          if (result) {
            return {
              content: getContentValue(result),
              sourceMap:
                typeof result.map === 'object'
                  ? JSON.stringify(result.map)
                  : undefined,
              moduleType: formatTransformModuleType(id)
              // TODO support meta, sourcemap and sideEffects
            };
          }
        }
      )
    };
  }

  private viteBuildEndToFarmBuildEnd(): JsPlugin['buildEnd'] {
    return {
      executor: this.wrap_executor((_, context) => {
        const hook = this.wrap_raw_plugin_hook(
          'buildEnd',
          this._rawPlugin.buildEnd,
          context
        );
        hook?.();
      })
    };
  }

  private viteCloseBundleToFarmFinish(): JsPlugin['finish'] {
    return {
      executor: this.wrap_executor(() => {
        const hook = this.wrap_raw_plugin_hook(
          'closeBundle',
          this._rawPlugin.closeBundle
        );
        hook?.();
      })
    };
  }

  private viteHandleHotUpdateToFarmUpdateModules(): JsPlugin['updateModules'] {
    return {
      executor: this.wrap_executor(
        async ({ paths }: { paths: [string, string][] }, ctx) => {
          const hook = this.wrap_raw_plugin_hook(
            'handleHotUpdate',
            this._rawPlugin.handleHotUpdate,
            ctx
          );

          const result = [];
          this._viteDevServer.moduleGraph.context = ctx;

          for (const [file, _] of paths) {
            const mods = this._viteDevServer.moduleGraph.getModulesByFile(
              file
            ) as unknown as ModuleNode[];

            const ctx: HmrContext = {
              file: file,
              timestamp: Date.now(),
              modules: mods ?? [],
              read: function (): string | Promise<string> {
                return readFile(file, 'utf-8');
              },
              server: this._viteDevServer as unknown as ViteDevServer
            };
            const updateMods: ModuleNode[] = await hook?.(ctx);

            if (updateMods) {
              result.push(...updateMods.map((mod) => mod.id));
            } else {
              result.push(...mods.map((mod) => mod.id));
            }
          }

          return [...new Set(result)];
        }
      )
    };
  }
}

function farmContextToViteContext(
  farmContext: CompilationContext,
  currentHandlingFile?: string,
  pluginName?: string,
  hookName?: string,
  config?: UserConfig
): PluginContext {
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
            relativePath: relative(importer, config.compilation.root),
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
    moduleIds: new Set<string>()[Symbol.iterator]()
  };

  return viteContext;
}

function farmConfigToViteConfig(config: UserConfig): ViteUserConfig {
  return {
    root: config.root,
    base: config.compilation?.output?.publicPath,
    publicDir: config.publicDir ?? 'public',
    mode: config.compilation?.mode,
    define: config.compilation?.define,
    resolve: {
      alias: config.compilation?.resolve?.alias,
      extensions: config.compilation?.resolve?.extensions,
      mainFields: config.compilation?.resolve?.mainFields,
      conditions: config.compilation?.resolve?.conditions,
      preserveSymlinks: config.compilation?.resolve?.symlinks === false
    },
    server: {
      hmr: Boolean(config.server?.hmr),
      port: config.server?.port,
      host: config.server?.host,
      strictPort: config.server?.strictPort,
      https: config.server?.https,
      proxy: config.server?.proxy as any,
      open: config.server?.open
      // other options are not supported in farm
    },
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore ignore this error
    isProduction: config.compilation?.mode === 'production',
    css: {
      devSourcemap: false
    },
    build: {
      outDir: config.compilation?.output?.path,
      sourcemap: Boolean(config.compilation?.sourcemap),
      minify: config.compilation?.minify,
      cssMinify: config.compilation?.minify
      // other options are not supported in farm
    }
  };
}

function viteConfigToFarmConfig(
  config: ViteUserConfig,
  origFarmConfig: UserConfig,
  pluginName: string
): UserConfig {
  const farmConfig: UserConfig = {};

  if (config.root) {
    farmConfig.root = config.root;
  }
  if (config.base) {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    if (!farmConfig.compilation.output) {
      farmConfig.compilation.output = {};
    }
    farmConfig.compilation.output.publicPath = config.base;
  }
  if (config.publicDir) {
    farmConfig.publicDir = config.publicDir;
  }
  if (config.mode === 'development' || config.mode === 'production') {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    farmConfig.compilation.mode = config.mode;
  }
  if (config.define) {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    farmConfig.compilation.define = config.define;
  }
  if (config.resolve) {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    if (!farmConfig.compilation.resolve) {
      farmConfig.compilation.resolve = {};
    }
    if (config.resolve.alias) {
      if (!Array.isArray(config.resolve.alias)) {
        farmConfig.compilation.resolve.alias = config.resolve.alias as Record<
          string,
          any
        >;
      } else {
        throw new Error(
          `Vite plugin ${pluginName} is not compatible with Farm for now. Because resolve.alias(called by hook ${pluginName}.config) is not supported in Farm`
        );
      }
    }

    farmConfig.compilation.resolve.extensions = config.resolve.extensions;
    farmConfig.compilation.resolve.mainFields = config.resolve.mainFields;
    farmConfig.compilation.resolve.conditions = config.resolve.conditions;
    farmConfig.compilation.resolve.symlinks =
      config.resolve.preserveSymlinks != true;
  }

  if (config.server) {
    if (!farmConfig.server) {
      farmConfig.server = {};
    }
    farmConfig.server.hmr = config.server.hmr;
    farmConfig.server.port = config.server.port;

    if (typeof config.server.host === 'string') {
      farmConfig.server.host = config.server.host;
    }

    farmConfig.server.strictPort = config.server.strictPort;
    farmConfig.server.https = Boolean(config.server.https);
    farmConfig.server.proxy = config.server.proxy as any;
    farmConfig.server.open = Boolean(config.server.open);
  }

  if (config.build) {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    if (!farmConfig.compilation.output) {
      farmConfig.compilation.output = {};
    }
    farmConfig.compilation.output.path = config.build.outDir;
    farmConfig.compilation.sourcemap = Boolean(config.build.sourcemap);
    farmConfig.compilation.minify = Boolean(config.build.minify);
  }

  deleteUndefinedPropertyDeeply(farmConfig);

  return merge({}, origFarmConfig, farmConfig);
}
