import type { ts, Diagnostic } from 'ts-morph';

export interface DtsPluginOptions {
  /**
   * Depends on the root directory
   *
   * By Default it base on 'root' option of your vite config
   */
  root?: string;

  /**
   * Declaration files output directory
   *
   * Can be specified a array to output to multiple directories
   *
   * By Default it base on 'build.outDir' option of your vite config
   */
  outputDir?: string | string[];

  /**
   * Manually set the root path of the entry files
   *
   * The output path of each file will be calculated base on it
   *
   * By Default it is the smallest public path for all files
   */
  entryRoot?: string;

  /**
   * Project init compilerOptions using by ts-morph
   *
   * @default null
   */
  compilerOptions?: ts.CompilerOptions | null;

  /**
   * Project init tsconfig.json file path by ts-morph
   *
   * Plugin also resolve include and exclude files from tsconfig.json
   *
   * @default 'tsconfig.json'
   */
  tsConfigPath?: string;

  /**
   * Whether transform dynamic import to static
   *
   * Force true when `rollupTypes` is effective
   *
   * eg. 'import('vue').DefineComponent' to 'import { DefineComponent } from "vue"'
   *
   * @default false
   */
  staticImport?: boolean;

  /**
   * Manual set include glob
   *
   * By Default it base on 'include' option of the tsconfig.json
   */
  include?: string | string[];

  /**
   * Manual set exclude glob
   *
   * By Default it base on 'exclude' option of the tsconfig.json, be 'node_module/**' when empty
   */
  exclude?: string | string[];

  /**
   * Bundled packages for rollup types
   *
   * See https://api-extractor.com/pages/configs/api-extractor_json/#bundledpackages
   *
   * @default []
   */
  bundledPackages?: string[];

  /**
   * Whether copy .d.ts source files into outputDir
   *
   * @default false
   * @remarks Before 2.0 it defaults to true
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
   * Skip type diagnostics means that type errors will not interrupt the build process
   *
   * But for the source files with type errors will not be emitted
   *
   * @default false
   * @remarks Before 1.7 it defaults to true
   */
  skipDiagnostics?: boolean;

  /**
   * Customize typescript lib folder path
   *
   * Should pass a relative path to root or a absolute path
   *
   * @default undefined
   */
  libFolderPath?: string;

  /**
   * Specify the log level of plugin
   *
   * By Default it base on 'logLevel' option of your vite config
   */
  /**
   * After emit diagnostic hook
   *
   * According to the length to judge whether there is any type error
   *
   * @default () => {}
   */
  afterDiagnostic?: (diagnostics: Diagnostic[]) => void | Promise<void>;
}
