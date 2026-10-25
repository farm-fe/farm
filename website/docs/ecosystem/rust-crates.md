# Rust crate directory

These 11 crates live in Farm's `rust-ecosystem/` directory; this is not a list of arbitrary third-party dependencies. Descriptions and versions refer to pinned source, with registry observations dated **2026-10-24**. Versions below are source manifest versions, not a claim of publication or stable public APIs.

[Pinned source and inventory](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem)

## Docs build components

These are integrated Docs support libraries, not independent documentation-site CLIs. Application authors should start with the [Farm Docs usage guide](/docs/farm-docs/usage).

### farmfe_docs_config

Docs configuration and validation/schema protocol; for native Docs integrations.

- Source version: `0.1.0`.
- State: Local candidate/integrated source; the audit found no crates.io entry under this exact name. No release-ahead installation command is provided.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-config) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-config/Cargo.toml).
- [License](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. Declared MIT; this is not a blanket clearance of upstream or transitive dependency licenses.

### farmfe_docs_content

Markdown/MDX semantics, headings, code, resources and source maps; for content tooling authors.

- Source version: `0.1.0`.
- State: Local candidate/integrated source; the audit found no crates.io entry under this exact name. No release-ahead installation command is provided.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-content) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-content/Cargo.toml).
- [License](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. Declared MIT; this is not a blanket clearance of upstream or transitive dependency licenses.

### farmfe_docs_plan

Prepared routes, links, assets and content plans; for native documentation build integrations.

- Source version: `0.1.0`.
- State: Local candidate/integrated source; the audit found no crates.io entry under this exact name. No release-ahead installation command is provided.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-plan) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-plan/Cargo.toml).
- [License](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. Declared MIT; this is not a blanket clearance of upstream or transitive dependency licenses.

### farmfe_docs_seo

Deterministic SEO and syndication resources from document plans; for documentation tool authors.

- Source version: `0.1.0`.
- State: Local candidate/integrated source; the audit found no crates.io entry under this exact name. No release-ahead installation command is provided.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-seo) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-seo/Cargo.toml).
- [License](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. Declared MIT; this is not a blanket clearance of upstream or transitive dependency licenses.

### farmfe_docs_snapshot

Source observations, input/output receipts and build verification; for build-system integrations.

- Source version: `0.1.0`.
- State: Local candidate/integrated source; the audit found no crates.io entry under this exact name. No release-ahead installation command is provided.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-snapshot) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/docs-snapshot/Cargo.toml).
- [License](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. Declared MIT; this is not a blanket clearance of upstream or transitive dependency licenses.

## Shared highlighting

### farmfe_syntax_highlight

Rust syntax highlighting, themes and static rendering shared by Docs and compiler plugins; for Rust tooling authors.

- Source version: `0.1.0`.
- State: Local candidate/integrated source; the audit found no crates.io entry under this exact name. No release-ahead installation command is provided.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/syntax-highlight) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/syntax-highlight/Cargo.toml).
- [License](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. Declared MIT; this is not a blanket clearance of upstream or transitive dependency licenses. [THIRD_PARTY_NOTICES.md](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/syntax-highlight/THIRD_PARTY_NOTICES.md) and theme notices must also be retained.

## Compilation and transformation

### farmfe_svgr_rs

SVG-to-React JSX/SWC transformation; for Rust transform integrations. This is a Farm fork of svgr-rs.

- Source version: `0.2.0`.
- State: [crates.io 0.2.0](https://crates.io/crates/farmfe_svgr_rs/0.2.0) exists; this does not establish that its archive equals the current source.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/svgr-rs) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/svgr-rs/Cargo.toml).
- [License](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/svgr-rs/LICENSE): MIT. Preserve original 2023 Cong-Cong Pan copyright and Farm fork attribution.

### farmfe_grass

Sass compiler fork with Farm source-map and import fixes, consumed by the native Sass plugin; for CSS tooling authors.

- Source version: `0.13.4-farm.1`.
- State: Local candidate/integrated source; the audit found no crates.io entry under this exact name. No release-ahead installation command is provided.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/grass) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/grass/Cargo.toml).
- [License](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/grass/LICENSE): MIT. Preserve 2020 Connor Skees copyright and [Grass upstream](https://github.com/connorskees/grass) attribution.

### farmfe_ecosystem_tailwindcss

Rust Tailwind compiler, utilities, variants, themes and scanning. JS/TS configuration and plugin compatibility remain incomplete.

- Source version: `0.1.0`.
- State: Internal integration; automated release configuration sets `release = false`. Registry availability was not verified by this audit; no published-release claim is made.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/tailwindcss) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/tailwindcss/Cargo.toml).
- [License](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. Declared MIT; this is not a blanket clearance of upstream or transitive dependency licenses. [Tailwind upstream notices](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-plugins/tailwindcss/THIRD_PARTY_NOTICES.md) accompany the corresponding distribution; full upstream compatibility is not claimed.

### farmfe_ecosystem_tailwindcss_node

Rust CSS/module resolution, imports, URLs, maps and dependency tracing; an integration component for the Tailwind compiler.

- Source version: `0.1.0`.
- State: Internal integration; automated release configuration sets `release = false`. Registry availability was not verified by this audit; no published-release claim is made.
- [Source](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/tailwindcss-node) · [Cargo.toml](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-ecosystem/tailwindcss-node/Cargo.toml).
- [License](https://github.com/farm-fe/farm/tree/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/LICENSE): MIT. Declared MIT; this is not a blanket clearance of upstream or transitive dependency licenses. [Tailwind upstream notices](https://github.com/farm-fe/farm/blob/07c61e2fd7acaf068708dca7cf2b223ac92a2a7b/rust-plugins/tailwindcss/THIRD_PARTY_NOTICES.md) accompany the corresponding distribution; full upstream compatibility is not claimed.

## Glob matching and traversal

### farmfe_ecosystem_glob

Pure Rust glob matching and import-glob traversal for Farm integrations. The scoped reference profiles are Vite **8.3.5**'s JavaScript transform/tinyglobby 0.2.17 (picomatch 4.0.7, fdir 6.5.0) and direct JavaScript HMR matching. This does not claim full picomatch, minimatch or Vite compatibility, including Vite's separate native Rolldown path.

- Source version: `0.1.0`.
- State: Local unpublished integration; no published-release claim or registry installation command is provided.
- [Source](https://github.com/farm-fe/farm/tree/79cdbe44c/rust-ecosystem/glob) · [Cargo.toml](https://github.com/farm-fe/farm/blob/79cdbe44c/rust-ecosystem/glob/Cargo.toml) · [Compatibility scope](https://github.com/farm-fe/farm/blob/79cdbe44c/rust-ecosystem/glob/README.md).
- Provenance: Derived from picomatch-rs `0.1.0`, upstream commit `ceb39c90fc719fbbab79216a139f31c174272ed3`; no NAPI wrapper or JavaScript RegExp is used. Preserve the MIT [UPSTREAM-LICENSE](https://github.com/farm-fe/farm/blob/79cdbe44c/rust-ecosystem/glob/UPSTREAM-LICENSE), including Jon Schlinkert's copyright, and Farm's MIT license. [UPSTREAM.patch](https://github.com/farm-fe/farm/blob/79cdbe44c/rust-ecosystem/glob/UPSTREAM.patch) records Farm's matcher changes.

## Release and provenance boundaries

This directory describes source capabilities; it does not replace distribution validation or license review. Ports, copied code and dependencies require their applicable upstream notices to be checked in actual npm/crate archives; unresolved permission evidence is not marked complete. The glob integration above is local and scoped; its source and compatibility profile are pinned separately from the original inventory.
