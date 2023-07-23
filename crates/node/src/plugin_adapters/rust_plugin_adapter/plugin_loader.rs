use core::panic;
use std::{ffi::OsStr, sync::Arc};

use farmfe_core::{config::Config, plugin::Plugin, VERSION};

use libloading::{Error, Library, Symbol};

/// load rust plugin from the specified path
///
/// # Safety
/// The plugin is loaded as a dynamic library and it may be unsafe. We use core_version to control the compatibility.
pub unsafe fn load_rust_plugin<P: AsRef<OsStr> + std::fmt::Display>(
  filename: P,
  config: &Config,
  options: String,
) -> Result<(Arc<dyn Plugin>, Library), Error> {
  type PluginCreate = unsafe fn(config: &Config, options: String) -> Arc<dyn Plugin>;

  let lib = Library::new(filename.as_ref())?;

  let core_version_fn: Symbol<unsafe fn() -> String> = lib.get(b"_core_version")?;
  let core_version = core_version_fn();

  if core_version != VERSION {
    panic!(
      "Incompatible Rust Plugin: Current core's version({}) is not compatible with the plugin `{:?}` used ({}).\nRefer to xxx(TODO) for details.",
      VERSION, core_version, filename
    );
  }

  let constructor: Symbol<PluginCreate> = lib.get(b"_plugin_create")?;
  let plugin = constructor(config, options);

  Ok((plugin, lib))
}
