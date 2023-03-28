use farmfe_core::{
  config::Config,
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::Plugin,
};
use module::TreeShakeModule;
use statement_graph::ImportInfo;

pub mod module;
pub mod statement_graph;

pub struct FarmPluginTreeShake;

impl FarmPluginTreeShake {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginTreeShake {
  fn name(&self) -> &'static str {
    "FarmPluginTreeShake"
  }

  /// tree shake useless modules and code, steps:
  /// 1. topo sort the module_graph, the cyclic modules will be marked as side_effects
  /// 2. generate tree_shake_modules based on the topo sorted modules
  /// 3. traverse the tree_shake_modules
  ///   3.1 mark entry modules as side_effects
  ///   3.2 if module is commonjs, mark all imported modules as [UsedExports::All]
  ///   3.3 else if module is esm and the module has side effects, add imported identifiers to [UsedExports::Partial] of the imported modules
  ///   3.4 else if module is esm and the module has no side effects, analyze the used statement based on the statement graph
  fn optimize_module_graph(
    &self,
    module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // topo sort the module_graph, the cyclic modules will be marked as side_effects
    let (topo_sorted_modules, cyclic_modules) = module_graph.toposort();

    // mark cyclic modules as side_effects
    for chain in cyclic_modules {
      for module_id in chain {
        let mut module = module_graph.module_mut(&module_id).unwrap();
        module.side_effects = true;
      }
    }

    // mark entry modules as side_effects
    for entry_module_id in module_graph.entries.clone() {
      let mut module = module_graph.module_mut(&entry_module_id).unwrap();
      module.side_effects = true;
    }

    let mut tree_shake_modules_ids = vec![];
    let mut tree_shake_modules_map = std::collections::HashMap::new();

    for module_id in topo_sorted_modules {
      let module = module_graph.module(&module_id).unwrap();
      let tree_shake_module = module::TreeShakeModule::new(module);
      tree_shake_modules_ids.push(tree_shake_module.module_id.clone());
      tree_shake_modules_map.insert(tree_shake_module.module_id.clone(), tree_shake_module);
    }

    let mut modules_to_remove = vec![];

    // traverse the tree_shake_modules
    for tree_shake_module_id in tree_shake_modules_ids {
      let tree_shake_module = tree_shake_modules_map.get(&tree_shake_module_id).unwrap();

      // if module is not esm, mark all imported modules as [UsedExports::All]
      if !matches!(
        tree_shake_module.module_system,
        farmfe_core::module::ModuleSystem::EsModule
      ) {
        let imports = tree_shake_module.imports().clone();
        drop(tree_shake_module);

        for import_info in &imports {
          let imported_module_id =
            module_graph.get_dep_by_source(&tree_shake_module_id, &import_info.source);
          let imported_tree_shake_module =
            tree_shake_modules_map.get_mut(&imported_module_id).unwrap();
          imported_tree_shake_module.used_exports = module::UsedExports::All;
        }
      } else {
        // if module is esm and the module has side effects, add imported identifiers to [UsedExports::Partial] of the imported modules
        if tree_shake_module.side_effects {
          let imports = tree_shake_module.imports();
          drop(tree_shake_module);

          for import_info in &imports {
            add_used_exports(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              import_info,
            );
          }
        } else {
          // analyze the statement graph start from the used statements
          let used_stmts = tree_shake_module.used_statements().clone();

          // if the module's used_stmts is empty, means this module should be removed
          if used_stmts.is_empty() {
            modules_to_remove.push(tree_shake_module_id.clone());
            continue;
          }

          // remove the unused statements from the module
          let module = module_graph.module_mut(&tree_shake_module_id).unwrap();
          let swc_module = &mut module.meta.as_script_mut().ast;
          let mut stmts_to_remove = swc_module
            .body
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
              if !used_stmts.contains(&index) {
                Some(index)
              } else {
                None
              }
            })
            .collect::<Vec<_>>();
          // remove from the end to the start
          stmts_to_remove.reverse();

          for stmt in stmts_to_remove {
            swc_module.body.remove(stmt);
          }

          // get used indents from the used statements
          let used_imports = tree_shake_module
            .imports()
            .into_iter()
            .filter_map(|import_info| {
              if used_stmts.contains(&import_info.stmt_id) {
                Some(import_info)
              } else {
                None
              }
            })
            .collect::<Vec<_>>();

          // mark the imported modules' used_exports based on used_imports
          for used_import_info in used_imports {
            add_used_exports(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              &used_import_info,
            );
          }
        }
      }
    }

    // remove the unused modules
    for module_id in modules_to_remove {
      module_graph.remove_module(&module_id);
    }

    Ok(None)
  }
}

fn add_used_exports(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  import_info: &ImportInfo,
) {
  let imported_module_id =
    module_graph.get_dep_by_source(tree_shake_module_id, &import_info.source);
  let imported_tree_shake_module = tree_shake_modules_map.get_mut(&imported_module_id).unwrap();

  for sp in &import_info.specifiers {
    match sp {
      statement_graph::ImportSpecifierInfo::Namespace(_) => {
        imported_tree_shake_module.used_exports = module::UsedExports::All;
      }
      statement_graph::ImportSpecifierInfo::Named { local, imported } => {
        if let Some(ident) = imported {
          imported_tree_shake_module
            .used_exports
            .add_used_export(module::UsedIdent::SwcIdent(ident.clone()));
        } else {
          imported_tree_shake_module
            .used_exports
            .add_used_export(module::UsedIdent::SwcIdent(local.clone()));
        }
      }
      statement_graph::ImportSpecifierInfo::Default(_) => {
        imported_tree_shake_module
          .used_exports
          .add_used_export(module::UsedIdent::Default);
      }
    }
  }
}
