// #![deny(clippy::all)]

// use farmfe_core::relative_path::RelativePath;
// use farmfe_core::serde_json::{self, Value};
// use farmfe_core::{config::Config, module::ModuleType, plugin::Plugin};
// use farmfe_macro_plugin::farm_plugin;
// use farmfe_toolkit::{fs, regex::Regex, resolve::follow_symlinks};
// use serde::{Deserialize, Serialize};
// use std::env;
// use std::env::consts::{ARCH, OS};
// use std::path::PathBuf;

// const PKG_NAME: &str = "@farmfe/plugin-vue-jsx";

// #[farm_plugin]
// #[derive(Debug, Deserialize, Serialize, Default)]
// #[serde(rename_all = "camelCase")]
// pub struct FarmPluginVueJsx {
//   vue_jsx_options: String,
// }

// impl FarmPluginVueJsx {
//   pub fn new(config: &Config, options: String) -> Self {
//     Self {
//       vue_jsx_options: options,
//     }
//   }
// }

// impl Plugin for FarmPluginVueJsx {
//   fn name(&self) -> &str {
//     "FarmPluginVueJsx"
//   }
// }

#![deny(clippy::all)]

use farmfe_core::{config::Config, plugin::Plugin};

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct FarmPluginVueJsx {
  vue_jsx_options: String,
}

impl FarmPluginVueJsx {
  fn new(config: &Config, options: String) -> Self {
    Self {
      vue_jsx_options: options,
    }
  }
}

impl Plugin for FarmPluginVueJsx {
  fn name(&self) -> &str {
    "FarmPluginVueJsx"
  }
}
