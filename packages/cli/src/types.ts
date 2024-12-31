export interface GlobalCliOptions {
  '--'?: string[];
  c?: boolean | string;
  config?: string;
  base?: string;
  m?: string;
  mode?: string;
  clearScreen?: boolean;
}

interface BaseServerOptions {
  host?: string;
  port?: number;
  open?: boolean;
  cors?: boolean;
  strictPort?: boolean;
}

export interface CleanOptions {
  recursive?: boolean;
}

export interface CliServerOptions extends BaseServerOptions {
  hmr?: boolean;
}

export interface CliBuildOptions {
  o?: string;
  outDir?: string;
  i?: string;
  input?: string;
  w?: boolean;
  watch?: boolean;
  l?: boolean;
  lazy?: boolean;
  sourcemap?: boolean;
  minify?: boolean;
  treeShaking?: boolean;
  format?: 'cjs' | 'esm';
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
    | 'library';
}

export interface CliPreviewOptions extends BaseServerOptions {
  outDir?: string;
}
