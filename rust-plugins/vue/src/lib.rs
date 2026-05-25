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
    PluginTransformHookResult, PluginUpdateModulesHookParam, UpdateType,
  },
  serde_json::{self, Value},
  swc_common::DUMMY_SP,
  swc_css_ast::{
    AttributeSelector, ComplexSelector, ComplexSelectorChildren, CompoundSelector, Ident,
    QualifiedRule, QualifiedRulePrelude, SubclassSelector, WqName,
  },
};

use farmfe_macro_plugin::farm_plugin;
use farmfe_toolkit::{
  css::{codegen_css_stylesheet, parse_css_stylesheet},
  swc_atoms::Atom,
  swc_css_visit::{VisitMut, VisitMutWith},
};

use fervid::{CompileOptions, PropsDestructureConfig};

mod consts;
pub mod descriptor;
mod filter;
mod options;
mod styles;

use crate::consts::{CE_VUE_SUFFIX, VUE_MODULE_TYPE, VUE_QUERY_KEY};
use crate::descriptor::{
  content_hash, narrow_virtual_updates, CustomBlockDescriptor, DescriptorCache, SfcDescriptor,
  StyleDescriptor,
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

  fn compile_sfc(
    &self,
    module_id: &str,
    resolved_path: &str,
    content: &str,
  ) -> farmfe_core::error::Result<CompiledSfc> {
    let is_ce = self.is_custom_element(resolved_path);

    let compile_result = fervid::compile(
      content,
      CompileOptions {
        filename: Cow::Borrowed(resolved_path),
        id: Cow::Owned(module_id.to_string()),
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
      resolved_path: resolved_path.to_string(),
      msg: format!("[plugin-vue] fervid failed to compile SFC: {e:?}"),
    })?;

    let import_base = normalize_path_for_import(resolved_path);
    let virtual_base = normalize_path_for_import(module_id);
    let mut prepend = String::new();
    let mut style_descriptors = Vec::new();
    let mut entries = Vec::new();

    for (idx, style) in compile_result.styles.into_iter().enumerate() {
      let scope_id = format!("data-v-{}", compile_result.file_hash);
      let virtual_id = style_virtual_id(
        &virtual_base,
        idx,
        &style.lang,
        style.is_scoped,
        Some(&scope_id),
      );
      let module_type = lang_to_module_type(&style.lang);
      let style_content_hash = content_hash(&style.code);
      prepend.push_str(&format!(
        "import {q}{base}?{query}{q};\n",
        q = '"',
        base = &import_base,
        query = virtual_id.split_once('?').map(|(_, q)| q).unwrap_or(""),
      ));
      entries.push((
        virtual_id.clone(),
        StyleEntry {
          content: style.code,
          module_type,
        },
      ));
      style_descriptors.push(StyleDescriptor {
        index: idx,
        lang: style.lang.trim().to_ascii_lowercase(),
        scoped: style.is_scoped,
        owner_module_id: virtual_base.clone(),
        owner_resolved_path: import_base.clone(),
        scope_id,
        content_hash: style_content_hash,
        virtual_id,
      });
    }

    let mut custom_block_descriptors = Vec::new();
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
      entries.push((
        virtual_id,
        StyleEntry {
          content: asset.content,
          module_type,
        },
      ));
    }

    let descriptor = SfcDescriptor {
      source_hash: compile_result.file_hash,
      main_content_hash: content_hash(&compile_result.code),
      is_custom_element: is_ce,
      styles: style_descriptors,
      custom_blocks: custom_block_descriptors,
    };

    Ok(CompiledSfc {
      code: compile_result.code,
      source_map: compile_result.source_map,
      prepend,
      descriptor,
      entries,
    })
  }

  fn register_compiled_sfc(&self, module_id: &str, resolved_path: &str, compiled: &CompiledSfc) {
    for (virtual_id, entry) in &compiled.entries {
      self.styles.insert(virtual_id.clone(), entry.clone());
    }

    self
      .descriptors
      .insert(module_id.to_string(), compiled.descriptor.clone());
    self.descriptors.insert(
      normalize_path_for_import(resolved_path),
      compiled.descriptor.clone(),
    );
  }

  fn rehydrate_sfc(&self, module_id: &str, resolved_path: &str) -> farmfe_core::error::Result<()> {
    let resolved_path = resolved_path
      .split_once('?')
      .map(|(path, _)| path)
      .unwrap_or(resolved_path);
    if !resolved_path.ends_with(".vue") || !self.matches_filter(resolved_path) {
      return Ok(());
    }

    let content =
      std::fs::read_to_string(resolved_path).map_err(|e| CompilationError::LoadError {
        resolved_path: resolved_path.to_string(),
        source: Some(Box::new(e)),
      })?;
    let module_id = module_id
      .split_once('?')
      .map(|(base, _)| base)
      .unwrap_or(module_id);
    let module_id = normalize_path_for_import(module_id);
    let compiled = self.compile_sfc(&module_id, resolved_path, &content)?;
    self.register_compiled_sfc(&module_id, resolved_path, &compiled);

    Ok(())
  }
}

struct CompiledSfc {
  code: String,
  source_map: Option<String>,
  prepend: String,
  descriptor: SfcDescriptor,
  entries: Vec<(String, StyleEntry)>,
}

/// True iff `query` contains the `vue` flag (we only inspect the key
/// because Farm parses query strings into `(key, value)` pairs and empty
/// flags surface as `("vue", "")`).
fn has_vue_query(query: &[(String, String)]) -> bool {
  query.iter().any(|(k, _)| k == VUE_QUERY_KEY)
}

fn query_value<'a>(query: &'a [(String, String)], key: &str) -> Option<&'a str> {
  query
    .iter()
    .find(|(query_key, _)| query_key == key)
    .map(|(_, value)| value.as_str())
}

fn scoped_style_scope_id(query: &[(String, String)]) -> Option<&str> {
  let is_scoped_style = has_vue_query(query)
    && query_value(query, "type") == Some("style")
    && query_value(query, "scoped") == Some("true");

  if is_scoped_style {
    query_value(query, "scopeId").filter(|scope_id| !scope_id.is_empty())
  } else {
    None
  }
}

struct VueScopeVisitor<'a> {
  scope_id: &'a str,
}

impl VisitMut for VueScopeVisitor<'_> {
  fn visit_mut_complex_selector(&mut self, selector: &mut ComplexSelector) {
    selector.visit_mut_children_with(self);

    for child in &mut selector.children {
      if let ComplexSelectorChildren::CompoundSelector(compound) = child {
        add_scope_attribute(compound, self.scope_id);
      }
    }
  }

  fn visit_mut_qualified_rule(&mut self, rule: &mut QualifiedRule) {
    if let QualifiedRulePrelude::SelectorList(selector_list) = &mut rule.prelude {
      selector_list.visit_mut_children_with(self);
    }

    rule.block.visit_mut_children_with(self);
  }
}

fn add_scope_attribute(compound: &mut CompoundSelector, scope_id: &str) {
  let already_scoped = compound.subclass_selectors.iter().any(|selector| {
    matches!(selector, SubclassSelector::Attribute(attribute) if attribute.name.value.value == scope_id)
  });

  if already_scoped {
    return;
  }

  let attribute = SubclassSelector::Attribute(Box::new(AttributeSelector {
    span: DUMMY_SP,
    name: WqName {
      span: DUMMY_SP,
      prefix: None,
      value: Ident {
        span: DUMMY_SP,
        value: Atom::from(scope_id),
        raw: None,
      },
    },
    matcher: None,
    value: None,
    modifier: None,
  }));

  let insert_index = compound
    .subclass_selectors
    .iter()
    .position(|selector| matches!(selector, SubclassSelector::PseudoElement(_)))
    .unwrap_or(compound.subclass_selectors.len());
  compound.subclass_selectors.insert(insert_index, attribute);
}

fn scope_compiled_css(
  css: String,
  scope_id: &str,
  resolved_path: &str,
) -> farmfe_core::error::Result<String> {
  let mut parsed = parse_css_stylesheet(resolved_path, Arc::new(css))?.ast;
  parsed.visit_mut_with(&mut VueScopeVisitor { scope_id });
  Ok(codegen_css_stylesheet(&parsed, false, None, false).0)
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
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    // Virtual style sub-block requests, e.g. `foo.vue?vue&type=style&idx=0&lang=css`.
    if has_vue_query(&param.query) {
      let normalized_module_id = normalize_path_for_import(&param.module_id);
      let entry = self
        .styles
        .get(&param.module_id)
        .or_else(|| self.styles.get(&normalized_module_id))
        .or_else(|| {
          self
            .rehydrate_sfc(&normalized_module_id, param.resolved_path)
            .ok()?;
          self
            .styles
            .get(&param.module_id)
            .or_else(|| self.styles.get(&normalized_module_id))
        });

      if let Some(entry) = entry {
        if !matches!(entry.module_type, ModuleType::Css) {
          let transformed = context.plugin_driver.transform(
            PluginTransformHookParam {
              module_id: param.module_id.clone(),
              content: entry.content.clone(),
              module_type: entry.module_type.clone(),
              resolved_path: param.resolved_path,
              query: param.query.clone(),
              meta: param.meta.clone(),
              source_map_chain: vec![],
            },
            context,
          )?;

          if transformed.module_type == Some(ModuleType::Css) {
            let content = if let Some(scope_id) = scoped_style_scope_id(&param.query) {
              scope_compiled_css(transformed.content, scope_id, param.resolved_path)?
            } else {
              transformed.content
            };

            return Ok(Some(PluginLoadHookResult {
              content,
              module_type: ModuleType::Css,
              source_map: None,
            }));
          }
        }

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

    let compiled = self.compile_sfc(&param.module_id, param.resolved_path, &param.content)?;
    self.register_compiled_sfc(&param.module_id, param.resolved_path, &compiled);

    let mut code = compiled.code;
    if !compiled.prepend.is_empty() {
      code.insert_str(0, &compiled.prepend);
    }

    if let Some(scope_id) = compiled
      .descriptor
      .styles
      .iter()
      .find(|style| style.scoped)
      .map(|style| &style.scope_id)
    {
      code.push_str(&format!(
        "\n{target} if (__farm_vue_sfc) __farm_vue_sfc.__scopeId = {q}{scope_id}{q};\n",
        target = sfc_runtime_target_statement(),
        q = '"',
        scope_id = escape_double_quotes(scope_id),
      ));
    }

    // Devtools metadata. fervid bakes `__hmrId` via its scope id during
    // codegen, but `__file` is plugin-orchestrator territory in every
    // mainstream SFC toolchain — append it in dev so Vue Devtools can
    // locate the component source.
    if !self.is_prod {
      code.push_str(&format!(
        "\n{target} if (__farm_vue_sfc) __farm_vue_sfc.__file = {q}{file}{q};\n",
        target = sfc_runtime_target_statement(),
        q = '"',
        file = escape_double_quotes(param.resolved_path),
      ));

      code.push_str(&vue_hmr_runtime_code(&normalize_path_for_import(
        param.resolved_path,
      )));
    }

    Ok(Some(PluginTransformHookResult {
      content: code,
      module_type: Some(ModuleType::Ts),
      source_map: compiled.source_map,
      ignore_previous_source_map: true,
    }))
  }

  fn update_modules(
    &self,
    params: &mut PluginUpdateModulesHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let mut next_paths = Vec::with_capacity(params.paths.len());

    for (path, update_type) in params.paths.drain(..) {
      if !matches!(update_type, UpdateType::Updated)
        || path.contains('?')
        || !path.ends_with(".vue")
        || !self.matches_filter(&path)
      {
        next_paths.push((path, update_type));
        continue;
      }

      let descriptor_key = normalize_path_for_import(&path);
      let Some(previous_descriptor) = self.descriptors.get(&descriptor_key) else {
        self.rehydrate_sfc(&descriptor_key, &path)?;
        next_paths.push((descriptor_key, update_type));
        continue;
      };
      let fallback_update_path = previous_descriptor
        .styles
        .first()
        .map(|style| style.owner_resolved_path.clone())
        .or_else(|| {
          previous_descriptor.custom_blocks.first().map(|block| {
            block
              .virtual_id
              .split_once('?')
              .map(|(base, _)| base)
              .unwrap_or(&path)
              .to_string()
          })
        })
        .unwrap_or_else(|| path.clone());

      let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => {
          next_paths.push((fallback_update_path, update_type));
          continue;
        }
      };

      let module_id = previous_descriptor
        .styles
        .first()
        .map(|style| {
          style
            .virtual_id
            .split_once('?')
            .map(|(base, _)| base)
            .unwrap_or(&path)
        })
        .or_else(|| {
          previous_descriptor.custom_blocks.first().map(|block| {
            block
              .virtual_id
              .split_once('?')
              .map(|(base, _)| base)
              .unwrap_or(&path)
          })
        })
        .unwrap_or(&path)
        .to_string();

      let compiled = self.compile_sfc(&module_id, &path, &content)?;
      let narrowed_updates = narrow_virtual_updates(&previous_descriptor, &compiled.descriptor);
      self.register_compiled_sfc(&module_id, &path, &compiled);

      if let Some(updates) = narrowed_updates {
        next_paths.extend(
          updates
            .into_iter()
            .map(|virtual_id| (virtual_id, UpdateType::Updated)),
        );
      } else {
        next_paths.push((fallback_update_path, update_type));
      }
    }

    params.paths = next_paths;
    Ok(Some(()))
  }
}

fn escape_double_quotes(s: &str) -> String {
  s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn sfc_runtime_target_statement() -> &'static str {
  "var __farm_vue_sfc = typeof _sfc_main !== 'undefined' ? _sfc_main : (typeof exports !== 'undefined' ? exports.default : undefined);"
}

fn vue_hmr_runtime_code(hmr_id: &str) -> String {
  format!(
    r#"
{target}
if (__farm_vue_sfc) __farm_vue_sfc.__hmrId = {q}{hmr_id}{q};
if (import.meta.hot && typeof window !== 'undefined' && window.__VUE_HMR_RUNTIME__ && __farm_vue_sfc) {{
  window.__VUE_HMR_RUNTIME__.createRecord(__farm_vue_sfc.__hmrId, __farm_vue_sfc);
  import.meta.hot.accept((mod) => {{
    if (!mod) return;
    var __farm_vue_updated = mod.default || (mod.exports && mod.exports.default);
    if (__farm_vue_updated) window.__VUE_HMR_RUNTIME__.reload(__farm_vue_updated.__hmrId, __farm_vue_updated);
  }});
}}
"#,
    target = sfc_runtime_target_statement(),
    q = '"',
    hmr_id = escape_double_quotes(hmr_id),
  )
}
