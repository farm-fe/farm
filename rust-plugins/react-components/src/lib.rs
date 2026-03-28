#![deny(clippy::all)]
#![feature(box_patterns)]
pub mod find_local_components;
pub mod finish_components;
pub mod generate_dts;
pub mod insert_import;
pub mod resolvers;

use std::{
  collections::HashSet,
  fmt,
  sync::{Arc, Mutex},
};

use serde::{
  de::{self, Visitor},
  Deserialize, Deserializer, Serialize, Serializer,
};

use farmfe_core::{
  config::{config_regex::ConfigRegex, Config},
  context::create_swc_source_map,
  module::ModuleType,
  plugin::{Plugin, PluginTransformHookResult},
  serde_json::{self},
  swc_common::source_map::{DefaultSourceMapGenConfig},
  swc_ecma_parser::{Syntax, TsSyntax},
};

use farmfe_macro_plugin::farm_plugin;

use farmfe_toolkit::{
  plugin_utils::{normalize_path::normalize_path, path_filter::PathFilter},
  script::{codegen_module, parse_module, CodeGenCommentsConfig, ParseScriptModuleResult},
  swc_ecma_codegen,
  swc_ecma_visit::VisitMutWith,
};

use find_local_components::ComponentInfo;
use finish_components::{finish_components, FinishComponentsParams};
use insert_import::{ImportModifier, InsertImportModifier};
use resolvers::ResolverOption;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
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
  pub local: Option<bool>,
  pub import_mode: Option<ImportMode>,
  pub include: Option<Vec<ConfigRegex>>,
  pub exclude: Option<Vec<ConfigRegex>>,
  pub resolvers: Option<Vec<ResolverOption>>,
}

#[derive(Debug)]
#[farm_plugin]
pub struct FarmPluginReactComponents {
  options: Options,
  components: Arc<Mutex<HashSet<ComponentInfo>>>,
}

impl FarmPluginReactComponents {
  pub fn new(config: &Config, options: String) -> Self {
    let options: Options = serde_json::from_str(&options).unwrap();
    let resolvers = options.resolvers.clone().unwrap_or(vec![]);
    let dirs = options.dirs.clone().unwrap_or(vec![]);
    let root_path = config.root.clone();
    let components = Arc::new(Mutex::new(HashSet::new()));
    finish_components(FinishComponentsParams {
      root_path: normalize_path(&root_path),
      resolvers,
      dirs,
      local: options.local.unwrap_or(true),
      dts: options.dts.clone().unwrap_or_default(),
      context_components: &components,
    });
    Self {
      options,
      components,
    }
  }
}

impl Plugin for FarmPluginReactComponents {
  fn name(&self) -> &str {
    "FarmPluginReactComponents"
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if param.module_type != ModuleType::Jsx && param.module_type != ModuleType::Tsx {
      return Ok(None);
    }
    let options = self.options.clone();
    let include = options.include.unwrap_or(vec![]);
    let exclude = options.exclude.unwrap_or(vec![]);
    let filter = PathFilter::new(&include, &exclude);
    if !filter.execute(&param.module_id) {
      return Ok(None);
    }
    let content = Arc::new(param.content.clone());
    let (cm, _) = create_swc_source_map(&param.module_id.clone().into(), content);
    let ParseScriptModuleResult {
      mut ast, comments, ..
    } = match parse_module(
      &param.module_id.clone().into(),
      Arc::new(param.content.clone()),
      Syntax::Typescript(TsSyntax {
        tsx: true,
        decorators: false,
        dts: false,
        no_early_errors: true,
        disallow_ambiguous_jsx_like: true,
      }),
      context.config.script.target.clone(),
    ) {
      Ok(res) => res,
      Err(err) => {
        println!("{}", err.to_string());
        panic!("Parse {} failed. See error details above.", param.module_id);
      }
    };
    let mut import_modifier = ImportModifier::new(self.components.clone());
    ast.visit_mut_with(&mut import_modifier);
    let used_components = import_modifier.used_components;
    let mut insert_import_modifier = InsertImportModifier::new(
      context.config.output.target_env.clone(),
      options.import_mode.unwrap_or(ImportMode::Absolute),
      param.resolved_path.to_owned(),
      used_components,
    );
    ast.visit_mut_with(&mut insert_import_modifier);
    let mut src_map = vec![];
    let transformed_content = codegen_module(
      &ast,
      cm.clone(),
      Some(&mut src_map),
      swc_ecma_codegen::Config::default(),
      Some(CodeGenCommentsConfig {
        comments: &comments,
        config: &context.config.comments,
      }),
    )
    .unwrap();

    let output_code = String::from_utf8(transformed_content).unwrap();

    let map = {
      let map = cm.build_source_map(&src_map, None, DefaultSourceMapGenConfig);
      let mut buf = vec![];
      map.to_writer(&mut buf).expect("failed to write sourcemap");
      Some(String::from_utf8(buf).unwrap())
    };

    Ok(Some(PluginTransformHookResult {
      content: output_code,
      source_map: map,
      module_type: Some(param.module_type.clone()),
      ignore_previous_source_map: false,
    }))
  }

  fn update_finished(
    &self,
    context: &Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let resolvers = self.options.resolvers.clone().unwrap_or(vec![]);
    let dirs = self.options.dirs.clone().unwrap_or(vec![]);
    let root_path = context.config.root.clone();
    finish_components(FinishComponentsParams {
      root_path: normalize_path(&root_path),
      resolvers,
      dirs,
      local: self.options.local.unwrap_or(true),
      dts: self.options.dts.clone().unwrap_or_default(),
      context_components: &self.components,
    });

    Ok(None)
  }
}
