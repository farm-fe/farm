use std::{collections::VecDeque, sync::Arc};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  hashbrown::{HashMap, HashSet},
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroup, ModuleGroupGraph},
    ModuleId, ModuleType,
  },
  plugin::{Plugin, PluginHookContext},
  resource::resource_pot::{ResourcePot, ResourcePotId},
};

pub struct FarmPluginPartialBundling {}

impl Plugin for FarmPluginPartialBundling {
  fn name(&self) -> &str {
    "FarmPluginPartialBundling"
  }

  fn analyze_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ModuleGroupGraph>> {
    let module_group_graph = module_group_graph_from_entries(
      &module_graph.entries.clone().into_iter().collect(),
      module_graph,
    );

    Ok(Some(module_group_graph))
  }

  /// The partial bundling algorithm's result should not be related to the order of the module group.
  /// Whatever the order of the module group is, the result should be the same.
  fn partial_bundling(
    &self,
    module_group: &mut ModuleGroup,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<ResourcePot>>> {
    let mut module_graph = context.module_graph.write();
    let mut resource_pots = vec![];

    let mut module_type_resource_pot_map = HashMap::<ModuleType, ResourcePot>::new();

    for module_id in module_group.modules() {
      let module = module_graph.module_mut(module_id).unwrap();

      // TODO remove this after the partial bundle is implemented
      if module.module_type.is_script() {
        module.module_type = ModuleType::Js;
      }

      let resource_pot = if module_type_resource_pot_map.contains_key(&module.module_type) {
        let resource_pot = module_type_resource_pot_map
          .get_mut(&module.module_type)
          .unwrap();
        module.resource_pot = Some(resource_pot.id.clone());
        resource_pot.add_module(module_id.clone());

        resource_pot
      } else {
        let id = if module_id.to_string().ends_with(".html") {
          module.id.to_string()
        } else {
          let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
          format!(
            "{}_{:?}_{}",
            now,
            module.module_type,
            module_group.id.to_string()
          )
        };
        let mut resource_pot = ResourcePot::new(
          // TODO design a better id for resource pot
          ResourcePotId::new(id),
          module.module_type.clone().into(),
          module_group.id.clone(),
        );
        module.resource_pot = Some(resource_pot.id.clone());
        resource_pot.add_module(module_id.clone());
        module_type_resource_pot_map.insert(module.module_type.clone(), resource_pot);

        module_type_resource_pot_map
          .get_mut(&module.module_type)
          .unwrap()
      };

      if module_graph.entries.contains(module_id) {
        resource_pot.entry_module = Some(module_id.clone());
      }
    }

    for resource_pot in module_type_resource_pot_map.into_values() {
      // module_group.add_resource_pot(resource_pot.id.clone());
      resource_pots.push(resource_pot);
    }

    Ok(Some(resource_pots))
  }
}

impl FarmPluginPartialBundling {
  pub fn new(_: &Config) -> Self {
    Self {}
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

    while queue.len() > 0 {
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

  for (dep, kind, _) in graph.dependencies(entry) {
    if kind.is_dynamic() {
      dynamic_entries.push(dep);
    } else {
      // visited all dep and its dependencies using BFS
      let mut queue = VecDeque::new();
      queue.push_back(dep.clone());

      while queue.len() > 0 {
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

        for (dep, kind, _) in graph.dependencies(&head) {
          if kind.is_dynamic() {
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
  use std::{collections::HashMap, sync::Arc};

  use farmfe_core::{
    context::CompilationContext,
    hashbrown::HashSet,
    parking_lot::RwLock,
    plugin::{Plugin, PluginHookContext},
  };
  #[cfg(test)]
  use farmfe_testing_helpers::construct_test_module_graph;
  use farmfe_testing_helpers::construct_test_module_group_graph;

  use crate::{module_group_from_entry as mgfe, FarmPluginPartialBundling};

  #[test]
  fn analyze_module_graph() {
    let plugin = FarmPluginPartialBundling {};
    let mut context = CompilationContext::new(Default::default(), vec![]).unwrap();
    let graph = construct_test_module_graph();

    let _ = std::mem::replace(&mut context.module_graph, RwLock::new(graph));
    let context = Arc::new(context);
    let mut module_graph = context.module_graph.write();

    let module_group_graph = plugin
      .analyze_module_graph(
        &mut *module_graph,
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
