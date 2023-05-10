export interface GlobalFarmCLIOptions {
  '--'?: string[];
  c?: boolean | string;
  config?: string;
  configPath?: string;
  m?: string;
  mode?: string;
  w?: boolean;
  watch?: boolean;
  port?: number;
}

export interface FarmCLIServerOptions {
  port?: string;
  open?: boolean;
  https?: boolean;
  hmr?: boolean;
  strictPort?: boolean;
}

export interface FarmCLIBuildOptions {
  outDir?: string;
  sourcemap?: boolean;
  minify?: boolean;
}

export interface FarmCLIPreviewOptions {
  open?: boolean;
  port?: number;
}
