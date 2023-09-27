import type { TransformOptions } from '@babel/core';

export type FilterPattern =
  | ReadonlyArray<string | RegExp>
  | string
  | RegExp
  | null;

/** Possible options for the extensions property */
export interface ExtensionOptions {
  typescript?: boolean;
}

/** Configuration options for @farmfe/js-plugin-solid. */
export interface Options {
  /**
   * A [picomatch](https://github.com/micromatch/picomatch) pattern, or array of patterns, which specifies the files
   * the plugin should operate on.
   */
  include?: FilterPattern;
  /**
   * A [picomatch](https://github.com/micromatch/picomatch) pattern, or array of patterns, which specifies the files
   * to be ignored by the plugin.
   */
  exclude?: FilterPattern;
  /**
   * This will inject solid-js/dev in place of solid-js in dev mode. Has no
   * effect in prod. If set to `false`, it won't inject it in dev. This is
   * useful for extra logs and debugging.
   *
   * @default true
   */
  dev: boolean;
  /**
   * This will force SSR code in the produced files. This is experimental
   * and mostly not working yet.
   *
   * @default false
   */
  ssr: boolean;
  /**
   * This will inject HMR runtime in dev mode. Has no effect in prod. If
   * set to `false`, it won't inject the runtime in dev.
   *
   * @default true
   */
  hot: boolean;
  /**
   * This registers additional extensions that should be processed by
   * @farmfe/js-plugin-solid.
   *
   * @default undefined
   */
  extensions?: (string | [string, ExtensionOptions])[];
  /**
   * Pass any additional babel transform options. They will be merged with
   * the transformations required by Solid.
   *
   * @default {}
   */
  babel:
    | TransformOptions
    | ((source: string, id: string, ssr: boolean) => TransformOptions)
    | ((source: string, id: string, ssr: boolean) => Promise<TransformOptions>);
  typescript: {
    /**
     * Forcibly enables jsx parsing. Otherwise angle brackets will be treated as
     * typescript's legacy type assertion var foo = <string>bar;. Also, isTSX:
     * true requires allExtensions: true.
     *
     * @default false
     */
    isTSX?: boolean;

    /**
     * Replace the function used when compiling JSX expressions. This is so that
     * we know that the import is not a type import, and should not be removed.
     *
     * @default React
     */
    jsxPragma?: string;

    /**
     * Replace the function used when compiling JSX fragment expressions. This
     * is so that we know that the import is not a type import, and should not
     * be removed.
     *
     * @default React.Fragment
     */
    jsxPragmaFrag?: string;

    /**
     * Indicates that every file should be parsed as TS or TSX (depending on the
     * isTSX option).
     *
     * @default false
     */
    allExtensions?: boolean;

    /**
     * Enables compilation of TypeScript namespaces.
     *
     * @default uses the default set by @babel/plugin-transform-typescript.
     */
    allowNamespaces?: boolean;

    /**
     * When enabled, type-only class fields are only removed if they are
     * prefixed with the declare modifier:
     *
     * > NOTE: This will be enabled by default in Babel 8
     *
     * @default false
     *
     * @example
     * ```ts
     * class A {
     *   declare foo: string; // Removed
     *   bar: string; // Initialized to undefined
     *    prop?: string; // Initialized to undefined
     *    prop1!: string // Initialized to undefined
     * }
     * ```
     */
    allowDeclareFields?: boolean;

    /**
     * When set to true, the transform will only remove type-only imports
     * (introduced in TypeScript 3.8). This should only be used if you are using
     * TypeScript >= 3.8.
     *
     * @default false
     */
    onlyRemoveTypeImports?: boolean;

    /**
     * When set to true, Babel will inline enum values rather than using the
     * usual enum output:
     *
     * This option differs from TypeScript's --isolatedModules behavior, which
     * ignores the const modifier and compiles them as normal enums, and aligns
     * Babel's behavior with TypeScript's default behavior.
     *
     * ```ts
     *  // Input
     *  const enum Animals {
     *    Fish
     *  }
     *  console.log(Animals.Fish);
     *
     *  // Default output
     *  var Animals;
     *
     *  (function (Animals) {
     *    Animals[Animals["Fish"] = 0] = "Fish";
     *  })(Animals || (Animals = {}));
     *
     *  console.log(Animals.Fish);
     *
     *  // `optimizeConstEnums` output
     *  console.log(0);
     * ```
     *
     * However, when exporting a const enum Babel will compile it to a plain
     * object literal so that it doesn't need to rely on cross-file analysis
     * when compiling it:
     *
     * ```ts
     * // Input
     * export const enum Animals {
     *   Fish,
     * }
     *
     * // `optimizeConstEnums` output
     * export var Animals = {
     *     Fish: 0,
     * };
     * ```
     *
     * @default false
     */
    optimizeConstEnums?: boolean;
  };
  /**
   * Pass any additional [babel-plugin-jsx-dom-expressions](https://github.com/ryansolid/dom-expressions/tree/main/packages/babel-plugin-jsx-dom-expressions#plugin-options).
   * They will be merged with the defaults sets by [babel-preset-solid](https://github.com/solidjs/solid/blob/main/packages/babel-preset-solid/index.js#L8-L25).
   *
   * @default {}
   */
  solid: {
    /**
     * The name of the runtime module to import the methods from.
     *
     * @default "solid-js/web"
     */
    moduleName?: string;

    /**
     * The output mode of the compiler.
     * Can be:
     * - "dom" is standard output
     * - "ssr" is for server side rendering of strings.
     * - "universal" is for using custom renderers from solid-js/universal
     *
     * @default "dom"
     */
    generate?: 'ssr' | 'dom' | 'universal';

    /**
     * Indicate whether the output should contain hydratable markers.
     *
     * @default false
     */
    hydratable?: boolean;

    /**
     * Boolean to indicate whether to enable automatic event delegation on camelCase.
     *
     * @default true
     */
    delegateEvents?: boolean;

    /**
     * Boolean indicates whether smart conditional detection should be used.
     * This optimizes simple boolean expressions and ternaries in JSX.
     *
     * @default true
     */
    wrapConditionals?: boolean;

    /**
     * Boolean indicates whether to set current render context on Custom Elements and slots.
     * Useful for seemingly Context API with Web Components.
     *
     * @default true
     */
    contextToCustomElements?: boolean;

    /**
     * Array of Component exports from module, that aren't included by default with the library.
     * This plugin will automatically import them if it comes across them in the JSX.
     *
     * @default ["For","Show","Switch","Match","Suspense","SuspenseList","Portal","Index","Dynamic","ErrorBoundary"]
     */
    builtIns?: string[];
  };
}
