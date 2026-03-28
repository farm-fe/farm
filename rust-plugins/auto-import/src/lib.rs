#![deny(clippy::all)]
mod addons;
mod finish_imports;
mod parser;
mod presets;

use std::{
  fmt,
  sync::{Arc, Mutex},
};

use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  module::ModuleType,
  plugin::Plugin,
  serde_json,
};

use addons::vue_template::vue_template_addon;
use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::plugin_utils::path_filter::PathFilter;
use finish_imports::FinishImportsParams;
use parser::scan_exports::Import;
use presets::PresetItem;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum ImportMode {
  Relative,
  Absolute,
}

#[derive(Clone, Debug)]
pub enum Dts {
  Bool(bool),
  Filename(String),
}

impl Serialize for Dts {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match *self {
      Dts::Bool(ref b) => serializer.serialize_bool(*b),
      Dts::Filename(ref s) => serializer.serialize_str(s),
    }
  }
}

impl<'de> Deserialize<'de> for Dts {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct StringOrBoolVisitor;
    impl<'de> Visitor<'de> for StringOrBoolVisitor {
      type Value = Dts;
      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        formatter.write_str("a boolean or a string")
      }
      fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(Dts::Bool(value))
      }
      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(Dts::Filename(value.to_owned()))
      }
    }
    deserializer.deserialize_any(StringOrBoolVisitor)
  }
}

impl Default for Dts {
  fn default() -> Self {
    Dts::Bool(true)
  }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  pub dirs: Option<Vec<ConfigRegex>>,
  pub dts: Option<Dts>,
  pub ignore: Option<Vec<ConfigRegex>>,
  pub presets: Option<Vec<PresetItem>>,
  pub import_mode: Option<ImportMode>,
  pub include: Option<Vec<ConfigRegex>>,
  pub exclude: Option<Vec<ConfigRegex>>,
  pub inject_at_end: Option<bool>,
}

#[farm_plugin]
pub struct FarmfePluginAutoImport {
  options: Options,
  collect_imports: Arc<Mutex<Vec<Import>>>,
}

impl FarmfePluginAutoImport {
  fn new(config: &Config, options: String) -> Self {
    let options: Options = serde_json::from_str(&options).unwrap();
    let collect_imports: Arc<Mutex<Vec<Import>>> = Arc::new(Mutex::new(vec![]));
    let dirs = options.dirs.clone().unwrap_or(vec![]);
    let root_path = config.root.clone();
    let presets = options.presets.clone().unwrap_or(vec![]);
    let ignore = options.ignore.clone().unwrap_or(vec![]);
    finish_imports::finish_imports(FinishImportsParams {
      root_path,
      presets,
      dirs,
      ignore,
      dts: options.dts.clone().unwrap_or_default(),
      context_imports: &collect_imports,
    });
    Self {
      options,
      collect_imports,
    }
  }
}

impl Plugin for FarmfePluginAutoImport {
  fn name(&self) -> &str {
    "FarmfePluginAutoImport"
  }
  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    _context: &Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if ![
      ModuleType::Jsx,
      ModuleType::Tsx,
      ModuleType::Js,
      ModuleType::Ts,
    ]
    .contains(&param.module_type)
    {
      return Ok(None);
    }
    let options = self.options.clone();
    let include = options.include.unwrap_or(vec![]);
    let exclude = options
      .exclude
      .unwrap_or(vec![ConfigRegex::new("node_modules")]);
    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(&param.module_id) {
      return Ok(None);
    } else {
      let imports = self.collect_imports.lock().unwrap();
      let mut content = param.content.clone();
      if param.resolved_path.ends_with(".vue") {
        vue_template_addon(&mut content, &imports);
      }
      let content =
        parser::inject_imports::inject_imports(&content, imports.clone().to_vec(), None, options.inject_at_end.unwrap_or(false));
      // let (cm, src) = create_swc_source_map(Source {
      //   path: PathBuf::from(param.resolved_path),
      //   content: Arc::new(content.clone()),
      // });
      // let map = {
      //   let map = build_source_map(cm, &src_map);
      //   let mut buf = vec![];
      //   map.to_writer(&mut buf).expect("failed to write sourcemap");
      //   Some(String::from_utf8(buf).unwrap())
      // };
      Ok(Some(farmfe_core::plugin::PluginTransformHookResult {
        content,
        source_map: None,
        module_type: Some(param.module_type.clone()),
        ignore_previous_source_map: false,
      }))
    }
  }

  fn update_finished(
    &self,
    context: &Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let dirs = self.options.dirs.clone().unwrap_or(vec![]);
    let root_path = context.config.root.clone();
    let presets = self.options.presets.clone().unwrap_or(vec![]);
    let ignore = self.options.ignore.clone().unwrap_or(vec![]);
    finish_imports::finish_imports(FinishImportsParams {
      root_path,
      presets,
      dirs,
      ignore,
      dts: self.options.dts.clone().unwrap_or_default(),
      context_imports: &self.collect_imports,
    });
    Ok(None)
  }
}
