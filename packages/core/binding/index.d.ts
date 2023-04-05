export type ModuleType =
  | 'ts'
  | 'js'
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

export * from './binding.js';
import { Compiler } from './binding.js';

export default Compiler;
export const bindingPath: string;

/// Parameter of the resolve hook
export interface PluginResolveHookParam {
  /// the start location to resolve `source`, being [None] if resolving a entry or resolving a hmr update.
  importer: { relativePath: string; queryString: string | null } | null;
  /// for example, [ResolveKind::Import] for static import (`import a from './a'`)
  kind: ResolveKind;
  /// resolvedPath. for example in index.ts (import App from "./App.vue")
  /// source should be "path.resolve(process.cwd(),'./App.vue')"
  source: String;
}

export interface PluginResolveHookResult {
  /// resolved path, normally a absolute path. you can also return a virtual path, and use [PluginLoadHookResult] to provide the content of the virtual path
  resolvedPath: String;
  /// whether this module should be external, if true, the module won't present in the final result
  external: boolean;
  /// whether this module has side effects, affects tree shaking
  sideEffects: boolean;
  /// the query parsed from specifier, for example, query should be `{ inline: true }` if specifier is `./a.png?inline`
  /// if you custom plugins, your plugin should be responsible for parsing query
  /// if you just want a normal query parsing like the example above, [crate::utils::parse_query] is for you
  query: [string, string][] | null;
  /// meta data of the module, will be passed to [PluginLoadHookParam] and [PluginTransformHookParam]
  meta: Record<string, string> | null;
}

export interface PluginLoadHookParam {
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string> | null;
}

export interface PluginLoadHookResult {
  /// the content of the module
  content: string;
  /// the type of the module, for example [ModuleType::Js] stands for a normal javascript file,
  /// usually end with `.js` extension
  moduleType: ModuleType;
}

export interface PluginTransformHookParam {
  /// source content after load or transformed result of previous plugin
  content: string;
  /// module type after load
  moduleType: ModuleType;
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string> | null;
}

export interface PluginTransformHookResult {
  /// transformed source content, will be passed to next plugin.
  content: string;
  /// you can change the module type after transform.
  moduleType?: ModuleType;
  /// transformed source map, all plugins' transformed source map will be stored as a source map chain.
  sourceMap?: String | null;
}

export interface Config {
  config?: {
    coreLibPath?: string;
    input?: Record<string, string>;
    output?: {
      filename?: string;
      path?: string;
      publicPath?: string;
      assetsFilename?: string;
    };
    resolve?: {
      extensions?: string[];
      alias?: Record<string, string>;
      mainFields?: string[];
      conditions?: string[];
      symlinks?: boolean;
      strictExports?: boolean;
    };
    define?: Record<string, string>;
    external?: string[];
    mode?: 'development' | 'production';
    root?: string;
    runtime?: {
      path: string;
      plugins?: string[];
      swcHelpersPath?: string;
    };
    assets?: {
      include?: string[];
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
    sourcemap?: boolean | 'all';
    partialBundling?: {
      moduleBuckets?: {
        name: string;
        test: string[];
      }[];
    };
    lazyCompilation?: boolean;
  };
  jsPlugins?: object[];
  // [rustPluginFilePath, jsonStringifiedOptions]
  rustPlugins?: [string, string][];
}
