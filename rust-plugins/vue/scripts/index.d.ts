export interface VueFeaturesOptions {
  /**
   * Drives the `__VUE_OPTIONS_API__` runtime define. Defaults to `true`,
   * matching `@vitejs/plugin-vue` and `unplugin-vue`.
   */
  optionsAPI?: boolean;
  /**
   * Drives `__VUE_PROD_DEVTOOLS__`. Defaults to `false`.
   */
  prodDevtools?: boolean;
  /**
   * Drives `__VUE_PROD_HYDRATION_MISMATCH_DETAILS__`. Defaults to `false`.
   */
  prodHydrationMismatchDetails?: boolean;
  /**
   * Enable reactive destructure for `defineProps`. Forwarded to fervid.
   */
  propsDestructure?: boolean;
  /**
   * Files matching this pattern are compiled as custom elements.
   *
   * @default /\.ce\.vue$/
   */
  customElement?: boolean | string | RegExp | (string | RegExp)[];
}

export interface VuePluginOptions {
  /**
   * Files matched by this filter are treated as Vue SFCs.
   *
   * @default /\.vue$/
   */
  include?: string | RegExp | (string | RegExp)[];
  /**
   * Files excluded from the Vue SFC pipeline.
   */
  exclude?: string | RegExp | (string | RegExp)[];

  /**
   * Force production mode regardless of Farm's compilation mode.
   */
  isProduction?: boolean;

  /**
   * Enable SSR codegen path. Experimental — fervid SSR support is partial.
   */
  ssr?: boolean;

  /**
   * Emit source maps for the compiled Vue output. Defaults to `true`.
   */
  sourceMap?: boolean;

  /**
   * @deprecated prefer `features.customElement`.
   */
  customElement?: boolean | string | RegExp | (string | RegExp)[];

  features?: VueFeaturesOptions;
}

/**
 * Farm Rust plugin to compile Vue 3 Single-File Components.
 *
 * Phase A — coarse HMR (full module reload on change). Granular HMR,
 * preprocessor re-scoping, custom blocks and type-dep tracking are tracked
 * as follow-up work; see the plugin docs for the full feature matrix.
 *
 * @example
 * ```ts
 * import vue from "@farmfe/plugin-vue";
 *
 * export default {
 *   plugins: [vue()],
 * };
 * ```
 */
declare const vue: (options?: VuePluginOptions) => [string, string];
export default vue;
