use crate::module::{module_graph::ModuleGraphEdge, Module, ModuleId};

pub struct PluginFreezeModuleHookParam<'a> {
  pub module: &'a mut Module,
  pub resolved_deps: Vec<(ModuleId, ModuleGraphEdge)>,
}
