use std::collections::HashMap;

use farmfe_core::{
  config::Mode,
  module::{
    module_graph::ModuleGraph,
    module_group::{ModuleGroupGraph, ModuleGroupId},
    ModuleId,
  },
  resource::{resource_pot_map::ResourcePotMap, Resource, ResourceType},
};

pub fn get_dynamic_resources_map(
  module_group_graph: &ModuleGroupGraph,
  module_group_id: &ModuleGroupId,
  resource_pot_map: &ResourcePotMap,
  resources_map: &HashMap<String, Resource>,
  module_graph: &ModuleGraph,
) -> HashMap<ModuleId, Vec<(String, ResourceType)>> {
  let mut dep_module_groups = vec![];

  module_group_graph.bfs(&module_group_id, &mut |mg_id| {
    if mg_id != module_group_id {
      dep_module_groups.push(mg_id.clone());
    }
  });

  let mut dynamic_resources_map = HashMap::<ModuleId, Vec<(String, ResourceType)>>::new();

  for mg_id in dep_module_groups {
    let mg = module_group_graph.module_group(&mg_id).unwrap();

    for rp_id in &mg.sorted_resource_pots(module_graph, resource_pot_map) {
      let rp = resource_pot_map.resource_pot(rp_id).unwrap_or_else(|| {
        panic!(
          "Resource pot {} not found in resource pot map",
          rp_id.to_string()
        )
      });

      if dynamic_resources_map.contains_key(&mg_id) {
        let resources = dynamic_resources_map.get_mut(&mg_id).unwrap();

        for r in rp.resources() {
          let resource = resources_map.get(r).unwrap();

          // Currently only support js and css
          if !matches!(resource.resource_type, ResourceType::Js | ResourceType::Css) {
            continue;
          }

          resources.push((resource.name.clone(), resource.resource_type.clone()));
        }
      } else {
        let mut resources = vec![];

        for r in rp.resources() {
          let resource = resources_map
            .get(r)
            .unwrap_or_else(|| panic!("{r} not found"));

          // Currently only support js and css
          if !matches!(resource.resource_type, ResourceType::Js | ResourceType::Css) {
            continue;
          }

          resources.push((resource.name.clone(), resource.resource_type.clone()));
        }

        dynamic_resources_map.insert(mg_id.clone(), resources);
      }
    }
  }

  dynamic_resources_map
}

pub fn get_dynamic_resources_code(
  dynamic_resources_map: &HashMap<ModuleId, Vec<(String, ResourceType)>>,
  mode: Mode,
) -> (String, String) {
  let mut dynamic_resources_code_vec = vec![];
  let mut dynamic_resources = vec![];
  let mut visited_resources = HashMap::new();

  // inject dynamic resources
  let mut dynamic_resources_map_vec = dynamic_resources_map.iter().collect::<Vec<_>>();
  dynamic_resources_map_vec.sort_by_key(|(module_id, _)| module_id.to_string());
  for (module_id, resources) in dynamic_resources_map_vec {
    let mut dynamic_resources_index = vec![];

    for (resource_name, resource_type) in resources {
      let key = format!("{resource_name}{resource_type:?}");

      if let Some(index) = visited_resources.get(&key) {
        dynamic_resources_index.push(format!("{}", *index));
        continue;
      }

      match resource_type {
        ResourceType::Js => {
          dynamic_resources.push(format!(r#"{{ path: '{resource_name}', type: 0 }}"#));
        }
        ResourceType::Css => {
          dynamic_resources.push(format!(r#"{{ path: '{resource_name}', type: 1 }}"#));
        }
        _ => {
          panic!("unsupported type ({resource_type:?}) when injecting dynamic resources")
        }
      }

      dynamic_resources_index.push(format!("{}", dynamic_resources.len() - 1));
      visited_resources.insert(key, dynamic_resources.len() - 1);
    }

    let id = module_id.id(mode.clone()).replace(r"\", r"\\");
    dynamic_resources_code_vec.push((id, dynamic_resources_index.join(",")));
  }

  let mut dynamic_resources_code = dynamic_resources_code_vec
    .into_iter()
    .map(|(id, resources_code)| format!(r#"'{id}': [{resources_code}]"#))
    .collect::<Vec<_>>()
    .join(",");

  dynamic_resources_code = format!("{{ {dynamic_resources_code} }}");

  (
    format!("[{}]", dynamic_resources.join(",")),
    dynamic_resources_code,
  )
}
