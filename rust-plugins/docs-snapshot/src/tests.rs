use super::*;
use farmfe_core::{
  config::{TargetEnv, persistent_cache::PersistentCacheConfig},
  module::ModuleId,
  resource::Resource,
};
use farmfe_docs_config::parse_config;
use serde_json::json;

fn text(path: &Path) -> String {
  path.to_str().unwrap().replace('\\', "/")
}

struct Fixture {
  _directory: tempfile::TempDir,
  config: Config,
  options: String,
  entry: String,
  dependency: String,
  seed: PathBuf,
  receipt: PathBuf,
}

impl Fixture {
  fn new() -> Self {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir(root.join("docs")).unwrap();
    fs::create_dir_all(root.join("node_modules/watched-package")).unwrap();
    fs::write(root.join("docs/index.md"), "# Hook fixture\n").unwrap();
    fs::write(root.join("entry.js"), "export default 1;").unwrap();
    fs::write(
      root.join("node_modules/watched-package/data.txt"),
      "original",
    )
    .unwrap();
    let site = parse_config(&json!({"title":"Hooks","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}}).to_string(), root).unwrap();
    let plan = farmfe_docs_plan::prepare(&site).unwrap();
    let plan_path = root.join("plan.json");
    fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let seed = root.join("seed.json");
    let receipt = root.join("receipt.json");
    let mut config = Config::default();
    config.root = text(root);
    config.output.target_env = TargetEnv::Node;
    config.persistent_cache = Box::new(PersistentCacheConfig::Bool(false));
    let options = json!({"mode":"capture","planPath":text(&plan_path),"receiptPath":text(&receipt),"seedPath":text(&seed),"target":"server","outputPath":text(&root.join("build"))}).to_string();
    let entry = text(&root.join("entry.js"));
    let dependency = text(&root.join("node_modules/watched-package"));
    Self {
      _directory: directory,
      config,
      options,
      entry,
      dependency,
      seed,
      receipt,
    }
  }

  fn snapshot(&self) -> Arc<FarmPluginDocsSnapshot> {
    Arc::new(FarmPluginDocsSnapshot::new(
      &self.config,
      self.options.clone(),
    ))
  }

  fn context(&self, plugins: Vec<Arc<dyn Plugin>>) -> Arc<CompilationContext> {
    Arc::new(CompilationContext::new(self.config.clone(), plugins).unwrap())
  }

  fn module(&self) -> Module {
    Module::new(ModuleId::new(&self.entry, "", &self.config.root))
  }

  fn load(
    &self,
    snapshot: &FarmPluginDocsSnapshot,
    context: &Arc<CompilationContext>,
    module: &Module,
  ) {
    snapshot
      .load(
        &PluginLoadHookParam {
          module_id: module.id.to_string(),
          resolved_path: &self.entry,
          query: vec![],
          meta: Default::default(),
        },
        context,
        &PluginHookContext::default(),
      )
      .unwrap();
  }

  fn watch(&self, context: &Arc<CompilationContext>, module: &Module) {
    context
      .add_watch_files(
        module.id.clone(),
        vec![ModuleId::new(&self.dependency, "", &self.config.root)],
      )
      .unwrap();
  }
}

fn freeze(
  snapshot: &FarmPluginDocsSnapshot,
  context: &Arc<CompilationContext>,
  module: &mut Module,
) -> Result<Option<()>> {
  snapshot.freeze_module(
    &mut PluginFreezeModuleHookParam {
      module,
      resolved_deps: vec![],
    },
    context,
  )
}

fn graph(module: &Module) -> ModuleGraph {
  let mut graph = ModuleGraph::new();
  graph.add_module(module.clone());
  graph
}

fn static_fixture() -> (Fixture, PathBuf) {
  let fixture = Fixture::new();
  let root = fixture._directory.path();
  let directory = root.join("static");
  fs::create_dir_all(directory.join("empty")).unwrap();
  fs::write(directory.join("asset.txt"), "original").unwrap();
  fs::write(directory.join("second.txt"), "second original").unwrap();
  let site = parse_config(&json!({"title":"Hooks","url":"https://example.com","staticDirs":[{"dir":"static"}],"docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}}).to_string(), root).unwrap();
  let plan = farmfe_docs_plan::prepare(&site).unwrap();
  let Options::Capture { plan_path, .. } = serde_json::from_str(&fixture.options).unwrap() else {
    unreachable!()
  };
  fs::write(plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
  (fixture, directory)
}

fn static_reader(
  context: &Arc<CompilationContext>,
  snapshot: &FarmPluginDocsSnapshot,
  directory: &Path,
) -> Module {
  let module = Module::new("virtual:farm-docs:application".into());
  snapshot
    .load(
      &PluginLoadHookParam {
        module_id: module.id.to_string(),
        resolved_path: "virtual:farm-docs:application",
        query: vec![],
        meta: Default::default(),
      },
      context,
      &PluginHookContext::default(),
    )
    .unwrap();
  context
    .add_watch_files(
      module.id.clone(),
      [
        directory.to_path_buf(),
        directory.join("asset.txt"),
        directory.join("second.txt"),
      ]
      .iter()
      .map(|path| ModuleId::new(&text(path), "", &context.config.root))
      .collect(),
    )
    .unwrap();
  module
}

#[test]
fn declared_static_root_is_observed_before_virtual_application_without_retry() {
  let (fixture, directory) = static_fixture();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = static_reader(&context, &snapshot, &directory);
  freeze(&snapshot, &context, &mut module).unwrap();
  snapshot
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap();
  assert!(!fixture.seed.exists());
  let state = snapshot.state.lock().unwrap();
  let records = state.modules.get(&module.id.to_string()).unwrap();
  assert_eq!(
    records.len(),
    3,
    "real application watches each file and the directory"
  );
  assert_eq!(records.iter().filter(|record| record.directory).count(), 1);
  assert!(
    records
      .iter()
      .all(|record| state.initial_sources.contains(&record.canonical_path))
  );
}

#[test]
fn declared_static_changes_after_prior_observation_are_rejected() {
  for change in [
    "content",
    "add-file",
    "remove-file",
    "add-empty-directory",
    "remove-empty-directory",
    "file-to-directory",
  ] {
    let (fixture, directory) = static_fixture();
    let context = fixture.context(vec![]);
    let snapshot = fixture.snapshot();
    snapshot.build_start(&context).unwrap();
    let mut module = static_reader(&context, &snapshot, &directory);
    match change {
      "content" => fs::write(directory.join("asset.txt"), "modified").unwrap(),
      "add-file" => fs::write(directory.join("added.txt"), "new").unwrap(),
      "remove-file" => fs::remove_file(directory.join("asset.txt")).unwrap(),
      "add-empty-directory" => fs::create_dir(directory.join("new-empty")).unwrap(),
      "remove-empty-directory" => fs::remove_dir(directory.join("empty")).unwrap(),
      "file-to-directory" => {
        fs::remove_file(directory.join("asset.txt")).unwrap();
        fs::create_dir(directory.join("asset.txt")).unwrap();
      }
      _ => unreachable!(),
    }
    assert!(
      freeze(&snapshot, &context, &mut module).is_err(),
      "accepted {change}"
    );
  }
}

#[test]
fn later_observation_cannot_replace_first_static_fact() {
  let (fixture, directory) = static_fixture();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  // Config validation canonicalizes static roots; Windows TempDir may use a
  // short-name spelling while the prepared root uses its expanded identity.
  let key = FarmPluginDocsSnapshot::source_key(&directory.canonicalize().unwrap()).unwrap();
  let first = snapshot
    .state
    .lock()
    .unwrap()
    .known
    .get(&key)
    .unwrap()
    .clone();
  fs::write(directory.join("asset.txt"), "modified").unwrap();
  let mut state = snapshot.state.lock().unwrap();
  let later = FarmPluginDocsSnapshot::observe_input(&directory, &state).unwrap();
  assert_eq!(first.canonical_path, later.canonical_path);
  assert_ne!(first.hash, later.hash);
  assert!(FarmPluginDocsSnapshot::remember_input(later, &mut state).is_err());
  assert_eq!(state.known.get(&key), Some(&first));
}

#[test]
fn source_lookup_key_matches_receipt_windows_path_spelling() {
  assert_eq!(
    FarmPluginDocsSnapshot::source_key(Path::new(r"C:\site\static")).unwrap(),
    "C:/site/static"
  );
  assert_eq!(
    FarmPluginDocsSnapshot::source_key(Path::new(r"\\?\C:\site\static")).unwrap(),
    "C:/site/static"
  );
  assert_eq!(
    FarmPluginDocsSnapshot::source_key(Path::new(r"\\?\UNC\server\share\static")).unwrap(),
    "//server/share/static"
  );
}

#[test]
fn changed_prepared_static_input_is_not_verified_again_at_build_start() {
  let (fixture, directory) = static_fixture();
  fs::write(directory.join("asset.txt"), "modified after prepare").unwrap();
  let context = fixture.context(vec![]);
  assert!(fixture.snapshot().build_start(&context).is_err());
}

#[test]
fn undeclared_directory_still_requires_discovery_with_declared_static_root() {
  let (fixture, directory) = static_fixture();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = static_reader(&context, &snapshot, &directory);
  fixture.watch(&context, &module);
  freeze(&snapshot, &context, &mut module).unwrap();
  let error = snapshot
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap_err();
  assert!(error.to_string().contains("DOCS_INPUT_DISCOVERY"));
  assert!(fixture.seed.exists());
}

#[test]
fn undeclared_file_still_requires_discovery_with_declared_static_files() {
  let (fixture, directory) = static_fixture();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = static_reader(&context, &snapshot, &directory);
  let file = Path::new(&fixture.dependency).join("data.txt");
  context
    .add_watch_files(
      module.id.clone(),
      vec![ModuleId::new(&text(&file), "", &context.config.root)],
    )
    .unwrap();
  freeze(&snapshot, &context, &mut module).unwrap();
  let error = snapshot
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap_err();
  assert!(error.to_string().contains("DOCS_INPUT_DISCOVERY"));
  assert!(fixture.seed.exists());
}

#[test]
fn declared_static_watch_is_rechecked_at_finish() {
  let (fixture, directory) = static_fixture();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = static_reader(&context, &snapshot, &directory);
  freeze(&snapshot, &context, &mut module).unwrap();
  snapshot
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap();
  context.module_graph.write().add_module(module);
  fs::write(directory.join("asset.txt"), "modified after graph").unwrap();
  let error = snapshot.finish(&Stats::default(), &context).unwrap_err();
  assert!(
    error.to_string().contains("source snapshot changed"),
    "{error}"
  );
}

#[cfg(unix)]
#[test]
fn declared_static_root_canonical_redirection_is_rejected() {
  let (fixture, directory) = static_fixture();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = static_reader(&context, &snapshot, &directory);
  let moved = fixture._directory.path().join("moved-static");
  fs::rename(&directory, &moved).unwrap();
  std::os::unix::fs::symlink(&moved, &directory).unwrap();
  assert!(freeze(&snapshot, &context, &mut module).is_err());
}

#[cfg(unix)]
#[test]
fn declared_static_symlink_after_prior_observation_is_rejected() {
  let (fixture, directory) = static_fixture();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = static_reader(&context, &snapshot, &directory);
  std::os::unix::fs::symlink(directory.join("asset.txt"), directory.join("linked.txt")).unwrap();
  assert!(freeze(&snapshot, &context, &mut module).is_err());
}

#[test]
fn controlled_file_read_accepts_new_watch_without_discovery_but_rechecks_bytes() {
  let fixture = Fixture::new();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&snapshot, &context, &module);
  let file = Path::new(&fixture.dependency).join("data.txt");
  let id = module.id.to_string();
  context.source_observations.read_file(&id, &file).unwrap();
  context
    .add_watch_files(
      module.id.clone(),
      vec![ModuleId::new(&text(&file), "", &fixture.config.root)],
    )
    .unwrap();
  freeze(&snapshot, &context, &mut module).unwrap();
  snapshot
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap();
  assert!(
    !fixture.seed.exists(),
    "controlled read must not request a full retry"
  );
  fs::write(&file, "changed after actual read").unwrap();
  assert!(context.source_observations.seal().is_err());
}

#[test]
fn derived_watch_requires_declared_producer_own_source_identity() {
  let fixture = Fixture::new();
  let context = fixture.context(vec![]);
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut reader = fixture.module();
  fixture.load(&snapshot, &context, &reader);
  let producer_file = Path::new(&fixture.dependency).join("data.txt");
  let producer = ModuleId::new(&text(&producer_file), "", &fixture.config.root);
  snapshot
    .load(
      &PluginLoadHookParam {
        module_id: producer.to_string(),
        resolved_path: &text(&producer_file),
        query: vec![],
        meta: Default::default(),
      },
      &context,
      &PluginHookContext::default(),
    )
    .unwrap();
  context
    .source_observations
    .consume_derived(
      context.source_observations.generation(),
      &reader.id.to_string(),
      &producer.to_string(),
    )
    .unwrap();
  context
    .add_watch_files(reader.id.clone(), vec![producer.clone()])
    .unwrap();
  freeze(&snapshot, &context, &mut reader).unwrap();
  snapshot
    .module_graph_build_end(&mut graph(&reader), &context)
    .unwrap();
  assert!(!fixture.seed.exists());
}

#[test]
fn cached_negative_observation_change_invalidates_without_poisoning_new_generation() {
  let fixture = Fixture::new();
  let context = fixture.context(vec![]);
  let first = fixture.snapshot();
  first.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&first, &context, &module);
  let missing = Path::new(&fixture.config.root).join("optional.config");
  context
    .source_observations
    .metadata(&module.id.to_string(), &missing)
    .unwrap();
  freeze(&first, &context, &mut module).unwrap();
  let cache = first.write_plugin_cache(&context).unwrap().unwrap();
  fs::write(&missing, "now present").unwrap();
  let next_context = fixture.context(vec![]);
  let restored = fixture.snapshot();
  restored.plugin_cache_loaded(&cache, &next_context).unwrap();
  restored.build_start(&next_context).unwrap();
  assert_eq!(
    restored
      .handle_persistent_cached_module(&module, &next_context)
      .unwrap(),
    Some(true)
  );
  assert!(
    next_context
      .source_observations
      .records()
      .unwrap()
      .is_empty()
  );
  fixture.load(&restored, &next_context, &module);
  freeze(&restored, &next_context, &mut module).unwrap();
}

fn disk_cache_fixture() -> (Fixture, Arc<CompilationContext>, PathBuf, PathBuf) {
  let mut fixture = Fixture::new();
  let base = PathBuf::from(&fixture.config.root).join("cache-origin");
  let PersistentCacheConfig::Obj(mut cache) =
    PersistentCacheConfig::get_default_config(&fixture.config.root)
  else {
    unreachable!()
  };
  cache.cache_dir = Some(text(&base));
  cache.namespace = "physical-cache-test".into();
  fixture.config.persistent_cache = Box::new(PersistentCacheConfig::Obj(cache));
  let context = fixture.context(vec![]);
  context
    .cache_manager
    .plugin_cache
    .set_cache("probe", b"first".to_vec());
  context.cache_manager.write_cache();
  fn manifests(root: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
      let entry = entry.unwrap();
      if entry.file_type().unwrap().is_dir() {
        manifests(&entry.path(), found);
      } else if entry.file_name() == "farm-cache.json" {
        found.push(entry.path());
      }
    }
  }
  let mut found = vec![];
  manifests(&base, &mut found);
  assert_eq!(found.len(), 1, "locate the real CacheManager output");
  let actual = found.pop().unwrap();
  assert_eq!(
    context
      .cache_manager
      .disk_cache_directory()
      .unwrap()
      .canonicalize()
      .unwrap(),
    actual.parent().unwrap().canonicalize().unwrap()
  );
  (fixture, context, base, actual)
}

fn assert_actual_cache_watch_rejected(directory: bool) {
  let (fixture, context, _, manifest) = disk_cache_fixture();
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&snapshot, &context, &module);
  let watched = if directory {
    manifest.parent().unwrap()
  } else {
    &manifest
  };
  context
    .add_watch_files(
      module.id.clone(),
      vec![ModuleId::new(&text(watched), "", &context.config.root)],
    )
    .unwrap();
  let error = freeze(&snapshot, &context, &mut module).unwrap_err();
  assert!(
    error
      .to_string()
      .contains("generated output or compiler cache")
  );
}

#[test]
fn actual_disk_cache_file_cannot_be_a_watched_source() {
  assert_actual_cache_watch_rejected(false);
}

#[test]
fn actual_disk_cache_directory_cannot_be_a_watched_source() {
  assert_actual_cache_watch_rejected(true);
}

#[test]
fn ancestor_watch_excludes_actual_disk_cache_writes() {
  let (fixture, context, base, _) = disk_cache_fixture();
  let snapshot = fixture.snapshot();
  snapshot.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&snapshot, &context, &module);
  context
    .add_watch_files(
      module.id.clone(),
      vec![ModuleId::new(&text(&base), "", &context.config.root)],
    )
    .unwrap();
  freeze(&snapshot, &context, &mut module).unwrap();
  assert!(
    snapshot
      .module_graph_build_end(&mut graph(&module), &context)
      .unwrap_err()
      .to_string()
      .contains("DOCS_INPUT_DISCOVERY")
  );
  let seed: Seed = read_json(&fixture.seed).unwrap();
  let source = seed
    .sources
    .iter()
    .find(|source| Path::new(&source.path).canonicalize().unwrap() == base.canonicalize().unwrap())
    .unwrap();
  context
    .cache_manager
    .plugin_cache
    .set_cache("probe", b"changed".to_vec());
  context.cache_manager.write_cache();
  verify_source(source).expect("compiler cache writes must not change an ancestor source snapshot");
}

#[test]
fn corrupt_snapshot_plugin_cache_requires_recompilation() {
  let fixture = Fixture::new();
  let context = fixture.context(vec![]);
  let baseline = fixture.snapshot();
  baseline.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&baseline, &context, &module);
  freeze(&baseline, &context, &mut module).unwrap();
  let bytes = baseline.write_plugin_cache(&context).unwrap().unwrap();
  let mut version: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  version["schemaVersion"] = json!(999);
  let mut unknown: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  unknown["unknown"] = json!(true);
  for bytes in [
    b"{".to_vec(),
    serde_json::to_vec(&version).unwrap(),
    serde_json::to_vec(&unknown).unwrap(),
  ] {
    let snapshot = fixture.snapshot();
    snapshot.plugin_cache_loaded(&bytes, &context).unwrap();
    snapshot.build_start(&context).unwrap();
    assert_eq!(
      snapshot
        .handle_persistent_cached_module(&module, &context)
        .unwrap(),
      Some(true)
    );
    fixture.load(&snapshot, &context, &module);
    freeze(&snapshot, &context, &mut module).unwrap();
    snapshot
      .module_graph_build_end(&mut graph(&module), &context)
      .unwrap();
    let repaired: Cache =
      serde_json::from_slice(&snapshot.write_plugin_cache(&context).unwrap().unwrap()).unwrap();
    assert_eq!(repaired.schema_version, 2);
    assert!(repaired.modules.contains_key(&module.id.to_string()));
  }
}

#[test]
fn explicit_dependency_directory_survives_discovery_retry_and_invalidates_cached_module() {
  let fixture = Fixture::new();
  let first = fixture.snapshot();
  let context = fixture.context(vec![]);
  first.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&first, &context, &module);
  fixture.watch(&context, &module);
  freeze(&first, &context, &mut module).unwrap();
  let error = first
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap_err();
  assert!(error.to_string().contains("DOCS_INPUT_DISCOVERY"));

  let retry = fixture.snapshot();
  retry.build_start(&context).unwrap();
  fixture.load(&retry, &context, &module);
  freeze(&retry, &context, &mut module).unwrap();
  retry
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap();
  let cached = retry.write_plugin_cache(&context).unwrap().unwrap();
  fs::remove_file(&fixture.seed).unwrap();
  let restored = fixture.snapshot();
  restored.plugin_cache_loaded(&cached, &context).unwrap();
  restored.build_start(&context).unwrap();
  assert_eq!(
    restored
      .handle_persistent_cached_module(&module, &context)
      .unwrap(),
    None
  );
  fs::write(Path::new(&fixture.dependency).join("data.txt"), "modified").unwrap();
  assert_eq!(
    restored
      .handle_persistent_cached_module(&module, &context)
      .unwrap(),
    Some(true)
  );
  assert!(freeze(&restored, &context, &mut module).is_err());
}

struct LateDirectoryWatch {
  directory: String,
}
impl Plugin for LateDirectoryWatch {
  fn name(&self) -> &str {
    "LateDirectoryWatch"
  }
  fn freeze_module(
    &self,
    param: &mut PluginFreezeModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    fs::read(Path::new(&self.directory).join("data.txt")).map_err(diagnostic)?;
    context.add_watch_files(
      param.module.id.clone(),
      vec![ModuleId::new(&self.directory, "", &context.config.root)],
    )?;
    Ok(None)
  }
}

#[test]
fn later_rust_freeze_watch_is_discovered_and_included_after_observed_retry() {
  let fixture = Fixture::new();
  let snapshot = fixture.snapshot();
  let context = fixture.context(vec![
    snapshot.clone(),
    Arc::new(LateDirectoryWatch {
      directory: fixture.dependency.clone(),
    }),
  ]);
  snapshot.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&snapshot, &context, &module);
  context
    .plugin_driver
    .freeze_module(
      &mut PluginFreezeModuleHookParam {
        module: &mut module,
        resolved_deps: vec![],
      },
      &context,
    )
    .unwrap();
  let error = snapshot
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap_err();
  assert!(error.to_string().contains("DOCS_INPUT_DISCOVERY"));

  let retry = fixture.snapshot();
  let context = fixture.context(vec![
    retry.clone(),
    Arc::new(LateDirectoryWatch {
      directory: fixture.dependency.clone(),
    }),
  ]);
  retry.build_start(&context).unwrap();
  fixture.load(&retry, &context, &module);
  context
    .plugin_driver
    .freeze_module(
      &mut PluginFreezeModuleHookParam {
        module: &mut module,
        resolved_deps: vec![],
      },
      &context,
    )
    .unwrap();
  retry
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap();
  let mut resource = Resource::default();
  resource.name = "entry.js".into();
  resource.bytes = b"export default 1;".to_vec();
  context
    .resources_map
    .lock()
    .insert(resource.name.clone(), resource);
  retry.finish(&context.stats, &context).unwrap();
  let receipt: Receipt = read_json(&fixture.receipt).unwrap();
  let expected_directory = fs::canonicalize(&fixture.dependency).unwrap();
  assert!(
    receipt.sources.iter().any(|source| source.directory
      && fs::canonicalize(&source.path).ok().as_ref() == Some(&expected_directory)),
    "missing watched directory in {:#?}",
    receipt.sources
  );
  fs::write(
    Path::new(&fixture.dependency).join("data.txt"),
    "changed after compilation",
  )
  .unwrap();
  assert!(
    receipt
      .validate(
        &receipt.plan_snapshot_id,
        Target::Server,
        &context.config.root
      )
      .is_err()
  );
}

#[test]
fn watch_source_loaded_after_its_reader_still_requires_observed_retry() {
  let fixture = Fixture::new();
  let snapshot = fixture.snapshot();
  let context = fixture.context(vec![]);
  snapshot.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&snapshot, &context, &module);
  let watched = text(&Path::new(&fixture.dependency).join("data.txt"));
  let watched_id = ModuleId::new(&watched, "", &context.config.root);
  let resolved_watch = watched_id.resolved_path(&context.config.root);
  snapshot
    .load(
      &PluginLoadHookParam {
        module_id: watched_id.to_string(),
        resolved_path: &resolved_watch,
        query: vec![],
        meta: Default::default(),
      },
      &context,
      &PluginHookContext::default(),
    )
    .unwrap();
  context
    .add_watch_files(module.id.clone(), vec![watched_id])
    .unwrap();
  freeze(&snapshot, &context, &mut module).unwrap();
  let error = snapshot
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap_err();
  assert!(error.to_string().contains("DOCS_INPUT_DISCOVERY"));
}

#[test]
fn shared_new_watch_source_invalidates_every_cached_reader_on_retry() {
  let fixture = Fixture::new();
  let entry_two = Path::new(&fixture.entry).with_file_name("second.js");
  fs::write(&entry_two, "export default 2;").unwrap();
  let mut first = fixture.module();
  let mut second = Module::new(ModuleId::new(&text(&entry_two), "", &fixture.config.root));
  let baseline = fixture.snapshot();
  let context = fixture.context(vec![]);
  baseline.build_start(&context).unwrap();
  fixture.load(&baseline, &context, &first);
  baseline
    .load(
      &PluginLoadHookParam {
        module_id: second.id.to_string(),
        resolved_path: &text(&entry_two),
        query: vec![],
        meta: Default::default(),
      },
      &context,
      &PluginHookContext::default(),
    )
    .unwrap();
  freeze(&baseline, &context, &mut first).unwrap();
  freeze(&baseline, &context, &mut second).unwrap();
  let cached = baseline.write_plugin_cache(&context).unwrap().unwrap();

  let discovery = fixture.snapshot();
  discovery.plugin_cache_loaded(&cached, &context).unwrap();
  discovery.build_start(&context).unwrap();
  assert_eq!(
    discovery
      .handle_persistent_cached_module(&first, &context)
      .unwrap(),
    None
  );
  assert_eq!(
    discovery
      .handle_persistent_cached_module(&second, &context)
      .unwrap(),
    None
  );
  fixture.watch(&context, &first);
  fixture.watch(&context, &second);
  freeze(&discovery, &context, &mut first).unwrap();
  freeze(&discovery, &context, &mut second).unwrap();
  let mut modules = graph(&first);
  modules.add_module(second.clone());
  assert!(
    discovery
      .module_graph_build_end(&mut modules, &context)
      .unwrap_err()
      .to_string()
      .contains("DOCS_INPUT_DISCOVERY")
  );
  let retry = fixture.snapshot();
  retry.plugin_cache_loaded(&cached, &context).unwrap();
  retry.build_start(&context).unwrap();
  assert_eq!(
    retry
      .handle_persistent_cached_module(&first, &context)
      .unwrap(),
    Some(true)
  );
  assert_eq!(
    retry
      .handle_persistent_cached_module(&second, &context)
      .unwrap(),
    Some(true)
  );
}

#[test]
fn watching_generated_output_is_rejected_instead_of_observing_an_empty_tree() {
  let mut fixture = Fixture::new();
  let output = Path::new(&fixture.entry).with_file_name("build");
  fs::create_dir(&output).unwrap();
  fs::write(output.join("generated.js"), "generated").unwrap();
  fixture.dependency = text(&output);
  let snapshot = fixture.snapshot();
  let context = fixture.context(vec![]);
  snapshot.build_start(&context).unwrap();
  let mut module = fixture.module();
  fixture.load(&snapshot, &context, &module);
  fixture.watch(&context, &module);
  assert!(
    freeze(&snapshot, &context, &mut module)
      .unwrap_err()
      .to_string()
      .contains("generated output or compiler cache")
  );
}
