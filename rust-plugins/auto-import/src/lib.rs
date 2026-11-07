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
  config::{Config, config_regex::ConfigRegex},
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
  fn new(_config: &Config, options: String) -> Self {
    let options: Options = serde_json::from_str(&options).unwrap();
    let collect_imports: Arc<Mutex<Vec<Import>>> = Arc::new(Mutex::new(vec![]));
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
  fn build_start(
    &self,
    context: &Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    // Discovery can parse user files. Run it in a fallible hook, not the native constructor.
    self.update_finished(context)
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
    let include = options.include.unwrap_or_default();
    let exclude = options
      .exclude
      .unwrap_or(vec![ConfigRegex::new("node_modules")]);
    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(&param.module_id) {
      Ok(None)
    } else {
      let imports = self.collect_imports.lock().unwrap();
      let mut content = param.content.clone();
      if param.resolved_path.ends_with(".vue") {
        vue_template_addon(&mut content, &imports);
      }
      let content = parser::inject_imports::inject_imports(
        param.resolved_path,
        &content,
        imports.clone().to_vec(),
        None,
        options.inject_at_end.unwrap_or(false),
      )?;
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
    let dirs = self.options.dirs.clone().unwrap_or_default();
    let root_path = context.config.root.clone();
    let presets = self.options.presets.clone().unwrap_or_default();
    let ignore = self.options.ignore.clone().unwrap_or_default();
    finish_imports::finish_imports(FinishImportsParams {
      root_path,
      presets,
      dirs,
      ignore,
      dts: self.options.dts.clone().unwrap_or_default(),
      context_imports: &self.collect_imports,
    })?;
    Ok(None)
  }
}

#[cfg(test)]
mod recovery_tests {
  use super::*;
  use farmfe_core::{
    context::CompilationContext, error::CompilationError, plugin::PluginTransformHookParam,
  };

  #[test]
  fn transform_preserves_diagnostic_then_recovers_without_poisoning_imports() {
    let config = Config::default();
    let plugin =
      FarmfePluginAutoImport::new(&config, r#"{"dts":false,"presets":["react"]}"#.into());
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    plugin.build_start(&context).unwrap();
    for content in ["export const value =", "const value = 1 2;"] {
      let param = PluginTransformHookParam {
        module_id: "stream.tsx".into(),
        resolved_path: "stream.tsx",
        content: content.into(),
        module_type: ModuleType::Tsx,
        query: vec![],
        meta: Default::default(),
        source_map_chain: vec![],
      };
      assert!(
        matches!(plugin.transform(&param, &context), Err(CompilationError::ParseError { resolved_path, .. }) if resolved_path == "stream.tsx")
      );
      let valid = PluginTransformHookParam {
        content:
          "export const View = () => { const [value] = useState(1); return <div>{value}</div> };"
            .into(),
        ..param
      };
      let result = plugin.transform(&valid, &context).unwrap().unwrap();
      assert!(result.content.contains("useState"));
      assert!(result.content.contains("import"));
    }
  }

  #[test]
  fn directory_discovery_failure_is_fallible_and_can_retry() {
    let dir =
      std::env::temp_dir().join(format!("farm-auto-import-recovery-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("exports.ts");
    std::fs::write(&file, "export const item =").unwrap();
    let config = Config {
      root: dir.to_string_lossy().into_owned(),
      ..Default::default()
    };
    let plugin =
      FarmfePluginAutoImport::new(&config, r#"{"dts":false,"dirs":["exports\\.ts$"]}"#.into());
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    assert!(plugin.build_start(&context).is_err());
    assert!(plugin.collect_imports.lock().unwrap().is_empty());
    std::fs::write(&file, "export const item = 1;").unwrap();
    plugin.build_start(&context).unwrap();
    assert!(
      plugin
        .collect_imports
        .lock()
        .unwrap()
        .iter()
        .any(|item| item.name == "item")
    );
    std::fs::write(&file, "export const item =").unwrap();
    assert!(plugin.update_finished(&context).is_err());
    assert!(
      plugin
        .collect_imports
        .lock()
        .unwrap()
        .iter()
        .any(|item| item.name == "item")
    );
    std::fs::remove_dir_all(dir).unwrap();
  }
}

#[cfg(test)]
mod lifecycle_tests {
  use super::*;
  use farmfe_core::context::CompilationContext;
  // Prepared supplemental crate test; not appended/compiled yet.
  // Uses the existing recovery_tests scope imports and FarmfePluginAutoImport.
  #[test]
  fn repeated_build_start_rescans_and_preserves_dts_on_failure() {
    let dir = std::env::temp_dir().join(format!(
      "farm-auto-import-dts-lifecycle-{}",
      std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("exports.ts");
    let dts = dir.join("generated.d.ts");
    std::fs::write(&source, "export const alpha = 1;").unwrap();
    let config = Config {
      root: dir.to_string_lossy().into_owned(),
      ..Default::default()
    };
    let plugin = FarmfePluginAutoImport::new(
      &config,
      r#"{"dirs":["exports\\.ts$"],"dts":"generated.d.ts"}"#.into(),
    );
    assert!(!dts.exists());
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    plugin.build_start(&context).unwrap();
    let first = std::fs::read_to_string(&dts).unwrap();
    assert!(first.contains("alpha"));
    std::fs::write(&dts, format!("{first}\n// sentinel")).unwrap();
    plugin.build_start(&context).unwrap();
    assert!(std::fs::read_to_string(&dts).unwrap().contains("sentinel"));
    std::fs::write(&source, "export const beta = 2;").unwrap();
    plugin.build_start(&context).unwrap();
    let second = std::fs::read_to_string(&dts).unwrap();
    assert!(second.contains("beta"));
    assert!(!second.contains("sentinel"));
    std::fs::write(&source, "export const beta =").unwrap();
    assert!(plugin.build_start(&context).is_err());
    assert_eq!(std::fs::read_to_string(&dts).unwrap(), second);
    assert!(
      plugin
        .collect_imports
        .lock()
        .unwrap()
        .iter()
        .any(|item| item.name == "beta")
    );
    std::fs::write(&source, "export const beta = 2;").unwrap();
    plugin.build_start(&context).unwrap();
    assert_eq!(std::fs::read_to_string(&dts).unwrap(), second);
    std::fs::remove_dir_all(dir).unwrap();
  }
}
