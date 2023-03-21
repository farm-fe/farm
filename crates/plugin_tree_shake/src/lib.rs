use farmfe_core::plugin::Plugin;

mod module;
mod statement_graph;

pub struct FarmPluginTreeShake;

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
    let mut tree_shale_modules_map = std::collections::HashMap::new();

    for module_id in topo_sorted_modules {
      let module = module_graph.module(&module_id).unwrap();
      let tree_shake_module = module::TreeShakeModule::new(module);
      tree_shake_modules_ids.push(tree_shake_module.module_id.clone());
      tree_shale_modules_map.insert(tree_shake_module.module_id.clone(), tree_shake_module);
    }

    // traverse the tree_shake_modules
    for tree_shake_module_id in tree_shake_modules_ids {
      let tree_shake_module = tree_shale_modules_map.get(&tree_shake_module_id).unwrap();

      // if module is not esm, mark all imported modules as [UsedExports::All]
      if !matches!(
        tree_shake_module.module_system,
        farmfe_core::module::ModuleSystem::EsModule
      ) {
        let imports = tree_shake_module.imports.clone();
        drop(tree_shake_module);

        for (_, imported_module_id) in &imports {
          let imported_tree_shake_module =
            tree_shale_modules_map.get_mut(&imported_module_id).unwrap();
          imported_tree_shake_module.used_exports = module::UsedExports::All;
        }
      } else {
        // if module is esm and the module has side effects, add imported identifiers to [UsedExports::Partial] of the imported modules
        if tree_shake_module.side_effects {
          let imports = tree_shake_module.imports.clone();
          drop(tree_shake_module);

          for (imported_ident, imported_module_id) in &imports {
            let imported_tree_shake_module =
              tree_shale_modules_map.get_mut(imported_module_id).unwrap();
            imported_tree_shake_module
              .used_exports
              .add_used_exports(vec![imported_ident.sym.to_string()]);
          }
        } else {
          // if module is esm and the module has no side effects, analyze the used statement based on the statement graph
          // let mut used_statements = vec![];
          let used_export = tree_shake_module.used_exports.clone();

          let exported_identifiers = match used_export {
            module::UsedExports::All => {
              // all exported identifiers are used
              tree_shake_module
                .exports
                .iter()
                .map(|(export, _)| export.sym.to_string())
                .collect()
            }
            module::UsedExports::Partial(identifiers) => {
              // some exported identifiers are used, check if the used identifiers are exported, otherwise log a warning
              for ident in &identifiers {
                if !tree_shake_module
                  .exports
                  .iter()
                  .any(|(export, _)| &export.sym == ident)
                {
                  println!(
                    "[warning] module `{}` does not export identifier `{:?}`",
                    ident, tree_shake_module.module_id
                  );
                }
              }
              identifiers
            }
          };

          // analyze the statement graph start from the used statements
        }
      }
    }

    Ok(None)
  }
}
