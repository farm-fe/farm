/**
 * The value can be a string or an object with raw content and module type.
 * @example
 * // As a string
 * const options: IPluginOptions = {
 *   'virtual-module.js': "export default 'Hello, world!';"
 * };
 *
 * // As an object
 * const options: IPluginOptions = {
 *   'virtual-module.js': {
 *     raw: "export default 'Hello, world!';",
 *     moduleType: "js"
 *   }
 * };
 */
type Value = string | {
  /**
   * The raw content of the module.
   * @example "export default 'Hello, world!';"
   */
  raw: string;
  /**
   * The type of the module.
   * @default "js"
   */
  moduleType: "js" | "ts" | "jsx" | "tsx" | 'css' | 'html' | 'json' | 'asset';
};
export type IPluginOptions = Record<string, Value>;
