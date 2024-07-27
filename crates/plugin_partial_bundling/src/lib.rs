use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::{collections::VecDeque, sync::Arc};

use farmfe_core::config::custom::CUSTOM_CONFIG_PARTIAL_BUNDLING_GROUPS_ENFORCE_MAP;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupGraph},
    ModuleId,
  },
  plugin::{Plugin, PluginHookContext},
  resource::resource_pot::ResourcePot,
};
use generate_module_buckets::{generate_module_buckets_map, group_module_buckets_by_module_group};
use generate_resource_pots::generate_resource_pots;

// mod module_bucket;
mod generate_module_buckets;
mod generate_module_pots;
mod generate_resource_pots;
mod merge_module_pots;
mod module_bucket;
mod module_pot;
mod utils;
/// Partial Bundling implementation for Farm.
/// See https://github.com/farm-fe/rfcs/pull/9
pub struct FarmPluginPartialBundling {
  partial_bundling_groups_enforce_map: RwLock<HashMap<String, bool>>,
}

impl Plugin for FarmPluginPartialBundling {
  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    *self.partial_bundling_groups_enforce_map.write().unwrap() = config
      .custom
      .get(CUSTOM_CONFIG_PARTIAL_BUNDLING_GROUPS_ENFORCE_MAP)
      .map(|s| {
        farmfe_core::serde_json::from_str(s)
          .expect("failed to parse partial bundling group enforce map")
      })
      .unwrap_or_default();

    Ok(Some(()))
  }

  fn name(&self) -> &str {
    "FarmPluginPartialBundling"
  }

  fn priority(&self) -> i32 {
    99
  }

  fn analyze_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ModuleGroupGraph>> {
    let module_group_graph = module_group_graph_from_entries(
      &module_graph
        .entries
        .clone()
        .into_iter()
        .map(|(entry, _)| entry)
        .collect(),
      module_graph,
    );

    Ok(Some(module_group_graph))
  }

  /// The partial bundling algorithm's result should not be related to the order of the module group.
  /// Whatever the order of the module group is, the result should be the same.
  /// See https://github.com/farm-fe/rfcs/blob/main/rfcs/003-partial-bundling/rfc.md for more design details.
  fn partial_bundling(
    &self,
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<ResourcePot>>> {
    // 0. handle enforceResources in separate hook

    // 1. get module group graph and module graph
    let module_graph = context.module_graph.read();
    let module_group_graph = context.module_group_graph.read();
    // 2. generate module buckets and group by module group
    let module_buckets_map = generate_module_buckets_map(modules, &module_graph);
    let module_group_buckets =
      group_module_buckets_by_module_group(&module_buckets_map, &module_group_graph, &module_graph);

    let enforce_map = self.partial_bundling_groups_enforce_map.read().unwrap();

    // 3. generate resource pots
    let resource_pots = generate_resource_pots(
      module_group_buckets,
      module_buckets_map,
      &module_graph,
      &context.config,
      &enforce_map,
    );

    Ok(Some(resource_pots))
  }
}

impl FarmPluginPartialBundling {
  pub fn new(_: &Config) -> Self {
    Self {
      partial_bundling_groups_enforce_map: RwLock::new(HashMap::new()),
    }
  }
}

pub fn module_group_graph_from_entries(
  entries: &Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
) -> ModuleGroupGraph {
  let mut module_group_graph = ModuleGroupGraph::new();
  let mut edges = vec![];
  let mut visited = HashSet::new();

  for entry in entries.clone() {
    if visited.contains(&entry) {
      continue;
    }

    let (group, dynamic_dependencies) = module_group_from_entry(&entry, module_graph);
    edges.extend(
      dynamic_dependencies
        .clone()
        .into_iter()
        .map(|dep| (group.id.clone(), dep)),
    );

    module_group_graph.add_module_group(group);

    visited.insert(entry);
    let mut queue = VecDeque::from(dynamic_dependencies);

    while !queue.is_empty() {
      let head = queue.pop_front().unwrap();

      if visited.contains(&head) {
        continue;
      }

      visited.insert(head.clone());

      let (group, dynamic_dependencies) = module_group_from_entry(&head, module_graph);
      edges.extend(
        dynamic_dependencies
          .clone()
          .into_iter()
          .map(|dep| (group.id.clone(), dep)),
      );

      module_group_graph.add_module_group(group);
      queue.extend(dynamic_dependencies);
    }
  }

  for (from, to) in &edges {
    module_group_graph.add_edge(from, to);
  }

  module_group_graph
}

/// get module group start from a entry. return (module group, dynamic dependencies)
/// traverse the module graph using bfs, stop when reach a dynamic dependency
fn module_group_from_entry(
  entry: &ModuleId,
  graph: &mut ModuleGraph,
) -> (ModuleGroup, Vec<ModuleId>) {
  let mut visited = HashSet::new();
  let mut module_group = ModuleGroup::new(entry.clone());
  let mut dynamic_entries = vec![];

  graph
    .module_mut(entry)
    .unwrap()
    .module_groups
    .insert(entry.clone());

  visited.insert(entry.clone());

  let deps = graph
    .dependencies(entry)
    .into_iter()
    .map(|(k, v)| (k, v.is_dynamic()))
    .collect::<Vec<_>>();

  for (dep, is_dynamic) in deps {
    if is_dynamic {
      dynamic_entries.push(dep);
    } else {
      // visited all dep and its dependencies using BFS
      let mut queue = VecDeque::new();
      queue.push_back(dep.clone());

      while !queue.is_empty() {
        let head = queue.pop_front().unwrap();

        if visited.contains(&head) {
          continue;
        }

        visited.insert(head.clone());
        module_group.add_module(head.clone());
        graph
          .module_mut(&head)
          .unwrap()
          .module_groups
          .insert(entry.clone());

        for (dep, edge) in graph.dependencies(&head) {
          if edge.is_dynamic() {
            dynamic_entries.push(dep);
          } else {
            queue.push_back(dep);
          }
        }
      }
    }
  }

  (module_group, dynamic_entries)
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;
  use std::{collections::HashMap, sync::Arc};

  use farmfe_core::{
    context::CompilationContext,
    parking_lot::RwLock,
    plugin::{Plugin, PluginHookContext},
  };
  #[cfg(test)]
  use farmfe_testing_helpers::construct_test_module_graph;
  use farmfe_testing_helpers::construct_test_module_group_graph;

  use crate::{module_group_from_entry as mgfe, FarmPluginPartialBundling};

  #[test]
  fn analyze_module_graph() {
    let plugin = FarmPluginPartialBundling {
      partial_bundling_groups_enforce_map: Default::default(),
    };
    let mut context = CompilationContext::new(Default::default(), vec![]).unwrap();
    let graph = construct_test_module_graph();

    let _ = std::mem::replace(&mut context.module_graph, Box::new(RwLock::new(graph)));
    let context = Arc::new(context);
    let mut module_graph = context.module_graph.write();

    let module_group_graph = plugin
      .analyze_module_graph(
        &mut module_graph,
        &context,
        &PluginHookContext {
          caller: None,
          meta: HashMap::new(),
        },
      )
      .unwrap()
      .unwrap();

    assert_eq!(module_group_graph.len(), 5);
    assert!(module_group_graph.has(&"A".into()));
    assert!(module_group_graph.has(&"B".into()));
    assert!(module_group_graph.has(&"D".into()));
    assert!(module_group_graph.has(&"F".into()));
    assert!(module_group_graph.has(&"G".into()));

    let module_group_a = module_group_graph.module_group(&"A".into()).unwrap();
    assert_eq!(module_group_a.id, "A".into());
    assert_eq!(
      module_group_a.modules(),
      &HashSet::from(["A".into(), "C".into()])
    );

    let module_group_b = module_group_graph.module_group(&"B".into()).unwrap();
    assert_eq!(module_group_b.id, "B".into());
    assert_eq!(
      module_group_b.modules(),
      &HashSet::from(["B".into(), "D".into(), "E".into()])
    );

    let module_group_d = module_group_graph.module_group(&"D".into()).unwrap();
    assert_eq!(module_group_d.id, "D".into());
    assert_eq!(module_group_d.modules(), &HashSet::from(["D".into()]));

    let module_group_f = module_group_graph.module_group(&"F".into()).unwrap();
    assert_eq!(module_group_f.id, "F".into());
    assert_eq!(
      module_group_f.modules(),
      &HashSet::from(["F".into(), "A".into(), "C".into()])
    );

    let module_group_g = module_group_graph.module_group(&"G".into()).unwrap();
    assert_eq!(module_group_g.id, "G".into());
    assert_eq!(module_group_g.modules(), &HashSet::from(["G".into()]));

    let test_pairs = vec![(
      "A",
      vec!["A", "F"],
      ("B", vec!["B"]),
      ("C", vec!["A", "F"]),
      ("D", vec!["D", "B"]),
      ("E", vec!["B"]),
      ("F", vec!["F"]),
      ("G", vec!["G"]),
    )];

    for tp in test_pairs {
      let m_a = module_graph.module_mut(&tp.0.into()).unwrap();
      assert_eq!(m_a.module_groups.len(), tp.1.len());

      for g_id in tp.1 {
        assert!(m_a.module_groups.contains(&g_id.into()));
      }
    }
  }

  #[test]
  fn module_group_from_entry() {
    let mut graph = construct_test_module_graph();

    let (module_group, de) = mgfe(&"A".into(), &mut graph);
    assert_eq!(de, vec!["F".into(), "D".into()]);
    assert_eq!(module_group.id, "A".into());
    assert_eq!(
      module_group.modules(),
      &HashSet::from(["A".into(), "C".into()])
    );
    assert!(graph
      .module(&"A".into())
      .unwrap()
      .module_groups
      .contains(&"A".into()));
    assert!(graph
      .module(&"C".into())
      .unwrap()
      .module_groups
      .contains(&"A".into()));
  }

  #[test]
  fn module_group_graph_from_entries() {
    let mut graph = construct_test_module_graph();

    let entries = vec!["A".into(), "B".into()];
    let module_group_graph = crate::module_group_graph_from_entries(&entries, &mut graph);
    let final_module_group_graph = construct_test_module_group_graph();

    assert_eq!(module_group_graph, final_module_group_graph);
  }
}
