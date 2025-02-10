use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::ModuleId,
  plugin::{PluginUpdateModulesHookParam, UpdateResult, UpdateType},
  serde_json,
  stats::CompilationPluginHookStats,
  HashMap,
};
use farmfe_utils::relative;

pub fn handle_update_modules(
  paths: Vec<(String, UpdateType)>,
  last_fail_module_ids: &[ModuleId],
  context: &Arc<CompilationContext>,
  update_result: &mut UpdateResult,
) -> farmfe_core::error::Result<Vec<(String, UpdateType)>> {
  let (before_paths, start_time) = if context.config.record {
    (
      paths.clone(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis(),
    )
  } else {
    (vec![], 0)
  };
  let paths = resolve_watch_graph_paths(paths, context);
  let paths = resolve_last_failed_module_paths(paths, last_fail_module_ids, context);

  if context.config.record {
    let end_time = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    context
      .stats
      .add_plugin_hook_stats(CompilationPluginHookStats {
        plugin_name: "InternalWatchGraphPlugin".to_string(),
        hook_name: "update_modules".to_string(),
        module_id: "".into(),
        hook_context: None,
        input: serde_json::to_string(&before_paths).unwrap(),
        output: serde_json::to_string(&paths).unwrap(),
        duration: end_time - start_time,
        start_time,
        end_time,
      })
  }
  let mut plugin_update_modules_hook_params = PluginUpdateModulesHookParam { paths };

  context
    .plugin_driver
    .update_modules(&mut plugin_update_modules_hook_params, context)?;

  let (before_params, start_time) = if context.config.record {
    (
      serde_json::to_string(&plugin_update_modules_hook_params).unwrap(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis(),
    )
  } else {
    ("".to_string(), 0)
  };
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

  let paths = [paths, additional_paths]
    .concat()
    .into_iter()
    .filter(|(p, _)| {
      let id = ModuleId::from_resolved_path_with_query(p, &context.config.root);
      module_graph.has_module(&id)
    })
    .collect::<Vec<_>>();

  // group the paths by same resolved_path
  let grouped_paths = paths.iter().fold(
    HashMap::<String, Vec<String>>::default(),
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

  let result: Vec<(String, UpdateType)> = filtered_paths
    .into_iter()
    .filter(|p| {
      let id = ModuleId::from_resolved_path_with_query(p, &context.config.root);
      module_graph.has_module(&id)
    })
    .map(|p| {
      if let Some((_, ty)) = paths.iter().find(|(pp, _)| *pp == p) {
        (p, ty.clone())
      } else {
        (p, UpdateType::Updated)
      }
    })
    .collect();

  if context.config.record {
    let end_time = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    context
      .stats
      .add_plugin_hook_stats(CompilationPluginHookStats {
        plugin_name: "InternalUpdateModulesPlugin".to_string(),
        hook_name: "update_modules".to_string(),
        module_id: "".into(),
        hook_context: None,
        input: before_params,
        output: serde_json::to_string(&UpdateModulesStatsResult {
          paths: result.clone(),
          update_result: update_result.clone(),
        })
        .unwrap(),
        duration: end_time - start_time,
        start_time,
        end_time,
      })
  }

  Ok(result)
}

#[derive(farmfe_core::serde::Serialize, farmfe_core::serde::Deserialize)]
#[serde(crate = "farmfe_core::serde", rename_all = "camelCase")]
struct UpdateModulesStatsResult {
  pub paths: Vec<(String, UpdateType)>,
  pub update_result: UpdateResult,
}

fn resolve_watch_graph_paths(
  paths: Vec<(String, UpdateType)>,
  context: &Arc<CompilationContext>,
) -> Vec<(String, UpdateType)> {
  let watch_graph = context.watch_graph.read();
  let module_graph = context.module_graph.read();

  // fetch watch file relation module, and replace watch file
  paths
    .into_iter()
    .flat_map(|(path, update_type)| {
      let id = ModuleId::new(&path, "", &context.config.root);

      if watch_graph.has_module(&id) {
        let r: Vec<(String, UpdateType)> = watch_graph
          .relation_roots(&id)
          .into_iter()
          .map(|item| {
            (
              item.resolved_path(&context.config.root),
              UpdateType::Updated,
            )
          })
          .collect();

        if module_graph.has_module(&ModuleId::new(path.as_str(), "", &context.config.root)) {
          return [r, vec![(path, update_type)]].concat();
        };

        if !r.is_empty() {
          r
        } else {
          vec![(path, update_type)]
        }
      } else {
        vec![(path, update_type)]
      }
    })
    .collect()
}

fn resolve_last_failed_module_paths(
  mut paths: Vec<(String, UpdateType)>,
  last_fail_module_ids: &[ModuleId],
  context: &Arc<CompilationContext>,
) -> Vec<(String, UpdateType)> {
  paths.extend(
    last_fail_module_ids
      .iter()
      .map(|id| (id.resolved_path(&context.config.root), UpdateType::Updated)),
  );

  paths
}
