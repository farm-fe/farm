use std::{path::PathBuf, process::Command};

use farmfe_compiler::Compiler;
use farmfe_core::{
  HashMap,
  config::{ModuleFormat, ModuleFormatConfig, TargetEnv, bool_or_obj::BoolOrObj},
};

mod common;

#[test]
fn aliased_import_and_star_reexport_across_dynamic_boundary_keep_live_bindings() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/concatenate_aliased_star");
  for minify in [false, true] {
    let output =
      std::env::temp_dir().join(format!("farm-aliased-star-{minify}-{}", std::process::id()));
    let mut config = common::create_config(root.clone(), crate_path.clone());
    config.input = HashMap::from_iter([(
      "main".into(),
      root.join("main.js").to_string_lossy().into_owned(),
    )]);
    config.runtime = common::generate_runtime(crate_path.clone(), true);
    config.concatenate_modules = true;
    config.output.target_env = TargetEnv::Node;
    config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
    config.output.filename = "[resourceName].mjs".into();
    config.output.entry_filename = "[entryName].mjs".into();
    config.output.path = output.to_string_lossy().into_owned();
    config.minify = Box::new(BoolOrObj::Bool(minify));
    let compiler = Compiler::new(config, vec![]).unwrap();
    compiler.compile().unwrap();
    compiler.write_resources_to_disk().unwrap();
    let script = "import assert from 'node:assert/strict'; import { pathToFileURL } from 'node:url'; const m = await import(pathToFileURL(process.argv[1])); assert.equal(m.c, 1); assert.equal(m.f(), 1); m.increment(); assert.equal(m.f(), 2); console.log('aliased-star-loaded');";
    let result = Command::new("node")
      .args(["--input-type=module", "--eval", script])
      .arg(output.join("main.mjs"))
      .output()
      .expect("node must be installed for runtime regression tests");
    assert!(
      result.status.success(),
      "minify={minify}: {}",
      String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("aliased-star-loaded"));
  }
}
