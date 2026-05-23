export interface IPluginOptions {
  /**
   * Specifies the scaling factor for the zooming icon.
   * @type {number}
   * @default 1.2
   */
  scale?: number;

  /**
   * Indicates whether the plugin should automatically install required dependencies.
   * @type {boolean}
   * @default true
   */
  autoInstall?: boolean;

  /**
   * Specifies the compiler used by the plugin. This setting determines how to process the code.
   * @type {"jsx" | "vue" | "solid" | "svelte"}
   * @default "jsx"
   */
  compiler?: "jsx" | "vue" | "solid" | "svelte";

  /**
   * Specifies the JSX compiler to use. This option is necessary when the selected compiler is processing JSX syntax.
   * @type {"react" | "preact"}
   * @default "react"
   * 
   * Options:
   * - "react": Use the React JSX transformer.
   * - "preact": Use the Preact JSX transformer.
   */
  jsx?: "react" | "preact";

  /**
   * Specifies default CSS styles to apply to the SVG element. Use standard CSS properties in camelCase.
   * @type {Partial<CSSStyleDeclaration>}
   */
  defaultStyle?: Partial<CSSStyleDeclaration>;

  /**
   * Specifies a default CSS class to apply to the SVG element.
   * @type {string}
   */
  defaultClass?: string;

  /**
   * Defines a collection of custom icons. This can include both local and remote SVG files.
   * Use [iconname] as a placeholder within the string to represent the icon name.
   * @type {Record<string, string>}
   * @example
   *  import icon from "~icons/local/[iconname].svg"
   *  import icon from "~icons/remote/[iconname].svg"
   */
  customCollections?: Record<string, string>;
}
