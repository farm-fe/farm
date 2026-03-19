export type SsrRuntimeCommand = 'dev' | 'preview';

export type SsrRuntimeAssets = {
  css: string[];
  preload: string[];
  scripts: string[];
};

export type SsrRuntimeMeta = {
  root: string;
  publicPath: string;
};
