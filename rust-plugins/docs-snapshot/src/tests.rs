use super::*;
use farmfe_core::{
  config::{persistent_cache::PersistentCacheConfig, TargetEnv},
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
  assert!(error
    .to_string()
    .contains("generated output or compiler cache"));
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
  assert!(snapshot
    .module_graph_build_end(&mut graph(&module), &context)
    .unwrap_err()
    .to_string()
    .contains("DOCS_INPUT_DISCOVERY"));
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
    assert_eq!(repaired.schema_version, 1);
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
  assert!(receipt
    .validate(
      &receipt.plan_snapshot_id,
      Target::Server,
      &context.config.root
    )
    .is_err());
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
  assert!(discovery
    .module_graph_build_end(&mut modules, &context)
    .unwrap_err()
    .to_string()
    .contains("DOCS_INPUT_DISCOVERY"));
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
  assert!(freeze(&snapshot, &context, &mut module)
    .unwrap_err()
    .to_string()
    .contains("generated output or compiler cache"));
}
