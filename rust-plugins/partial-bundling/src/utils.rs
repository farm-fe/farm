use farmfe_core::{
  hashbrown::HashMap,
  module::{module_graph::ModuleGraph, ModuleId},
};
use std::{
  collections::{HashSet, VecDeque},
  path::PathBuf,
};

pub fn try_get_filename(module_id: &ModuleId, named_map: &HashMap<ModuleId, String>) -> String {
  let mut result = Vec::new();

  let mut filepath = PathBuf::from(module_id.to_string());

  if filepath.extension().is_some() {
    filepath.set_extension("");
  }

  while let Some(filename) = filepath.file_name() {
    result.insert(0, filename.to_string_lossy().to_string());

    filepath.pop();

    let name = result.join("_").replace('/', "_");

    if !named_map.values().any(|named| named == &name) {
      return name;
    }
  }

  result.join("_").replace('/', "_")
}

pub fn is_subset<T: PartialEq>(v1: &[T], v2: &[T]) -> bool {
  v1.iter().all(|item| v2.contains(item))
}

pub fn ids_to_string<'a, I: Iterator<Item = &'a K>, K>(resources: I) -> String
where
  K: ToString + 'a,
{
  let mut module_group_ids = resources.map(|id| id.to_string()).collect::<Vec<String>>();

  module_group_ids.sort();

  module_group_ids
    .clone()
    .into_iter()
    .collect::<Vec<String>>()
    .join("_")
}

#[derive(Debug, Clone)]
pub struct ModuleChain {
  pub has_dynamic: bool,
  pub chains: Vec<(bool, ModuleId)>,
  last: ModuleId,
  visit: HashSet<String>,
}

pub fn find_module_importer_chains(
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
) -> Vec<ModuleChain> {
  let mut queen = VecDeque::from([ModuleChain {
    has_dynamic: false,
    chains: Vec::from([(false, module_id.clone())]),
    last: module_id.clone(),
    visit: HashSet::new(),
  }]);
  let mut result: Vec<ModuleChain> = Vec::new();

  let mut module_groups = module_graph
    .module(module_id)
    .map(|item| item.module_groups.clone())
    .unwrap();

  'out: while !queen.is_empty() {
    let mut module_chain = queen.pop_back().unwrap();

    let last_import = &module_chain.last;

    let last_import_modules_groups = &module_graph.module(last_import).unwrap().module_groups;
    if !module_groups
      .iter()
      .any(|item| last_import_modules_groups.contains(item))
    {
      continue;
    }

    let dynamic_importer = module_graph
      .module_importer(last_import)
      .into_iter()
      .filter(|item| {
        module_graph
          .edge_info(item, last_import)
          .is_some_and(|item| item.is_dynamic())
      })
      .collect::<HashSet<_>>();

    for dynamic_importer_id in dynamic_importer {
      let mut p = module_chain.clone();

      p.has_dynamic = true;

      p.chains.push((true, dynamic_importer_id.clone()));

      p.last = dynamic_importer_id;

      queen.push_back(p);
    }

    for module_groups_id in last_import_modules_groups {
      let importer_module_groups = &module_graph.module(module_groups_id).unwrap().module_groups;

      let hash = module_groups_id.hash();

      if !module_groups
        .iter()
        .any(|item| importer_module_groups.contains(item))
        || module_chain.visit.contains(&hash)
      {
        continue;
      }

      module_chain.visit.insert(hash);

      let mut new_module_chain = module_chain.clone();

      new_module_chain.last = module_groups_id.clone();

      if module_graph.entries.contains_key(module_groups_id) {
        new_module_chain
          .chains
          .push((false, module_groups_id.clone()));
        result.push(new_module_chain.clone());
        module_groups.remove(module_groups_id);

        if module_groups.is_empty() {
          break 'out;
        }

        continue;
      }

      queen.push_back(new_module_chain);
    }
  }

  result
}
