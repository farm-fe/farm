export interface TailwindCSSOptions {
  /**
   * Paths or patterns to scan for TailwindCSS candidates.
   */
  content?: string[];
}

declare const binPath: (options?: TailwindCSSOptions) => [string, typeof options];
export default binPath;
