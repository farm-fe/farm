//! Rust implementation of `@tailwindcss/node` compile utilities.
//!
//! This crate provides the compilation orchestration layer originally found in
//! the [`@tailwindcss-node`](https://github.com/tailwindlabs/tailwindcss/tree/main/packages/%40tailwindcss-node/src)
//! TypeScript package.
//!
//! It includes:
//! - CSS / JS module resolution (ESM + CJS dual-resolver pattern) — [`resolve`]
//! - Stylesheet loading and URL rewriting — [`urls`], [`compile`]
//! - JS module dependency tracing — [`get_module_dependencies`]
//! - Cross-platform path normalization — [`normalize_path`]
//! - Source‑map utilities — [`source_maps`]
//! - CSS AST node representation — [`compile::AstNode`]
//! - Top‑level `compile` and `compile_ast` orchestration — [`compile`]
//! - Unstable `load_design_system` API — [`compile::load_design_system`]
//! - Environment variable (DEBUG flag) resolution — [`env`]
//! - CSS optimization via `lightningcss` — [`optimize`]
//! - Performance instrumentation (hit counts + timers) — [`instrumentation`]

pub mod compile;
pub mod env;
pub mod get_module_dependencies;
pub mod instrumentation;
pub mod normalize_path;
pub mod optimize;
pub mod resolve;
pub mod source_maps;
pub mod urls;
