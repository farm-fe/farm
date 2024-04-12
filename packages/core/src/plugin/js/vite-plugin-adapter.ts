import {
  JsPlugin,
  CompilationContext,
  PluginRenderResourcePotParams,
  PluginFinalizeResourcesHookParams,
  ResourcePotInfo,
  Resource
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
  FARM_CSS_MODULES_SUFFIX,
  transformFarmConfigToRollupNormalizedOutputOptions,
  transformResourceInfo2RollupRenderedChunk,
  transformRollupResource2FarmResource,
  VITE_PLUGIN_DEFAULT_MODULE_TYPE,
  normalizePath,
  revertNormalizePath,
  normalizeAdapterVirtualModule,
  isStartsWithSlash,
  removeQuery
} from './utils.js';
import type { ResolvedUserConfig, UserConfig } from '../../config/types.js';
import type { Server } from '../../server/index.js';

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
import fse from 'fs-extra';
import {
  Config,
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult
} from '../../../binding/index.js';
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
import { applyHtmlTransform } from './apply-html-transform.js';
import merge from '../../utils/merge.js';
import { CompilationMode } from '../../config/env.js';

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
  transformHtml: JsPlugin['transformHtml'];

  // filter for js plugin to improve performance
  filters: string[];

  constructor(
    rawPlugin: Plugin,
    farmConfig: UserConfig,
    filters: string[],
    logger: Logger,
    mode: CompilationMode
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
      // closeBundle: () => (this.finish = this.viteCloseBundleToFarmFinish()),
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
      transformIndexHtml: () =>
        (this.transformHtml = this.viteTransformIndexHtmlToFarmTransformHtml()),
      'writeBundle|closeBundle': () =>
        (this.writeResources = this.viteWriteCloseBundleToFarmWriteResources())
    };
    const alwaysExecutedHooks = ['buildStart'];
    const productionOnlyHooks = [
      'renderChunk',
      'generateBundle',
      'renderStart',
      'closeBundle',
      'writeBundle'
    ];

    // convert hooks
    for (const [hookNameGroup, fn] of Object.entries(hooksMap)) {
      const hookNames = hookNameGroup.split('|');

      for (const hookName of hookNames) {
        if (
          rawPlugin[hookName as keyof Plugin] ||
          alwaysExecutedHooks.includes(hookName)
        ) {
          if (mode !== 'production' && productionOnlyHooks.includes(hookName)) {
            continue;
          }

          fn();
        }
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

  get api() {
    return this._rawPlugin.api;
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

  async configureDevServer(devServer: Server) {
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
      isSsrBuild: this._farmConfig.compilation?.output?.targetEnv === 'node',
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
        isSsrBuild: this._farmConfig.compilation.output?.targetEnv === 'node'
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
          `Farm does not support '${name}' property of vite plugin ${this.name} hook ${hookName} for now. '${name}' property will be ignored.`
        );
      };
      const supportedHooks = ['transformIndexHtml'];
      // TODO support order, if a hook has order, it should be split into two plugins
      if (hook.order && !supportedHooks.includes(hookName)) {
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
          context: CompilationContext,
          hookContext?: { caller?: string; meta: Record<string, unknown> }
        ): Promise<PluginResolveHookResult> => {
          if (
            VitePluginAdapter.isFarmInternalVirtualModule(params.source) ||
            (params.importer &&
              VitePluginAdapter.isFarmInternalVirtualModule(params.importer)) ||
            hookContext?.caller === this.name + '.resolveId'
          ) {
            return null;
          }

          const hook = this.wrapRawPluginHook(
            'resolveId',
            this._rawPlugin.resolveId,
            context
          );
          const absImporterPath = normalizePath(
            path.resolve(process.cwd(), params.importer ?? '')
          );
          let resolveIdResult: ResolveIdResult = await hook?.(
            decodeStr(params.source),
            absImporterPath,
            { isEntry: params.kind === 'entry' }
          );

          if (isString(resolveIdResult)) {
            resolveIdResult = normalizeAdapterVirtualModule(resolveIdResult);
            return {
              resolvedPath: removeQuery(encodeStr(resolveIdResult)),
              query: customParseQueryString(resolveIdResult),
              sideEffects: false,
              external: false,
              meta: {}
            };
          } else if (isObject(resolveIdResult)) {
            const resolveId = normalizeAdapterVirtualModule(
              resolveIdResult?.id
            );
            return {
              resolvedPath: removeQuery(encodeStr(resolveId)),
              query: customParseQueryString(resolveId),
              sideEffects: Boolean(resolveIdResult?.moduleSideEffects),
              // TODO support relative and absolute external
              external: Boolean(resolveIdResult?.external),
              meta: resolveIdResult.meta ?? {}
            };
          }

          // handles paths starting with / in the vite plugin,
          // returning the correct path if the file exists in our root path
          const rootAbsolutePath = path.join(
            this._farmConfig.root,
            params.source
          );

          if (
            isStartsWithSlash(params.source) &&
            fse.pathExistsSync(rootAbsolutePath)
          ) {
            return {
              resolvedPath: removeQuery(encodeStr(rootAbsolutePath)),
              query: customParseQueryString(rootAbsolutePath),
              sideEffects: false,
              external: false,
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
        resolvedPaths: this.filters
      },
      executor: this.wrapExecutor(
        async (
          params: PluginLoadHookParam,
          context: CompilationContext
        ): Promise<PluginLoadHookResult> => {
          if (VitePluginAdapter.isFarmInternalVirtualModule(params.moduleId)) {
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
          const resolvedPath = normalizePath(decodeStr(params.resolvedPath));

          // append query
          const id = formatId(resolvedPath, params.query);
          const result = await hook?.(id, isSSR ? { ssr: true } : undefined);

          if (result) {
            let map = undefined;

            if (typeof result === 'object' && result.map) {
              if (typeof result.map === 'string') {
                map = result.map;
              } else if (typeof result.map === 'object') {
                map = JSON.stringify(result.map);
              }
            }

            return {
              content: getContentValue(result),
              // only support css as first class citizen for vite plugins
              moduleType: formatLoadModuleType(id),
              sourceMap: map
              // does not support meta and sideEffects
            };
          }
        }
      )
    };
  }

  private viteTransformToFarmTransform(): JsPlugin['transform'] {
    // default module type and asset can be transformed by vite transform hook
    const moduleTypesCouldTransform = [
      VITE_PLUGIN_DEFAULT_MODULE_TYPE,
      'asset'
    ];
    return {
      filters: {
        resolvedPaths: this.filters
      },
      executor: this.wrapExecutor(
        async (
          params: PluginTransformHookParam,
          context: CompilationContext
        ): Promise<PluginTransformHookResult> => {
          if (VitePluginAdapter.isFarmInternalVirtualModule(params.moduleId)) {
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
          const resolvedPath = normalizePath(decodeStr(params.resolvedPath));
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
              moduleType: moduleTypesCouldTransform.includes(params.moduleType)
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
            const filename = normalizePath(file);
            const ctx: HmrContext = {
              file: filename,
              timestamp: Date.now(),
              modules: (mods ?? []).map(
                (m) =>
                  ({
                    ...m,
                    id: normalizePath(m.id),
                    file: normalizePath(m.file)
                  } as ModuleNode)
              ),
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

          return [...new Set(result)].map((id) => revertNormalizePath(id));
        }
      )
    };
  }

  private viteHandleRenderChunkToFarmRenderResourcePot(): JsPlugin['renderResourcePot'] {
    return {
      filters: {
        moduleIds: this.filters
      },
      executor: this.wrapExecutor(
        async (param: PluginRenderResourcePotParams, ctx) => {
          if (param.resourcePotInfo.resourcePotType !== 'js') {
            return;
          }

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
      filters: {
        moduleIds: this.filters
      },
      executor: this.wrapExecutor(async (param: ResourcePotInfo, context) => {
        if (param.resourcePotType !== 'js') {
          return;
        }

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
        async (param: PluginFinalizeResourcesHookParams, context) => {
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

          const result = Object.entries(bundles).reduce((res, [key, val]) => {
            res[key] = transformRollupResource2FarmResource(
              val,
              param.resourcesMap[key]
            );
            return res;
          }, {} as PluginFinalizeResourcesHookParams['resourcesMap']);

          return result;
        }
      )
    };
  }

  private viteTransformIndexHtmlToFarmTransformHtml(): JsPlugin['transformHtml'] {
    const rawTransformHtmlHook: any = this._rawPlugin.transformIndexHtml;
    const order: 'pre' | 'post' | 'normal' =
      rawTransformHtmlHook?.order ??
      rawTransformHtmlHook?.enforce ??
      this._rawPlugin.enforce ??
      'normal';

    const orderMap: Record<string, 0 | 1 | 2> = {
      pre: 0,
      normal: 1,
      post: 2
    };

    return {
      order: orderMap[order] ?? 1,
      executor: this.wrapExecutor(
        async (params: { htmlResource: Resource }, context) => {
          const { htmlResource } = params;
          const hook = this.wrapRawPluginHook(
            'transformIndexHtml',
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore ignore type error
            this._rawPlugin.transformIndexHtml,
            context
          );

          const result = await this.callViteTransformIndexHtmlHook(
            htmlResource,
            hook
          );

          if (result) {
            htmlResource.bytes = [...Buffer.from(result)];
          }

          return htmlResource;
        }
      )
    };
  }

  private viteWriteCloseBundleToFarmWriteResources(): JsPlugin['writeResources'] {
    return {
      executor: this.wrapExecutor(
        async (param: PluginFinalizeResourcesHookParams, context) => {
          const hook = this.wrapRawPluginHook(
            'writeBundle',
            this._rawPlugin.writeBundle,
            context
          );

          if (hook) {
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

          const closeBundle = this.wrapRawPluginHook(
            'closeBundle',
            this._rawPlugin.closeBundle
          );
          return closeBundle?.();
        }
      )
    };
  }

  private async callViteTransformIndexHtmlHook(
    resource: Resource,
    transformIndexHtmlHook?: (...args: any[]) => Promise<string>,
    bundles?: OutputBundle
  ) {
    const html = Buffer.from(resource.bytes).toString();
    const result = await transformIndexHtmlHook?.(html, {
      path: resource.name,
      filename: resource.name,
      server: bundles === undefined ? this._viteDevServer : undefined,
      bundle: bundles,
      chunk: transformResourceInfo2RollupResource(resource)
    });

    if (result && typeof result !== 'string') {
      return applyHtmlTransform(html, result);
    } else if (typeof result === 'string') {
      return result;
    }
  }

  // skip farm lazy compilation virtual module for vite plugin
  public static isFarmInternalVirtualModule(id: string) {
    return (
      id.startsWith(VIRTUAL_FARM_DYNAMIC_IMPORT_PREFIX) ||
      // css has been handled before the virtual module is created
      FARM_CSS_MODULES_SUFFIX.test(id)
    );
  }
}
