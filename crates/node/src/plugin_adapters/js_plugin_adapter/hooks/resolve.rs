use napi::bindgen_prelude::FromNapiValue;
use std::sync::Arc;

use farmfe_core::{
  config::config_regex::ConfigRegex,
  context::CompilationContext,
  error::Result,
  plugin::{PluginHookContext, PluginResolveHookParam, PluginResolveHookResult},
};

use crate::{
  new_js_plugin_hook,
  plugin_adapters::js_plugin_adapter::thread_safe_js_plugin_hook::ThreadSafeJsPluginHook,
};

pub struct JsPluginResolveHook {
  tsfn: ThreadSafeJsPluginHook,
  filters: PluginResolveHookFilters,
}

impl JsPluginResolveHook {
  new_js_plugin_hook!(
    PluginResolveHookFilters,
    JsPluginResolveHookFilters,
    PluginResolveHookParam,
    PluginResolveHookResult
  );

  pub fn call(
    &self,
    param: PluginResolveHookParam,
    ctx: Arc<CompilationContext>,
    hook_context: PluginHookContext,
  ) -> Result<Option<PluginResolveHookResult>> {
    let filtered = self.filters.importers.iter().any(|i| {
      if let Some(importer) = &param.importer {
        i.is_match(importer.to_string().as_str())
      } else {
        i.is_match("None")
      }
    }) && self
      .filters
      .sources
      .iter()
      .any(|f| f.is_match(&param.source));

    if filtered {
      self
        .tsfn
        .call::<PluginResolveHookParam, PluginResolveHookResult>(param, ctx, Some(hook_context))
    } else {
      Ok(None)
    }
  }
}

/// Resolve hook filters, works as `||`. If any importers or sources matches any regex item in the Vec, we treat it as filtered.
#[napi(object)]
pub struct JsPluginResolveHookFilters {
  pub importers: Vec<String>,
  pub sources: Vec<String>,
}

#[derive(Debug)]
struct PluginResolveHookFilters {
  pub importers: Vec<ConfigRegex>,
  pub sources: Vec<ConfigRegex>,
}

impl From<JsPluginResolveHookFilters> for PluginResolveHookFilters {
  fn from(f: JsPluginResolveHookFilters) -> Self {
    Self {
      importers: f
        .importers
        .into_iter()
        .map(|f| ConfigRegex::new(&f))
        .collect(),
      sources: f
        .sources
        .into_iter()
        .map(|f| ConfigRegex::new(&f))
        .collect(),
    }
  }
}
