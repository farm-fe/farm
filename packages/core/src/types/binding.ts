/// reexport generate binding, compiles a type reference
export * from '../../binding/binding.js';

export type ModuleType = LiteralUnion<
  'ts' | 'js' | 'jsx' | 'tsx' | 'css' | 'html' | 'asset'
>;

export type ResolveKind =
  | { entry: string }
  | LiteralUnion<
      | 'import'
      | 'dynamicImport'
      | 'require'
      | 'cssAtImport'
      | 'cssUrl'
      | 'scriptSrc'
      | 'linkHref'
      | 'hmrUpdate'
    >;

import type { WatchOptions } from 'chokidar';
import type { JsPlugin, LiteralUnion } from '../plugin/type.js';
import {
  JsMinifyOptions,
  ScriptDecoratorsConfig,
  ScriptParseConfig,
  SwcPresetEnvOptions
} from './swc-config.js';
// export const bindingPath: string;

/// Parameter of the resolve hook
export interface PluginResolveHookParam {
  /// the start location to resolve `source`, being [None] if resolving a entry or resolving a hmr update.
  importer: string | null;
  /// for example, [ResolveKind::Import] for static import (`import a from './a'`)
  kind: ResolveKind;
  /// source of the import. for example in index.ts (import App from "./App.vue")
  /// source should be './App.vue'
  source: string;
}

export interface PluginResolveHookResult {
  /// resolved path, normally a absolute path. you can also return a virtual path, and use [PluginLoadHookResult] to provide the content of the virtual path
  resolvedPath: string;
  /// whether this module should be external, if true, the module won't present in the final result
  external?: boolean;
  /// whether this module has side effects, affects tree shaking
  sideEffects?: boolean;
  /// the query parsed from specifier, for example, query should be `{ inline: true }` if specifier is `./a.png?inline`
  /// if you custom plugins, your plugin should be responsible for parsing query
  /// if you just want a normal query parsing like the example above, [crate::utils::parse_query] is for you
  query?: [string, string][];
  /// meta data of the module, will be passed to [PluginLoadHookParam] and [PluginTransformHookParam]
  meta?: Record<string, string>;
}

export interface PluginLoadHookParam {
  moduleId: string;
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
  /// source map of the module
  sourceMap?: string | null;
}

export interface PluginTransformHookParam {
  moduleId: string;
  /// source content after load or transformed result of previous plugin
  content: string;
  /// module type after load
  moduleType: ModuleType;
  resolvedPath: string;
  query: [string, string][];
  meta: Record<string, string> | null;
  sourceMapChain: string[];
}

export interface PluginTransformHookResult {
  /// transformed source content, will be passed to next plugin.
  content: string;
  /// you can change the module type after transform.
  moduleType?: ModuleType;
  /// transformed source map, all plugins' transformed source map will be stored as a source map chain.
  sourceMap?: string | null;
  // ignore previous source map. if true, the source map chain will be cleared. and this result should return a new source map that combines all previous source map.
  ignorePreviousSourceMap?: boolean;
}

type BrowserTargetsRecord = Partial<
  Record<
    | 'chrome'
    | 'opera'
    | 'edge'
    | 'firefox'
    | 'safari'
    | 'ie'
    | 'ios'
    | 'android'
    | 'node'
    | 'electron',
    string
  >
> & { [key: string]: string };

export interface OutputConfig {
  /**
   * Configure the file name of the output files which contain the entry modules. Prior to filename
   */
  entryFilename?: string;
  /**
   * Configure the name of all the output files
   */
  filename?: string;
  /**
   * Output dir that production files are emitted to.
   */
  path?: string;
  /**
   * resource loading prefix. for example, if publicPath is `https://xxx.cdn.com`,
   * then the url output files in html will be `https://xxx.cdn.com/index_ecad.xxxx.js`
   *
   * default by `output.targetEnv`, if node, publicPath is `./`, if browser, publicPath is `/`
   */
  publicPath?: string;
  /**
   * the same as `filename`, but only for static assets like `.png`, `.jpg`
   */
  assetsFilename?: string;
  /**
   * Target execution environment of production files, browser or node. browser is equal to `browser-es2017`, node is equal to `node16`.
   * You can also set target env version like `node16`, `node-legacy`, 'browser-legacy`, 'browser-es2015', 'browser-2017', 'browser-esnext'. Farm will automatically downgrade syntax and inject polyfill according to the specified target env.
   * @default 'browser'
   */
  targetEnv?:
    | 'browser'
    | 'node'
    | 'node16'
    | 'node-legacy'
    | 'node-next'
    | 'browser-legacy'
    | 'browser-es2015'
    | 'browser-es2017'
    | 'browser-esnext'
    | 'library';
  /**
   * output module format
   */
  format?: 'cjs' | 'esm';
  /**
   * clean output.path automatically or not
   */
  clean?: boolean;
  /**
   * Whether to show print file size of final output files.
   */
  showFileSize?: boolean;
}

export interface ResolveConfig {
  /**
   * Configure the suffix when parsing dependencies. For example, when parsing ./index, if it is not resolved, the suffix parsing will be automatically added, such as trying ./index.tsx, ./index.css, etc.
   * @default ["tsx", "ts", "jsx", "js", "mjs", "json", "html", "css"]
   */
  extensions?: string[];
  /**
   * Configure parsing alias. Alias is prefix replacement, for example /@/pages/index will be replaced by /root/src/pages/index. If you want an exact match, you can add $, for example stream$ will only replace stream, but not stream/xxx.
   */
  // TODO customResolver?: ResolverFunction | ResolverObject
  alias?:
    | Record<string, string>
    | Array<{ find: string | RegExp; replacement: string }>;
  /**
   * When parsing dependencies under node_modules, the fields and order configured in mainFields will be parsed from package.json. For package.json
   * @default ["exports", "browser", "module", "main"]
   */
  mainFields?: string[];
  /**
   * Conditions of node package module spec
   */
  conditions?: string[];
  /**
   * When parsing a file, whether to track the real directory corresponding to the symlink, and start parsing the next dependency from the real directory. If pnpm is used to manage dependencies, this option must be configured as true.
   * @default true
   */
  symlinks?: boolean;
  /**
   * Whether to strictly follow the exports defined in exports in package.json. If set to true, when exports is defined in package.json, but exports does not define the corresponding export, an error will be reported directly. If set to true, it will continue to try other entries according to mainFields.
   * @default false
   */
  strictExports?: boolean;
  /**
   * If some modules can not be resolved, auto external it.
   * @default false
   */
  autoExternalFailedResolve?: boolean;
  /**
   *
   * @default []
   */
  dedupe?: string[];
}

export interface RuntimeConfig {
  /**
   * Customize a Runtime to replace Farm's built-in Runtime.
   * Note: t is not recommended to configure this option under normal circumstances, because once this option is configured, the pointed runtime needs to be compatible with Farm's runtime
   */
  path?: string;
  /**
   * Configure the Runtime plug-in, through the Runtime plug-in, you can intervene in Runtime behavior, such as module loading, resource loading, etc.
   */
  plugins?: string[];
  /**
   * Customize path of @swc/helpers
   * Note: It's not recommended to set this options
   */
  swcHelpersPath?: string;
  /**
   * Configure the namespace of Farm Runtime to ensure that the execution of different products under the same window or global can be isolated from each other.
   * By default, the name field of the project package.json is used as the namespace.
   */
  namespace?: string;
  /**
   * Whether to isolate the farm entry script, the default is false.
   * If set to true, the farm entry script will be emitted as a separate file.
   */
  isolate?: boolean;
}

export interface ScriptConfig {
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
    | 'es2022'
    | 'esnext';
  // config swc parser
  parser?: ScriptParseConfig;
  decorators?: ScriptDecoratorsConfig;
  /**
   * Configure the swc plugin array.
   */
  plugins?: {
    /**
     * the package name of the swc plugin
     */
    name: string;
    /**
     * Configuration items passed to swc plugin

     */
    options?: Record<string, any>;
    /**
     * Which modules to execute the plug-in, must be configured, support resolvedPaths and moduleTypes these two filter items, if both are specified at the same time, take the union.
     */
    filters?: {
      resolvedPaths?: string[];
      moduleTypes?: ModuleType[];
    };
  }[];
  /**
   * keep output entry file top level await, it is useful when building library
   *
   * @default false
   */
  nativeTopLevelAwait?: boolean;
  /**
   * https://www.typescriptlang.org/tsconfig/#importsNotUsedAsValues
   *
   * @default "remove"
   */
  importNotUsedAsValues?:
    | 'remove'
    | 'preserve'
    | {
        /**
         * modules that match any of this regex string array would treated as 'preserve'
         * otherwise, it would be 'remove'
         */
        preserve?: string[];
      };
}

export interface CssConfig {
  /**
   * Configure css modules
   */
  modules?: {
    // Configure which paths will be processed as css modules, using regular strings
    // defaults to `.module.css` or `.module.scss` or `.module.less`
    paths?: string[];
    // configure the generated css class name, the default is `[name]-[hash]`
    indentName?: string;
    /**
     *
     * - `asIs` - Do not convert the local variable name
     * - `lowerCamel` - Convert the local variable name to lower camel case e.g: `fooBar`
     * - `upperCamel` - Convert the local variable name to upper camel case e.g: `FooBar`
     * - `snake` - Convert the local variable name to snake case e.g: `foo_bar`
     *
     * @default 'asIs'
     */
    localsConversion?: 'asIs' | 'lowerCamel' | 'upperCamel' | 'snake';
  } | null;
  /**
   * Configure CSS compatibility prefixes, such as -webkit-.
   */
  prefixer?: {
    targets?: string[] | string | BrowserTargetsRecord;
  } | null;
  /**
   * You SHOULD NOT use this option. It's preserved vite css options for compatibility of vite plugins
   */
  _viteCssOptions?: any;
}

export interface GlobalBuiltinCacheKeyStrategy {
  /** @default true */
  define?: boolean;
  /** @default true */
  buildDependencies?: boolean;
  /** @default true */
  lockfile?: boolean;
  /** @default true */
  packageJson?: boolean;
  /** @default true */
  env?: boolean;
}

export interface PersistentCacheConfig {
  namespace?: string;
  cacheDir?: string;
  buildDependencies?: string[];
  moduleCacheKeyStrategy?: {
    timestamp?: boolean;
    hash?: boolean;
  };
  envs?: Record<string, string>;
  /**
   * Whether to ignore the built-in keys of the cache, such as define, buildDependencies, lockfile, etc.
   * If these keys are not ignored, the cache will be fully invalidated when these keys change.
   * @default {
   *  define: false,
   *  buildDependencies: true,
   *  lockfile: false
   * }
   */
  globalBuiltinCacheKeyStrategy?: GlobalBuiltinCacheKeyStrategy;
}

export interface PartialBundlingConfig {
  /**
   * Farm tries to generate resource numbers as closer as possible to this config value for initial resource loading or a dynamic resource loading.
   * @default 25
   */
  targetConcurrentRequests?: number;
  /**
   * The minimum size of each generated resources before minify and gzip.
   * @default 20KB
   */
  targetMinSize?: number;
  /**
   * The maximum size of generated resources before minify and gzip.
   * @default 1500KB
   */
  targetMaxSize?: number;
  /**
   * A group of modules that should be placed together when bundling.
   */
  groups?: {
    name: string;
    test: string[];
    groupType?: 'mutable' | 'immutable';
    resourceType?: 'all' | 'initial' | 'async';
  }[];
  /**
   * Array to match the modules that should always be in the same bundles, ignore all other constraints.
   */
  enforceResources?: {
    name: string;
    test: string[];
  }[];
  /**
   * Enforce target concurrent requests for every resource loading, when true, smaller resource will be merged into bigger resource to meet the target concurrent requests. this may cause issue for css resource, be careful to use this option.
   */
  enforceTargetConcurrentRequests?: boolean;
  /**
   * Enforce target min size for every resource, when tue, smaller resource will be merged into bigger resource to meet the target concurrent requests. this may cause issue for css resource, be careful to use this option
   */
  enforceTargetMinSize?: boolean;
  /**
   * Regex array to match the immutable modules.
   * @default ["node_modules"]
   */
  immutableModules?: string[];
}

export interface PresetEnvConfig {
  include?: string[];
  exclude?: string[];
  options?: SwcPresetEnvOptions;
  /**
   * @see https://babeljs.io/docs/assumptions
   */
  assumptions?: any;
}

export interface Config {
  config?: {
    coreLibPath?: string;
    /**
     * Compilation entries
     *
     * tip: if the value is `null` or `undefined`, it will be ignored
     */
    input?: Record<string, string | undefined | null>;
    /**
     * Compilation outputs
     */
    output?: OutputConfig;
    resolve?: ResolveConfig;
    /**
     * Global variable injection, the configured variable name and value will be injected into the product at compile time. Farm injects process.env.NODE_ENV and some variables used by Farm itself such as FARM_HMR_PORT by default
     */
    define?: Record<string, any>;
    /**
     * Configure the imports that are external, and the imports that are external will not appear in the compiled product.
     */
    external?: (string | Record<string, string>)[];
    externalNodeBuiltins?: boolean | string[];
    mode?: 'development' | 'production';
    root?: string;
    runtime?: RuntimeConfig;
    watch?: boolean | WatchOptions;
    assets?: {
      include?: string[];
      publicDir?: string;
      mode?: 'node' | 'browser';
    };
    script?: ScriptConfig;
    css?: CssConfig;
    html?: {
      base?: string;
    };
    /**
     * Configure whether to enable sourcemap, optional configuration items and descriptions are as follows:
      - true: Only generate sourcemap for files not under node_modules, and generate a separate sourcemap file
      - false: turn off sourcemap
      - inline: Only generate sourcemap for files not under node_modules, and inline sourcemap into the product, do not generate a separate file
      - all: generate sourcemap for all files, and generate a separate sourcemap file
      - all-inline: Generate sourcemaps for all files, and inline sourcemaps into the product, do not generate separate files
     */
    sourcemap?: boolean | 'inline' | 'all' | 'all-inline';
    /**
     * Configure the behavior of Farm's partial bundling. For details, please refer to https://farmfe.org/docs/features/partial-bundling
     */
    partialBundling?: PartialBundlingConfig;
    /**
     * Whether to enable lazy compilation, configure to false to disabled. See https://farmfe.org/docs/features/lazy-compilation
     */
    lazyCompilation?: boolean;
    /**
     * Whether to enable tree shake, set to false to disable. See https://farmfe.org/docs/features/tree-shake
     */
    treeShaking?: boolean;
    minify?: boolean | JsMinifyOptions;
    record?: boolean;
    progress?: boolean;
    presetEnv?: boolean | PresetEnvConfig;
    persistentCache?: boolean | PersistentCacheConfig;
    comments?: boolean | 'license';
    custom?: Record<string, any>;
    concatenateModules?: boolean;
  };
  jsPlugins?: JsPlugin[];
  // [rustPluginFilePath, jsonStringifiedOptions]
  rustPlugins?: [string, string][];
}
