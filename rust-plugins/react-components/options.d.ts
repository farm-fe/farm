interface Resolver {
  /**
   * The module name that the resolver will handle.
   */
  module: string;

  /**
   * Optional prefix to prepend to the generated import statements.
   */
  prefix?: string;

  /**
   * Specifies whether to import styles along with components. 
   * This can be a boolean to simply enable/disable importing styles, or a string to specify a custom path.
   * 
   * @example
   *  'style/index.js' // Custom path to style file
   */
  importStyle?: boolean | string;
}

export interface IPluginOptions {
  /**
   * Specifies the directories to search for components. Paths should be relative to the project root.
   */
  dirs?: string[],

  /**
   * Array of custom resolver configurations that define how components should be resolved.
   */
  resolvers?: Resolver[],

  /**
   * Determines the mode of importing, either as absolute or relative paths.
   *
   * @default "absolute"
   */
  importMode?: "absolute" | "relative",

  /**
   * Specifies whether the plugin is valid for local components.
   *
   * @default true
   */
  local?: boolean,

  /**
   * Controls whether styles are automatically imported with components. 
   * Can be set to a boolean to toggle this feature or a string to specify a path for the style files.
   *
   * @default false
   */
  importStyle?: boolean | string,

  /**
   * Generates a `components.d.ts` file for TypeScript types declaration. 
   * This can be set to a boolean to enable/disable, or a string to specify a custom path for the declaration file.
   * The default behavior is to enable this feature if TypeScript is detected in the project.
   */
  dts?: boolean | string,

  /**
   * Defines regex patterns to include components for automatic imports. 
   * This is not for filtering components to register. Use actual regex patterns to specify targets.
   *
   * @example
   *  ["src/components"] // Only components under `src/components` are included.
   */
  include?: string[],

  /**
   * Defines regex patterns to exclude components from being automatically imported.
   *
   * @example
   *  ["node_modules"] // Components in `node_modules` are excluded.
   */
  exclude?: string[],
}
