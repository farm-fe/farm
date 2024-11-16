use std::{collections::HashSet, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::{module_graph::ModuleGraph, ModuleId},
};
use rustc_hash::FxHashMap;

pub fn find_hmr_boundaries(
  update_module_ids: &Vec<ModuleId>,
  context: &Arc<CompilationContext>,
) -> FxHashMap<String, Vec<Vec<String>>> {
  let mut boundaries = FxHashMap::default();
  let module_graph = context.module_graph.read();

  for id in update_module_ids {
    let mut stack = vec![id.clone()];
    let mut visited = HashSet::new();
    let mut res = vec![];
    let all_path_accepted =
      find_hmr_accepted_recursively(id, &module_graph, &mut stack, &mut visited, &mut res);
    // if any of the path is not accepted, reload the whole page
    if !all_path_accepted {
      return FxHashMap::default();
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
  if module_graph.entries.contains_key(id) {
    return false;
  }

  // self accepted, non script modules are not self-acceptable for now
  if module.module_type.is_script() && module.meta.as_script().hmr_self_accepted {
    res.push(stack.clone());

    return true;
  }

  // check if any of the importers accepts the module
  if !visited.contains(id) {
    visited.insert(id.clone());

    let parents = module_graph.dependents_ids(id);

    for parent in parents {
      // check if the parent accepts the module
      let parent_module = module_graph.module(&parent).unwrap();

      if !parent_module.module_type.is_script() {
        return false;
      }
      // if the importer accepts the module, push
      if parent_module
        .meta
        .as_script()
        .hmr_accepted_deps
        .contains(id)
      {
        let mut cloned_stack = stack.clone();
        cloned_stack.push(parent.clone());
        res.push(cloned_stack);
        // skip self recursive check if accepts the module
        continue;
      }

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
  use std::{collections::HashSet, sync::Arc};

  use farmfe_core::{
    config::{Config, Mode},
    context::CompilationContext,
    module::{module_graph::ModuleGraph, ModuleMetaData, ModuleType, ScriptModuleMetaData},
    parking_lot::RwLock,
  };
  use farmfe_testing_helpers::construct_test_module_graph;
  use rustc_hash::FxHashMap;

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

    context.module_graph = Box::new(RwLock::new(module_graph));
    Arc::new(context)
  }
  #[test]
  fn find_hmr_boundaries_1() {
    let mut module_graph = construct_test_module_graph();

    let module_a = module_graph.module_mut(&"A".into()).unwrap();
    module_a.module_type = ModuleType::Js;
    module_a.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: true,
      ..Default::default()
    }));

    let context = create_context(module_graph);
    let boundaries = find_hmr_boundaries(&vec!["F".into()], &context);

    assert_eq!(boundaries, FxHashMap::default());
  }

  #[test]
  fn find_hmr_boundaries_2() {
    let mut module_graph = construct_test_module_graph();

    let module_d = module_graph.module_mut(&"D".into()).unwrap();
    module_d.module_type = ModuleType::Js;
    module_d.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: true,
      ..Default::default()
    }));
    let module_c = module_graph.module_mut(&"C".into()).unwrap();
    module_c.module_type = ModuleType::Js;
    module_c.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: true,
      ..Default::default()
    }));

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
      .collect::<FxHashMap<_, _>>()
    );
  }

  #[test]
  fn find_hmr_boundaries_3() {
    let mut module_graph = construct_test_module_graph();

    let module_d = module_graph.module_mut(&"D".into()).unwrap();
    module_d.module_type = ModuleType::Js;
    module_d.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: true,
      ..Default::default()
    }));

    let context = create_context(module_graph);
    let boundaries = find_hmr_boundaries(&vec!["F".into()], &context);
    // Be careful, the order of the paths may not be guaranteed. check the order if the test fails.
    assert_eq!(boundaries, FxHashMap::default());
  }

  #[test]
  fn find_hmr_boundaries_deps_1() {
    let mut module_graph = construct_test_module_graph();

    let module_d = module_graph.module_mut(&"D".into()).unwrap();
    module_d.module_type = ModuleType::Js;
    module_d.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: false,
      hmr_accepted_deps: HashSet::from(["F".into()]),
      ..Default::default()
    }));
    let module_c = module_graph.module_mut(&"C".into()).unwrap();
    module_c.module_type = ModuleType::Js;
    module_c.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: false,
      hmr_accepted_deps: HashSet::from(["F".into()]),
      ..Default::default()
    }));

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
      .collect::<FxHashMap<_, _>>()
    );
  }

  #[test]
  fn find_hmr_boundaries_deps_2() {
    let mut module_graph = construct_test_module_graph();

    let module_d = module_graph.module_mut(&"D".into()).unwrap();
    module_d.module_type = ModuleType::Js;
    module_d.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: false,
      hmr_accepted_deps: HashSet::from(["F".into()]),
      ..Default::default()
    }));

    let module_f = module_graph.module_mut(&"F".into()).unwrap();
    module_f.module_type = ModuleType::Js;
    module_f.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: false,
      ..Default::default()
    }));

    let context = create_context(module_graph);
    let boundaries = find_hmr_boundaries(&vec!["F".into()], &context);
    // Be careful, the order of the paths may not be guaranteed. check the order if the test fails.
    assert_eq!(boundaries, FxHashMap::default());
  }

  #[test]
  fn find_hmr_boundaries_deps_3() {
    let mut module_graph = construct_test_module_graph();

    let module_b = module_graph.module_mut(&"B".into()).unwrap();
    module_b.module_type = ModuleType::Js;
    module_b.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: false,
      hmr_accepted_deps: HashSet::from(["E".into()]),
      ..Default::default()
    }));

    let module_e = module_graph.module_mut(&"E".into()).unwrap();
    module_e.module_type = ModuleType::Js;
    module_e.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: false,
      ..Default::default()
    }));

    let module_g = module_graph.module_mut(&"G".into()).unwrap();
    module_g.module_type = ModuleType::Js;
    module_g.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
      hmr_self_accepted: false,
      ..Default::default()
    }));

    let context = create_context(module_graph);
    let boundaries = find_hmr_boundaries(&vec!["G".into()], &context);
    // Be careful, the order of the paths may not be guaranteed. check the order if the test fails.
    assert_eq!(
      boundaries,
      vec![("G".into(), vec![vec!["G".into(), "E".into(), "B".into()]])]
        .into_iter()
        .collect::<FxHashMap<_, _>>()
    );
  }
}
