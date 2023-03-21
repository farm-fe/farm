use farmfe_core::{
  module::{Module, ModuleId, ModuleSystem},
  swc_ecma_ast::Ident,
};

use crate::statement_graph::StatementGraph;

#[derive(Debug, Clone)]
pub enum UsedExports {
  All,
  Partial(Vec<String>),
}

impl UsedExports {
  pub fn add_used_exports(&mut self, used_exports: Vec<String>) {
    match self {
      UsedExports::All => {}
      UsedExports::Partial(self_used_exports) => {
        self_used_exports.extend(used_exports);
      }
    }
  }
}

pub struct TreeShakeModule {
  pub module_id: ModuleId,
  pub side_effects: bool,
  pub stmt_graph: StatementGraph,
  pub imports: Vec<(Ident, ModuleId)>,
  pub exports: Vec<(Ident, usize)>,
  // used exports will be analyzed when tree shaking
  pub used_exports: UsedExports,
  pub module_system: ModuleSystem,
}

impl TreeShakeModule {
  pub fn new(module: &Module) -> Self {
    // 1. generate statement graph
    let ast = &module.meta.as_script().ast;
    let stmt_graph = StatementGraph::new(ast);

    // 2. analyze imports and exports
    let mut imports = vec![];
    let mut exports = vec![];

    // 3. set default used exports
    let used_exports = if module.side_effects {
      UsedExports::All
    } else {
      UsedExports::Partial(vec![])
    };

    Self {
      module_id: module.id.clone(),
      stmt_graph,
      imports,
      exports,
      used_exports,
      side_effects: module.side_effects,
      module_system: module.meta.as_script().module_system.clone(),
    }
  }
}
