use std::{collections::HashMap, path::PathBuf};

use farmfe_core::config::{
  bool_or_obj::BoolOrObj, config_regex::ConfigRegex,
  partial_bundling::PartialBundlingEnforceResourceConfig, Mode, TargetEnv,
};
mod common;
use crate::common::{
  assert_compiler_result_with_config, create_compiler_with_args, AssertCompilerResultConfig,
};

#[allow(dead_code)]
#[cfg(test)]
fn test(file: String, crate_path: String) {
  let file_path_buf = PathBuf::from(file.clone());
  let create_path_buf = PathBuf::from(crate_path);
  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {:?}", cwd);

  let entry_name = "index".to_string();

  let runtime_entry = cwd.to_path_buf().join("runtime.ts");

  let compiler =
    create_compiler_with_args(cwd.to_path_buf(), create_path_buf, |mut config, plugins| {
      config.mode = Mode::Production;

      if runtime_entry.is_file() {
        let runtime_entry = runtime_entry.to_string_lossy().to_string();
        config.runtime.path = runtime_entry;
      }

      config.input = HashMap::from_iter(vec![(entry_name.clone(), file)]);

      config.minify = Box::new(BoolOrObj::Bool(false));
      config.tree_shaking = false;

      config.external = vec![ConfigRegex::new("(^node:.*)"), ConfigRegex::new("^fs$")];
      config.output.target_env = TargetEnv::Node;

      // multiple bundle
      config.partial_bundling.enforce_resources = vec![PartialBundlingEnforceResourceConfig {
        test: vec![ConfigRegex::new("^bundle2.*")],
        name: "bundle2".to_string(),
      }];

      (config, plugins)
    });

  compiler.compile().unwrap();

  assert_compiler_result_with_config(
    &compiler,
    AssertCompilerResultConfig {
      entry_name: Some(entry_name),
      ignore_emitted_field: false,
    },
  );
}

// farmfe_testing::testing! {"tests/fixtures/runtime/bundle/**/index.ts", test}
// farmfe_testing::testing! {"tests/fixtures/runtime/bundle/cjs/normal/**/index.ts", test}
farmfe_testing::testing! {"tests/fixtures/runtime/bundle/cjs/hybrid/exportStar/**/index.ts", test}
// farmfe_testing::testing! {"tests/fixtures/runtime/bundle/cjs/export/exportNamespace/**/index.ts", test}
