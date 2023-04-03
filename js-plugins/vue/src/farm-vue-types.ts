import {
  SFCDescriptor,
  SFCScriptCompileOptions,
  SFCTemplateCompileOptions,
  SFCStyleCompileOptions,
} from '@vue/compiler-sfc';

export interface outputData {
  id: string;
  message: string | object;
}
export interface QueryObj {
  [key: string]: string | number | boolean;
}
export interface StylesCodeCache {
  [key: string]: string;
}

export type CacheDescriptor = Record<string, SFCDescriptor>;

export declare namespace Less {
  interface RootFileInfo {
    rewriteUrls?: boolean | undefined;
    filename: string;
    relativeUrls: boolean;
    rootpath: string;
    currentDirectory: string;
    entryPath: string;
    rootFilename: string;
    reference: boolean;
  }

  class PluginManager {
    constructor(less: LessStatic);
    addPreProcessor(preProcessor: PreProcessor, priority?: number): void;
    addFileManager(fileManager: FileManager): void;
  }

  interface Plugin {
    install: (less: LessStatic, pluginManager: PluginManager) => void;
    minVersion?: [number, number, number] | undefined;
  }

  interface PreProcessor {
    process: (src: string, extra: PreProcessorExtraInfo) => string;
  }

  interface PreProcessorExtraInfo {
    context: {
      pluginManager: PluginManager;
    };

    fileInfo: RootFileInfo;

    imports: {
      [key: string]: any;
    };
  }

  interface FileLoadResult {
    /** Full resolved path to file. */
    filename: string;

    /** The contents of the file, as a string. */
    contents: string;
  }

  interface FileLoadError {
    /** Error object if an error occurs. */
    error: unknown;
  }

  class FileManager extends AbstractFileManager {
    supports(
      filename: string,
      currentDirectory: string,
      options: LoadFileOptions,
      environment: Environment
    ): boolean;

    loadFile(
      filename: string,
      currentDirectory: string,
      options: LoadFileOptions,
      environment: Environment
    ): Promise<FileLoadResult>;

    loadFileSync(
      filename: string,
      currentDirectory: string,
      options: LoadFileOptions,
      environment: Environment
    ): FileLoadResult | FileLoadError;
  }

  class AbstractFileManager {
    getPath(filename: string): string;
    tryAppendLessExtension(filename: string): string;
    alwaysMakePathsAbsolute(): boolean;
    isPathAbsolute(path: string): boolean;
    join(basePath: string, laterPath: string): string;
    pathDiff(url: string, baseUrl: string): string;
    supportsSync(
      filename: string,
      currentDirectory: string,
      options: LoadFileOptions,
      environment: Environment
    ): boolean;
  }

  interface LoadFileOptions {
    paths?: string[] | undefined;
    prefixes?: string[] | undefined;
    ext?: string | undefined;
    rawBuffer?: any;
    syncImport?: boolean | undefined;
  }

  interface Environment {
    encodeBase64(str: string): string;
    mimeLookup(filename: string): string;
    charsetLookup(mime: string): string;
    getSourceMapGenerator(): any;
  }

  interface SourceMapOption {
    sourceMapURL?: string | undefined;
    sourceMapBasepath?: string | undefined;
    sourceMapRootpath?: string | undefined;
    outputSourceFiles?: boolean | undefined;
    sourceMapFileInline?: boolean | undefined;
  }

  interface StaticOptions {
    async: boolean;
    fileAsync: boolean;
    modifyVars: { [variable: string]: string };
  }

  interface ImportManager {
    contents: { [fileName: string]: string };
  }

  interface Options {
    sourceMap?: SourceMapOption | undefined;
    /** Filename of the main file to be passed to less.render() */
    filename?: string | undefined;
    /** The locations for less looking for files in @import rules */
    paths?: string[] | undefined;
    /** True, if run the less parser and just reports errors without any output. */
    lint?: boolean | undefined;
    /** Pre-load global Less.js plugins */
    plugins?: Plugin[] | undefined;
    /** @deprecated If true, compress using less built-in compression. */
    compress?: boolean | undefined;
    strictImports?: boolean | undefined;
    /** If true, allow imports from insecure https hosts. */
    insecure?: boolean | undefined;
    depends?: boolean | undefined;
    maxLineLen?: number | undefined;
    /** @deprecated If false, No color in compiling. */
    color?: boolean | undefined;
    /** @deprecated False by default. */
    ieCompat?: boolean | undefined;
    /** @deprecated If true, enable evaluation of JavaScript inline in `.less` files. */
    javascriptEnabled?: boolean | undefined;
    /** Whether output file information and line numbers in compiled CSS code. */
    dumpLineNumbers?: 'comment' | string | undefined;
    /** Add a path to every generated import and url in output css files. */
    rootpath?: string | undefined;
    /** Math mode options for avoiding symbol conficts on math expressions. */
    math?:
      | 'always'
      | 'strict'
      | 'parens-division'
      | 'parens'
      | 'strict-legacy'
      | number
      | undefined;
    /** If true, stops any warnings from being shown. */
    silent?: boolean | undefined;
    /** Without this option, Less attempts to guess at the output unit when it does maths. */
    strictUnits?: boolean | undefined;
    /** Defines a variable that can be referenced by the file. */
    globalVars?:
      | {
          [key: string]: string;
        }
      | undefined;
    /** Puts Var declaration at the end of base file. */
    modifyVars?:
      | {
          [key: string]: string;
        }
      | undefined;
    /** Read files synchronously in Node.js */
    syncImport?: boolean | undefined;
  }

  interface RenderError {
    column: number;
    extract: string[];
    filename: string;
    index: number;
    line: number;
    message: string;
    type: string;
  }

  interface RenderOutput {
    css: string;
    map: string;
    imports: string[];
  }

  interface RefreshOutput {
    endTime: Date;
    startTime: Date;
    sheets: number;
    totalMilliseconds: number;
  }
}

export interface LessStatic {
  options: Less.StaticOptions;

  importManager?: Less.ImportManager | undefined;
  sheets: HTMLLinkElement[];

  modifyVars(vars: { [name: string]: string }): Promise<Less.RefreshOutput>;

  refreshStyles(): void;

  render(
    input: string,
    callback: (
      error: Less.RenderError,
      output: Less.RenderOutput | undefined
    ) => void
  ): void;
  render(
    input: string,
    options: Less.Options,
    callback: (
      error: Less.RenderError,
      output: Less.RenderOutput | undefined
    ) => void
  ): void;

  render(input: string): Promise<Less.RenderOutput>;
  render(input: string, options: Less.Options): Promise<Less.RenderOutput>;

  refresh(
    reload?: boolean,
    modifyVars?: { [variable: string]: string },
    clearFileCache?: boolean
  ): Promise<Less.RefreshOutput>;

  version: number[];

  watch(): void;

  FileManager: typeof Less.FileManager;
  PluginManager: typeof Less.PluginManager;
}

export interface FarmVuePluginOptions {
  include?: string | RegExp | (string | RegExp)[];
  exclude?: string | RegExp | (string | RegExp)[];
  isProduction?: boolean;
  sourceMap?: boolean;
  script?: Partial<Pick<SFCScriptCompileOptions, 'babelParserPlugins'>>;
  template?: Partial<
    Pick<
      SFCTemplateCompileOptions,
      | 'compiler'
      | 'compilerOptions'
      | 'preprocessOptions'
      | 'preprocessCustomRequire'
      | 'transformAssetUrls'
    >
  >;
  style?: Partial<Pick<SFCStyleCompileOptions, 'trim'>>;
}

export interface ResolvedOptions {
  include: (string | RegExp)[];
  exclude: (string | RegExp)[];
  isProduction: boolean;
  sourceMap: boolean;
  script: Partial<Pick<SFCScriptCompileOptions, 'babelParserPlugins'>>;
  template: Partial<
    Pick<
      SFCTemplateCompileOptions,
      | 'compiler'
      | 'compilerOptions'
      | 'preprocessOptions'
      | 'preprocessCustomRequire'
      | 'transformAssetUrls'
    >
  >;
  style: Partial<Pick<SFCStyleCompileOptions, 'trim'>>;
}

export type QueryVue = {
  vue?: 'true';
  lang?: 'less' | 'css';
  t?: number;
  index?: number;
  scoped?: string;
  hash?: string;
  [key: string]: any;
};

export type Union<A, B> = A & B;
