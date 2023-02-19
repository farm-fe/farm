use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  hashbrown::HashSet,
  module::{module_graph::ModuleGraph, ModuleId},
};

pub fn find_hmr_boundaries(
  update_module_ids: &Vec<ModuleId>,
  context: &Arc<CompilationContext>,
) -> HashMap<String, Vec<Vec<String>>> {
  let mut boundaries = HashMap::new();
  let module_graph = context.module_graph.read();

  for id in update_module_ids {
    let mut stack = vec![id.clone()];
    let mut visited = HashSet::new();
    let mut res = vec![];
    let all_path_accepted =
      find_hmr_accepted_recursively(id, &module_graph, &mut stack, &mut visited, &mut res);

    if !all_path_accepted {
      continue;
    }

    boundaries.insert(
      id.id(context.config.mode.clone()),
      res
        .into_iter()
        .map(|v| {
          v.into_iter()
            .map(|id| id.id(context.config.mode.clone()))
            .collect()
        })
        .collect(),
    );
  }

  boundaries
}

fn find_hmr_accepted_recursively(
  id: &ModuleId,
  module_graph: &ModuleGraph,
  stack: &mut Vec<ModuleId>,
  visited: &mut HashSet<ModuleId>,
  res: &mut Vec<Vec<ModuleId>>,
) -> bool {
  let module = module_graph.module(id).unwrap();
  // There is a path from the module to the root that does not have HMR accepted
  if module_graph.entries.contains(id) {
    return false;
  } else if module.module_type.is_script() {
    let hmr_accepted = module.meta.as_script().hmr_accepted;

    if hmr_accepted {
      res.push(stack.clone());

      return true;
    }
  }

  if !visited.contains(id) {
    visited.insert(id.clone());

    let parents = module_graph.dependents_ids(id);

    for parent in parents {
      stack.push(parent.clone());
      let all_path_accepted =
        find_hmr_accepted_recursively(&parent, module_graph, stack, visited, res);
      stack.pop();

      if !all_path_accepted {
        return false;
      }
    }
  }

  true
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, sync::Arc};

  use farmfe_core::{
    config::{Config, Mode},
    context::CompilationContext,
    module::{module_graph::ModuleGraph, ModuleMetaData, ModuleType, ScriptModuleMetaData},
    parking_lot::RwLock,
  };
  use farmfe_testing_helpers::construct_test_module_graph;

  use super::find_hmr_boundaries;

  fn create_context(module_graph: ModuleGraph) -> Arc<CompilationContext> {
    let mut context = CompilationContext::new(
      Config {
        mode: Mode::Development,
        ..Default::default()
      },
      vec![],
    )
    .unwrap();

    context.module_graph = RwLock::new(module_graph);
    Arc::new(context)
  }
  #[test]
  fn find_hmr_boundaries_1() {
    let mut module_graph = construct_test_module_graph();

    let module_a = module_graph.module_mut(&"A".into()).unwrap();
    module_a.module_type = ModuleType::Js;
    module_a.meta = ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_accepted: true,
      ..Default::default()
    });

    let context = create_context(module_graph);
    let boundaries = find_hmr_boundaries(&vec!["F".into()], &context);

    assert_eq!(boundaries, HashMap::new());
  }

  #[test]
  fn find_hmr_boundaries_2() {
    let mut module_graph = construct_test_module_graph();

    let module_d = module_graph.module_mut(&"D".into()).unwrap();
    module_d.module_type = ModuleType::Js;
    module_d.meta = ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_accepted: true,
      ..Default::default()
    });
    let module_c = module_graph.module_mut(&"C".into()).unwrap();
    module_c.module_type = ModuleType::Js;
    module_c.meta = ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_accepted: true,
      ..Default::default()
    });

    let context = create_context(module_graph);
    let boundaries = find_hmr_boundaries(&vec!["F".into()], &context);
    // Be careful, the order of the paths may not be guaranteed. check the order if the test fails.
    assert_eq!(
      boundaries,
      vec![(
        "F".into(),
        vec![vec!["F".into(), "D".into()], vec!["F".into(), "C".into()]]
      )]
      .into_iter()
      .collect::<HashMap<_, _>>()
    );
  }

  #[test]
  fn find_hmr_boundaries_3() {
    let mut module_graph = construct_test_module_graph();

    let module_d = module_graph.module_mut(&"D".into()).unwrap();
    module_d.module_type = ModuleType::Js;
    module_d.meta = ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_accepted: true,
      ..Default::default()
    });

    let context = create_context(module_graph);
    let boundaries = find_hmr_boundaries(&vec!["F".into()], &context);
    // Be careful, the order of the paths may not be guaranteed. check the order if the test fails.
    assert_eq!(boundaries, HashMap::new());
  }
}
