use crate::module::{Module, ModuleId, module_graph::ModuleGraphEdge};

pub struct PluginFreezeModuleHookParam<'a> {
  pub module: &'a mut Module,
  pub resolved_deps: Vec<(ModuleId, ModuleGraphEdge)>,
}
