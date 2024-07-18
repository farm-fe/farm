
export interface ScriptParseConfig {
  esConfig?: {
    jsx?: boolean;
    fnBind?: boolean;
    // Enable decorators.
    decorators?: boolean;

    // babel: `decorators.decoratorsBeforeExport`
    // Effective only if `decorator` is true.
    decoratorsBeforeExport?: boolean;
    exportDefaultFrom?: boolean;
    // Stage 3.
    importAssertions?: boolean;
    privateInObject?: boolean;
    allowSuperOutsideMethod?: boolean;
    allowReturnOutsideFunction?: boolean;
  };
  tsConfig?: {
    tsx?: boolean;
    decorators?: boolean;
    /// `.d.ts`
    dts?: boolean;
    noEarlyErrors?: boolean;
  };
}

export interface ScriptDecoratorsConfig {
  legacyDecorator: boolean;
  decoratorMetadata: boolean;
  /**
   * The version of the decorator proposal to use. 2021-12 or 2022-03
   * @default 2021-12
   */
  decoratorVersion: '2021-12' | '2022-03' | null;
  /**
   * @default []
   */
  includes: string[];
  /**
   * @default ["node_modules/"]
   */
  excludes: string[];
}

/**
 * Configuration ported from babel-preset-env
 */
export interface SwcPresetEnvOptions {
  mode?: 'usage' | 'entry';
  debug?: boolean;
  dynamicImport?: boolean;

  loose?: boolean;

  /// Skipped es features.
  ///
  /// e.g.)
  ///  - `core-js/modules/foo`
  skip?: string[];

  include?: string[];

  exclude?: string[];

  /**
   * The version of the used core js.
   *
   */
  coreJs?: string;

  targets?: any;

  path?: string;

  shippedProposals?: boolean;

  /**
   * Enable all transforms
   */
  forceAllTransforms?: boolean;
}

interface TerserEcmaVersion {

}

interface TerserManglePropertiesOptions {}

export interface TerserCompressOptions {
  arguments?: boolean;
  arrows?: boolean;

  booleans?: boolean;

  booleans_as_integers?: boolean;

  collapse_vars?: boolean;

  comparisons?: boolean;

  computed_props?: boolean;

  conditionals?: boolean;

  dead_code?: boolean;

  defaults?: boolean;

  directives?: boolean;

  drop_console?: boolean;

  drop_debugger?: boolean;

  ecma?: TerserEcmaVersion;

  evaluate?: boolean;

  expression?: boolean;

  global_defs?: any;

  hoist_funs?: boolean;

  hoist_props?: boolean;

  hoist_vars?: boolean;

  ie8?: boolean;

  if_return?: boolean;

  inline?: 0 | 1 | 2 | 3;

  join_vars?: boolean;

  keep_classnames?: boolean;

  keep_fargs?: boolean;

  keep_fnames?: boolean;

  keep_infinity?: boolean;

  loops?: boolean;
  // module        : false,

  negate_iife?: boolean;

  passes?: number;

  properties?: boolean;

  pure_getters?: any;

  pure_funcs?: string[];

  reduce_funcs?: boolean;

  reduce_vars?: boolean;

  sequences?: any;

  side_effects?: boolean;

  switches?: boolean;

  top_retain?: any;

  toplevel?: any;

  typeofs?: boolean;

  unsafe?: boolean;

  unsafe_passes?: boolean;

  unsafe_arrows?: boolean;

  unsafe_comps?: boolean;

  unsafe_function?: boolean;

  unsafe_math?: boolean;

  unsafe_symbols?: boolean;

  unsafe_methods?: boolean;

  unsafe_proto?: boolean;

  unsafe_regexp?: boolean;

  unsafe_undefined?: boolean;

  unused?: boolean;

  const_to_let?: boolean;

  module?: boolean;
}

/**
 * @example ToSnakeCase<'indentLevel'> == 'indent_level'
 */
type ToSnakeCase<T extends string> = T extends `${infer A}${infer B}`
  ? `${A extends Lowercase<A> ? A : `_${Lowercase<A>}`}${ToSnakeCase<B>}`
  : T;

/**
 * @example ToSnakeCaseProperties<{indentLevel: 3}> == {indent_level: 3}
 */
type ToSnakeCaseProperties<T> = {
  [K in keyof T as K extends string ? ToSnakeCase<K> : K]: T[K];
};

export interface TerserMangleOptions {
  props?: TerserManglePropertiesOptions;

  toplevel?: boolean;

  keep_classnames?: boolean;

  keep_fnames?: boolean;

  keep_private_props?: boolean;

  ie8?: boolean;

  safari10?: boolean;

  reserved?: string[];
}

export interface JsMinifyOptions {
  compress?: ToSnakeCaseProperties<TerserCompressOptions> | boolean;

  mangle?: ToSnakeCaseProperties<TerserMangleOptions> | boolean;

  include?: string[];

  /**
   * @example ['.min.js$']
   */
  exclude?: string[];

  /**
   *
   * @default 'minify-module'
   */
  mode?: 'minify-module' | 'minify-resource-pot';

  /**
   * @default true
   */
  moduleDecls?: boolean;
}
