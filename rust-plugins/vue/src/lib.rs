#![deny(clippy::all)]
//! `@farmfe/plugin-vue` — Farm Rust plugin that compiles Vue 3 SFCs by
//! delegating to the [`fervid`] compiler.
//!
//! Phase A scope (see plan in PR description):
//! - load `.vue` files and tag them with `ModuleType::Custom("vue")`;
//! - transform the main module via `fervid::compile`, emitting one JS
//!   module that imports each `<style>` block as a `?vue&type=style…`
//!   virtual module;
//! - inject the `__VUE_OPTIONS_API__`, `__VUE_PROD_DEVTOOLS__` and
//!   `__VUE_PROD_HYDRATION_MISMATCH_DETAILS__` define flags;
//! - append `__file` / `__hmrId` devtools metadata in development.
//!
//! Phase 2 adds a descriptor cache and custom-block virtual modules as
//! foundations for granular HMR. Preprocessor re-scoping, type-dep tracking
//! and `inlineTemplate: false` remain gated on Farm/fervid hook support.

use std::borrow::Cow;
use std::sync::Arc;

use farmfe_core::{
  config::{Config, Mode},
  context::CompilationContext,
  error::CompilationError,
  module::ModuleType,
  plugin::{
    Plugin, PluginHookContext, PluginLoadHookParam, PluginLoadHookResult, PluginTransformHookParam,
    PluginTransformHookResult,
  },
  serde_json::{self, Value},
};

use farmfe_macro_plugin::farm_plugin;

use fervid::{CompileOptions, PropsDestructureConfig};

mod consts;
pub mod descriptor;
mod filter;
mod options;
mod styles;

use crate::consts::{CE_VUE_SUFFIX, VUE_MODULE_TYPE, VUE_QUERY_KEY};
use crate::descriptor::{
  content_hash, CustomBlockDescriptor, DescriptorCache, SfcDescriptor, StyleDescriptor,
};
use crate::filter::{CustomElementFilter, Filter};
use crate::options::VuePluginOptions;
use crate::styles::{
  custom_block_virtual_id, lang_to_module_type, style_virtual_id, StyleEntry, StyleRegistry,
};

#[farm_plugin]
pub struct FarmPluginVue {
  filter: Filter,
  custom_element_filter: CustomElementFilter,
  is_prod: bool,
  ssr: bool,
  source_map: bool,
  props_destructure: Option<bool>,
  styles: StyleRegistry,
  descriptors: DescriptorCache,
}

impl FarmPluginVue {
  pub fn new(config: &Config, options: String) -> Self {
    let opts: VuePluginOptions = if options.trim().is_empty() {
      VuePluginOptions::default()
    } else {
      serde_json::from_str(&options).unwrap_or_default()
    };

    let features = opts.features.clone().unwrap_or_default();

    let is_prod = opts
      .is_production
      .unwrap_or(matches!(config.mode, Mode::Production));

    let filter = Filter::new(opts.include.clone(), opts.exclude.clone());
    let custom_element_filter =
      CustomElementFilter::new(opts.custom_element.clone(), features.custom_element.clone());

    Self {
      filter,
      custom_element_filter,
      is_prod,
      ssr: opts.ssr.unwrap_or(false),
      source_map: opts.source_map.unwrap_or(true),
      props_destructure: features.props_destructure,
      styles: StyleRegistry::default(),
      descriptors: DescriptorCache::default(),
    }
  }

  fn is_custom_element(&self, resolved_path: &str) -> bool {
    if resolved_path.ends_with(CE_VUE_SUFFIX) {
      return true;
    }
    self.custom_element_filter.matches(resolved_path)
  }

  fn matches_filter(&self, resolved_path: &str) -> bool {
    self.filter.matches(resolved_path)
  }

  #[doc(hidden)]
  pub fn cached_descriptor_for_test(&self, module_id: &str) -> Option<SfcDescriptor> {
    self.descriptors.get(module_id)
  }
}

/// True iff `query` contains the `vue` flag (we only inspect the key
/// because Farm parses query strings into `(key, value)` pairs and empty
/// flags surface as `("vue", "")`).
fn has_vue_query(query: &[(String, String)]) -> bool {
  query.iter().any(|(k, _)| k == VUE_QUERY_KEY)
}

fn normalize_path_for_import(path: &str) -> String {
  path.replace('\\', "/")
}

impl Plugin for FarmPluginVue {
  fn name(&self) -> &str {
    "FarmPluginVue"
  }

  fn priority(&self) -> i32 {
    // Higher than the default so we win against generic loaders for `.vue`.
    101
  }

  fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    // Inject the three Vue runtime define flags if the user has not
    // already set them. We default to the `unplugin-vue` shape:
    // optionsAPI = true, prodDevtools = false, prodHydrationMismatchDetails = false.
    let defaults: [(&str, Value); 3] = [
      ("__VUE_OPTIONS_API__", Value::Bool(true)),
      ("__VUE_PROD_DEVTOOLS__", Value::Bool(false)),
      (
        "__VUE_PROD_HYDRATION_MISMATCH_DETAILS__",
        Value::Bool(false),
      ),
    ];
    for (key, default) in defaults {
      config.define.entry(key.to_string()).or_insert(default);
    }

    // De-duplicate `vue` so users don't accidentally ship two copies, but
    // skip it when targeting Node so SSR builds can pull in a different
    // server runtime.
    if !config.output.target_env.is_node() {
      let dedupe = &mut config.resolve.dedupe;
      if !dedupe.iter().any(|d| d == "vue") {
        dedupe.push("vue".to_string());
      }
    }

    Ok(Some(()))
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    // Virtual style sub-block requests, e.g. `foo.vue?vue&type=style&idx=0&lang=css`.
    if has_vue_query(&param.query) {
      let normalized_module_id = normalize_path_for_import(&param.module_id);
      if let Some(entry) = self
        .styles
        .get(&param.module_id)
        .or_else(|| self.styles.get(&normalized_module_id))
      {
        return Ok(Some(PluginLoadHookResult {
          content: entry.content,
          module_type: entry.module_type,
          source_map: None,
        }));
      }
      return Ok(None);
    }

    // Main `.vue` file load: read once and tag with the `vue` module type
    // so our transform hook (and only ours) picks it up.
    if param.resolved_path.ends_with(".vue") && self.matches_filter(param.resolved_path) {
      let content =
        std::fs::read_to_string(param.resolved_path).map_err(|e| CompilationError::LoadError {
          resolved_path: param.resolved_path.to_string(),
          source: Some(Box::new(e)),
        })?;
      return Ok(Some(PluginLoadHookResult {
        content,
        module_type: ModuleType::Custom(VUE_MODULE_TYPE.to_string()),
        source_map: None,
      }));
    }

    Ok(None)
  }

  fn transform(
    &self,
    param: &PluginTransformHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    let is_vue = matches!(&param.module_type, ModuleType::Custom(t) if t == VUE_MODULE_TYPE);
    if !is_vue {
      return Ok(None);
    }

    let is_ce = self.is_custom_element(param.resolved_path);

    let compile_result = fervid::compile(
      &param.content,
      CompileOptions {
        filename: Cow::Borrowed(param.resolved_path),
        id: Cow::Owned(param.module_id.clone()),
        is_prod: Some(self.is_prod),
        is_custom_element: Some(is_ce),
        ssr: Some(self.ssr),
        props_destructure: self.props_destructure.map(|enabled| {
          if enabled {
            PropsDestructureConfig::True
          } else {
            PropsDestructureConfig::False
          }
        }),
        gen_default_as: None,
        source_map: Some(self.source_map),
        // We expose only a boolean toggle in v0; the richer
        // `TransformAssetUrlsConfig::EnabledOptions(...)` shape is left for
        // a follow-up because its inner `Rc` makes it `!Send + !Sync` and
        // would require parsing JSON into a side-allocated cache.
        transform_asset_urls: None,
      },
    )
    .map_err(|e| CompilationError::TransformError {
      resolved_path: param.resolved_path.to_string(),
      msg: format!("[plugin-vue] fervid failed to compile SFC: {e:?}"),
    })?;

    // Register each emitted style/custom block under its virtual id and
    // prepend an `import` so it participates in the module graph.
    let import_base = normalize_path_for_import(param.resolved_path);
    let virtual_base = normalize_path_for_import(&param.module_id);
    let mut prepend = String::new();
    let mut style_descriptors = Vec::new();
    if !compile_result.styles.is_empty() {
      for (idx, style) in compile_result.styles.into_iter().enumerate() {
        let virtual_id = style_virtual_id(&virtual_base, idx, &style.lang, style.is_scoped);
        let module_type = lang_to_module_type(&style.lang);
        let style_content_hash = content_hash(&style.code);
        // The first `?` in the import path is the start of the query; the
        // rest is `vue&type=style&…`. Farm preserves the entire query on
        // both load and transform sides.
        prepend.push_str(&format!(
          "import {q}{base}?{query}{q};\n",
          q = '"',
          base = &import_base,
          query = virtual_id.split_once('?').map(|(_, q)| q).unwrap_or(""),
        ));
        self.styles.insert(
          virtual_id.clone(),
          StyleEntry {
            content: style.code,
            module_type,
          },
        );
        style_descriptors.push(StyleDescriptor {
          index: idx,
          lang: style.lang.trim().to_ascii_lowercase(),
          scoped: style.is_scoped,
          content_hash: style_content_hash,
          virtual_id,
        });
      }
    }

    let mut custom_block_descriptors = Vec::new();
    if !compile_result.other_assets.is_empty() {
      for (idx, asset) in compile_result.other_assets.into_iter().enumerate() {
        let virtual_id = custom_block_virtual_id(&virtual_base, idx, &asset.tag_name);
        let module_type = ModuleType::Custom(asset.tag_name.trim().to_ascii_lowercase());
        prepend.push_str(&format!(
          "import {q}{base}?{query}{q};\n",
          q = '"',
          base = &import_base,
          query = virtual_id.split_once('?').map(|(_, q)| q).unwrap_or(""),
        ));
        custom_block_descriptors.push(CustomBlockDescriptor {
          index: idx,
          tag_name: asset.tag_name.trim().to_ascii_lowercase(),
          content_hash: content_hash(&asset.content),
          virtual_id: virtual_id.clone(),
        });
        self.styles.insert(
          virtual_id,
          StyleEntry {
            content: asset.content,
            module_type,
          },
        );
      }
    }

    self.descriptors.insert(
      param.module_id.clone(),
      SfcDescriptor {
        source_hash: compile_result.file_hash,
        is_custom_element: is_ce,
        styles: style_descriptors,
        custom_blocks: custom_block_descriptors,
      },
    );

    let mut code = compile_result.code;
    if !prepend.is_empty() {
      code.insert_str(0, &prepend);
    }

    // Devtools metadata. fervid bakes `__hmrId` via its scope id during
    // codegen, but `__file` is plugin-orchestrator territory in every
    // mainstream SFC toolchain — append it in dev so Vue Devtools can
    // locate the component source.
    if !self.is_prod {
      code.push_str(&format!(
        "\nif (typeof _sfc_main !== 'undefined') _sfc_main.__file = {q}{file}{q};\n",
        q = '"',
        file = escape_double_quotes(param.resolved_path),
      ));
    }

    Ok(Some(PluginTransformHookResult {
      content: code,
      module_type: Some(ModuleType::Ts),
      source_map: compile_result.source_map,
      ignore_previous_source_map: true,
    }))
  }
}

fn escape_double_quotes(s: &str) -> String {
  s.replace('\\', "\\\\").replace('"', "\\\"")
}
