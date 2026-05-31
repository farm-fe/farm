export interface VueJsxPluginOptions {
  /** Convert `on` / `nativeOn` attrs to use @vue/babel-helper-vue-transform-on */
  transformOn?: boolean;
  /** Inject PatchFlags for optimized VNode updates */
  optimize?: boolean;
  /** Custom element detection patterns (regex strings) */
  customElementPatterns?: string[];
  /** Merge attribute objects with mergeProps from Vue */
  mergeProps?: boolean;
  /** Enable object slot detection via _isSlot helper */
  enableObjectSlots?: boolean;
  /** Custom pragma (e.g. h), overrides createVNode */
  pragma?: string;
  /** Resolve TypeScript types in defineComponent */
  resolveType?: boolean;
}
