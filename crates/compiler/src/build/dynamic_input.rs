use std::{collections::VecDeque, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::{module_graph::ModuleGraph, ModuleId},
  HashSet,
};

/// Copy module deeply starting from module_id, suffixed the copied module with scope
fn copy_module_deeply(module_id: ModuleId, scope: &str, module_graph: &mut ModuleGraph) -> bool {
  let mut queue = VecDeque::from(vec![module_id]);
  let mut visited = HashSet::default();
  let mut copied = false;

  while let Some(module_id) = queue.pop_front() {
    if visited.contains(&module_id) {
      continue;
    }

    // remove the edge from the module_id to its dependencies, and
    // if the dep module does not have any other parent, just remove and rename the module suffixed with scope and create a new edge
    // if the dep module has other parent, remove the edge, clone the module, rename the module suffixed with scope
    for dep in module_graph.dependencies_ids(&module_id) {
      let scoped_id: ModuleId =
        format!("{}.{}{}", dep.relative_path(), scope, dep.query_string()).into();
      // if the module is already renamed, then skip
      if module_graph.has_edge(&module_id, &scoped_id) {
        continue;
      }

      copied = true;

      let (mut module, edge) = if module_graph.dependents_ids(&dep).len() > 1 {
        let mut module = module_graph.module(&dep).unwrap();
        if !module.module_type.is_script() || !module.module_type.is_css() {
          continue;
        }
        let edge = module_graph.remove_edge(&module_id, &dep).unwrap();

        (None, edge)
      } else {
        let edge = module_graph.remove_edge(&module_id, &dep).unwrap();
        let module = module_graph.remove_module(&module_id);
        if !module.module_type.is_script() || !module.module_type.is_css() {
          continue;
        }

        (Some(module), edge)
      };

      // the scoped module is already existed, clone the edge
      if module_graph.has_module(&scoped_id) {
        module_graph
          .add_edge(&module_id, &scoped_id, edge.unwrap())
          .unwrap();
      } else {
        // clone the module if needed and rename it
        let mut cloned_module = module.unwrap_or(module_graph.module(&dep).unwrap().clone());
        cloned_module.id = scoped_id.clone();
        cloned_module.module_type = cloned_module.module_type.to_custom(scope);
        module_graph.add_module(cloned_module);
        module_graph
          .add_edge(&module_id, &scoped_id, edge.unwrap())
          .unwrap();
      }

      queue.push_back(dep);
    }

    visited.insert(module_id);
  }

  copied
}

/// If scope of dynamic input is set, then we will make a copy of all the modules starting from the dynamic input module in the module graph.
pub fn handle_dynamic_input(
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) -> bool {
  // if there is new dynamic input handled, the generate stage of hmr should execute synchronously
  let mut handled = false;

  for item in &*context.dynamic_input {
    let input_name = item.key();
    let dynamic_input = item.value();

    if let Some(scope) = &dynamic_input.scope {
      if let Some((module_id, _)) = module_graph
        .entries
        .iter()
        .find(|(_, entry)| entry.as_str() == input_name.as_str())
      {
        let res = copy_module_deeply(module_id.clone(), scope, module_graph);
        handled = handled || res;
      }
    }
  }

  handled
}
