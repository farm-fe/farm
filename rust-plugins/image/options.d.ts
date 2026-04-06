export interface IPluginOptions {
  /**
   * Determines whether the processed images should be returned as `Image` DOM elements.
   * If set to true, the plugin will convert the image data into `Image` elements that can be directly used in the DOM.
   * This is useful when you need to manipulate or display images dynamically on the web page.
   * 
   * @type {boolean}
   * @default false
   */
  dom?: boolean;

  /**
   * Specifies an array of regex patterns to include specific image files for the plugin to process.
   * Each string in the array should be a valid regular expression used to match file paths.
   * This option is used to target images in specific directories or with certain file names that need processing.
   * 
   * @example
   *  ["assets/images/.*\\.(jpg|png)$"] // Includes all jpg and png images in 'assets/images' directory
   * 
   * @type {string[]}
   */
  include?: string[];

  /**
   * Specifies an array of regex patterns to exclude specific image files from the plugin processing.
   * Each string in the array should be a valid regular expression used to match file paths.
   * Use this to avoid processing images in certain directories or with specific characteristics.
   * 
   * @example
   *  ["assets/images/.*\\-thumbnail\\.(jpg|png)$"] // Excludes all thumbnail jpg and png images in 'assets/images' directory
   * 
   * @type {string[]}
   */
  exclude?: string[];
}
