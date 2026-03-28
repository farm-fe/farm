#![deny(clippy::all)]

mod utils;

use farmfe_core::{
  config::Config,
  error::Result,
  module::ModuleType,
  plugin::{Plugin, PluginLoadHookResult, PluginResolveHookResult},
  serde_json,
};
use farmfe_toolkit::script::module_type_from_id;
use std::{collections::HashMap, path::Path, sync::Arc};
use thiserror::Error;
use utils::{normalize_path, path_join};

use farmfe_macro_plugin::farm_plugin;

const VIRTUAL_PREFIX: &str = "farm-virtual:";

/// Error types specific to virtual module operations
#[derive(Debug, Error)]
pub enum VirtualModuleError {
  #[error("Failed to parse virtual options: {0}")]
  OptionsParseError(#[from] serde_json::Error),
  #[error("Path operation failed: {0}")]
  PathError(#[from] utils::PathError),
}

/// A plugin that handles virtual modules in the Farm build system
#[derive(Debug)]
#[farm_plugin]
pub struct FarmPluginVirtualModule {
  virtual_modules: HashMap<String, Value>,
  resolved_paths: HashMap<String, Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
enum Value {
  Struct(StructValue),
  Str(String),
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct StructValue {
  raw: String,
  module_type: Option<ModuleType>,
}
impl FarmPluginVirtualModule {
  fn new(_: &Config, options: String) -> Self {
    let virtual_modules: HashMap<String, Value> =
      serde_json::from_str(&options).expect("Failed to parse virtual module options");

    let mut resolved_paths = HashMap::new();
    for (module_id, content) in &virtual_modules {
      if let Ok(resolved_path) = utils::resolve_path(module_id) {
        resolved_paths.insert(resolved_path, content.clone());
      }
    }

    Self {
      virtual_modules,
      resolved_paths,
    }
  }

  /// Checks if a given source is a virtual module
  fn is_virtual_module(&self, source: &str) -> bool {
    self.virtual_modules.contains_key(source)
  }

  /// Resolves a virtual module path
  fn resolve_virtual_path(&self, source: &str) -> Option<PluginResolveHookResult> {
    if self.is_virtual_module(source) {
      Some(PluginResolveHookResult {
        resolved_path: format!("{}{}", VIRTUAL_PREFIX, source),
        ..Default::default()
      })
    } else {
      None
    }
  }

  /// Resolves a relative import within a virtual module
  fn resolve_relative_import(
    &self,
    source: &str,
    importer: &str,
    root: &str,
  ) -> Option<PluginResolveHookResult> {
    let importer_path = if importer.starts_with(VIRTUAL_PREFIX) {
      &importer[VIRTUAL_PREFIX.len()..]
    } else {
      importer
    };

    let parts = [root, importer_path];
    let absolute_path = path_join(&parts).ok()?;

    let resolved = Path::new(&absolute_path).with_file_name(source);
    let resolved = normalize_path(resolved).to_string_lossy().to_string();

    if self.resolved_paths.contains_key(&resolved) {
      Some(PluginResolveHookResult {
        resolved_path: format!("{}{}", VIRTUAL_PREFIX, resolved),
        ..Default::default()
      })
    } else {
      None
    }
  }

  /// Loads content for a virtual module
  fn load_virtual_module(&self, id: &str) -> Option<PluginLoadHookResult> {
    self
      .virtual_modules
      .get(id)
      .or_else(|| self.resolved_paths.get(id))
      .map(|value| PluginLoadHookResult {
        content: match value {
          Value::Struct(s) => s.raw.clone(),
          Value::Str(s) => s.clone(),
        },
        module_type: match value {
          Value::Struct(s) => s.module_type
            .clone()
            .unwrap_or(module_type_from_id(id).unwrap_or(ModuleType::Js)),
          Value::Str(_) => module_type_from_id(id).unwrap_or(ModuleType::Js),
        },
        source_map: None,
      })
  }
}

impl Plugin for FarmPluginVirtualModule {
  fn name(&self) -> &str {
    "FarmPluginVirtual"
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    context: &Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    // Try direct virtual module resolution
    if let Some(result) = self.resolve_virtual_path(&param.source) {
      return Ok(Some(result));
    }

    // Try relative import resolution
    if let Some(importer) = &param.importer {
      if let Some(result) = self.resolve_relative_import(
        &param.source,
        importer.relative_path(),
        &context.config.root,
      ) {
        return Ok(Some(result));
      }
    }

    Ok(None)
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    if param.resolved_path.starts_with(VIRTUAL_PREFIX) {
      let id = &param.resolved_path[VIRTUAL_PREFIX.len()..];
      return Ok(self.load_virtual_module(id));
    }
    Ok(None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_virtual_module_creation() {
    let config = Config::default();
    let options = r#"{"test.js": "console.log('test');"}"#.to_string();

    let plugin = FarmPluginVirtualModule::new(&config, options);
    assert!(plugin.is_virtual_module("test.js"));
  }
}
