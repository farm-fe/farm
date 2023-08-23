#![deny(clippy::all)]
#![allow(clippy::ptr_arg)]
#![feature(trivial_bounds)]
#![feature(is_some_and)]
#![feature(const_trait_impl)]
#![feature(hasher_prefixfree_extras)]

// #![feature(unsize)]
// #![feature(trait_upcasting)]

pub mod cache;
pub mod common;
pub mod config;
pub mod context;
pub mod error;
pub mod module;
pub mod plugin;
pub mod resource;
pub mod stats;
pub mod record;

pub use farmfe_macro_cache_item::cache_item;

/// Version of this core crate, if the core data structures changed, and the changes will affect the memory layout,
/// like adding or removing a field, this version should be bumped. So plugin loader can recognize compatibility of the dynamic library plugins and the core.
pub const VERSION: &str = "0.1.0";

// re-export common external crates
pub use dashmap;
pub use glob;
pub use hashbrown;
pub use parking_lot;
pub use petgraph;
#[cfg(feature = "profile")]
pub use puffin;
pub use rayon;
pub use regex;
pub use relative_path;
pub use rkyv;
pub use rkyv_dyn;
pub use rkyv_typename;
pub use serde;
pub use serde_json;
pub use swc_common;
pub use swc_css_ast;
pub use swc_ecma_ast;
pub use swc_ecma_parser;
pub use swc_html_ast;

#[macro_export]
macro_rules! farm_profile_scope {
  ($s:expr) => {
    #[cfg(feature = "profile")]
    let msg = farmfe_utils::transform_string_to_static_str($s);
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_scope!(msg);
  };
}

#[macro_export]
macro_rules! farm_profile_function {
  ($s:expr) => {
    #[cfg(feature = "profile")]
    let msg = farmfe_utils::transform_string_to_static_str($s);
    #[cfg(feature = "profile")]
    farmfe_core::puffin::profile_function!(msg);
  };
}
