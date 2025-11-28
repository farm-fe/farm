mod common;
use crate::common::{assert_compiler_result, create_compiler_with_args};
use std::path::PathBuf;

use farmfe_core::config::ModuleFormatConfig;
use farmfe_core::config::{config_regex::ConfigRegex, ModuleFormat, TargetEnv};
use farmfe_core::HashMap;

fn test(file: String, crate_path: String) {
  let file_path_buf = PathBuf::from(file.clone());
  let create_path_buf = PathBuf::from(crate_path);
  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {cwd:?}");

  let entry_name = "index".to_string();
  let normolized_file = file.replace('\\', "/");
  let compiler =
    create_compiler_with_args(cwd.to_path_buf(), create_path_buf, |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(entry_name.clone(), file.clone())]);

      config.output.target_env = if normolized_file.contains("browser") {
        TargetEnv::Browser
      } else {
        TargetEnv::Node
      };

      if normolized_file.contains("/object") {
        config
          .output
          .external_globals
          .insert("jquery".to_string(), "$".to_string());
      } else {
        config.external = vec![ConfigRegex::new("^jquery$")];
      }

      config.output.format = if normolized_file.contains("cjs") {
        ModuleFormatConfig::Single(ModuleFormat::CommonJs)
      } else {
        ModuleFormatConfig::Single(ModuleFormat::EsModule)
      };

      (config, plugins)
    });

  compiler.compile().unwrap();

  assert_compiler_result(&compiler, Some(&entry_name));
}

farmfe_testing::testing! {"tests/fixtures/external/**/index.ts", test}
