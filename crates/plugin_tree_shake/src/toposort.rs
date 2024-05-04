use std::collections::{HashMap, HashSet};

use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
};

use crate::{
  module::{self, TreeShakeModule},
  statement_graph,
  tree_shake_modules::utils::strip_context,
};

pub fn toposort(
  module_graph: &ModuleGraph,
  pre_shaking_module_map: &mut HashMap<ModuleId, TreeShakeModule>,
) -> (Vec<ModuleId>, Vec<ModuleId>) {
  fn dfs(
    entry: &ModuleId,
    graph: &ModuleGraph,
    stack: &mut Vec<ModuleId>,
    visited: &mut HashSet<ModuleId>,
    result: &mut Vec<ModuleId>,
    pre_shaking_module_map: &mut HashMap<ModuleId, TreeShakeModule>,
    cyclic_node: &mut Vec<ModuleId>,
  ) {
    // collect cycle
    if stack.iter().any(|m| m == entry) {
      // while see ahead of the stack, mark parse import specify
      let import_circle_module_id = stack.iter().last().unwrap();
      if let Some(import_circle_module) = pre_shaking_module_map.get_mut(&import_circle_module_id) {
        let imports = import_circle_module
          .imports()
          .into_iter()
          .filter(|import_info| {
            graph
              .get_dep_by_source_optional(
                import_circle_module_id,
                &import_info.source,
                Some(ResolveKind::Import),
              )
              .is_some()
          })
          .collect::<Vec<_>>();

        let shake_module = pre_shaking_module_map.get_mut(&entry).unwrap();

        for shake_module_import_info in imports {
          for specify in shake_module_import_info.specifiers {
            match specify {
              statement_graph::ImportSpecifierInfo::Namespace(_) => {
                shake_module
                  .used_exports
                  .add_used_export(import_circle_module_id, &module::UsedIdent::ExportAll);
              }

              statement_graph::ImportSpecifierInfo::Named { local, imported } => {
                if let Some(ident) = imported {
                  if ident.as_str() == "default" {
                    shake_module
                      .used_exports
                      .add_used_export(import_circle_module_id, &module::UsedIdent::Default);
                  } else {
                    shake_module.used_exports.add_used_export(
                      import_circle_module_id,
                      &module::UsedIdent::SwcIdent(strip_context(&ident)),
                    );
                  }
                } else {
                  shake_module.used_exports.add_used_export(
                    import_circle_module_id,
                    &module::UsedIdent::SwcIdent(strip_context(&local)),
                  );
                }
              }
              statement_graph::ImportSpecifierInfo::Default(_) => {
                shake_module
                  .used_exports
                  .add_used_export(import_circle_module_id, &module::UsedIdent::Default);
              }
            }
          }
        }
      };

      cyclic_node.push(entry.clone());
      return;
    } else if visited.contains(entry) {
      // skip visited module
      return;
    }

    visited.insert(entry.clone());
    stack.push(entry.clone());

    let deps = graph.dependencies(entry);

    for (dep, _) in &deps {
      dfs(
        dep,
        graph,
        stack,
        visited,
        result,
        pre_shaking_module_map,
        cyclic_node,
      )
    }

    // visit current entry
    result.push(stack.pop().unwrap());
  }

  let mut result = vec![];
  let mut stack = vec![];
  let mut cyclic_node = vec![];

  // sort entries to make sure it is stable
  let mut entries = module_graph.entries.iter().collect::<Vec<_>>();
  entries.sort();

  let mut visited = HashSet::new();

  for (entry, _) in entries {
    let mut res = vec![];
    dfs(
      entry,
      module_graph,
      &mut stack,
      &mut visited,
      &mut res,
      pre_shaking_module_map,
      &mut cyclic_node,
    );

    result.extend(res);
  }

  result.reverse();

  (result, cyclic_node)
}
