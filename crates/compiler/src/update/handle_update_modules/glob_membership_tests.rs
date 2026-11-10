use super::*;
use farmfe_core::{
  config::Config,
  module::{Module, ModuleType, meta_data::ModuleMetaData},
};
use farmfe_plugin_script::FarmPluginScript;

fn fixture(cached_metadata: bool) -> (Arc<CompilationContext>, String, Vec<String>) {
  let config = Config {
    root: std::env::temp_dir()
      .join("farm-glob-membership")
      .to_string_lossy()
      .into_owned(),
    ..Default::default()
  };
  let plugin = Arc::new(FarmPluginScript::new(&config));
  let context = Arc::new(CompilationContext::new(config, vec![plugin]).unwrap());
  let root = &context.config.root;
  let directory = std::path::Path::new(root).join("candidates");
  let path = directory.join("probe.tsx").to_string_lossy().into_owned();
  let css = ModuleId::new("styles.css", "", root);
  context
    .module_graph
    .write()
    .add_module(Module::new(css.clone()));
  // A candidate may remain watched by Tailwind while absent from the module graph.
  context
    .add_watch_files(css, vec![ModuleId::new(&path, "", root)])
    .unwrap();
  let mut importers = Vec::new();
  for name in ["eager.ts", "lazy.ts"] {
    let id = ModuleId::new(name, "", root);
    importers.push(id.resolved_path(root));
    let mut module = Module::new(id);
    module.module_type = ModuleType::Js;
    module.meta = Box::new(ModuleMetaData::Script(Box::default()));
    let watches = serde_json::json!([[{
      "root": directory, "pattern": "*.tsx", "negative": false,
      "exhaustive": false, "case_insensitive": false
    }, {
      "root": directory, "pattern": "excluded*.tsx", "negative": true,
      "exhaustive": false, "case_insensitive": false
    }]]);
    let custom = &mut module.meta.as_script_mut().custom;
    custom.insert("farm:glob:v1".into(), Box::new(watches.to_string()));
    if cached_metadata {
      // Clone serializes the custom entries to their cached bytes representation.
      *custom = custom.clone();
    }
    context.module_graph.write().add_module(module);
  }
  (context, path, importers)
}

#[test]
fn watched_glob_membership_reaches_importers_after_empty_match_and_restore() {
  for cached in [false, true] {
    let (context, path, importers) = fixture(cached);
    // No candidate module exists: covers an empty match and repeated same-path membership events.
    for kind in [UpdateType::Added, UpdateType::Removed, UpdateType::Added] {
      let paths = handle_update_modules(
        vec![(path.clone(), kind)],
        &[],
        &context,
        &mut UpdateResult::default(),
      )
      .unwrap();
      for importer in &importers {
        assert!(
          paths
            .iter()
            .any(|(file, kind)| file == importer && matches!(kind, UpdateType::Updated)),
          "missing importer {importer}: {paths:?}, cached={cached}"
        );
      }
      assert!(
        !paths
          .iter()
          .any(|(file, kind)| file == &path && matches!(kind, UpdateType::Updated)),
        "membership must not create an unrelated bare update"
      );
    }
  }
}

#[test]
fn watched_non_matching_and_negative_glob_members_do_not_invalidate_importers() {
  let (context, path, importers) = fixture(false);
  for filename in ["excluded-probe.tsx", "probe.txt"] {
    let file = std::path::Path::new(&path)
      .with_file_name(filename)
      .to_string_lossy()
      .into_owned();
    context
      .add_watch_files(
        ModuleId::new("styles.css", "", &context.config.root),
        vec![ModuleId::new(&file, "", &context.config.root)],
      )
      .unwrap();
    let paths = handle_update_modules(
      vec![(file, UpdateType::Added)],
      &[],
      &context,
      &mut UpdateResult::default(),
    )
    .unwrap();
    assert!(!paths.iter().any(|(file, _)| importers.contains(file)));
    assert!(
      paths
        .iter()
        .all(|(file, kind)| !matches!(kind, UpdateType::Updated) || file.ends_with("styles.css")),
      "non-matching notifications must not become compile targets: {paths:?}"
    );
  }
}

#[test]
fn membership_notifications_preserve_query_instances_without_bare_targets() {
  let (context, path, importers) = fixture(false);
  for query in ["?raw", "?url"] {
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
    vec![(path.clone(), UpdateType::Added)],
    &[],
    &context,
    &mut UpdateResult::default(),
  )
  .unwrap();
  for importer in importers {
    assert!(
      paths
        .iter()
        .any(|(file, kind)| file == &importer && matches!(kind, UpdateType::Updated))
    );
  }
  for query in ["?raw", "?url"] {
    assert!(
      paths
        .iter()
        .any(|(file, _)| file == &format!("{path}{query}"))
    );
  }
  assert!(
    !paths
      .iter()
      .any(|(file, kind)| file == &path && matches!(kind, UpdateType::Updated))
  );
}

#[test]
fn membership_hook_runs_once_and_preserves_explicit_same_path_compile_target() {
  use farmfe_core::plugin::Plugin;
  use std::sync::atomic::{AtomicUsize, Ordering};
  struct Observer(Arc<AtomicUsize>);
  impl Plugin for Observer {
    fn name(&self) -> &str {
      "membership-test-observer"
    }
    fn update_modules(
      &self,
      params: &mut PluginUpdateModulesHookParam,
      _context: &Arc<CompilationContext>,
    ) -> farmfe_core::error::Result<Option<()>> {
      self.0.fetch_add(1, Ordering::SeqCst);
      let added = params
        .paths
        .iter()
        .filter_map(|(path, kind)| matches!(kind, UpdateType::Added).then_some(path.clone()))
        .collect::<Vec<_>>();
      assert_eq!(added.len(), 1, "one original membership notification");
      for path in added {
        params.paths.push((path, UpdateType::Updated));
      }
      Ok(None)
    }
  }
  let calls = Arc::new(AtomicUsize::new(0));
  let context = Arc::new(
    CompilationContext::new(
      Config {
        root: std::env::temp_dir().to_string_lossy().into_owned(),
        ..Default::default()
      },
      vec![Arc::new(Observer(calls.clone()))],
    )
    .unwrap(),
  );
  let source = ModuleId::new("new-member.ts", "", &context.config.root);
  let path = source.resolved_path(&context.config.root);
  let css = ModuleId::new("styles.css", "", &context.config.root);
  context
    .module_graph
    .write()
    .add_module(Module::new(css.clone()));
  context.add_watch_files(css.clone(), vec![source]).unwrap();
  let paths = handle_update_modules(
    vec![(path.clone(), UpdateType::Added)],
    &[],
    &context,
    &mut UpdateResult::default(),
  )
  .unwrap();
  assert_eq!(calls.load(Ordering::SeqCst), 1);
  assert!(
    paths
      .iter()
      .any(|(file, kind)| file == &path && matches!(kind, UpdateType::Updated))
  );
  assert!(paths.iter().any(
    |(file, kind)| file == &css.resolved_path(&context.config.root)
      && matches!(kind, UpdateType::Updated)
  ));
}

#[test]
fn plugin_replaced_and_appended_membership_events_are_preserved() {
  use farmfe_core::plugin::Plugin;
  struct ReplaceNotifications;
  impl Plugin for ReplaceNotifications {
    fn name(&self) -> &str {
      "replace-membership-notifications"
    }
    fn update_modules(
      &self,
      params: &mut PluginUpdateModulesHookParam,
      _context: &Arc<CompilationContext>,
    ) -> farmfe_core::error::Result<Option<()>> {
      let original = params
        .paths
        .iter()
        .find(|(_, kind)| matches!(kind, UpdateType::Added))
        .unwrap()
        .0
        .clone();
      params.paths.retain(|(path, _)| path != &original);
      // These now belong to the plugin, even though the first equals the input notification.
      params.paths.push((original.clone(), UpdateType::Added));
      params.paths.push((original.clone(), UpdateType::Removed));
      params.paths.push((original.clone(), UpdateType::Added));
      params.paths.push((original, UpdateType::Updated));
      Ok(None)
    }
  }
  let context = Arc::new(
    CompilationContext::new(
      Config {
        root: std::env::temp_dir().to_string_lossy().into_owned(),
        ..Default::default()
      },
      vec![Arc::new(ReplaceNotifications)],
    )
    .unwrap(),
  );
  let source = ModuleId::new("new-member.ts", "", &context.config.root);
  let path = source.resolved_path(&context.config.root);
  let css = ModuleId::new("styles.css", "", &context.config.root);
  context
    .module_graph
    .write()
    .add_module(Module::new(css.clone()));
  context.add_watch_files(css, vec![source]).unwrap();
  let paths = handle_update_modules(
    vec![(path.clone(), UpdateType::Added)],
    &[],
    &context,
    &mut UpdateResult::default(),
  )
  .unwrap();
  let returned = paths
    .iter()
    .filter(|(file, _)| file == &path)
    .collect::<Vec<_>>();
  assert_eq!(
    returned
      .iter()
      .filter(|(_, kind)| matches!(kind, UpdateType::Added))
      .count(),
    2
  );
  assert_eq!(
    returned
      .iter()
      .filter(|(_, kind)| matches!(kind, UpdateType::Removed))
      .count(),
    1
  );
  assert_eq!(
    returned
      .iter()
      .filter(|(_, kind)| matches!(kind, UpdateType::Updated))
      .count(),
    1
  );
}
