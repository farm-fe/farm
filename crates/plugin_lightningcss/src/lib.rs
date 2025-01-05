use std::sync::Arc;

use farmfe_core::config::css::NameConversion;
use farmfe_core::config::custom::get_config_css_modules_local_conversion;
use farmfe_core::enhanced_magic_string::collapse_sourcemap::{
  collapse_sourcemap_chain, CollapseSourcemapOptions,
};
use farmfe_core::module::CustomModuleMetaData;
use farmfe_core::module::{ModuleId, ModuleMetaData, ModuleType};
use farmfe_core::plugin::{PluginLoadHookResult, PluginTransformHookResult, ResolveKind};
use farmfe_core::serde::Serialize;
use farmfe_core::serde_json;
use farmfe_core::swc_common::plugin::diagnostics::PluginCorePkgDiagnosticsResolver;
use farmfe_core::{
  config::Config, context::CompilationContext, deserialize, parking_lot::Mutex, plugin::Plugin,
};
use farmfe_macro_cache_item::cache_item;
use farmfe_toolkit::common::load_source_original_source_map;
use farmfe_toolkit::fs::read_file_utf8;
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::regex::Regex;
use farmfe_toolkit::resolve::path_start_with_alias::is_start_with_alias;
use farmfe_toolkit::script::module_type_from_id;
use farmfe_utils::{relative, stringify_query};
use lightningcss::css_modules::{CssModuleExports, CssModuleReference};
use lightningcss::printer::PrinterOptions;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, StyleSheet};
use lightningcss::targets::{Features, Targets};
use lightningcss::values::url::Url;
use lightningcss::visit_types;
use lightningcss::visitor::{Visit, VisitTypes, Visitor};
use rkyv::Deserialize;
use lightningcss::bundler::{Bundler, FileProvider};
use std::path::Path;
pub const FARM_CSS_MODULES: &str = "farm_lightning_css_modules";
use farmfe_toolkit::sourcemap::SourceMap;
use lightningcss::traits::{IntoOwned, ToCss};
use std::collections::HashMap;

use farmfe_core::config::{AliasItem, TargetEnv};
use farmfe_core::plugin::PluginAnalyzeDepsHookResultEntry;

mod transform_css_to_script;
mod source_replacer;

fn to_static(
  stylesheet: StyleSheet,
  options: ParserOptions<'static, 'static>,
) -> StyleSheet<'static, 'static> {
  let sources = stylesheet.sources.clone();
  let rules = stylesheet.rules.clone().into_owned();

  StyleSheet::new(sources, rules, options)
}

lazy_static! {
  pub static ref FARM_CSS_MODULES_SUFFIX: Regex =
    Regex::new(&format!("(?:\\?|&){FARM_CSS_MODULES}")).unwrap();
}

fn is_farm_css_modules(path: &str) -> bool {
  FARM_CSS_MODULES_SUFFIX.is_match(path)
}

fn is_farm_css_modules_type(module_type: &ModuleType) -> bool {
  if let ModuleType::Custom(c) = module_type {
    return c.as_str() == FARM_CSS_MODULES;
  }

  false
}

pub fn is_source_ignored(source: &str) -> bool {
  source.starts_with("http://")
    || source.starts_with("https://")
    || source.starts_with("/")
    || source.starts_with("data:")
    || source.starts_with('#')
}
struct DepVisitor {
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
  alias: Vec<AliasItem>,
}

impl DepVisitor {
  pub fn new(alias: Vec<AliasItem>) -> Self {
    Self {
      deps: vec![],
      alias,
    }
  }

  fn insert_dep(&mut self, dep: PluginAnalyzeDepsHookResultEntry) -> bool {
    // ignore http and /
    if is_source_ignored(&dep.source) && !is_start_with_alias(&self.alias, &dep.source) {
      return false;
    }

    self.deps.push(dep);
    true
  }
}

impl<'i> Visitor<'i> for DepVisitor {
  type Error = Infallible;

  fn visit_types(&self) -> VisitTypes {
    visit_types!(URLS | LENGTHS)
  }

  fn visit_url(&mut self, url: &mut Url<'i>) -> Result<(), Self::Error> {
    self.insert_dep(PluginAnalyzeDepsHookResultEntry {
      source: url.url.to_string(),
      kind: ResolveKind::CssUrl,
    });
    Ok(())
  }
}

#[cache_item]
struct LightingCssModuleMetaData {
  pub ast: String,
}

impl Clone for LightingCssModuleMetaData {
  fn clone(&self) -> Self {
    Self {
      ast: self.ast.clone(),
    }
  }
}

fn flatten_exports(exports: &CssModuleExports) -> HashMap<String, String> {
  let mut res = HashMap::new();
  for (name, export) in exports {
    let mut classes = export.name.clone();
    for composes in &export.composes {
      classes.push(' ');
      classes.push_str(match composes {
        CssModuleReference::Local { name } => name,
        CssModuleReference::Global { name } => name,
        _ => unreachable!(),
      })
    }
    res.insert(name.clone(), classes);
  }
  res
}

#[cache_item]
struct LightningCssModulesCache {
  content_map: HashMap<String, String>,
  sourcemap_map: HashMap<String, String>,
}

pub struct FarmPluginLightningCss {
  css_modules_paths: Vec<Regex>,
  content_map: Mutex<HashMap<String, String>>,
  sourcemap_map: Mutex<HashMap<String, String>>,
  locals_conversion: NameConversion,
  ast_map: Mutex<HashMap<String, String>>,
}

impl Plugin for FarmPluginLightningCss {
  fn name(&self) -> &str {
    "FarmPluginLightningCss"
  }

  fn priority(&self) -> i32 {
    -99
  }

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let cache = deserialize!(cache, LightningCssModulesCache);
    let mut content_map = self.content_map.lock();

    for (k, v) in cache.content_map {
      content_map.insert(k, v);
    }

    let mut sourcemap_map = self.sourcemap_map.lock();

    for (k, v) in cache.sourcemap_map {
      sourcemap_map.insert(k, v);
    }

    Ok(Some(()))
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if is_farm_css_modules(&param.module_id) {
      return Ok(Some(PluginLoadHookResult {
        content: self
          .content_map
          .lock()
          .get(&param.module_id)
          .unwrap()
          .clone(),
        module_type: ModuleType::Custom(FARM_CSS_MODULES.to_string()),
        source_map: None,
      }));
    }
    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if matches!(module_type, ModuleType::Css) {
        let content = read_file_utf8(param.resolved_path)?;

        let map =
          load_source_original_source_map(&content, param.resolved_path, "/*# sourceMappingURL");
        return Ok(Some(PluginLoadHookResult {
          content,
          module_type,
          source_map: map,
        }));
      }
    }

    Ok(None)
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<PluginTransformHookResult>> {
    if is_farm_css_modules_type(&param.module_type) {
      return Ok(Some(PluginTransformHookResult {
        content: param.content.clone(),
        module_type: Some(ModuleType::Css),
        source_map: self.sourcemap_map.lock().get(&param.module_id).cloned(),
        ignore_previous_source_map: false,
      }));
    }
    if matches!(param.module_type, ModuleType::Css) {
      let enable_css_modules = context.config.css.modules.is_some();

      if enable_css_modules && self.is_path_match_css_modules(&param.module_id) {}

      let mut query = param.query.clone();
      query.push((FARM_CSS_MODULES.to_string(), "".to_string()));
      let query_string = stringify_query(&query);

      let css_modules_module_id =
        ModuleId::new(param.resolved_path, &query_string, &context.config.root);

      let cache_id = css_modules_module_id.to_string();

      let dynamic_import_of_composes: HashMap<String, String> = HashMap::default();

      let fs = FileProvider::new();
      let mut bundler = Bundler::new(
        &fs,
        None,
        ParserOptions {
          ..Default::default()
        },
      );

      let ast = bundler.bundle(Path::new(&param.resolved_path)).unwrap();
      let stylesheet = ast.to_css(Default::default()).unwrap();
      self
        .ast_map
        .lock()
        .insert(cache_id, serde_json::to_string(&ast).unwrap());

      let mut export_names = HashMap::new();

      if let Some(exports) = stylesheet.exports {
        export_names = flatten_exports(&exports);
      }

      let code = format!(
        r#"
  import "{}";
  {}
  export default {{{}}}
  "#,
        css_modules_module_id.to_string(),
        dynamic_import_of_composes
          .into_iter()
          .fold(Vec::new(), |mut acc, (from, name)| {
            acc.push(format!("import {name} from \"{from}\""));
            acc
          })
          .join(";\n"),
        export_names
          .keys()
          .fold(Vec::new(), |mut acc, name| {
            acc.push(format!("{}: {}", name, export_names.get(name).unwrap()));
            acc
          })
          .join(",\n")
      );

      if !param.source_map_chain.is_empty() {
        let source_map_chain = param
          .source_map_chain
          .iter()
          .map(|s| SourceMap::from_slice(s.as_bytes()).expect("failed to parse sourcemap"))
          .collect::<Vec<_>>();

        let root = context.config.root.clone();
        let collapsed_sourcemap = collapse_sourcemap_chain(
          source_map_chain,
          CollapseSourcemapOptions {
            remap_source: Some(Box::new(move |src| format!("/{}", relative(&root, src)))),
            ..Default::default()
          },
        );
        let mut buf = vec![];
        collapsed_sourcemap
          .to_writer(&mut buf)
          .expect("failed to write sourcemap");
        let map = String::from_utf8(buf).unwrap();
        self
          .sourcemap_map
          .lock()
          .insert(css_modules_module_id.to_string(), map);
      }

      return Ok(Some(PluginTransformHookResult {
        content: code,
        module_type: Some(ModuleType::Js),
        source_map: None,
        ignore_previous_source_map: true,
      }));
    }

    Ok(None)
  }

  fn parse(
    &self,
    param: &farmfe_core::plugin::PluginParseHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::module::ModuleMetaData>> {
    if matches!(param.module_type, ModuleType::Css) {
      let stylesheet = if is_farm_css_modules(&param.module_id.to_string()) {
        let cached_ast_str = self
          .ast_map
          .lock()
          .remove(&param.module_id.to_string())
          .unwrap_or_else(|| panic!("ast not found {:?}", param.module_id.to_string()));

        let ast: StyleSheet = serde_json::from_str(&cached_ast_str).unwrap();
        ast
      } else {
        let fs = FileProvider::new();
        let mut bundler = Bundler::new(
          &fs,
          None,
          ParserOptions {
            ..Default::default()
          },
        );

        let ast = bundler.bundle(Path::new(&param.resolved_path)).unwrap();
        ast
      };

      let ast_str = serde_json::to_string(&stylesheet).unwrap();

      let meta = ModuleMetaData::Custom(Box::new(LightingCssModuleMetaData { ast: ast_str }) as _);
      return Ok(Some(meta));
    } else {
      Ok(None)
    }
  }
  fn config(&self, _config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }
  fn build_start(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }
  fn resolve(
    &self,
    _param: &farmfe_core::plugin::PluginResolveHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    Ok(None)
  }

  fn process_module(
    &self,
    param: &mut farmfe_core::plugin::PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let enable_prefix = context.config.css.prefixer.is_some();
    let mut css_stylesheet = match &mut param.meta {
      ModuleMetaData::Custom(meta) => {
        let meta = meta.downcast_ref::<LightingCssModuleMetaData>().unwrap();
        let ast: StyleSheet = serde_json::from_str(&meta.ast).unwrap();
        ast
      }
      _ => return Ok(None),
    };

    if enable_prefix {
      // TODO prefix css
      css_stylesheet
        .minify(MinifyOptions {
          targets: Targets {
            include: Features::VendorPrefixes,
            ..Default::default()
          },
          ..Default::default()
        })
        .unwrap();

      let css_str = serde_json::to_string(&css_stylesheet).unwrap();
      param.meta =
        ModuleMetaData::Custom(Box::new(LightingCssModuleMetaData { ast: css_str }) as _);
      return Ok(None);
    }

    return Ok(None);
  }

  fn analyze_deps(
    &self,
    param: &mut farmfe_core::plugin::PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if param.module.module_type == ModuleType::Css {
      let meta: &mut String = param.module.meta.as_custom_mut().downcast_mut();
      let ast: StyleSheet = serde_json::from_str(&meta.ast).unwrap();

      let mut dep_analyzer = DepVisitor::new(context.config.resolve.alias.clone());
      ast.visit(&mut dep_analyzer);
      let rules = ast.rules;

      for rule in rules.0.iter() {
        if let CssRule::Import(import) = rule {
          let url = import.url;
          dep_analyzer.insert_dep(PluginAnalyzeDepsHookResultEntry {
            source: url.to_string(),
            kind: ResolveKind::CssAtImport,
          });
        }
      }
      param.deps.extend(dep_analyzer.deps);
    }
    Ok(None)
  }

  fn finalize_module(
    &self,
    _param: &mut farmfe_core::plugin::PluginFinalizeModuleHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn build_end(&self, context: &Arc<CompilationContext>) -> farmfe_core::error::Result<Option<()>> {
    if !matches!(context.config.mode, farmfe_core::config::Mode::Development)
      || !matches!(context.config.output.target_env, TargetEnv::Browser)
    {
      return Ok(None);
    }
    let css_modules = context.module_graph.write().modules().into_iter()
    .filter_map(|m| {
      if matches!(m.module_type, ModuleType::Css) {
        Some(m.id.clone())
      } else {
        None
      }
    })
    .collect::<Vec<ModuleId>>();

    transform_css_to_script::transform_css_to_script_modules(css_modules, context)?;

    Ok(None)
  }

  fn generate_start(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn optimize_module_graph(
    &self,
    _module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn analyze_module_graph(
    &self,
    _module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::module::module_group::ModuleGroupGraph>> {
    Ok(None)
  }

  fn partial_bundling(
    &self,
    _modules: &Vec<ModuleId>,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<farmfe_core::resource::resource_pot::ResourcePot>>> {
    Ok(None)
  }

  fn process_resource_pots(
    &self,
    _resource_pots: &mut Vec<&mut farmfe_core::resource::resource_pot::ResourcePot>,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn render_start(
    &self,
    _config: &Config,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn render_resource_pot_modules(
    &self,
    _resource_pot: &farmfe_core::resource::resource_pot::ResourcePot,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::resource::resource_pot::ResourcePotMetaData>>
  {
    Ok(None)
  }

  fn render_resource_pot(
    &self,
    _resource_pot: &farmfe_core::plugin::PluginRenderResourcePotHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginRenderResourcePotHookResult>>
  {
    Ok(None)
  }

  fn augment_resource_hash(
    &self,
    _render_pot_info: &farmfe_core::resource::resource_pot::ResourcePotInfo,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<String>> {
    Ok(None)
  }

  fn optimize_resource_pot(
    &self,
    _resource: &mut farmfe_core::resource::resource_pot::ResourcePot,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn generate_resources(
    &self,
    _resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
    _context: &Arc<CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginGenerateResourcesHookResult>>
  {
    Ok(None)
  }

  fn process_generated_resources(
    &self,
    _resources: &mut farmfe_core::plugin::PluginGenerateResourcesHookResult,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn handle_entry_resource(
    &self,
    _resource: &mut farmfe_core::plugin::PluginHandleEntryResourceHookParams,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn finalize_resources(
    &self,
    _param: &mut farmfe_core::plugin::PluginFinalizeResourcesHookParams,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn generate_end(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn finish(
    &self,
    _stat: &farmfe_core::stats::Stats,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn update_modules(
    &self,
    _params: &mut farmfe_core::plugin::PluginUpdateModulesHookParams,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn module_graph_updated(
    &self,
    _param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParams,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn update_finished(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
  }

  fn handle_persistent_cached_module(
    &self,
    _module: &farmfe_core::module::Module,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<bool>> {
    Ok(None)
  }

  fn write_plugin_cache(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    Ok(None)
  }
}

impl FarmPluginLightningCss {
  pub fn new(config: &Config) -> Self {
    Self {
      css_modules_paths: config
        .css
        .modules
        .as_ref()
        .map(|item| {
          item
            .paths
            .iter()
            .map(|item| Regex::new(item).expect("Config `css.modules.paths` is not valid Regex"))
            .collect()
        })
        .unwrap_or_default(),
      content_map: Mutex::new(Default::default()),
      sourcemap_map: Mutex::new(Default::default()),
      locals_conversion: get_config_css_modules_local_conversion(config),
      ast_map: Mutex::new(Default::default()),
    }
  }

  pub fn is_path_match_css_modules(&self, path: &str) -> bool {
    self
      .css_modules_paths
      .iter()
      .any(|regex| regex.is_match(path))
  }
}
