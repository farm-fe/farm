export interface IPluginOptions {
  limit?: number;
  publicPath?: string;
  emitFiles?: boolean;
  filename?: string;
  destDir?: string;
  sourceDir?: string;
  include?: string[];
  exclude?: string[];
}
