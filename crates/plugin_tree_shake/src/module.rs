use farmfe_core::{
  module::{Module, ModuleId, ModuleSystem},
  swc_ecma_ast::Ident,
};

use crate::statement_graph::{ImportInfo, StatementGraph, StatementId};

#[derive(Debug, Clone)]
pub enum UsedIdent {
  SwcIdent(Ident),
  Default,
}

#[derive(Debug, Clone)]
pub enum UsedExports {
  All,
  Partial(Vec<UsedIdent>),
}

impl UsedExports {
  pub fn add_used_export(&mut self, used_export: UsedIdent) {
    match self {
      UsedExports::All => {
        *self = UsedExports::All;
      }
      UsedExports::Partial(self_used_exports) => self_used_exports.push(used_export),
    }
  }
}

pub struct TreeShakeModule {
  pub module_id: ModuleId,
  pub side_effects: bool,
  pub stmt_graph: StatementGraph,
  // used exports will be analyzed when tree shaking
  pub used_exports: UsedExports,
  pub module_system: ModuleSystem,
}

impl TreeShakeModule {
  pub fn new(module: &Module) -> Self {
    // 1. generate statement graph
    let ast = &module.meta.as_script().ast;
    let stmt_graph = StatementGraph::new(ast);

    // 2. set default used exports
    let used_exports = if module.side_effects {
      UsedExports::All
    } else {
      UsedExports::Partial(vec![])
    };

    Self {
      module_id: module.id.clone(),
      stmt_graph,
      used_exports,
      side_effects: module.side_effects,
      module_system: module.meta.as_script().module_system.clone(),
    }
  }

  pub fn imports(&self) -> Vec<ImportInfo> {
    let mut imports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(import) = &stmt.imports {
        imports.push(import.clone());
      }
    }

    imports
  }

  pub fn exports(&self) -> Vec<(Ident, StatementId)> {
    return vec![];
  }

  pub fn used_statements(&self) -> Vec<StatementId> {
    // 1. get used exports
    let used_exports_idents = self.used_exports_idents();

    // 2. analyze used statements starting from used exports

    return vec![];
  }

  pub fn used_exports_idents(&self) -> Vec<UsedIdent> {
    // match &self.used_exports {
    //   UsedExports::All => {
    //     // all exported identifiers are used
    //     self
    //       .exports()
    //       .iter()
    //       .map(|(export, _)| export.clone())
    //       .collect()
    //   }
    //   UsedExports::Partial(used_idents) => {
    //     // some exported identifiers are used, check if the used identifiers are exported, otherwise log a warning
    //     for ident in identifiers {
    //       if !self.exports().iter().any(|(export, _)| export == ident) {
    //         println!(
    //           "[warning] module `{}` does not export identifier `{:?}`",
    //           ident, self.module_id
    //         );
    //       }
    //     }
    //     identifiers.clone()
    //   }
    // }
    vec![]
  }
}
