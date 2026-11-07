use farmfe_compiler::Compiler;
use farmfe_core::{
  HashMap,
  config::{
    Config, Mode, SourcemapConfig, bool_or_obj::BoolOrObj, persistent_cache::PersistentCacheConfig,
    preset_env::PresetEnvConfig,
  },
  context::CompilationContext,
  error::Result,
  module::ModuleId,
  plugin::{Plugin, PluginProcessModuleHookParam, UpdateType},
};
use std::{
  path::PathBuf,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};
mod common;

struct CountImporters(Arc<AtomicUsize>);
impl Plugin for CountImporters {
  fn name(&self) -> &str {
    "count-glob-importer-transforms"
  }
  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    _: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if ["eager.js", "lazy.js"].contains(&param.module_id.to_string().as_str()) {
      self.0.fetch_add(1, Ordering::SeqCst);
    }
    Ok(None)
  }
}

fn verify_membership_updates(disk_cache: bool) {
  static NEXT: AtomicUsize = AtomicUsize::new(0);
  let root = std::env::temp_dir().join(format!(
    "farm-glob-update-{}-{}",
    std::process::id(),
    NEXT.fetch_add(1, Ordering::SeqCst)
  ));
  std::fs::create_dir_all(root.join("members")).unwrap();
  let root = farmfe_toolkit::resolve::follow_symlinks(root);
  std::fs::write(root.join("index.js"), "import './styles.css'; import './eager.js'; import './lazy.js'; if(import.meta.hot) import.meta.hot.accept();").unwrap();
  std::fs::write(root.join("styles.css"), ".probe{color:red}").unwrap();
  for (name, eager) in [("eager.js", true), ("lazy.js", false)] {
    std::fs::write(root.join(name), format!("export const members=import.meta.glob(['./members/*.js','!./members/excluded*.js'],{{eager:{eager}}}); if(import.meta.hot) import.meta.hot.accept();")).unwrap();
  }
  let mut config = Config {
    root: root.to_string_lossy().into_owned(),
    input: HashMap::from_iter([("index".into(), "./index.js".into())]),
    runtime: common::generate_runtime(PathBuf::from(env!("CARGO_MANIFEST_DIR")), false),
    mode: Mode::Development,
    lazy_compilation: false,
    progress: false,
    sourcemap: Box::new(SourcemapConfig::Bool(false)),
    minify: Box::new(BoolOrObj::Bool(false)),
    tree_shaking: Box::new(BoolOrObj::Bool(false)),
    preset_env: Box::new(PresetEnvConfig::Bool(false)),
    persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
    ..Default::default()
  };
  if disk_cache {
    let mut cache = PersistentCacheConfig::get_default_config(&config.root).as_obj(&config.root);
    cache.memory = false;
    cache.cache_dir = Some(root.join(".cache").to_string_lossy().into_owned());
    cache.module_cache_key_strategy.timestamp = false;
    cache.module_cache_key_strategy.hash = true;
    *config.persistent_cache = PersistentCacheConfig::Obj(cache);
  }
  let count = Arc::new(AtomicUsize::new(0));
  let create = || {
    Compiler::new(
      config.clone(),
      vec![Arc::new(CountImporters(count.clone()))],
    )
    .unwrap()
  };
  let mut compiler = create();
  compiler.compile().unwrap();
  compiler.flush_cache().unwrap();
  assert_eq!(count.load(Ordering::SeqCst), 2);
  if disk_cache {
    drop(compiler);
    compiler = create();
    compiler.compile().unwrap();
    compiler.flush_cache().unwrap();
    assert_eq!(
      count.load(Ordering::SeqCst),
      2,
      "both glob importers must be restored from disk cache"
    );
  }
  let css = ModuleId::new("styles.css", "", &config.root);
  for (name, kind, exists, matches) in [
    ("first.js", UpdateType::Added, true, true),
    ("first.js", UpdateType::Removed, false, true),
    ("first.js", UpdateType::Added, true, true),
    ("first.js", UpdateType::Removed, false, true),
    ("fresh.js", UpdateType::Added, true, true),
    ("excluded-file.js", UpdateType::Added, true, false),
    ("unrelated.txt", UpdateType::Added, true, false),
  ] {
    let path = root.join("members").join(name);
    if exists {
      std::fs::write(&path, "export default 'member';").unwrap();
    } else {
      std::fs::remove_file(&path).unwrap();
    }
    let id = ModuleId::new(&path.to_string_lossy(), "", &config.root);
    // Reproduce Tailwind's independent watch registration without requiring a native dylib in this test.
    compiler
      .context()
      .add_watch_files(css.clone(), vec![id.clone()])
      .unwrap();
    let update = compiler
      .update(
        vec![(path.to_string_lossy().into_owned(), kind)],
        || {},
        true,
        true,
      )
      .unwrap();
    for importer in ["eager.js", "lazy.js"] {
      assert_eq!(
        update.updated_module_ids.contains(&importer.into()),
        matches,
        "{name}: {importer}"
      );
    }
    assert_eq!(
      compiler.context().module_graph.read().has_module(&id),
      exists && matches,
      "{name} graph membership"
    );
    compiler.flush_cache().unwrap();
  }
  drop(compiler);
  std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn eager_and_lazy_globs_add_delete_restore_from_empty() {
  verify_membership_updates(false);
}
#[test]
fn cached_eager_and_lazy_globs_keep_membership_updates() {
  verify_membership_updates(true);
}
