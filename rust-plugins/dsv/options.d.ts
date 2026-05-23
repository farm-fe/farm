export interface IPluginOptions {
  /**
   * Specifies an array of regex patterns to include files in the plugin process.
   * Each string in the array should be a valid regular expression used to match file paths.
   * 
   * @example
   *  ["src/.*\\.dsv$", "lib/.*\\.dsv$"] // Includes all TypeScript files in 'src' and 'lib' directories
   * 
   * @type {string[]}
   */
  include?: string[];

  /**
   * Specifies an array of regex patterns to exclude files from the plugin process.
   * Each string in the array should be a valid regular expression used to match file paths.
   * 
   * @example
   *  ["src/.*\\.test\\.dsv$"] // Excludes all test files in 'src' directory
   * 
   * @type {string[]}
   */
  exclude?: string[];
}
