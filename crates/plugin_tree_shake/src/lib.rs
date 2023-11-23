use farmfe_core::{
  config::Config,
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::Plugin,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
};
use farmfe_toolkit::script::swc_try_with::resolve_module_mark;
use module::{TreeShakeModule, UsedExports};
use statement_graph::{ExportInfo, ImportInfo};

pub mod module;
pub mod remove_useless_stmts;
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
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // topo sort the module_graph, the cyclic modules will be marked as side_effects
    let (topo_sorted_modules, cyclic_modules) = {
      farmfe_core::farm_profile_scope!("tree shake toposort".to_string());
      module_graph.toposort()
    };

    // mark cyclic modules as side_effects
    for chain in cyclic_modules {
      for module_id in chain {
        let module = module_graph.module_mut(&module_id).unwrap();
        module.side_effects = true;
      }
    }

    // mark entry modules as side_effects
    for (entry_module_id, _) in module_graph.entries.clone() {
      let module = module_graph.module_mut(&entry_module_id).unwrap();
      module.side_effects = true;
    }

    let mut tree_shake_modules_ids = vec![];
    let mut tree_shake_modules_map = std::collections::HashMap::new();

    module_graph
      .modules_mut()
      .into_par_iter()
      .filter(|m| m.module_type.is_script() && !m.external)
      .for_each(|module| {
        let meta = module.meta.as_script_mut();

        if meta.top_level_mark != 0 || meta.unresolved_mark != 0 {
          return;
        }

        let ast = &mut meta.ast;

        let (unresolved_mark, top_level_mark) =
          resolve_module_mark(ast, module.module_type.is_typescript(), context);
        module.meta.as_script_mut().unresolved_mark = unresolved_mark.as_u32();
        module.meta.as_script_mut().top_level_mark = top_level_mark.as_u32();
      });

    for module_id in topo_sorted_modules {
      let module = module_graph.module(&module_id).unwrap();

      // skip non script modules and external modules
      if !module.module_type.is_script() || module.external {
        if !module.module_type.is_script() && !module.external {
          // mark all non script modules' script dependencies as side_effects
          for dep_id in module_graph.dependencies_ids(&module_id) {
            let dep_module = module_graph.module_mut(&dep_id).unwrap();

            if !dep_module.module_type.is_script() {
              continue;
            }

            dep_module.side_effects = true;
          }
        }

        continue;
      }

      let tree_shake_module = module::TreeShakeModule::new(module);
      tree_shake_modules_ids.push(tree_shake_module.module_id.clone());
      tree_shake_modules_map.insert(tree_shake_module.module_id.clone(), tree_shake_module);
    }

    let mut modules_to_remove = vec![];

    // traverse the tree_shake_modules
    for tree_shake_module_id in tree_shake_modules_ids {
      let tree_shake_module = tree_shake_modules_map
        .get_mut(&tree_shake_module_id)
        .unwrap();

      // if module is not esm, mark all imported modules as [UsedExports::All]
      if !matches!(
        tree_shake_module.module_system,
        farmfe_core::module::ModuleSystem::EsModule
      ) {
        for (dep_id, _) in module_graph.dependencies(&tree_shake_module_id) {
          let dep_tree_shake_module = tree_shake_modules_map.get_mut(&dep_id);

          if let Some(dep_tree_shake_module) = dep_tree_shake_module {
            dep_tree_shake_module.used_exports = UsedExports::All;
          }
        }
      } else {
        // if module is esm and the module has side effects, add imported identifiers to [UsedExports::Partial] of the imported modules
        if tree_shake_module.side_effects {
          let imports = tree_shake_module.imports();
          let exports = tree_shake_module.exports();

          for import_info in &imports {
            add_used_exports_by_import_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              import_info,
            );
          }

          for export_info in &exports {
            add_used_exports_by_export_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              export_info,
            );
          }
        } else {
          let tree_shake_module = tree_shake_modules_map
            .get_mut(&tree_shake_module_id)
            .unwrap();

          if tree_shake_module.used_exports.is_empty()
            && !tree_shake_module.contains_self_executed_stmt
          {
            // if the module's used_exports is empty, and this module does not have self-executed statements, then this module is useless
            // which means this module is not used and should be removed
            modules_to_remove.push(tree_shake_module_id.clone());
            continue;
          }

          // remove useless statements and useless imports/exports identifiers, then all preserved import info and export info will be added to the used_exports.
          let (used_imports, used_exports_from) = remove_useless_stmts::remove_useless_stmts(
            &tree_shake_module_id,
            module_graph,
            &tree_shake_modules_map,
          );

          for import_info in used_imports {
            add_used_exports_by_import_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              &import_info,
            );
          }

          for export_info in used_exports_from {
            add_used_exports_by_export_info(
              &mut tree_shake_modules_map,
              &*module_graph,
              &tree_shake_module_id,
              &export_info,
            );
          }
        }
      }

      // add all dynamic imported dependencies as [UsedExports::All]
      for (dep, edge) in module_graph.dependencies(&tree_shake_module_id) {
        if edge.is_dynamic() {
          let tree_shake_module = tree_shake_modules_map.get_mut(&dep).unwrap();
          tree_shake_module.side_effects = true;
          tree_shake_module.used_exports = UsedExports::All;
        }
      }
    }

    // remove the unused modules
    for module_id in modules_to_remove {
      module_graph.remove_module(&module_id);
    }

    // update used_exports in module_graph
    for module in module_graph.modules_mut() {
      if let Some(tree_shake_module) = tree_shake_modules_map.get(&module.id) {
        let mut used_exports = tree_shake_module.used_exports.to_string_vec();
        used_exports.sort();
        module.used_exports = used_exports;
      }
    }

    Ok(Some(()))
  }
}

// Add all imported to used_exports
fn add_used_exports_by_import_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  import_info: &ImportInfo,
) {
  let imported_module_id =
    module_graph.get_dep_by_source(tree_shake_module_id, &import_info.source);
  let imported_module = module_graph.module(&imported_module_id).unwrap();

  if imported_module.external || !imported_module.module_type.is_script() {
    return;
  }

  let imported_tree_shake_module = tree_shake_modules_map
    .get_mut(&imported_module_id)
    .unwrap_or_else(|| {
      panic!("imported module not found: {:?}", imported_module_id);
    });

  if import_info.specifiers.is_empty() {
    imported_tree_shake_module.used_exports = module::UsedExports::All;
    imported_tree_shake_module.side_effects = true;
    return;
  }

  for sp in &import_info.specifiers {
    match sp {
      statement_graph::ImportSpecifierInfo::Namespace(_) => {
        imported_tree_shake_module.used_exports = module::UsedExports::All;
      }
      statement_graph::ImportSpecifierInfo::Named { local, imported } => {
        if let Some(ident) = imported {
          if *ident == "default" {
            imported_tree_shake_module
              .used_exports
              .add_used_export(&module::UsedIdent::Default);
          } else {
            imported_tree_shake_module
              .used_exports
              .add_used_export(&module::UsedIdent::SwcIdent(strip_context(ident)));
          }
        } else {
          imported_tree_shake_module
            .used_exports
            .add_used_export(&module::UsedIdent::SwcIdent(strip_context(local)));
        }
      }
      statement_graph::ImportSpecifierInfo::Default(_) => {
        imported_tree_shake_module
          .used_exports
          .add_used_export(&module::UsedIdent::Default);
      }
    }
  }
}

/// All all exported to used_exports
fn add_used_exports_by_export_info(
  tree_shake_modules_map: &mut std::collections::HashMap<ModuleId, TreeShakeModule>,
  module_graph: &ModuleGraph,
  tree_shake_module_id: &ModuleId,
  export_info: &ExportInfo,
) {
  if let Some(source) = &export_info.source {
    let exported_module_id = module_graph.get_dep_by_source(tree_shake_module_id, source);
    let exported_module = module_graph.module(&exported_module_id).unwrap();

    if exported_module.external {
      return;
    }

    let exported_tree_shake_module = tree_shake_modules_map.get_mut(&exported_module_id).unwrap();

    for sp in &export_info.specifiers {
      match sp {
        statement_graph::ExportSpecifierInfo::Namespace(_) => {
          exported_tree_shake_module.used_exports = module::UsedExports::All;
        }
        statement_graph::ExportSpecifierInfo::Named { local, .. } => {
          if local == &"default".to_string() {
            exported_tree_shake_module
              .used_exports
              .add_used_export(&module::UsedIdent::Default);
          } else {
            exported_tree_shake_module
              .used_exports
              .add_used_export(&module::UsedIdent::SwcIdent(strip_context(local)));
          }
        }
        statement_graph::ExportSpecifierInfo::Default => {
          exported_tree_shake_module
            .used_exports
            .add_used_export(&module::UsedIdent::Default);
        }
        statement_graph::ExportSpecifierInfo::All(used_idents) => {
          if let Some(used_idents) = used_idents {
            for ident in used_idents {
              if ident == "*" {
                exported_tree_shake_module.used_exports = module::UsedExports::All;
              } else {
                exported_tree_shake_module
                  .used_exports
                  .add_used_export(&strip_context(ident));
              }
            }
          } else {
            exported_tree_shake_module.used_exports = module::UsedExports::All;
          }
        }
      }
    }
  }
}

fn strip_context(ident: &String) -> String {
  let ident_split = ident.split('#').collect::<Vec<_>>();
  ident_split[0].to_string()
}
