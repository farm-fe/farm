export interface TailwindCSSOptions {
  /**
   * Reserved for explicit paths/globs. Current scanning follows Farm modules.
   */
  content?: string[];
  /** Emit flattened palette aliases such as --blue-500 in @layer base. Defaults to false. */
  colorVariables?: boolean;
  /**
   * JSON-serializable TailwindCSS configuration.
   *
   * When omitted, built-in defaults are used. JS config files are not loaded.
   */
  config?: Record<string, unknown>;
}

export default function tailwindcss(
  options?: TailwindCSSOptions
): [string, TailwindCSSOptions];
