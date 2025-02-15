use crate::module::Module;

use super::analyze_deps::PluginAnalyzeDepsHookResultEntry;

pub struct PluginFinalizeModuleHookParam<'a> {
  pub module: &'a mut Module,
  pub deps: &'a mut Vec<PluginAnalyzeDepsHookResultEntry>,
}
