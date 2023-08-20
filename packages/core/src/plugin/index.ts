import {
  Config,
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult
} from '../../binding/index.js';
import { DevServer } from '../index.js';

interface CompilationContext {
  resolve(
    param: PluginResolveHookParam,
    hookContext: { meta: Record<string, string>; caller: string }
  ): Promise<PluginResolveHookResult>;
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

  finish?: { executor: Callback<Record<string, never>, void> };
}

export { rustPluginResolver } from './rustPluginResolver.js';
