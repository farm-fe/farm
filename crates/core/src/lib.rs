#![deny(clippy::all)]
// #![feature(unsize)]
// #![feature(trait_upcasting)]

pub mod cache;
pub mod config;
pub mod context;
pub mod error;
pub mod module;
pub mod plugin;
pub mod resource;
pub mod stats;

/// Version of this core crate, if the core data structures changed, and the changes will affect the memory layout,
/// like adding or removing a field, this version should be bumped. So plugin loader can recognize compatibility of the dynamic library plugins and the core.
pub const VERSION: &str = "0.1.0";

// re-export common external crates
pub use parking_lot;
pub use rayon;
pub use relative_path;
pub use serde;
pub use serde_json;
pub use swc_common;
pub use swc_ecma_ast;
