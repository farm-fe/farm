use std::{path::PathBuf, process::Command};

use farmfe_compiler::Compiler;
use farmfe_core::{
  HashMap,
  config::{ModuleFormat, ModuleFormatConfig, TargetEnv, bool_or_obj::BoolOrObj},
};

mod common;

#[test]
fn string_module_names_preserve_live_bindings() {
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/string_module_names");
  for commonjs in [false, true] {
    for tree_shaking in [false, true] {
      for concatenate in [false, true] {
        for minify in [false, true] {
          let output = std::env::temp_dir().join(format!(
            "farm-string-names-{commonjs}-{tree_shaking}-{concatenate}-{minify}-{}",
            std::process::id()
          ));
          let mut config = common::create_config(root.clone(), crate_path.clone());
          config.input = HashMap::from_iter([(
            "main".into(),
            root.join("main.js").to_string_lossy().into_owned(),
          )]);
          config.runtime = common::generate_runtime(crate_path.clone(), true);
          config.concatenate_modules = concatenate;
          config.tree_shaking = Box::new(BoolOrObj::Bool(tree_shaking));
          config.output.target_env = TargetEnv::Node;
          config.output.format = ModuleFormatConfig::Single(if commonjs {
            ModuleFormat::CommonJs
          } else {
            ModuleFormat::EsModule
          });
          let extension = if commonjs { "cjs" } else { "mjs" };
          config.output.filename = format!("[resourceName].{extension}");
          config.output.entry_filename = format!("[entryName].{extension}");
          config.output.path = output.to_string_lossy().into_owned();
          config.minify = Box::new(BoolOrObj::Bool(minify));
          let compiler = Compiler::new(config, vec![]).unwrap();
          compiler.compile().unwrap();
          compiler.write_resources_to_disk().unwrap();
          let script = "import assert from 'node:assert/strict'; import { pathToFileURL } from 'node:url'; import { createRequire } from 'node:module'; const m = process.argv[1].endsWith('.cjs') ? createRequire(import.meta.url)(process.argv[1]) : await import(pathToFileURL(process.argv[1])); assert.deepEqual(m.values(), [1,1,1,1,1,1]); assert.equal(m['public-name'], 1); assert.deepEqual(await m.boundaryValues(), [1,1,1,1,1,1]); m.increment(); assert.deepEqual(m.values(), [2,2,2,2,2,2]); assert.deepEqual(await m.boundaryValues(), [2,2,2,2,2,2]); console.log('string-names-loaded');";
          let result = Command::new("node")
            .args(["--input-type=module", "--eval", script])
            .arg(output.join(format!("main.{extension}")))
            .output()
            .expect("node must be installed for runtime regression tests");
          assert!(
            result.status.success(),
            "commonjs={commonjs}, tree_shaking={tree_shaking}, concatenate={concatenate}, minify={minify}: {}",
            String::from_utf8_lossy(&result.stderr)
          );
          assert!(String::from_utf8_lossy(&result.stdout).contains("string-names-loaded"));
        }
      }
    }
  }
}

#[test]
fn library_existing_external_import_and_string_star_reexports() {
  use farmfe_core::config::config_regex::ConfigRegex;
  let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let root = crate_path.join("tests/fixtures/string_module_names/library");
  let output = std::env::temp_dir().join(format!("farm-string-library-{}", std::process::id()));
  let mut config = common::create_config(root.clone(), crate_path.clone());
  config.input = HashMap::from_iter([(
    "main".into(),
    root.join("main.js").to_string_lossy().into_owned(),
  )]);
  config.runtime = common::generate_runtime(crate_path, true);
  config.concatenate_modules = true;
  config.tree_shaking = Box::new(BoolOrObj::Bool(true));
  config.minify = Box::new(BoolOrObj::Bool(false));
  config.external = vec![ConfigRegex::new(r"external\.mjs$")];
  config.output.target_env = TargetEnv::Library;
  config.output.format = ModuleFormatConfig::Single(ModuleFormat::EsModule);
  config.output.filename = "[resourceName].mjs".into();
  config.output.entry_filename = "[entryName].mjs".into();
  config.output.path = output.to_string_lossy().into_owned();
  let compiler = Compiler::new(config, vec![]).unwrap();
  compiler.compile().unwrap();
  compiler.write_resources_to_disk().unwrap();
  std::fs::copy(root.join("external.mjs"), output.join("external.mjs")).unwrap();
  let script = "import assert from 'node:assert/strict'; import { pathToFileURL } from 'node:url'; const m = await import(pathToFileURL(process.argv[1])); assert.equal(m.read(), 1); for (const key of ['s-k','1-name','123','']) assert.equal(m[key], 1); m.increment(); assert.equal(m.read(), 2); for (const key of ['s-k','1-name','123','']) assert.equal(m[key], 2);";
  let result = Command::new("node")
    .args(["--input-type=module", "--eval", script])
    .arg(output.join("main.mjs"))
    .output()
    .unwrap();
  assert!(
    result.status.success(),
    "{}",
    String::from_utf8_lossy(&result.stderr)
  );
}
