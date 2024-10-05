export interface GlobalCliOptions {
  '--'?: string[];
  c?: boolean | string;
  config?: string;
  base?: string;
  m?: string;
  mode?: 'development' | 'production' | string;
  clearScreen?: boolean;
  timeUnit?: 'ms' | 's';
}

export interface CleanOptions {
  recursive?: boolean;
}

export interface CliServerOptions {
  port?: number;
  open?: boolean;
  hmr?: boolean;
  cors?: boolean;
  strictPort?: boolean;
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
    | 'library'
    | 'library-browser'
    | 'library-node';
}

export interface CliPreviewOptions {
  host?: string | boolean;
  port?: number;
  open?: boolean | string;
  outDir?: string;
  strictPort?: boolean;
}
