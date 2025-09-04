use farmfe_macro_cache_item::cache_item;

use crate::{module::Module, plugin::ResolveKind};

#[derive(Clone)]
pub struct PluginAnalyzeDepsHookParam<'a> {
  pub module: &'a Module,
  /// analyzed deps from previous plugins, you can push new entries to it for your plugin.
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
#[cache_item]
pub struct PluginAnalyzeDepsHookResultEntry {
  pub source: String,
  pub kind: ResolveKind,
}
