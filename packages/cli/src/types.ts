export interface FarmCLICommonOptions {
  _?: string[];
  c?: boolean | string;
  config?: string;
  m?: 'development' | 'production' | string;
  mode?: 'development' | 'production' | string;
  base?: string;
  clearScreen?: boolean;
}

export interface FarmCLIServerOptions {
  _?: string[];
  root?: string;
  l?: boolean;
  lazy?: boolean;
  host?: string;
  port?: string | number;
  open?: boolean;
  hmr?: boolean;
  cors?: boolean;
  strictPort?: boolean;
}

export interface FarmCLIBuildOptions {
  _?: string[];
  root?: string;
  input?: string;
  outDir?: string;
  sourcemap?: boolean;
  minify?: boolean;
  treeShaking?: boolean;
  format?: 'cjs' | 'esm' | string;
  target?:
    | 'browser'
    | 'node'
    | 'node16'
    | 'node-legacy'
    | 'node-next'
    | 'browser-legacy'
    | 'browser-es2015'
    | 'browser-es2017'
    | 'browser-esnext'
    | string;
}

export interface NormalizedFarmCLIBuildOptions extends FarmCLIBuildOptions {
  format: 'cjs' | 'esm';
  target:
    | 'browser'
    | 'node'
    | 'node16'
    | 'node-legacy'
    | 'node-next'
    | 'browser-legacy'
    | 'browser-es2015'
    | 'browser-es2017'
    | 'browser-esnext';
}

export interface GlobalFarmCLIOptions {
  _?: string[];
  c?: boolean | string;
  config?: string;
  configPath?: string;
  m?: string;
  base?: string;
  mode?: 'development' | 'production';
  w?: boolean;
  watch?: boolean;
  watchPath?: string;
  port?: number;
  lazy?: boolean;
  l?: boolean;
  clearScreen?: boolean;
}

export interface ICleanOptions {
  _?: string[];
  root?: string;
  path?: string;
  recursive?: boolean;
}

export interface FarmCLIPreviewOptions {
  _?: string[];
  root?: string;
  open?: boolean;
  port?: number | string;
}
