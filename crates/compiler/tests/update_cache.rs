use std::{
  path::PathBuf,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};

use farmfe_compiler::Compiler;
use farmfe_core::{
  HashMap,
  config::{
    Config, Mode, SourcemapConfig, bool_or_obj::BoolOrObj, persistent_cache::PersistentCacheConfig,
    preset_env::PresetEnvConfig,
  },
  context::CompilationContext,
  error::Result,
  module::ModuleType,
  plugin::{Plugin, PluginProcessModuleHookParam, UpdateType},
  rkyv,
};
use farmfe_toolkit::css::{codegen_css_stylesheet, parse_css_stylesheet};

mod common;

struct GeneratedStyles(Arc<AtomicBool>);

impl Plugin for GeneratedStyles {
  fn name(&self) -> &str {
    "GeneratedStyles"
  }

  fn process_module(
    &self,
    param: &mut PluginProcessModuleHookParam,
    _context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if *param.module_type == ModuleType::Css {
      let generation = if self.0.load(Ordering::SeqCst) {
        "new"
      } else {
        "old"
      };
      param.meta.as_css_mut().ast = parse_css_stylesheet(
        "styles.css",
        Arc::new(format!(".{generation}{{color:red}}")),
      )
      .unwrap()
      .ast;
    }
    Ok(None)
  }
}

fn assert_explicit_update_rebuilds_generated_styles(timestamp: bool) {
  let root = std::env::temp_dir().join(format!(
    "farm-explicit-cache-{}-{}-{timestamp}",
    std::process::id(),
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos(),
  ));
  std::fs::create_dir(&root).unwrap();
  // Match resolver identity when the OS temp directory has symlinked ancestors.
  let root = farmfe_toolkit::resolve::follow_symlinks(root);
  let stylesheet = root.join("styles.css");
  std::fs::write(&stylesheet, ".source{color:red}").unwrap();
  std::fs::write(
    root.join("index.js"),
    "import './styles.css'; import './sibling.js'; if(import.meta.hot) import.meta.hot.accept();",
  )
  .unwrap();
  std::fs::write(root.join("sibling.js"), "console.log('unchanged sibling');").unwrap();
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
    ..Default::default()
  };
  let mut cache = PersistentCacheConfig::get_default_config(&config.root).as_obj(&config.root);
  cache.memory = true;
  cache.module_cache_key_strategy.timestamp = timestamp;
  cache.module_cache_key_strategy.hash = true;
  config.persistent_cache = Box::new(PersistentCacheConfig::Obj(cache));
  let generation = Arc::new(AtomicBool::new(false));
  let compiler =
    Compiler::new(config, vec![Arc::new(GeneratedStyles(generation.clone()))]).unwrap();
  compiler.compile().unwrap();
  let context = compiler.context();
  let css_id = "styles.css".into();
  assert!(
    context.cache_manager.module_cache.has_cache(&css_id),
    "missing root-relative CSS cache for root {}",
    context.config.root
  );
  let old_cache = context.cache_manager.module_cache.get_cache(&css_id);
  assert!(
    codegen_css_stylesheet(&old_cache.module.meta.as_css().ast, false, None, false)
      .0
      .contains(".old")
  );
  let sibling_before = farmfe_core::serialize!(
    &context
      .cache_manager
      .module_cache
      .get_cache(&"sibling.js".into())
  );
  let source_before = std::fs::read(&stylesheet).unwrap();
  let mtime_before = std::fs::metadata(&stylesheet).unwrap().modified().unwrap();

  generation.store(true, Ordering::SeqCst);
  let update = compiler
    .update(
      vec![(
        stylesheet.to_string_lossy().into_owned(),
        UpdateType::Updated,
      )],
      || {},
      true,
      true,
    )
    .unwrap();

  let updated_cache = context.cache_manager.module_cache.get_cache(&css_id);
  let generated =
    codegen_css_stylesheet(&updated_cache.module.meta.as_css().ast, false, None, false).0;
  assert!(
    generated.contains(".new"),
    "explicit update restored stale generated CSS: {generated}"
  );
  assert!(
    update.mutable_resources.contains(".new"),
    "generated update must contain the rebuilt styles: {}",
    update.mutable_resources
  );
  assert_eq!(source_before, std::fs::read(&stylesheet).unwrap());
  assert_eq!(
    mtime_before,
    std::fs::metadata(&stylesheet).unwrap().modified().unwrap()
  );
  assert_eq!(
    sibling_before,
    farmfe_core::serialize!(
      &context
        .cache_manager
        .module_cache
        .get_cache(&"sibling.js".into())
    )
  );
  drop(compiler);
  std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_update_rebuilds_generated_styles_with_timestamp_cache() {
  assert_explicit_update_rebuilds_generated_styles(true);
}

#[test]
fn explicit_update_rebuilds_generated_styles_with_hash_cache() {
  assert_explicit_update_rebuilds_generated_styles(false);
}

#[cfg(unix)]
#[test]
fn explicit_update_rebuilds_generated_styles_under_symlinked_temp_root() {
  let base = std::env::temp_dir().join(format!(
    "farm-cache-temp-alias-{}-{}",
    std::process::id(),
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  ));
  std::fs::create_dir(&base).unwrap();
  let physical = base.join("physical");
  std::fs::create_dir(&physical).unwrap();
  // Canonicalize the link target so this controlled alias does not introduce
  // a second alias through the host's own temporary-directory ancestry.
  let physical = std::fs::canonicalize(physical).unwrap();
  let logical = base.join("logical");
  std::os::unix::fs::symlink(&physical, &logical).unwrap();
  let output = std::process::Command::new(std::env::current_exe().unwrap())
    .args([
      "explicit_update_rebuilds_generated_styles_with_hash_cache",
      "--exact",
      "--test-threads=1",
    ])
    .env("TMPDIR", &logical)
    .env_remove("FARM_UPDATE_SNAPSHOTS")
    .env_remove("INSTA_UPDATE")
    .output()
    .unwrap();
  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    output.status.success() && stdout.contains("1 passed"),
    "cache regression under aliased temp root failed:\n{stdout}\n{}",
    String::from_utf8_lossy(&output.stderr)
  );
  std::fs::remove_file(logical).unwrap();
  std::fs::remove_dir(physical).unwrap();
  std::fs::remove_dir(base).unwrap();
}
