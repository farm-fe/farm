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
  FARM_CSS_MODULE_SUFFIX
} from './utils.js';
import type { UserConfig } from '../../config/types.js';
import type { DevServer } from '../../server/index.js';

// only use types from vite and we do not install vite as a dependency
import type {
  Plugin,
  UserConfig as ViteUserConfig,
  HmrContext,
  ViteDevServer,
  ModuleNode,
  ConfigEnv
} from 'vite';
import type { ResolveIdResult } from 'rollup';
import path from 'path';
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
import { farmContextToViteContext } from './farm-to-vite-context.js';
import {
  farmConfigToViteConfig,
  proxyViteConfig,
  viteConfigToFarmConfig
} from './farm-to-vite-config.js';
import { VIRTUAL_FARM_DYNAMIC_IMPORT_PREFIX } from '../../compiler/index.js';

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

    if (!rawPlugin.name) {
      throw new Error(
        `Vite plugin ${rawPlugin} is not compatible with Farm for now. Because plugin name is required in Farm.`
      );
    }

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

    // if other unsupported vite plugins hooks are used, throw error
    const unsupportedHooks = [
      'transformIndexHtml',
      'renderChunk',
      'argumentChunkHash',
      'generateBundle',
      'writeBundle',
      'renderError',
      'resolveDynamicImport',
      'resolveFileUrl',
      'resolveImportMeta',
      'transformIndexHtml',
      'shouldTransformCachedModule',
      'banner',
      'footer',
      'renderStart'
    ];

    for (const hookName of unsupportedHooks) {
      if (this._rawPlugin[hookName as keyof Plugin]) {
        throw new Error(
          `Vite plugin ${this.name} is not compatible with Farm for now. Because it uses hook "${hookName}" which is not supported by Farm.`
        );
      }
    }
  }

  // call both config and configResolved
  async config(config: UserConfig['compilation'], configEnv: ConfigEnv) {
    this._farmConfig.compilation = config;
    this._viteConfig = farmConfigToViteConfig(this._farmConfig);

    const configHook = this.wrapRawPluginHook('config', this._rawPlugin.config);

    if (configHook) {
      this._viteConfig = proxyViteConfig(
        merge(
          this._viteConfig,
          await configHook(
            proxyViteConfig(this._viteConfig, this.name),
            this.getViteConfigEnv(configEnv)
          ),
          this.getViteConfigEnv(configEnv)
        ),
        this.name
      );
      this._farmConfig = viteConfigToFarmConfig(
        this._viteConfig,
        this._farmConfig,
        this.name
      );
    }

    return this._farmConfig.compilation;
  }

  async configResolved() {
    // this._farmConfig.compilation = config;
    // this._viteConfig = farmConfigToViteConfig(this._farmConfig);
    const configResolvedHook = this.wrapRawPluginHook(
      'configResolved',
      this._rawPlugin.configResolved
    );

    if (configResolvedHook) {
      await configResolvedHook(this._viteConfig);
    }
  }

  async configDevServer(devServer: DevServer) {
    const hook = this.wrapRawPluginHook(
      'configureServer',
      this._rawPlugin.configureServer
    );

    this._viteDevServer = createViteDevServerAdapter(
      this.name,
      this._viteConfig
    );

    if (hook) {
      await hook(this._viteDevServer);
      this._viteDevServer.middlewareCallbacks.forEach((cb) => {
        devServer.app().use((ctx, next) => {
          return cb(ctx.req, ctx.res, next);
        });
      });
    }
  }

  private getViteConfigEnv(configEnv: ConfigEnv): ConfigEnv {
    return {
      ssrBuild: this._farmConfig.compilation.output?.targetEnv === 'node',
      command: configEnv.command,
      mode: configEnv.mode
    };
  }

  private shouldExecutePlugin() {
    const command =
      this._farmConfig.compilation?.mode === 'production' ? 'build' : 'serve';

    if (typeof this._rawPlugin.apply === 'function') {
      return this._rawPlugin.apply(this._viteConfig, {
        mode: this._farmConfig.compilation.mode,
        command,
        ssrBuild: this._farmConfig.compilation.output?.targetEnv === 'node'
      });
    } else if (this._rawPlugin.apply === undefined) {
      return true;
    }

    return this._rawPlugin.apply === command;
  }

  private wrapExecutor(executor: (...args: any[]) => any) {
    return async (...args: any[]) => {
      if (this.shouldExecutePlugin()) {
        return await executor(...args);
      }
    };
  }

  private wrapRawPluginHook(
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
      executor: this.wrapExecutor((_, context) => {
        const hook = this.wrapRawPluginHook(
          'buildStart',
          this._rawPlugin.buildStart,
          context
        );
        return hook?.();
      })
    };
  }

  private viteResolveIdToFarmResolve(): JsPlugin['resolve'] {
    return {
      filters: { sources: ['.*'], importers: this.filters },
      executor: this.wrapExecutor(
        async (
          params: PluginResolveHookParam,
          context: CompilationContext
        ): Promise<PluginResolveHookResult> => {
          if (
            params.importer &&
            VitePluginAdapter.isFarmInternalVirtualModule(
              params.importer.relativePath
            )
          ) {
            return null;
          }

          const hook = this.wrapRawPluginHook(
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
              meta: {}
            };
          } else if (isObject(resolveIdResult)) {
            return {
              resolvedPath: removeQuery(encodeStr(resolveIdResult?.id)),
              query: customParseQueryString(resolveIdResult!.id),
              sideEffects: Boolean(resolveIdResult?.moduleSideEffects),
              // TODO support relative and absolute external
              external: Boolean(resolveIdResult?.external),
              meta: resolveIdResult.meta ?? {}
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
      executor: this.wrapExecutor(
        async (
          params: PluginLoadHookParam,
          context: CompilationContext
        ): Promise<PluginLoadHookResult> => {
          if (
            VitePluginAdapter.isFarmInternalVirtualModule(params.resolvedPath)
          ) {
            return null;
          }

          const hook = this.wrapRawPluginHook(
            'load',
            this._rawPlugin.load,
            context,
            params.moduleId
          );

          const isSSR =
            this._farmConfig.compilation.output?.targetEnv === 'node';
          const resolvedPath = decodeStr(params.resolvedPath);

          // append query
          const id = formatId(resolvedPath, params.query);
          const result = await hook?.(id, isSSR ? { ssr: true } : undefined);

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
      executor: this.wrapExecutor(
        async (
          params: PluginTransformHookParam,
          context: CompilationContext
        ): Promise<PluginTransformHookResult> => {
          if (
            VitePluginAdapter.isFarmInternalVirtualModule(params.resolvedPath)
          ) {
            return null;
          }

          const hook = this.wrapRawPluginHook(
            'transform',
            this._rawPlugin.transform,
            context,
            params.moduleId
          );
          const isSSR =
            this._farmConfig.compilation.output?.targetEnv === 'node';
          const resolvedPath = decodeStr(params.resolvedPath);
          // append query
          const id = formatId(resolvedPath, params.query);

          const result = await hook?.(
            params.content,
            id,
            isSSR ? { ssr: true } : undefined
          );

          if (result) {
            return {
              content: getContentValue(result),
              sourceMap:
                typeof result.map === 'object' && result.map !== null
                  ? JSON.stringify(result.map)
                  : undefined,
              moduleType: formatTransformModuleType(id)
              // TODO support meta and sideEffects
            };
          }
        }
      )
    };
  }

  private viteBuildEndToFarmBuildEnd(): JsPlugin['buildEnd'] {
    return {
      executor: this.wrapExecutor((_, context) => {
        const hook = this.wrapRawPluginHook(
          'buildEnd',
          this._rawPlugin.buildEnd,
          context
        );
        return hook?.();
      })
    };
  }

  private viteCloseBundleToFarmFinish(): JsPlugin['finish'] {
    return {
      executor: this.wrapExecutor(() => {
        const hook = this.wrapRawPluginHook(
          'closeBundle',
          this._rawPlugin.closeBundle
        );
        return hook?.();
      })
    };
  }

  private viteHandleHotUpdateToFarmUpdateModules(): JsPlugin['updateModules'] {
    return {
      executor: this.wrapExecutor(
        async ({ paths }: { paths: [string, string][] }, ctx) => {
          const hook = this.wrapRawPluginHook(
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

  // skip farm lazy compilation virtual module for vite plugin
  public static isFarmInternalVirtualModule(id: string) {
    return (
      id.startsWith(VIRTUAL_FARM_DYNAMIC_IMPORT_PREFIX) ||
      // css has been handled before the virtual module is created
      FARM_CSS_MODULE_SUFFIX.test(id)
    );
  }
}
