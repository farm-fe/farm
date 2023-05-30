export interface GlobalFarmCLIOptions {
  '--'?: string[];
  c?: boolean | string;
  config?: string;
  configPath?: string;
  m?: string;
  mode?: 'development' | 'production';
  w?: boolean;
  watch?: boolean;
  watchPath?: string;
  port?: number;
  lazy?: boolean;
  l?: boolean;
  clearScreen?: boolean;
}

export interface FarmCLIServerOptions {
  port?: string;
  open?: boolean;
  https?: boolean;
  hmr?: boolean;
  strictPort?: boolean;
}

export interface FarmCLIBuildOptions {
  input?: string;
  outDir?: string;
  sourcemap?: boolean;
  minify?: boolean;
  treeShaking?: boolean;
}

export interface FarmCLIPreviewOptions {
  open?: boolean;
  port?: number;
}
