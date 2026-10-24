use std::sync::Arc;

#[cfg(test)]
mod watch_query_tests {
  use super::*;
  use farmfe_core::{config::Config, module::Module};

  #[test]
  fn query_only_source_without_watch_dependents_does_not_create_bare_module() {
    let mut config = Config::default();
    config.root = std::env::temp_dir().to_string_lossy().into_owned();
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let path = std::path::Path::new(&context.config.root)
      .join("product.svg")
      .to_string_lossy()
      .into_owned();
    for query in ["?dom", "?media"] {
      context
        .module_graph
        .write()
        .add_module(Module::new(ModuleId::new(
          &path,
          query,
          &context.config.root,
        )));
    }
    let paths = handle_update_modules(
      vec![(path.clone(), UpdateType::Updated)],
      &[],
      &context,
      &mut UpdateResult::default(),
    )
    .unwrap();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().all(|(p, _)| {
      !ModuleId::from_resolved_path_with_query(p, &context.config.root)
        .query_string()
        .is_empty()
    }));
  }

  #[test]
  fn watched_source_keeps_its_query_modules_in_the_update_paths() {
    let mut config = Config::default();
    config.root = std::env::temp_dir().to_string_lossy().into_owned();
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let path = std::path::Path::new(&context.config.root)
      .join("product.svg")
      .to_string_lossy()
      .into_owned();
    let source = ModuleId::new(&path, "", &context.config.root);
    let image = ModuleId::new(&path, "?dom", &context.config.root);
    let media = ModuleId::new(&path, "?media", &context.config.root);
    let css = ModuleId::new("styles.css", "", &context.config.root);
    context
      .module_graph
      .write()
      .add_module(Module::new(image.clone()));
    context.module_graph.write().add_module(Module::new(media));
    context
      .module_graph
      .write()
      .add_module(Module::new(css.clone()));
    context.add_watch_files(css.clone(), vec![source]).unwrap();
    let paths = handle_update_modules(
      vec![(path.clone(), UpdateType::Updated)],
      &[],
      &context,
      &mut UpdateResult::default(),
    )
    .unwrap();
    assert!(
      paths
        .iter()
        .any(|(p, _)| p == &image.resolved_path_with_query(&context.config.root)),
      "Update the existing query module"
    );
    assert!(
      !paths.iter().any(
        |(p, _)| ModuleId::from_resolved_path_with_query(p, &context.config.root)
          == ModuleId::new(&path, "", &context.config.root)
      ),
      "Do not create a bare module without an HMR boundary"
    );
    assert!(
      paths
        .iter()
        .any(|(p, _)| p == &css.resolved_path(&context.config.root))
    );
  }
}

use farmfe_core::{
  HashMap,
  context::CompilationContext,
  module::ModuleId,
  plugin::{PluginUpdateModulesHookParam, UpdateResult, UpdateType},
  serde_json,
  stats::CompilationPluginHookStats,
};

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
    .collect::<Vec<_>>();

  // group the paths by same resolved_path
  let grouped_paths = paths.iter().fold(
    HashMap::<String, Vec<String>>::default(),
    |mut acc, (path, _)| {
      let resolved_path = ModuleId::from_resolved_path_with_query(path, &context.config.root)
        .resolved_path(&context.config.root);

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

      let module_id = ModuleId::from_resolved_path_with_query(&resolved_path, &context.config.root);
      let has_bare_update = paths.iter().any(|path| {
        ModuleId::from_resolved_path_with_query(path, &context.config.root) == module_id
      });
      let mut result = if has_bare_update {
        vec![resolved_path.clone()]
      } else {
        vec![]
      };

      for path in paths {
        // if /root/index.ts and /root/index.ts?foo=bar are both in paths, we should remove the query module
        if path != resolved_path {
          let child_module_id =
            ModuleId::from_resolved_path_with_query(&path, &context.config.root);

          if child_module_id == module_id {
            continue;
          }

          let dependents = module_graph.dependents_ids(&child_module_id);

          if has_bare_update && dependents.contains(&module_id) && dependents.len() == 1 {
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
      let query_paths = if module_graph.has_module(&id) {
        vec![]
      } else {
        module_graph
          .module_ids_by_file(&id)
          .into_iter()
          .map(|module| {
            (
              module.resolved_path_with_query(&context.config.root),
              update_type.clone(),
            )
          })
          .collect::<Vec<_>>()
      };
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

        if module_graph.has_module(&id) {
          return [r, vec![(path, update_type)]].concat();
        };
        if !query_paths.is_empty() {
          return [r, query_paths].concat();
        }

        if !r.is_empty() {
          r
        } else {
          vec![(path, update_type)]
        }
      } else {
        if query_paths.is_empty() {
          vec![(path, update_type)]
        } else {
          query_paths
        }
      }
    })
    .collect()
}

fn resolve_last_failed_module_paths(
  mut paths: Vec<(String, UpdateType)>,
  last_failed_module_ids: &[ModuleId],
  context: &Arc<CompilationContext>,
) -> Vec<(String, UpdateType)> {
  let paths_ids = paths
    .iter()
    .map(|(p, _)| ModuleId::new(p, "", &context.config.root))
    .collect::<Vec<_>>();

  let last_failed_module_ids = last_failed_module_ids
    .iter()
    .filter(|id| !paths_ids.contains(id));

  paths.extend(
    last_failed_module_ids.map(|id| (id.resolved_path(&context.config.root), UpdateType::Updated)),
  );

  paths
}
