mod common;
use crate::common::{assert_compiler_result, create_compiler_with_args};
use std::{collections::HashMap, path::PathBuf};

use farmfe_core::config::{
  config_regex::ConfigRegex,
  external::{ExternalConfig, ExternalConfigItem, ExternalObject},
  ModuleFormat, TargetEnv,
};

fn test(file: String, crate_path: String) {
  let file_path_buf = PathBuf::from(file.clone());
  let create_path_buf = PathBuf::from(crate_path);
  let cwd = file_path_buf.parent().unwrap();
  println!("testing test case: {:?}", cwd);

  let entry_name = "index".to_string();
let normolized_file = file.replace('\\', "/");
  let compiler =
    create_compiler_with_args(cwd.to_path_buf(), create_path_buf, |mut config, plugins| {
      config.input = HashMap::from_iter(vec![(entry_name.clone(), file.clone())]);

      if normolized_file.contains("/browser/") || normolized_file.contains("/node/") {
        config.output.target_env = if normolized_file.contains("browser") {
          TargetEnv::Browser
        } else {
          TargetEnv::Node
        };
      }

      if normolized_file.contains("/normal/") || normolized_file.contains("/object/") || true {
        config.external = ExternalConfig(vec![if normolized_file.contains("/object") {
          ExternalConfigItem::Object(ExternalObject {
            pattern: ConfigRegex::new("^jquery$"),
            global_name: "$".to_string(),
          })
        } else {
          ExternalConfigItem::Default(ConfigRegex::new("^jquery$"))
        }]);
      }

      if normolized_file.contains("/cjs/") || normolized_file.contains("/esm/") {
        config.output.format = if normolized_file.contains("cjs") {
          ModuleFormat::CommonJs
        } else {
          ModuleFormat::EsModule
        };
      }

      (config, plugins)
    });

  compiler.compile().unwrap();

  assert_compiler_result(&compiler, Some(&entry_name));
}

farmfe_testing::testing! {"tests/fixtures/external/**/*.ts", test}
