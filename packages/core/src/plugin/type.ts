import {
  Config,
  JsUpdateResult,
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult
} from '../../binding/index.js';
import { DevServer } from '../index.js';

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
}

type Callback<P, R> = (
  param: P,
  context?: CompilationContext,
  hookContext?: { caller?: string; meta: Record<string, unknown> }
) => Promise<R> | R;
type JsPluginHook<F, P, R> = { filters: F; executor: Callback<P, R> };

export interface JsPlugin {
  name: string;
  priority?: number;

  config?: Callback<Config['config'], Config['config']>;

  /**
   * runs in development mode only
   * @param server
   * @returns
   */
  configDevServer?: (server: DevServer) => void;

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
      { updateResult: JsUpdateResult; paths: [string, string][] },
      JsUpdateResult | undefined | null | void
    >;
  };
}

export { rustPluginResolver } from './rust/rustPluginResolver.js';
