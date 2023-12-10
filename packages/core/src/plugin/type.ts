import {
  Config,
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult
} from '../../binding/index.js';
import { Compiler, DevServer } from '../index.js';

export interface CompilationContextEmitFileParams {
  resolvedPath: string;
  name: string;
  content: number[];
  resourceType: 'runtime' | 'js' | 'css' | 'html' | string;
}

export interface CompilationContext {
  resolve(
    param: PluginResolveHookParam,
    hookContext: { meta: Record<string, string>; caller: string }
  ): Promise<PluginResolveHookResult>;

  addWatchFile(currentFile: string, targetFile: string): void;
  emitFile(params: CompilationContextEmitFileParams): void;
  getWatchFiles(): string[];
  warn(message: string): void;
  error(message: string): void;
  sourceMapEnabled(id: string): boolean;

  viteGetModulesByFile(file: string): {
    url: string;
    id: string;
    file: string;
    type: 'js' | 'css';
  }[];
  viteGetImporters(file: string): {
    url: string;
    id: string;
    file: string;
    type: 'js' | 'css';
  }[];
}

type ModuleId = string;

export interface ResourcePot {
  id: string;
  name: string;
  resourcePotType: string;
  modules: ModuleId[];
  meta: any;
  entryModule?: ModuleId;
  resources: string[];
  moduleGroups: string[];
  immutable: boolean;
}

interface RenderedModule {
  id: ModuleId;
  renderedContent: string;
  renderedMap?: string;
  renderedLength: number;
  originalLength: number;
}

export interface ResourcePotInfo {
  id: string;
  resourcePotType: string;
  content: string;
  dynamicImports: string[];
  exports: string[];
  facadeModuleId?: string;
  fileName: string;
  implicitlyLoadedBefore: string[];
  imports: string[];
  importedBindings: Record<string, string[]>;
  isDynamicEntry: boolean;
  isEntry: boolean;
  isImplicitEntry: boolean;
  map?: string;
  modules: Record<ModuleId, RenderedModule>;
  moduleIds: ModuleId[];
  name: string;
  preliminaryFileName: string;
  referencedFiles: string[];
  ty: string;
}
export interface RenderResourcePotParams {
  content: string;
  resourcePotInfo: ResourcePotInfo;
}
export interface RenderResourcePotResult {
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

export type FinalizeResourcesHookParams = {
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

  config?: Callback<Config['config'], Config['config']>;

  // config?: (
  //   config: Config['config'],
  //   configEnv?: ConfigEnv
  // ) => Config['config'] | Promise<Config['config']>;

  // configResolved?: (config: Config['config']) => void;

  /**
   * runs in development mode only
   * @param server
   * @returns
   */
  configDevServer?: (server: DevServer) => void;
  /**
   * runs in production mode only
   * @param server
   * @returns
   */
  configCompiler?: (compiler: Compiler) => void;

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

  finish?: { executor: Callback<Record<string, never>, void> };

  updateModules?: {
    executor: Callback<
      { paths: [string, string][] },
      string[] | undefined | null | void
    >;
  };

  renderStart?: {
    executor: Callback<Config['config'], void>;
  };

  renderResourcePot?: {
    executor: Callback<RenderResourcePotParams, RenderResourcePotResult>;
  };

  augmentResourceHash?: {
    executor: Callback<ResourcePotInfo, string>;
  };

  finalizeResources?: {
    executor: Callback<
      FinalizeResourcesHookParams,
      FinalizeResourcesHookParams
    >;
  };

  pluginCacheLoaded?: {
    executor: Callback<number[], undefined | null | void>;
  };

  writePluginCache?: {
    executor: Callback<undefined, number[]>;
  };
}

export { rustPluginResolver } from './rust/rustPluginResolver.js';
