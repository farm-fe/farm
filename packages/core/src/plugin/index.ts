import {
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult,
} from '../../binding/index.js';

interface CompilationContext {
  resolve(param: PluginResolveHookParam): Promise<PluginResolveHookResult>;
}

type Callback<P, R> = (param: P, context: CompilationContext) => Promise<R> | R;
type JsPluginHook<F, P, R> = { filters: F; executor: Callback<P, R> };

export interface JsPlugin {
  resolve: JsPluginHook<
    {
      importers: string[];
      specifiers: string[];
    },
    PluginResolveHookParam,
    PluginResolveHookResult
  >;

  load: JsPluginHook<
    { resolved_paths: string[] },
    PluginLoadHookParam,
    PluginLoadHookResult
  >;

  transform: JsPluginHook<
    { resolved_paths: string[] },
    PluginTransformHookParam,
    PluginTransformHookResult
  >;
}

export { rustPluginResolver } from './rustPluginResolver.js';
