import type { Diagnostic, ts } from 'ts-morph';

export interface DtsPluginOptions {
  /**
   * match files
   *
   * @default
   * ```ts
   * [".ts$", ".tsx$"]
   * ```
   **/
  resolvedPaths?: string[];

  /**
   * Depends on the root directory
   */
  root?: string;

  /**
   * Declaration files output directory
   */
  outputDir?: string | string[];

  /**
   * set the root path of the entry files
   */
  entryRoot?: string;

  /**
   * Project init compilerOptions using by ts-morph
   */
  compilerOptions?: ts.CompilerOptions | null;

  /**
   * Project init tsconfig.json file path by ts-morph
   */
  tsConfigPath?: string;

  staticImport?: boolean;

  clearPureImport?: boolean;

  insertTypesEntry?: boolean;

  /**
   * set include glob
   */
  include?: string | string[];

  /**
   * set exclude glob
   */
  exclude?: string | string[];

  /**
   * Whether copy .d.ts source files into outputDir
   *
   * @default false
   */
  copyDtsFiles?: boolean;

  /**
   * Whether emit nothing when has any diagnostic
   *
   * @default false
   */
  noEmitOnError?: boolean;

  /**
   * Whether skip typescript diagnostics
   *
   * @default true
   */
  skipDiagnostics?: boolean;

  /**
   * Customize typescript lib folder path
   *
   * @default undefined
   */
  libFolderPath?: string;

  /**
   * According to the length to judge whether there is any type error
   */
  afterDiagnostic?: (diagnostics: Diagnostic[]) => void | Promise<void>;

  /**
   * Set which paths should exclude when transform aliases
   *
   * If it's regexp, it will test the original import path directly
   *
   * @default []
   */
  aliasesExclude?: (string | RegExp)[];
}
