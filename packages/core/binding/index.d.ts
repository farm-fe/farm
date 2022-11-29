export type ModuleType =
  | 'js'
  | 'ts'
  | 'jsx'
  | 'tsx'
  | 'css'
  | 'html'
  | 'asset'
  | string;

export type ResolveKind =
  | 'entry'
  | 'import'
  | 'dynamicImport'
  | 'require'
  | 'cssAtImport'
  | 'cssUrl'
  | 'scriptSrc'
  | 'linkHref'
  | string;

export * from './binding';

/// Parameter of the resolve hook
export interface PluginResolveHookParam {
  /// the specifier would like to resolve, for example, './index'
  specifier: String;
  /// the start location to resolve `specifier`, being [None] if resolving a entry or resolving a hmr update.
  importer: String | null;
  /// for example, [ResolveKind::Import] for static import (`import a from './a'`)
  kind: ResolveKind;
  /// if this hook is called by the compiler, its value is [None]
  /// if this hook is called by other plugins, its value is set by the caller plugins.
  caller: String | null;
}

export interface PluginResolveHookResult {
  /// resolved id, normally a resolved path.
  id: String;
  /// whether this module should be external, if true, the module won't present in the final result
  external: boolean;
  /// whether this module has side effects, affects tree shaking
  side_effects: boolean;
  /// the package.json of the resolved id, if [None], using root package.json(where farm.config placed) by default
  package_json_info: Record<string, string> | null;
  /// the query parsed from specifier, for example, query should be `{ inline: true }` if specifier is `./a.png?inline`
  /// if you custom plugins, your plugin should be responsible for parsing query
  /// if you just want a normal query parsing like the example above, [crate::utils::parse_query] is for you
  query: Record<string, string> | null;
}

export interface PluginLoadHookParam {
  id: string;
  query: Record<string, string>;
  /// if this hook is called by the compiler, its value is [None]
  /// if this hook is called by other plugins, its value is set by the caller plugins.
  caller: string | null;
}

export interface PluginLoadHookResult {
  /// the source content of the module
  source: string;
  /// the type of the module, for example [ModuleType::Js] stands for a normal javascript file,
  /// usually end with `.js` extension
  module_type: ModuleType;
}

export interface PluginTransformHookParam {
  /// source content after load or transformed result of previous plugin
  source: string;
  /// module type after load
  module_type: ModuleType;
  id: string;
  query: Record<string, string>;
}

export interface PluginTransformHookResult {
  /// transformed source content, will be passed to next plugin.
  source: string;
  /// you can change the module type after transform.
  module_type: ModuleType | null;
  /// transformed source map, all plugins' transformed source map will be stored as a source map chain.
  source_map: String | null;
}

export interface Config {
  config?: {
    input?: Record<string, string>;
    output?: {
      filename?: string;
      path?: string;
      publicPath?: string;
    };
    resolve?: {
      extensions?: string[];
      alias?: Record<string, string>;
      mainFields?: string[];
      conditions?: string[];
      symlinks: boolean;
    };
    external?: string[];
    mode?: 'development' | 'production';
    root?: string;
    runtime?: {
      path: string;
      plugins?: string[];
    };
    script?: {
      // specify target es version
      target?:
        | 'es3'
        | 'es5'
        | 'es2015'
        | 'es2016'
        | 'es2017'
        | 'es2018'
        | 'es2019'
        | 'es2020'
        | 'es2021'
        | 'es2022';
      // config swc parser
      parser?: {
        esConfig?: {
          jsx?: boolean;
          fnBind: boolean;
          // Enable decorators.
          decorators: boolean;

          // babel: `decorators.decoratorsBeforeExport`
          //
          // Effective only if `decorator` is true.
          decoratorsBeforeExport: boolean;
          exportDefaultFrom: boolean;
          // Stage 3.
          importAssertions: boolean;
          privateInObject: boolean;
          allowSuperOutsideMethod: boolean;
          allowReturnOutsideFunction: boolean;
        };
        tsConfig?: {
          tsx: boolean;
          decorators: boolean;
          /// `.d.ts`
          dts: boolean;
          noEarlyErrors: boolean;
        };
      };
    };
  };
  jsPlugins?: object[];
  // [rustPluginFilePath, jsonStringifiedOptions]
  rustPlugins?: [string, string][];
}
