use std::{collections::HashMap, path::PathBuf, sync::Arc};

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
  println!("handle paths: {:?}", paths);
  // group the paths by same resolved_path
  let grouped_paths = paths.iter().fold(HashMap::new(), |mut acc, (path, _)| {
    let resolved_path = path.split('?').next().unwrap().to_string();

    acc
      .entry(resolved_path)
      .or_insert_with(Vec::new)
      .push(path.to_string());

    acc
  });

  println!("grouped_paths: {:?}", grouped_paths);

  let mut module_graph = context.module_graph.write();

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
        if path != resolved_path {
          let child_module_id: ModuleId = relative(&context.config.root, &path).into();
          let dependents = module_graph.dependents_ids(&child_module_id);
          println!("{:?}'s dependents: {:?}", child_module_id, dependents);
          if dependents.contains(&module_id) && dependents.len() == 1 {
            module_graph.remove_module(&child_module_id);
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
    paths
      .into_iter()
      .filter(|(path, _)| filtered_paths.contains(path))
      .collect(),
  )
}
