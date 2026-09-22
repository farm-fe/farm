use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{module_graph::ModuleGraph, module_group::{ModuleGroup, ModuleGroupGraph}, ModuleId},
  plugin::{Plugin, PluginHookContext},
  resource::resource_pot::ResourcePot,
};
use generate_module_buckets::{generate_module_buckets_map, group_module_buckets_by_module_group};
use generate_resource_pots::generate_resource_pots;

mod generate_module_buckets;
mod generate_module_pots;
mod generate_resource_pots;
mod merge_module_pots;
mod module_bucket;
mod module_pot;
mod utils;

pub struct FarmPluginPartialBundling {}

impl Plugin for FarmPluginPartialBundling {
  fn name(&self) -> &str { "FarmPluginPartialBundling" }
  fn priority(&self) -> i32 { 99 }

  fn analyze_module_graph(
    &self,
    module_graph: &mut ModuleGraph,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ModuleGroupGraph>> {
    Ok(Some(module_group_graph_from_entries(
      &module_graph.entries.clone().into_iter().map(|(entry, _)| entry).collect(),
      module_graph,
    )))
  }

  fn partial_bundling(
    &self,
    modules: &Vec<ModuleId>,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<ResourcePot>>> {
    let module_graph = context.module_graph.read();
    let module_group_graph = context.module_group_graph.read();
    let module_buckets_map = generate_module_buckets_map(modules, &module_graph);
    let module_group_buckets = group_module_buckets_by_module_group(
      &module_buckets_map, &module_group_graph, &module_graph,
    );
    Ok(Some(generate_resource_pots(
      module_group_buckets, module_buckets_map, &module_graph,
      &context.config.partial_bundling,
    )))
  }
}

impl FarmPluginPartialBundling {
  pub fn new(_: &Config) -> Self { Self {} }
}

pub fn module_group_graph_from_entries(
  entries: &Vec<ModuleId>,
  module_graph: &mut ModuleGraph,
) -> ModuleGroupGraph {
  let mut graph = ModuleGroupGraph::new();
  let mut edges = Vec::new();
  let mut visited = HashSet::new();
  let entry_ids: HashSet<ModuleId> = entries.iter().cloned().collect();
  let mut pending_dynamic = Vec::new();

  // Build every entry group before walking dynamic dependencies. This is
  // important when an entry is also the target of a dynamic import.
  for entry in entries.iter().cloned() {
    if visited.contains(&entry) { continue; }
    let (group, deps) = module_group_from_entry(&entry, module_graph);
    for dep in &deps {
      if entry_ids.contains(dep) { edges.push((group.id.clone(), dep.clone())); }
      else { pending_dynamic.push(dep.clone()); }
    }
    visited.insert(entry);
    graph.add_module_group(group);
  }

  let mut queue = VecDeque::from(pending_dynamic);
  while let Some(head) = queue.pop_front() {
    if visited.contains(&head) { continue; }
    visited.insert(head.clone());
    let (group, deps) = module_group_from_entry(&head, module_graph);
    for dep in deps {
      if entry_ids.contains(&dep) {
        edges.push((group.id.clone(), dep));
      } else {
        queue.push_back(dep);
      }
    }
    graph.add_module_group(group);
  }

  for (from, to) in edges {
    if graph.has(&from) && graph.has(&to) { graph.add_edge(&from, &to); }
  }
  graph
}

fn module_group_from_entry(
  entry: &ModuleId,
  graph: &mut ModuleGraph,
) -> (ModuleGroup, Vec<ModuleId>) {
  let mut visited = HashSet::new();
  let mut group = ModuleGroup::new(entry.clone());
  let mut dynamic_entries = Vec::new();
  graph.module_mut(entry).unwrap().module_groups.insert(entry.clone());
  visited.insert(entry.clone());

  for (dep, edge) in graph.dependencies(entry) {
    if edge.is_dynamic() {
      if !dynamic_entries.contains(&dep) { dynamic_entries.push(dep); }
      continue;
    }
    let mut queue = VecDeque::from([dep]);
    while let Some(head) = queue.pop_front() {
      if visited.contains(&head) { continue; }
      visited.insert(head.clone());
      group.add_module(head.clone());
      graph.module_mut(&head).unwrap().module_groups.insert(group.id.clone());
      for (child, edge) in graph.dependencies(&head) {
        if edge.is_dynamic() {
          if !dynamic_entries.contains(&child) { dynamic_entries.push(child); }
        } else { queue.push_back(child); }
      }
    }
  }
  (group, dynamic_entries)
}
