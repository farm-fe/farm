//! Rust implementation of `@tailwindcss/node` compile utilities.
//!
//! This crate provides the compilation orchestration layer originally found in
//! [`@tailwindcss-node/src/compile.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/compile.ts).
//!
//! It includes:
//! - CSS / JS module resolution (ESM + CJS dual-resolver pattern)
//! - Stylesheet loading and URL rewriting
//! - JS module dependency tracing
//! - Path normalization
//! - Source‑map utilities
//! - CSS AST node representation
//! - Top‑level `compile` and `compile_ast` orchestration
//! - Unstable `load_design_system` API

pub mod compile;
pub mod get_module_dependencies;
pub mod normalize_path;
pub mod resolve;
pub mod source_maps;
pub mod urls;
