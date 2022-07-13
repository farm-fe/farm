import {
  JsPluginResolveHookParam,
  JsPluginResolveHookResult,
} from '../../../binding';

interface CompilationContext {
  resolve(): JsPluginResolveHookResult;
}

type Callback<P, R> = (param: P, context: CompilationContext) => Promise<R> | R;
type JsPluginHook<F, P, R> = [F, Callback<P, R>];

export interface Plugin {
  resolve: JsPluginHook<
    {
      importer: string[];
      specifier: string[];
    },
    JsPluginResolveHookParam,
    JsPluginResolveHookResult
  >;
}
