//! Types and Tools for the plugin system.
//! This crate should only be used by Rust plugins.

pub mod swc_ast;
pub mod swc_transforms;

pub use libloading;

pub fn load_core_lib(core_lib_path: &str) -> libloading::Library {
  unsafe { libloading::Library::new(core_lib_path).unwrap() }
}
