import {
  JsPlugin,
  CompilationContext,
  RenderResourcePotParams,
  FinalizeResourcesHookParams,
  ResourcePotInfo
} from '../type.js';
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
  FARM_CSS_MODULE_SUFFIX,
  transformFarmConfigToRollupNormalizedOutputOptions,
  transformResourceInfo2RollupRenderedChunk,
  transformRollupResource2FarmResource,
  VITE_PLUGIN_DEFAULT_MODULE_TYPE
} from './utils.js';
import type { ResolvedUserConfig, UserConfig } from '../../config/types.js';
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
import type {
  ResolveIdResult,
  RenderChunkHook,
  OutputBundle,
  FunctionPluginHooks
} from 'rollup';
import path from 'path';
import {
  Config,
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
  farmUserConfigToViteConfig,
  proxyViteConfig,
  viteConfigToFarmConfig
} from './farm-to-vite-config.js';
import { VIRTUAL_FARM_DYNAMIC_IMPORT_PREFIX } from '../../compiler/index.js';
import {
  transformResourceInfo2RollupResource,
  transformFarmConfigToRollupNormalizedInputOptions
} from './utils.js';
import { Logger } from '../../index.js';

type OmitThis<T extends (this: any, ...args: any[]) => any> = T extends (
  this: any,
  ...args: infer A
) => infer R
  ? (...arg: A) => R
  : T;
type ObjectHook<T, O = Record<string, any>> =
  | T
  | ({ handler: T; order?: 'pre' | 'post' | null } & O);

/// turn a vite plugin to farm js plugin
export class VitePluginAdapter implements JsPlugin {
  name = 'to-be-override';
  priority = 0;

  private _rawPlugin: Plugin;
  private _farmConfig: UserConfig;
  private _viteConfig: ViteUserConfig;
  private _viteDevServer: ViteDevServerAdapter;
  private _logger: Logger;

  buildStart: JsPlugin['buildStart'];
  resolve: JsPlugin['resolve'];
  load: JsPlugin['load'];
  transform: JsPlugin['transform'];
  buildEnd: JsPlugin['buildEnd'];
  finish: JsPlugin['finish'];
  updateModules: JsPlugin['updateModules'];
  renderResourcePot: JsPlugin['renderResourcePot'];
  renderStart: JsPlugin['renderStart'];
  augmentResourceHash?: JsPlugin['augmentResourceHash'];
  finalizeResources: JsPlugin['finalizeResources'];
  writeResources: JsPlugin['writeResources'];

  // filter for js plugin to improve performance
  filters: string[];

  constructor(
    rawPlugin: Plugin,
    farmConfig: UserConfig,
    filters: string[],
    logger: Logger
  ) {
    this.name = rawPlugin.name;
    if (!rawPlugin.name) {
      throw new Error(
        `Vite plugin ${rawPlugin} is not compatible with Farm for now. Because plugin name is required in Farm.`
      );
    }

    this.priority = convertEnforceToPriority(rawPlugin.enforce);
    this._rawPlugin = rawPlugin;
    this._farmConfig = farmConfig;
    this._viteConfig = farmUserConfigToViteConfig(farmConfig);
    this._logger = logger;

    this.filters = filters;

    const hooksMap = {
      buildStart: () =>
        (this.buildStart = this.viteBuildStartToFarmBuildStart()),
      resolveId: () => (this.resolve = this.viteResolveIdToFarmResolve()),
      load: () => (this.load = this.viteLoadToFarmLoad()),
      transform: () => (this.transform = this.viteTransformToFarmTransform()),
      buildEnd: () => (this.buildEnd = this.viteBuildEndToFarmBuildEnd()),
      closeBundle: () => (this.finish = this.viteCloseBundleToFarmFinish()),
      handleHotUpdate: () =>
        (this.updateModules = this.viteHandleHotUpdateToFarmUpdateModules()),
      renderChunk: () =>
        (this.renderResourcePot =
          this.viteHandleRenderChunkToFarmRenderResourcePot()),
      renderStart: () =>
        (this.renderStart = this.viteRenderStartToFarmRenderStart()),
      augmentChunkHash: () =>
        (this.augmentResourceHash =
          this.viteAugmentChunkHashToFarmAugmentResourceHash()),
      generateBundle: () =>
        (this.finalizeResources =
          this.viteGenerateBundleToFarmFinalizeResources()),
      writeBundle: () =>
        (this.writeResources = this.viteWriteBundleToFarmWriteResources())
    };
    const alwaysExecutedHooks = ['buildStart'];

    // convert hooks
    for (const [hookName, fn] of Object.entries(hooksMap)) {
      if (
        rawPlugin[hookName as keyof Plugin] ||
        alwaysExecutedHooks.includes(hookName)
      ) {
        fn();
      }
    }

    // if other unsupported vite plugins hooks are used, throw error
    const unsupportedHooks = [
      'moduleParsed',
      'renderError',
      'resolveDynamicImport',
      'resolveFileUrl',
      'resolveImportMeta',
      'shouldTransformCachedModule',
      'banner',
      'footer'
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
  async config(config: UserConfig) {
    this._farmConfig = config;
    this._viteConfig = farmUserConfigToViteConfig(this._farmConfig);

    const configHook = this.wrapRawPluginHook('config', this._rawPlugin.config);

    if (configHook) {
      this._viteConfig = proxyViteConfig(
        merge(
          this._viteConfig,
          await configHook(
            proxyViteConfig(this._viteConfig, this.name, this._logger),
            this.getViteConfigEnv()
          )
        ),
        this.name,
        this._logger
      );

      this._farmConfig = viteConfigToFarmConfig(
        this._viteConfig,
        this._farmConfig,
        this.name
      );
    }

    return this._farmConfig;
  }

  async configResolved(config: ResolvedUserConfig) {
    this._farmConfig = config;
    this._viteConfig = proxyViteConfig(
      farmUserConfigToViteConfig(config),
      this.name,
      this._logger
    );

    if (!this._rawPlugin.configResolved) return;

    const configResolvedHook = this.wrapRawPluginHook(
      'configResolved',
      this._rawPlugin.configResolved
    );

    if (configResolvedHook) {
      await configResolvedHook(this._viteConfig);
    }
  }

  async configureDevServer(devServer: DevServer) {
    const hook = this.wrapRawPluginHook(
      'configureServer',
      this._rawPlugin.configureServer
    );

    this._viteDevServer = createViteDevServerAdapter(
      this.name,
      this._viteConfig,
      devServer
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

  private getViteConfigEnv(): ConfigEnv {
    return {
      ssrBuild: this._farmConfig.compilation?.output?.targetEnv === 'node',
      command:
        this._farmConfig.compilation?.mode === 'production' ? 'build' : 'serve',
      mode: this._farmConfig.compilation?.mode
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
    hook?: ObjectHook<(...args: any[]) => any, { sequential?: boolean }>,
    farmContext?: CompilationContext,
    currentHandlingFile?: string
  ): (
    ...args: any[]
  ) => any | undefined | Promise<(...args: any[]) => any | undefined> {
    if (hook === undefined) {
      return undefined;
    }

    if (typeof hook === 'object') {
      if (!hook.handler) {
        return undefined;
      }

      const logWarn = (name: string) => {
        this._logger.warn(
          `Farm does not support '${name}' property of vite plugin ${this.name} hook ${hookName} for now. It will be ignored.`
        );
      };
      // TODO support order, if a hook has order, it should be split into two plugins
      if (hook.order) {
        logWarn('order');
      }
      if (hook.sequential) {
        logWarn('sequential');
      }

      hook = hook.handler;
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
        if (this._viteDevServer) {
          this._viteDevServer.moduleGraph.context = context;
        }
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
            VitePluginAdapter.isFarmInternalVirtualModule(params.importer)
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
            params.importer ?? ''
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
              moduleType:
                params.moduleType === VITE_PLUGIN_DEFAULT_MODULE_TYPE
                  ? formatTransformModuleType(id)
                  : params.moduleType
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

  private viteHandleRenderChunkToFarmRenderResourcePot(): JsPlugin['renderResourcePot'] {
    return {
      executor: this.wrapExecutor(
        async (param: RenderResourcePotParams, ctx) => {
          const hook = this.wrapRawPluginHook(
            'renderChunk',
            this._rawPlugin.renderChunk,
            ctx
          );

          const result: ReturnType<RenderChunkHook> = await hook(
            param.content,
            transformResourceInfo2RollupRenderedChunk(param.resourcePotInfo),
            {},
            {
              chunks: {}
            }
          );

          if (result) {
            if (typeof result === 'string') {
              return { content: result };
            } else if (typeof result === 'object') {
              return { content: result.code, sourceMap: result.map };
            }
          }
        }
      )
    };
  }

  private viteRenderStartToFarmRenderStart(): JsPlugin['renderStart'] {
    return {
      executor: this.wrapExecutor(async (param: Config['config'], ctx) => {
        const hook = this.wrapRawPluginHook(
          'renderStart',
          this._rawPlugin.renderStart,
          ctx
        ) as OmitThis<FunctionPluginHooks['renderStart']>;

        await hook(
          transformFarmConfigToRollupNormalizedOutputOptions(param),
          transformFarmConfigToRollupNormalizedInputOptions(param)
        );
      })
    };
  }

  private viteAugmentChunkHashToFarmAugmentResourceHash(): JsPlugin['augmentResourceHash'] {
    return {
      executor: this.wrapExecutor(async (param: ResourcePotInfo, context) => {
        const hook = this.wrapRawPluginHook(
          'augmentChunkHash',
          this._rawPlugin.augmentChunkHash,
          context
        ) as OmitThis<FunctionPluginHooks['augmentChunkHash']>;

        const hash = await hook?.(
          transformResourceInfo2RollupRenderedChunk(param)
        );

        return hash;
      })
    };
  }

  private viteGenerateBundleToFarmFinalizeResources(): JsPlugin['finalizeResources'] {
    return {
      executor: this.wrapExecutor(
        async (param: FinalizeResourcesHookParams, context) => {
          const hook = this.wrapRawPluginHook(
            'generateBundle',
            this._rawPlugin.generateBundle,
            context
          );

          const bundles = Object.entries(param.resourcesMap).reduce(
            (res, [key, val]) => {
              res[key] = transformResourceInfo2RollupResource(val);
              return res;
            },
            {} as OutputBundle
          );

          await hook?.(
            transformFarmConfigToRollupNormalizedOutputOptions(param.config),
            bundles
          );

          // call transformIndexHtml hook after finalizeResources hook
          const transformIndexHtmlHook = this.wrapRawPluginHook(
            'transformIndexHtml',
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore ignore type error
            this._rawPlugin.transformIndexHtml,
            context
          );

          for (const resource of Object.values(param.resourcesMap)) {
            if (resource.resourceType === 'html') {
              const result = await transformIndexHtmlHook?.(
                Buffer.from(resource.bytes).toString(),
                {
                  path: resource.name,
                  filename: resource.name,
                  server: this._viteDevServer,
                  bundle: bundles,
                  chunk: transformResourceInfo2RollupResource(resource)
                }
              );

              if (result && typeof result !== 'string') {
                throw new Error(
                  `Vite plugin "${this.name}" is not compatible with Farm for now. Cause it uses transformIndexHtmlHook and return non-string value. Farm only supports string return for transformIndexHtmlHook`
                );
              } else if (typeof result === 'string') {
                resource.bytes = [...Buffer.from(result)];
              }
            }
          }

          return Object.entries(bundles).reduce((res, [key, val]) => {
            res[key] = transformRollupResource2FarmResource(
              val,
              param.resourcesMap[key]
            );
            return res;
          }, {} as FinalizeResourcesHookParams['resourcesMap']);
        }
      )
    };
  }

  private viteWriteBundleToFarmWriteResources(): JsPlugin['writeResources'] {
    return {
      executor: this.wrapExecutor(
        async (param: FinalizeResourcesHookParams, context) => {
          const hook = this.wrapRawPluginHook(
            'writeBundle',
            this._rawPlugin.writeBundle,
            context
          );

          const bundles = Object.entries(param.resourcesMap).reduce(
            (res, [key, val]) => {
              res[key] = transformResourceInfo2RollupResource(val);
              return res;
            },
            {} as OutputBundle
          );

          await hook?.(
            transformFarmConfigToRollupNormalizedOutputOptions(param.config),
            bundles
          );
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
