#![feature(box_patterns)]
#![feature(exact_size_is_empty)]
#![feature(let_chains)]

use farmfe_core::{
  config::{Config, Mode},
  plugin::Plugin,
};

use crate::remove_hot_update::remove_useless_hot_update_stmts;

pub mod fill_module_mark;
pub mod init_tree_shake_module_map;
pub mod mark_initial_side_effects;
pub mod module;
pub mod remove_hot_update;
pub mod statement_graph;
pub mod tree_shake_modules;

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
  /// 1. topo sort the module_graph
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
    // 1. fill module mark
    fill_module_mark::fill_module_mark(module_graph, context);

    // 2. init tree_shake_modules_map
    let mut tree_shake_modules_map =
      init_tree_shake_module_map::init_tree_shake_module_map(module_graph, context);

    // 3. handle default side effects
    let entry_module_ids = mark_initial_side_effects::mark_initial_side_effects(
      module_graph,
      &mut tree_shake_modules_map,
    );

    // 4. traverse the tree_shake_modules, and remove the unused statements
    let modules_to_remove = tree_shake_modules::tree_shake_modules(
      entry_module_ids,
      module_graph,
      &mut tree_shake_modules_map,
    );

    // 5. update used_exports in module_graph
    for module in module_graph.modules_mut() {
      if let Some(tree_shake_module) = tree_shake_modules_map.get(&module.id) {
        let mut used_exports = tree_shake_module.handled_used_exports.to_string_vec();
        used_exports.sort();

        module.used_exports = used_exports;
      }
    }

    // 6. remove the unused modules
    for module_id in modules_to_remove {
      module_graph.remove_module(&module_id);
    }

    // 7. remove useless hot update statements if production
    if matches!(context.config.mode, Mode::Production) {
      remove_useless_hot_update_stmts(module_graph);
    }

    Ok(Some(()))
  }
}
