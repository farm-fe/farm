export interface IPluginOptions {
  /** Enable Rust documentation metadata, heading IDs, GFM and admonitions. */
  docs?: boolean;
  /** Docs mode only. Auto uses Markdown for .md and MDX otherwise. */
  format?: 'auto' | 'markdown' | 'mdx';
  include?: string;
  exclude?: string;
  development?: boolean;
  providerImportSource?: string;
  jsx?: boolean;
  pragma?: string;
  pragmaFrag?: string;
  pragmaImportSource?: string;
  jsxImportSource?: string;
  filepath?: string;
  /** Native parser options; unspecified constructs keep their CommonMark defaults. */
  parse?: { constructs?: { gfmTable?: boolean } };
}
