import { Compiler, ResolvedUserConfig, Server, UserConfig } from '../index.js';
import {
  Config,
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult
} from '../types/binding.js';

// https://stackoverflow.com/questions/61047551/typescript-union-of-string-and-string-literals
// eslint-disable-next-line @typescript-eslint/ban-types
type LiteralUnion<T extends string> = T | (string & {});

type ResourcePotType = LiteralUnion<
  'runtime' | 'js' | 'css' | 'html' | 'asset'
>;

export interface CompilationContextEmitFileParams {
  resolvedPath: string;
  name: string;
  content: number[];
  resourceType: 'runtime' | 'js' | 'css' | 'html' | string;
}

export interface ViteModule {
  url: string;
  id: string;
  file: string;
  type: 'js' | 'css';
}

export interface CompilationContext {
  resolve(
    param: PluginResolveHookParam,
    hookContext: { meta: Record<string, unknown>; caller: string }
  ): Promise<PluginResolveHookResult>;

  addWatchFile(currentFile: string, targetFile: string): void;
  emitFile(params: CompilationContextEmitFileParams): void;
  getWatchFiles(): string[];
  warn(message: string): void;
  error(message: string): void;
  sourceMapEnabled(id: string): boolean;

  viteGetModulesByFile(file: string): ViteModule[];
  viteGetModuleById(id: string): ViteModule;
  viteGetImporters(file: string): ViteModule[];
}

type ModuleId = string;
export interface ResourcePot {
  id: string;
  name: string;
  resourcePotType: ResourcePotType;
  modules: ModuleId[];
  meta: {
    renderedModules: Record<ModuleId, RenderedModule>;
    renderedContent: string;
    renderedMapChain: string[];
    customData: Record<string, string>;
  };
  entryModule?: ModuleId;
  resources: string[];
  moduleGroups: string[];
  immutable: boolean;
  info: ResourcePotInfo;
}

export interface RenderedModule {
  id: ModuleId;
  renderedContent: string;
  renderedMap?: string;
  renderedLength: number;
  originalLength: number;
}

export interface ResourcePotInfo {
  id: string;
  name: string;
  resourcePotType: ResourcePotType;
  map?: string;
  modules: Record<ModuleId, RenderedModule>;
  moduleIds: ModuleId[];
  data: JsResourcePotInfoData;
  custom: Record<string, string>;
}

export interface JsResourcePotInfoData {
  dynamicImports: string[];
  exports: string[];
  imports: string[];
  importedBindings: Record<string, string[]>;
  isDynamicEntry: boolean;
  isEntry: boolean;
  isImplicitEntry: boolean;
}

export interface PluginRenderResourcePotParams {
  content: string;
  sourceMapChain: string[];
  resourcePotInfo: ResourcePotInfo;
}
export interface PluginRenderResourcePotResult {
  content: string;
  sourceMap?: string;
}

export interface Resource {
  name: string;
  bytes: number[];
  emitted: boolean;
  resourceType: string;
  origin: { type: 'ResourcePot' | 'Module'; value: string };
  info?: ResourcePotInfo;
}

export type PluginFinalizeResourcesHookParams = {
  resourcesMap: Record<string, Resource>;
  config: Config['config'];
};

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
  hookContext?: { caller?: string; meta: Record<string, unknown> }
) => Promise<R | null | undefined> | R | null | undefined;
type JsPluginHook<F, P, R> = { filters: F; executor: Callback<P, R> };

export interface JsPlugin {
  name: string;
  priority?: number;

  config?: (config: UserConfig) => UserConfig | Promise<UserConfig>;

  configResolved?: (config: ResolvedUserConfig) => void | Promise<void>;

  /**
   * runs in development mode only
   * @param server
   * @returns
   */
  configureDevServer?: (server: Server) => void | Promise<void>;
  /**
   * @param compiler
   * @returns
   */
  configureCompiler?: (compiler: Compiler) => void | Promise<void>;

  buildStart?: { executor: Callback<Record<string, never>, void> };

  resolve?: JsPluginHook<
    {
      importers: string[];
      sources: string[];
    },
    PluginResolveHookParam,
    PluginResolveHookResult
  >;

  load?: JsPluginHook<
    { resolvedPaths: string[] },
    PluginLoadHookParam,
    PluginLoadHookResult
  >;

  transform?: JsPluginHook<
    { resolvedPaths?: string[]; moduleTypes?: string[] },
    PluginTransformHookParam,
    PluginTransformHookResult
  >;

  buildEnd?: { executor: Callback<Record<string, never>, void> };

  renderStart?: {
    executor: Callback<Config['config'], void>;
  };

  renderResourcePot?: JsPluginHook<
    {
      resourcePotTypes?: ResourcePotType[];
      moduleIds?: string[];
    },
    PluginRenderResourcePotParams,
    PluginRenderResourcePotResult
  >;

  augmentResourceHash?: JsPluginHook<
    {
      resourcePotTypes?: ResourcePotType[];
      moduleIds?: string[];
    },
    ResourcePotInfo,
    string
  >;

  finalizeResources?: {
    executor: Callback<
      PluginFinalizeResourcesHookParams,
      PluginFinalizeResourcesHookParams['resourcesMap']
    >;
  };

  transformHtml?: {
    /** 0: pre, 1: normal, 2: post */
    order?: 0 | 1 | 2;
    executor: Callback<{ htmlResource: Resource }, Resource>;
  };

  writeResources?: {
    executor: (
      param: PluginFinalizeResourcesHookParams
    ) => void | Promise<void>;
  };

  pluginCacheLoaded?: {
    executor: Callback<number[], undefined | null | void>;
  };

  writePluginCache?: {
    executor: Callback<undefined, number[]>;
  };

  finish?: { executor: Callback<Record<string, never>, void> };
  updateFinished?: { executor: Callback<Record<string, never>, void> };

  updateModules?: {
    executor: Callback<
      { paths: [string, string][] },
      string[] | undefined | null | void
    >;
  };
}

export { rustPluginResolver } from './rust/rustPluginResolver.js';
export type {
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult
} from '../types/binding.js';
