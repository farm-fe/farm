use std::{ffi::OsStr, sync::Arc};

use farmfe_core::{
  VERSION,
  config::Config,
  error::{CompilationError, Result},
  plugin::Plugin,
};

use libloading::{Library, Symbol};

/// load rust plugin from the specified path
///
/// # Safety
/// The plugin is loaded as a dynamic library and it may be unsafe. We use core_version to control the compatibility.
pub unsafe fn load_rust_plugin<P: AsRef<OsStr> + std::fmt::Display>(
  filename: P,
  config: &Config,
  options: String,
) -> Result<(Arc<dyn Plugin>, Library)> {
  type PluginCreate = unsafe fn(config: &Config, options: String) -> Arc<dyn Plugin>;

  let load_error = |error: libloading::Error| CompilationError::GenericError(error.to_string());
  let lib = Library::new(filename.as_ref()).map_err(load_error)?;
  let core_version_fn: Symbol<unsafe fn() -> String> =
    lib.get(b"_core_version").map_err(load_error)?;
  let core_version = core_version_fn();

  if core_version != VERSION {
    return Err(CompilationError::GenericError(format!(
      "\nIncompatible Rust Plugin: core ABI {} does not match plugin ABI {:?}, plugin path: {}.
Install a plugin built for this exact core ABI, or rebuild the plugin against the matching Farm source and Rust toolchain.
The ABI is separate from the npm package version; matching only the 0.5 prefix is insufficient.
Plugin authors: use the toolchain pinned in that Farm checkout's rust-toolchain.toml.\n",
      VERSION, core_version, filename
    )));
  }

  let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create").map_err(load_error)?;
  let plugin = constructor(config, options);

  Ok((plugin, lib))
}
