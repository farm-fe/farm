use std::{path::PathBuf, process::Command, sync::Arc};

use farmfe_compiler::Compiler;
use farmfe_core::{
  HashMap,
  config::{ModuleFormat, ModuleFormatConfig, TargetEnv, bool_or_obj::BoolOrObj},
  context::CompilationContext,
  error::Result,
  module::ModuleType,
  plugin::{
    Plugin, PluginHookContext, PluginLoadHookParam, PluginLoadHookResult, PluginResolveHookParam,
    PluginResolveHookResult,
  },
};

mod common;

struct VirtualPlugin;
impl Plugin for VirtualPlugin {
  fn name(&self) -> &str {
    "virtual-dynamic-regression"
  }
  fn priority(&self) -> i32 {
    1000
  }
  fn resolve(
    &self,
    param: &PluginResolveHookParam,
    _: &Arc<CompilationContext>,
    _: &PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    Ok(
      (param.source == "virtual:answer").then(|| PluginResolveHookResult {
        // The Vite adapter encodes NUL this way before passing resolved IDs to Rust.
        resolved_path: "\\0virtual:answer.js".into(),
        ..Default::default()
      }),
    )
  }
  fn load(
    &self,
    param: &PluginLoadHookParam,
    _: &Arc<CompilationContext>,
    _: &PluginHookContext,
  ) -> Result<Option<PluginLoadHookResult>> {
    Ok(
      param
        .resolved_path
        .ends_with("\\0virtual:answer.js")
        .then(|| PluginLoadHookResult {
          content: "export default 42; export const named = 'virtual-loaded';".into(),
          module_type: ModuleType::Js,
          source_map: None,
        }),
    )
  }
}

#[test]
fn self_and_cyclic_dynamic_imports_keep_cached_identity() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/virtual_dynamic_import");
  for (entry, expression) in [
    ("self_entry.js", "a.loadSelf()"),
    ("cyclic_entry_a.js", "a.loadB().then(b=>b.loadA())"),
  ] {
    let output = std::env::temp_dir().join(format!("farm-{entry}-runtime-{}", std::process::id()));
    let mut config = common::create_config(root.clone(), crate_path.clone());
    config.input =
      HashMap::from_iter([("a".into(), root.join(entry).to_string_lossy().into_owned())]);
    config.runtime = common::generate_runtime(crate_path.clone(), true);
    config.output.target_env = TargetEnv::Node;
    config.output.format = ModuleFormatConfig::Single(ModuleFormat::CommonJs);
    config.output.entry_filename = "[entryName].cjs".into();
    config.output.path = output.to_string_lossy().into_owned();
    config.minify = Box::new(BoolOrObj::Bool(true));
    let compiler = Compiler::new(config, vec![]).unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
      let result = compiler.compile();
      sender.send((compiler, result)).unwrap();
    });
    let (compiler, result) = receiver
      .recv_timeout(std::time::Duration::from_secs(30))
      .expect("self dynamic import compilation must terminate");
    result.unwrap();
    compiler.write_resources_to_disk().unwrap();
    let script = format!(
      "const assert=require('node:assert/strict');const a=require(process.argv[1]);{expression}.then(m=>{{assert.equal(m.state,a.state);console.log('cycle-loaded');}});"
    );
    let result = Command::new("node")
      .args(["--eval", &script])
      .arg(output.join("a.cjs"))
      .output()
      .unwrap();
    assert!(
      result.status.success(),
      "{}",
      String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("cycle-loaded"));
  }
}

#[test]
fn synthetic_runtime_modules_can_be_rebuilt_by_compiler_update() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/virtual_dynamic_import");
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.mode = farmfe_core::config::Mode::Development;
  config.input = HashMap::from_iter([(
    "index".into(),
    root.join("entry_b.js").to_string_lossy().into_owned(),
  )]);
  config.runtime = common::generate_runtime(crate_path, true);
  let compiler = Compiler::new(config, vec![]).unwrap();
  compiler.compile().unwrap();
  let runtime_id = compiler
    .context()
    .module_graph
    .read()
    .modules()
    .into_iter()
    .find(|module| module.id.relative_path() == "@farmfe/runtime/src/module-system.ts.farm-runtime")
    .unwrap()
    .id
    .clone();
  let result = compiler
    .update(
      vec![(
        runtime_id.resolved_path(root.to_str().unwrap()),
        farmfe_core::plugin::UpdateType::Updated,
      )],
      || {},
      true,
      true,
    )
    .unwrap();
  assert!(result.updated_module_ids.contains(&runtime_id));
}

#[test]
fn default_reexports_execute_with_valid_entry_bindings() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/virtual_dynamic_import");
  let output = std::env::temp_dir().join(format!("farm-default-reexport-{}", std::process::id()));
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.input = HashMap::from_iter([
    (
      "index".into(),
      root.join("default_entry.js").to_string_lossy().into_owned(),
    ),
    (
      "alias".into(),
      root
        .join("named_default_entry.js")
        .to_string_lossy()
        .into_owned(),
    ),
    (
      "mixed".into(),
      root
        .join("mixed_default_entry.js")
        .to_string_lossy()
        .into_owned(),
    ),
  ]);
  config.runtime = common::generate_runtime(crate_path, true);
  config.output.target_env = TargetEnv::Node;
  config.output.filename = "[resourceName].mjs".into();
  config.output.entry_filename = "[entryName].mjs".into();
  config.output.path = output.to_string_lossy().into_owned();
  config.minify = Box::new(BoolOrObj::Bool(true));
  let compiler = Compiler::new(config, vec![]).unwrap();
  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();
  for (entry, expression, expected) in [
    ("index", "m.default", "42"),
    ("alias", "m.Alias", "42"),
    ("mixed", "m.default, m.Alias, m.named", "42 42 7"),
  ] {
    let script = format!(
      "import {{pathToFileURL}} from 'node:url'; const m = await import(pathToFileURL(process.argv[1])); console.log({expression});"
    );
    let result = Command::new("node")
      .args(["--input-type=module", "--eval", &script])
      .arg(output.join(format!("{entry}.mjs")))
      .output()
      .unwrap();
    assert!(
      result.status.success(),
      "{}",
      String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), expected);
  }
}

#[test]
fn an_entry_can_also_be_a_dynamic_import_target() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/virtual_dynamic_import");
  let output = std::env::temp_dir().join(format!("farm-dynamic-entry-{}", std::process::id()));
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.input = HashMap::from_iter([
    (
      "a".into(),
      root.join("entry_a.js").to_string_lossy().into_owned(),
    ),
    (
      "b".into(),
      root.join("entry_b.js").to_string_lossy().into_owned(),
    ),
  ]);
  config.runtime = common::generate_runtime(crate_path, true);
  config.output.target_env = TargetEnv::Node;
  config.output.format = ModuleFormatConfig::Single(ModuleFormat::CommonJs);
  config.output.entry_filename = "[entryName].cjs".into();
  config.output.path = output.to_string_lossy().into_owned();
  let compiler = Compiler::new(config, vec![]).unwrap();
  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();
  let result = Command::new("node")
    .arg(output.join("a.cjs"))
    .output()
    .unwrap();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  assert!(String::from_utf8_lossy(&result.stdout).contains("entry-loaded"));
}

#[test]
fn imported_and_independent_entries_keep_the_shared_runtime() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/virtual_dynamic_import");
  for (scenario, first, second, script, expected) in [
    (
      "sync",
      "sync_entry_a.js",
      "entry_b.js",
      "require(process.argv[1]);",
      "entry-loaded",
    ),
    (
      "independent",
      "independent_entry_a.js",
      "independent_entry_b.js",
      "require(process.argv[1]);require(process.argv[2]);",
      "second-entry-loaded 42",
    ),
  ] {
    let output = std::env::temp_dir().join(format!("farm-{scenario}-entry-{}", std::process::id()));
    let mut config = common::create_config(root.clone(), crate_path.clone());
    config.input = HashMap::from_iter([
      ("a".into(), root.join(first).to_string_lossy().into_owned()),
      ("b".into(), root.join(second).to_string_lossy().into_owned()),
    ]);
    config.runtime = common::generate_runtime(crate_path.clone(), true);
    config.output.target_env = TargetEnv::Node;
    config.output.format = ModuleFormatConfig::Single(ModuleFormat::CommonJs);
    config.output.entry_filename = "[entryName].cjs".into();
    config.output.path = output.to_string_lossy().into_owned();
    let compiler = Compiler::new(config, vec![]).unwrap();
    compiler.compile().unwrap();
    compiler.write_resources_to_disk().unwrap();
    let result = Command::new("node")
      .args(["--eval", script])
      .args([output.join("a.cjs"), output.join("b.cjs")])
      .output()
      .unwrap();
    assert!(
      result.status.success(),
      "{}",
      String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains(expected));
  }
}

#[test]
fn inline_and_split_entries_preserve_module_identity_and_dynamic_loading() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/virtual_dynamic_import");
  let output = std::env::temp_dir().join(format!("farm-mixed-runtime-{}", std::process::id()));
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.input = HashMap::from_iter([
    (
      "a".into(),
      root
        .join("mixed_inline_entry_a.js")
        .to_string_lossy()
        .into_owned(),
    ),
    (
      "b".into(),
      root
        .join("mixed_split_entry_b.js")
        .to_string_lossy()
        .into_owned(),
    ),
    (
      "c".into(),
      root
        .join("mixed_split_entry_c.js")
        .to_string_lossy()
        .into_owned(),
    ),
  ]);
  config.runtime = common::generate_runtime(crate_path, true);
  config.output.target_env = TargetEnv::Node;
  config.output.format = ModuleFormatConfig::Single(ModuleFormat::CommonJs);
  config.output.entry_filename = "[entryName].cjs".into();
  config.output.path = output.to_string_lossy().into_owned();
  config.minify = Box::new(BoolOrObj::Bool(true));
  let compiler = Compiler::new(config, vec![]).unwrap();
  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();
  let a = std::fs::read_to_string(output.join("a.cjs")).unwrap();
  let b = std::fs::read_to_string(output.join("b.cjs")).unwrap();
  assert!(
    !a.contains("require(\"./farm_internal_runtime"),
    "A must inline runtime for this regression"
  );
  assert!(
    b.contains("require(\"./farm_internal_runtime"),
    "B must load the standalone runtime for this regression"
  );
  let script = "const assert=require('node:assert/strict');const a=require(process.argv[1]);const ms=global.__farm_default_namespace__.m;const b=require(process.argv[2]);assert.equal(global.__farm_default_namespace__.m,ms);assert.equal(b.value,7);Promise.all([a.loadOwn(),b.loadInline()]).then(([lazy,fromB])=>{assert.equal(lazy.token,'inline-lazy-loaded');assert.equal(fromB.state,a.state);console.log('identity-preserved');});";
  let result = Command::new("node")
    .args(["--eval", script])
    .args([output.join("a.cjs"), output.join("b.cjs")])
    .output()
    .unwrap();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  assert!(String::from_utf8_lossy(&result.stdout).contains("identity-preserved"));
}

#[test]
fn virtual_dynamic_import_emits_safe_files_and_loads_them() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/virtual_dynamic_import");
  let output = std::env::temp_dir().join(format!("farm-virtual-import-{}", std::process::id()));
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.input = HashMap::from_iter([(
    "index".into(),
    root.join("index.js").to_string_lossy().into_owned(),
  )]);
  config.runtime = common::generate_runtime(crate_path.clone(), true);
  config.output.target_env = TargetEnv::Node;
  config.output.format = ModuleFormatConfig::Single(ModuleFormat::CommonJs);
  config.output.filename = "[resourceName].[ext]".into();
  config.output.entry_filename = "[entryName].cjs".into();
  config.output.path = output.to_string_lossy().into_owned();
  config.minify = Box::new(BoolOrObj::Bool(true));
  let compiler = Compiler::new(config.clone(), vec![Arc::new(VirtualPlugin)]).unwrap();
  compiler.compile().unwrap();
  for name in compiler.context().resources_map.lock().keys() {
    assert!(
      !name
        .chars()
        .any(|c| c.is_control() || "<>:\"\\|?*'".contains(c)),
      "unsafe emitted name {name:?}"
    );
  }
  compiler.write_resources_to_disk().unwrap();
  let result = Command::new("node")
    .arg(output.join("index.cjs"))
    .output()
    .expect("node must be installed for runtime regression tests");
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  assert!(String::from_utf8_lossy(&result.stdout).contains("42 virtual-loaded"));

  // Exercise the HTML minifier path that originally failed while parsing the
  // dynamic resource map's unescaped virtual filename.
  config.input = HashMap::from_iter([(
    "index".into(),
    root.join("index.html").to_string_lossy().into_owned(),
  )]);
  config.output.target_env = TargetEnv::Browser;
  config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
  config.output.entry_filename = "[entryName].[ext]".into();
  let compiler = Compiler::new(config, vec![Arc::new(VirtualPlugin)]).unwrap();
  compiler.compile().unwrap();
  let resources = compiler.context().resources_map.lock();
  let html = resources
    .values()
    .find(|resource| resource.name.ends_with(".html"))
    .unwrap();
  assert!(!html.bytes.contains(&0));
  farmfe_toolkit::html::parse_html_document(
    &html.name,
    Arc::new(String::from_utf8(html.bytes.clone()).unwrap()),
  )
  .unwrap();
  let runtime = resources
    .values()
    .find(|resource| {
      matches!(
        resource.resource_type,
        farmfe_core::resource::ResourceType::Runtime
      )
    })
    .unwrap();
  let worker_runtime = output.join("worker-runtime.js");
  std::fs::write(&worker_runtime, &runtime.bytes).unwrap();
  let result = Command::new("node")
    .arg(root.join("worker_runtime_check.cjs"))
    .arg(worker_runtime)
    .output()
    .unwrap();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
  assert!(String::from_utf8_lossy(&result.stdout).contains("worker-runtime-loaded"));
}
