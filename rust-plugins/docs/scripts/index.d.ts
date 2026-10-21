export interface DocsPrototypeOptions {
  /** Absolute path or path relative to the Farm root. Prototype protocol v1. */
  snapshotPath: string;
}
export interface DocsSiteOptions {
  /** JSON-serializable data validated by Rust. Public TS configuration types are provided by @farmfe/docs. */
  site: unknown;
  /** Node-only preflight. Emits a private Rust content plan for child builds. */
  prepare?: boolean;
  /** Rust-owned browser development plan; mutually exclusive with prepare. */
  development?: boolean;
}
export interface DocsPreparedOptions {
  /** Private preflight artifact, verified by Rust before loading compiled content. */
  planPath: string;
}
export type DocsPluginOptions = DocsPrototypeOptions | DocsSiteOptions | DocsPreparedOptions;
export default function docs(options: DocsPluginOptions): [string, DocsPluginOptions];
