export interface TailwindCSSOptions {
  /**
   * Paths or patterns to scan for TailwindCSS candidates.
   */
  content?: string[];
  /**
   * JSON-serializable TailwindCSS configuration.
   *
   * When omitted, the plugin loads `tailwindcss.config.js` from the current
   * working directory if it exists.
   */
  config?: Record<string, unknown>;
}

export default function tailwindcss(
  options?: TailwindCSSOptions
): [string, TailwindCSSOptions];
