use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::ModuleId,
  plugin::{PluginUpdateModulesHookParams, UpdateResult, UpdateType},
};
use farmfe_utils::relative;

pub fn handle_update_modules(
  paths: Vec<(String, UpdateType)>,
  context: &Arc<CompilationContext>,
  update_result: &mut UpdateResult,
) -> farmfe_core::error::Result<Vec<(String, UpdateType)>> {
  let mut plugin_update_modules_hook_params = PluginUpdateModulesHookParams { paths };

  context
    .plugin_driver
    .update_modules(&mut plugin_update_modules_hook_params, context)?;

  let paths = plugin_update_modules_hook_params.paths;
  let mut module_graph = context.module_graph.write();

  let mut additional_paths = vec![];

  for (path, _) in &paths {
    if !path.contains('?') {
      let path_id = ModuleId::new(path, "", &context.config.root);
      let ids = module_graph.module_ids_by_file(&path_id);

      for id in ids {
        let resolved_path = id.resolved_path_with_query(&context.config.root);

        if !paths.iter().any(|(p, _)| *p == resolved_path) {
          additional_paths.push((resolved_path, UpdateType::Updated));
        }
      }
    }
  }

  let paths = vec![paths, additional_paths].concat();

  // group the paths by same resolved_path
  let grouped_paths = paths.iter().fold(
    HashMap::<String, Vec<String>>::new(),
    |mut acc, (path, _)| {
      let resolved_path = path.split('?').next().unwrap().to_string();

      acc.entry(resolved_path).or_default().push(path.to_string());

      acc
    },
  );

  let mut module_group_graph = context.module_group_graph.write();

  // if there are multiple paths for the same resolved_path and one of them is a ancestor of another, we should remove child module in module graph
  let filtered_paths = grouped_paths
    .into_iter()
    .flat_map(|(resolved_path, paths)| {
      if paths.len() == 1 {
        return paths;
      }

      let module_id: ModuleId = relative(&context.config.root, &resolved_path).into();
      let mut result = vec![resolved_path.clone()];

      for path in paths {
        // if /root/index.vue and /root/index.vue?foo=bar are both in paths, we should remove /root/index.vue?foo=bar
        if path != resolved_path {
          let child_module_id: ModuleId = relative(&context.config.root, &path).into();
          let dependents = module_graph.dependents_ids(&child_module_id);

          if dependents.contains(&module_id) && dependents.len() == 1 {
            let removed_module = module_graph.remove_module(&child_module_id);

            for module_group_id in removed_module.module_groups {
              let module_group = module_group_graph
                .module_group_mut(&module_group_id)
                .unwrap();
              module_group.remove_module(&child_module_id);
            }

            update_result.removed_module_ids.push(child_module_id)
          } else {
            result.push(path);
          }
        }
      }

      result
    })
    .collect::<Vec<_>>();

  Ok(
    filtered_paths
      .into_iter()
      .map(|p| {
        if let Some((_, ty)) = paths.iter().find(|(pp, _)| *pp == p) {
          (p, ty.clone())
        } else {
          (p, UpdateType::Updated)
        }
      })
      .collect(),
  )
}
