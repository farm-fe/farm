# @farmfe/plugin-docs

Native Rust content planning for Farm Docs: content discovery, validation, route/module generation and the development content plan. The npm entry loads the matching native plugin and returns a Farm Rust plugin descriptor; it does not compile content in JavaScript.

For a complete documentation site use `defineDocsConfig` from `@farmfe/docs` and its public CLI workflow. Registering this plugin by itself does not perform browser/server compilation and static rendering.

The default export accepts `DocsPluginOptions` from `scripts/index.d.ts`: a serializable `site` with optional `prepare` or `development`, a private prepared `planPath`, or the prototype `snapshotPath` protocol. Prepare/development modes are mutually exclusive. Prepared plans and prototype snapshots are workflow protocols, not portable user configuration or public build artifacts.

Use Core and platform binaries from the same verified distribution. Current Farm Docs distribution and isolated consumer checks are in progress; platform metadata alone does not establish binary compatibility. LICENSE covers Farm code; THIRD_PARTY_NOTICES.md and licenses contain the known embedded syntax resource notices.
