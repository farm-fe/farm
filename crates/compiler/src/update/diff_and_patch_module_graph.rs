//! diff the module_graph and update_module_graph, analyze the changes and then patch the module_graph

use std::collections::VecDeque;

use farmfe_core::{
  hashbrown::{HashMap, HashSet},
  module::{
    module_graph::{ModuleGraph, ModuleGraphEdge},
    Module, ModuleId,
  },
};

/// the diff result of a module's dependencies
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModuleDepsDiffResult {
  /// added dependencies
  pub added: Vec<(ModuleId, ModuleGraphEdge)>,
  /// removed dependencies
  pub removed: Vec<(ModuleId, ModuleGraphEdge)>,
}

pub type ModuleDepsDiffResultMap = Vec<(ModuleId, ModuleDepsDiffResult)>;
/// the diff result of a module, this records all related changes of the module graph
/// for example, deeply added or removed dependencies also be recorded here
#[derive(Debug, Default, Clone)]
pub struct DiffResult {
  pub deps_changes: ModuleDepsDiffResultMap,
  pub added_modules: HashSet<ModuleId>,
  pub removed_modules: HashSet<ModuleId>,
}
#[cfg(test)]
impl DiffResult {
  pub fn readable_print(&self) {
    // print the diff result in a readable way
    // for example:
    // a ->
    //    added: b, c
    //    removed: d
    // b -> ..
    // removed_modules: e, f
    // added_modules: g, h

    for (module_id, deps_diff_result) in &self.deps_changes {
      println!("{:?} -> ", module_id);

      println!(
        "  added: {:?}",
        deps_diff_result
          .added
          .iter()
          .map(|(id, edge)| (
            id,
            edge
              .items()
              .iter()
              .map(|item| item.kind.clone())
              .collect::<Vec<_>>()
          ))
          .collect::<Vec<_>>()
      );

      println!(
        "  removed: {:?}",
        deps_diff_result
          .removed
          .iter()
          .map(|(id, edge)| (
            id,
            edge
              .items()
              .iter()
              .map(|item| item.kind.clone())
              .collect::<Vec<_>>()
          ))
          .collect::<Vec<_>>()
      );
    }

    println!("added_modules: {:?}", self.added_modules);

    println!("removed_modules: {:?}", self.removed_modules);
  }
}

/// diff the module_graph and update_module_graph, return the diff result
pub fn diff_module_graph(
  start_points: Vec<ModuleId>,
  module_graph: &ModuleGraph,
  update_module_graph: &ModuleGraph,
) -> DiffResult {
  let mut res: DiffResult = DiffResult {
    deps_changes: vec![],
    added_modules: HashSet::new(),
    removed_modules: HashSet::new(),
  };

  let (diff_result, added_modules, remove_modules) =
    diff_module_deps(&start_points, module_graph, update_module_graph);

  res.deps_changes.extend(diff_result);
  res.added_modules.extend(added_modules);
  res.removed_modules.extend(remove_modules);

  res
}

/// patch the module_graph according to the diff result
pub fn patch_module_graph(
  start_points: Vec<ModuleId>,
  diff_result: &DiffResult,
  module_graph: &mut ModuleGraph,
  update_module_graph: &mut ModuleGraph,
) -> HashMap<ModuleId, Module> {
  let mut removed_modules = HashMap::new();
  let mut added_edge_info = HashMap::<(ModuleId, ModuleId), ModuleGraphEdge>::new();

  for (module_id, deps_diff_result) in diff_result.deps_changes.iter() {
    for (removed_dep, _) in &deps_diff_result.removed {
      module_graph.remove_edge(module_id, removed_dep).unwrap();
    }

    for (added_dep, _) in &deps_diff_result.added {
      let edge_info = update_module_graph
        .edge_info(module_id, added_dep)
        .unwrap()
        .clone();
      added_edge_info.insert((module_id.clone(), added_dep.clone()), edge_info);
    }
  }

  // add new modules first, as we need to add edges to them later
  for added in &diff_result.added_modules {
    let module = update_module_graph.take_module(added);
    module_graph.add_module(module);
  }

  for ((from, to), edge_info) in added_edge_info {
    module_graph.add_edge(&from, &to, edge_info).unwrap();
  }

  // remove removed modules
  for removed in &diff_result.removed_modules {
    let removed_module = module_graph.remove_module(removed);
    removed_modules.insert(removed.clone(), removed_module);
  }

  // we must remove updated module at last, cause petgraph will remove edge when remove node
  for updated in start_points {
    let module = {
      let mut m = update_module_graph.take_module(&updated);
      let previous_module = module_graph.module(&updated).unwrap();
      m.module_groups = previous_module.module_groups.clone();
      m.resource_pot = previous_module.resource_pot.clone();
      m
    };

    module_graph.replace_module(module);
  }

  removed_modules
}

/// diff the module_graph and update_module_graph, return the diff result
/// for example:
/// ```ignore
/// 1. when the deps not changed
/// module_graph:
/// a -> b -> c
///   \-> d
/// update_module_graph:
/// a(changed) -> b -> c
///   \-> d
/// diff_result:
/// (ModuleDepsDiffResult { added: [], removed: [] }, HashSet::new(), HashSet::new())
///
/// 2. when the deps changed
/// module_graph:
/// a -> b -> c
///  \-> d
/// update_module_graph:
/// a(changed) ->(dep removed) b -> c
///   \-> d
///   \->(dep added) f
/// diff_result:
///   ({
///     a: ModuleDepsDiffResult { added: [f], removed: [b] }
///     b: ModuleDepsDiffResult { added: [], removed: [c] }
///    }, [f], [b, c])
///
/// 3. when the deps added with new module depend on existing module
/// module_graph:
/// a -> b -> c
/// update_module_graph:
/// a(changed) -> b -> c
///  \->(dep added) d -> c(existing module)
/// diff_result:
///  ({
///    a: ModuleDepsDiffResult { added: [d], removed: [] }
///    d: ModuleDepsDiffResult { added: [c], removed: [] }
///  }, [d], [])
///
/// 4. when the deps removed with removed module  depend on existing module
/// module_graph:
/// a -> b -> c
///  \-> d -> c
/// update_module_graph:
/// a(changed) -> b -> c
///  \->(dep removed) d -> c(existing module)
/// diff_result:
/// ({
///  a: ModuleDepsDiffResult { added: [], removed: [d] }
///  d: ModuleDepsDiffResult { added: [], removed: [c] }
/// }, [], [d])
/// ```
///
/// See test cases for more examples
fn diff_module_deps(
  module_ids: &Vec<ModuleId>,
  module_graph: &ModuleGraph,
  update_module_graph: &ModuleGraph,
) -> (
  ModuleDepsDiffResultMap,
  HashSet<ModuleId>,
  HashSet<ModuleId>,
) {
  // added_deps of to_id(the to node of the removed edge) -> from_id
  let mut all_added_deps_reverse = HashMap::new();
  // removed_deps of to_id(the to node of the removed edge) -> from_id
  let mut all_removed_deps_reverse = HashMap::new();
  let mut added_modules = HashSet::new();

  let mut diff_result = Vec::new();

  for module_id in module_ids {
    // Find the added and removed dependencies of current updated module
    let mut added_deps = Vec::new();
    let mut removed_deps = Vec::new();

    let deps = module_graph.dependencies_ids(module_id);
    let update_deps = update_module_graph.dependencies_ids(module_id);

    for dep in &deps {
      if !update_deps.contains(dep) {
        let edge_info = module_graph.edge_info(module_id, dep).unwrap();
        removed_deps.push((dep.clone(), edge_info.clone()));
      } else {
        // deal with edge info changes, e.g. static import changed to dynamic import
        let edge_info = module_graph.edge_info(module_id, dep).unwrap();
        let update_edge_info = update_module_graph.edge_info(module_id, dep).unwrap();

        if edge_info != update_edge_info {
          removed_deps.push((dep.clone(), edge_info.clone()));
          added_deps.push((dep.clone(), update_edge_info.clone()));
        }
      }
    }

    for dep in update_deps {
      if !deps.contains(&dep) {
        let edge_info = update_module_graph.edge_info(module_id, &dep).unwrap();
        added_deps.push((dep, edge_info.clone()));
      }
    }
    // the deps not changed
    if !added_deps.is_empty() || !removed_deps.is_empty() {
      if !added_deps.is_empty() {
        for (added_dep, _) in &added_deps {
          all_added_deps_reverse
            .entry(added_dep.clone())
            .or_insert_with(HashSet::new)
            .insert(module_id.clone());

          if !module_graph.has_module(added_dep) {
            added_modules.insert(added_dep.clone());
          }
        }
        // all_added_deps.insert(module_id.clone(), added_deps.clone());
      }

      if !removed_deps.is_empty() {
        for (removed_dep, _) in &removed_deps {
          all_removed_deps_reverse
            .entry(removed_dep.clone())
            .or_insert_with(HashSet::new)
            .insert(module_id.clone());
        }
        // all_removed_deps.insert(module_id.clone(), removed_deps.clone());
      }

      diff_result.push((
        module_id.clone(),
        ModuleDepsDiffResult {
          added: added_deps.clone(),
          removed: removed_deps.clone(),
        },
      ));
    }
  }

  if diff_result.is_empty() {
    return (diff_result, HashSet::new(), HashSet::new());
  }

  let all_removed_deps = all_removed_deps_reverse
    .iter()
    .map(|(k, _)| k.clone())
    .collect::<HashSet<_>>();
  let mut removed_modules_vec = all_removed_deps
    .into_iter()
    .filter_map(|id| {
      // entry should not be removed for any reason
      if module_graph.entries.contains_key(&id) {
        return None;
      }

      let dependents = module_graph.dependents_ids(&id);
      // if all the edges of the dependents are removed, then the module should be removed
      let is_all_dependents_edge_removed =
        if let Some(removed_dependents) = all_removed_deps_reverse.get(&id) {
          dependents
            .iter()
            .all(|dept| removed_dependents.contains(dept))
        } else {
          false
        };
      let is_edge_added = all_added_deps_reverse.contains_key(&id);

      if is_all_dependents_edge_removed && !is_edge_added {
        Some(id)
      } else {
        None
      }
    })
    .collect::<Vec<_>>();
  removed_modules_vec.sort();
  let mut removed_modules: HashSet<ModuleId> = removed_modules_vec.clone().into_iter().collect();

  // Find all added and removed children deeply of added and removed dependencies
  let mut added_modules_vec = added_modules.clone().into_iter().collect::<Vec<_>>();
  added_modules_vec.sort();
  let mut added_queue = VecDeque::from(added_modules_vec);
  let mut added_visited = HashSet::new();

  while let Some(dep) = added_queue.pop_front() {
    if added_visited.contains(&dep) {
      continue;
    }

    added_visited.insert(dep.clone());

    let children = update_module_graph.dependencies_ids(&dep);
    let mut children_added = vec![];

    if module_graph.has_module(&dep) {
      panic!("The module({:?}) exists in previous module graph, this should never happen and there is a internal bug inside farm. Please report it via issues", dep);
    }

    for child in children {
      let edge_info = update_module_graph.edge_info(&dep, &child).unwrap();
      children_added.push((child.clone(), edge_info.clone()));
      all_added_deps_reverse
        .entry(child.clone())
        .or_insert_with(HashSet::new)
        .insert(dep.clone());

      if !module_graph.has_module(&child) {
        added_queue.push_back(child.clone());
        added_modules.insert(child);
      } else {
        // the removed module is added again, so we should remove it from removed_modules
        if removed_modules.contains(&child) {
          removed_modules.remove(&child);
          removed_modules_vec.retain(|item| item != &child);
        }
      }
    }

    if children_added.is_empty() {
      continue;
    }

    diff_result.push((
      dep,
      ModuleDepsDiffResult {
        added: children_added,
        removed: vec![], // no removed children for added dependencies
      },
    ));
  }

  let mut removed_queue = VecDeque::from(removed_modules_vec);
  let mut removed_visited = HashSet::new();

  while let Some(dep) = removed_queue.pop_front() {
    let children = module_graph.dependencies_ids(&dep);
    let mut children_removed = vec![];

    for child in children {
      let visited_key = (dep.clone(), child.clone());

      if removed_visited.contains(&visited_key) {
        continue;
      }

      removed_visited.insert(visited_key);

      let edge_info = module_graph.edge_info(&dep, &child).unwrap();
      children_removed.push((child.clone(), edge_info.clone()));

      let dependents = module_graph.dependents_ids(&child);
      let is_edge_added = all_added_deps_reverse.contains_key(&child);
      // if all dependents of child are removed, then child should be removed
      if dependents.iter().all(|dept| removed_modules.contains(dept))
        && !module_graph.entries.contains_key(&child)
        && !is_edge_added
      {
        removed_queue.push_back(child.clone());
        removed_modules.insert(child);
      }
    }

    if children_removed.is_empty() {
      continue;
    }

    diff_result.push((
      dep,
      ModuleDepsDiffResult {
        added: vec![], // no added children for removed dependencies
        removed: children_removed,
      },
    ));
  }

  (diff_result, added_modules, removed_modules)
}

#[cfg(test)]
mod test_diff_module_deps;
#[cfg(test)]
mod test_diff_module_graph;
#[cfg(test)]
mod test_patch_module_graph;
