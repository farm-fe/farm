#![deny(clippy::all)]
#![feature(unsize)]

pub mod cache;
pub mod config;
pub mod context;
pub mod error;
pub mod module;
pub mod plugin;
pub mod resource;
pub mod stats;
pub mod utils;

/// Version of this core crate, if the core data structures changed, and the changes will affect the memory layout,
/// like adding or removing a field, this version should be bumped. So plugin loader can recognize compatibility of the dynamic library plugins and the core.
pub const VERSION: &str = "0.1.0";

// re-export common external crates
pub use rayon;
pub use serde;
pub use serde_json;
