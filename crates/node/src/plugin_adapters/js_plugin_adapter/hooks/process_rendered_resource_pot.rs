use std::sync::Arc;

use farmfe_core::{
  config::config_regex::ConfigRegex,
  context::CompilationContext,
  error::Result,
  resource::resource_pot::{ResourcePot, ResourcePotId, ResourcePotType},
  serde::{Deserialize, Serialize},
  HashMap,
};
use farmfe_toolkit::html::codegen_html_document;
use napi::bindgen_prelude::FromNapiValue;

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::{
    module_hook_common::{css_codegen, js_codegen},
    thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
  },
};

#[napi(object)]
pub struct JsPluginProcessRenderedResourcePotHookFilters {
  pub resource_pot_types: Vec<String>,
  pub module_ids: Vec<String>,
}

#[derive(Debug)]
pub struct PluginProcessRenderedResourcePotHookFilters {
  pub resource_pot_types: Vec<ResourcePotType>,
  pub module_ids: Vec<ConfigRegex>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "farmfe_core::serde")]
pub struct JsResourcePot {
  pub id: ResourcePotId,
  pub name: String,
  pub resource_pot_type: ResourcePotType,
  pub module_ids: Vec<String>,
  pub custom: HashMap<String, String>,

  pub content: String,
  pub source_map_chain: Vec<Arc<String>>,

  pub is_dynamic_entry: bool,
  pub is_entry: bool,
}

impl JsResourcePot {
  pub fn new(resource_pot: &ResourcePot, context: &Arc<CompilationContext>) -> Self {
    let (code, source_map) = match &resource_pot.resource_pot_type {
      ResourcePotType::DynamicEntryJs | ResourcePotType::Js => {
        let mut comments = resource_pot.meta.as_js().comments.clone();
        let cm = context.meta.get_resource_pot_source_map(&resource_pot.id);
        js_codegen(&resource_pot.meta.as_js().ast, &mut comments, cm, context).unwrap()
      }
      ResourcePotType::Css => {
        let cm = context.meta.get_resource_pot_source_map(&resource_pot.id);
        css_codegen(&resource_pot.meta.as_css().ast, cm, context).unwrap()
      }
      ResourcePotType::Html => (
        codegen_html_document(&resource_pot.meta.as_html().ast, false),
        None,
      ),
      ResourcePotType::Custom(_) => {
        unreachable!("custom resource pot type can not be handled by js plugins")
      }
    };

    Self {
      id: resource_pot.id.clone(),
      name: resource_pot.name.clone(),
      resource_pot_type: resource_pot.resource_pot_type.clone(),
      module_ids: resource_pot
        .modules()
        .iter()
        .map(|m| m.to_string())
        .collect(),
      custom: Default::default(),
      content: code,
      source_map_chain: if let Some(source_map) = source_map {
        vec![Arc::new(source_map)]
      } else {
        vec![]
      },
      is_dynamic_entry: resource_pot.is_dynamic_entry,
      is_entry: resource_pot.entry_module.is_some(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "farmfe_core::serde")]
pub struct JsPluginProcessRenderedResourcePotHookResult {
  pub content: String,
  pub source_map: Option<String>,
  pub ignore_previous_source_map: Option<bool>,
}

pub struct JsPluginProcessRenderedResourcePotHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginProcessRenderedResourcePotHookFilters,
}

impl From<JsPluginProcessRenderedResourcePotHookFilters>
  for PluginProcessRenderedResourcePotHookFilters
{
  fn from(f: JsPluginProcessRenderedResourcePotHookFilters) -> Self {
    Self {
      resource_pot_types: f
        .resource_pot_types
        .into_iter()
        .map(|ty| ty.into())
        .collect(),
      module_ids: f
        .module_ids
        .into_iter()
        .map(|p| (ConfigRegex::new(&p)))
        .collect(),
    }
  }
}

impl JsPluginProcessRenderedResourcePotHook {
  new_js_plugin_hook!(
    PluginProcessRenderedResourcePotHookFilters,
    JsPluginProcessRenderedResourcePotHookFilters,
    JsResourcePot,
    JsPluginProcessRenderedResourcePotHookResult
  );

  pub fn call(
    &self,
    param: JsResourcePot,
    ctx: Arc<CompilationContext>,
  ) -> Result<Option<JsPluginProcessRenderedResourcePotHookResult>> {
    if self
      .filters
      .resource_pot_types
      .contains(&param.resource_pot_type)
      || self
        .filters
        .module_ids
        .iter()
        .any(|f| param.module_ids.iter().any(|id| f.is_match(id)))
    {
      self.tsfn.call(param, ctx, None)
    } else {
      Ok(None)
    }
  }
}
