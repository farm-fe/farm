import {
  PluginResolveHookParam,
  PluginResolveHookResult,
} from '../../../binding';

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

  // load: JsPluginHook<{ filters: { ids: string[] }}>;
}

export { rustPluginResolver } from './rustPluginResolver';
