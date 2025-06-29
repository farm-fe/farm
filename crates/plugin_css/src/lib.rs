#![feature(box_patterns)]
#![feature(rustc_private)]
#![feature(trait_upcasting)]

use std::sync::Arc;

use farmfe_core::config::css::NameConversion;
use farmfe_core::config::custom::get_config_css_modules_local_conversion;
use farmfe_core::config::AliasItem;
use farmfe_core::module::meta_data::script::CommentsMetaData;
use farmfe_core::plugin::GeneratedResource;
use farmfe_core::resource::meta_data::ResourcePotMetaData;
use farmfe_core::{
  config::Config,
  context::CompilationContext,
  deserialize,
  enhanced_magic_string::collapse_sourcemap::{collapse_sourcemap_chain, CollapseSourcemapOptions},
  error::CompilationError,
  module::{module_graph::ModuleGraph, ModuleId, ModuleMetaData, ModuleType},
  parking_lot::Mutex,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginGenerateResourcesHookResult, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginParseHookParam, PluginResolveHookParam,
    PluginTransformHookResult, ResolveKind,
  },
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  serde_json, serialize,
  swc_css_ast::Stylesheet,
};
use farmfe_core::{DashMap, HashMap};
use farmfe_macro_cache_item::cache_item;
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::resolve::DYNAMIC_EXTENSION_PRIORITY;
use farmfe_toolkit::{
  fs::read_file_utf8,
  hash::sha256,
  regex::Regex,
  script::module_type_from_id,
  sourcemap::SourceMap as JsonSourceMap,
  sourcemap::{load_source_original_sourcemap, trace_module_sourcemap, SourceMap},
  swc_css_visit::VisitMutWith,
};
use farmfe_utils::{parse_query, relative, stringify_query};
use rkyv::Deserialize;
use source_replacer::SourceReplacer;

use crate::adapter::adapter_trait::{
  CodegenContext, CreateResourcePotMetadataContext, CssModuleReference, CssModulesContext,
  CssPluginAdapter, CssPluginProcesser, LightningCss, ParseOption,
};
mod adapter;

pub const FARM_CSS_MODULES: &str = "farm_css_modules";

lazy_static! {
  pub static ref FARM_CSS_MODULES_SUFFIX: Regex =
    Regex::new(&format!("(?:\\?|&){FARM_CSS_MODULES}")).unwrap();
}

mod dep_analyzer;
mod source_replacer;
pub mod transform_css_to_script;

pub struct FarmPluginCssResolve {}

impl FarmPluginCssResolve {
  pub fn new(_config: &Config) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginCssResolve {
  fn name(&self) -> &str {
    "FarmPluginCssResolve"
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    context: &Arc<CompilationContext>,
    hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    if let Some(caller) = &hook_context.caller {
      if caller.as_str() == "FarmPluginCss" {
        return Ok(None);
      }
    }

    if is_farm_css_modules(&param.source) {
      let split = param.source.split('?').collect::<Vec<&str>>();
      let strip_query_path = split[0].to_string();
      let query = parse_query(&param.source);

      return Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
        resolved_path: strip_query_path,
        query,
        ..Default::default()
      }));
    } else if matches!(param.kind, ResolveKind::CssAtImport | ResolveKind::CssUrl) {
      // if dep starts with '~', means it's from node_modules.
      let source = if let Some(striped_source) = param.source.strip_prefix('~') {
        striped_source.to_string()
      } else {
        param.source.clone()
      };
      // fix #1230
      let extensions = if matches!(param.kind, ResolveKind::CssAtImport) {
        let mut ext = vec!["css"];
        // fix #1450
        for e in ["sass", "scss", "less"] {
          if context.config.resolve.extensions.contains(&e.to_string()) {
            ext.insert(0, e)
          }
        }

        ext
      } else {
        vec![]
      };

      let resolve_css = |source: String| {
        if let Ok(Some(res)) = context.plugin_driver.resolve(
          &PluginResolveHookParam {
            source,
            ..param.clone()
          },
          context,
          &PluginHookContext {
            caller: Some("FarmPluginCss".to_string()),
            meta: HashMap::from_iter([(
              DYNAMIC_EXTENSION_PRIORITY.to_string(),
              serde_json::to_string(&extensions).unwrap(),
            )]),
          },
        ) {
          return Some(res);
        }

        None
      };
      // try relative path first
      if !source.starts_with('.') {
        if let Some(res) = resolve_css(format!("./{source}")) {
          return Ok(Some(res));
        }
      }
      // try original source in case it's in node_modules.
      if let Some(res) = resolve_css(source) {
        return Ok(Some(res));
      }
    }

    Ok(None)
  }
}

#[cache_item(farmfe_core)]
struct CssModulesCache {
  content_map: HashMap<String, Arc<String>>,
  sourcemap_map: HashMap<String, String>,
}

pub struct FarmPluginCss {
  css_modules_paths: Vec<Regex>,
  ast_map: Mutex<HashMap<String, (Stylesheet, CommentsMetaData)>>,
  content_map: DashMap<String, Arc<String>>,
  sourcemap_map: Mutex<HashMap<String, String>>,
  locals_conversion: NameConversion,
  css_processer: CssPluginProcesser,
}

// fn prefixer(stylesheet: &mut Stylesheet, css_prefixer_config: &CssPrefixerConfig) {
//   let mut prefixer = swc_css_prefixer::prefixer(swc_css_prefixer::options::Options {
//     env: css_prefixer_config.targets.clone(),
//   });
//   prefixer.visit_mut_stylesheet(stylesheet);
// }

impl Plugin for FarmPluginCss {
  fn name(&self) -> &str {
    "FarmPluginCss"
  }

  /// This plugin should be executed at last
  fn priority(&self) -> i32 {
    -99
  }

  /// Just load the cache, if the cache is invalidated, it will be reset when transform.
  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let cache = deserialize!(cache, CssModulesCache);
    for (k, v) in cache.content_map {
      self.content_map.insert(k, v);
    }

    let mut sourcemap_map = self.sourcemap_map.lock();

    for (k, v) in cache.sourcemap_map {
      sourcemap_map.insert(k, v);
    }

    Ok(Some(()))
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if is_farm_css_modules(&param.module_id) {
      return Ok(Some(PluginLoadHookResult {
        content: (**self.content_map.get(&param.module_id).unwrap().value()).clone(),
        module_type: ModuleType::Custom(FARM_CSS_MODULES.to_string()),
        source_map: None,
      }));
    };
    // for internal css plugin, we do not support ?xxx.css. It should be handled by external plugins.
    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if matches!(module_type, ModuleType::Css) {
        let content = read_file_utf8(param.resolved_path)?;

        let map =
          load_source_original_sourcemap(&content, param.resolved_path, "/*# sourceMappingURL");

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
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
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

      // css modules
      if enable_css_modules && self.is_path_match_css_modules(&param.module_id) {
        let mut query = param.query.clone();
        query.push((FARM_CSS_MODULES.to_string(), "".to_string()));
        let query_string = stringify_query(&query);

        let css_modules_module_id =
          ModuleId::new(param.resolved_path, &query_string, &context.config.root);

        let content = Arc::new(param.content.clone());

        // css modules
        let css_modules = self.css_processer.css_modules(CssModulesContext {
          module_id: &css_modules_module_id,
          content,
          context,
          content_map: &self.content_map,
        })?;

        let mut dynamic_import_of_composes = HashMap::default();
        let mut export_names = Vec::new();

        if let Some(v) = css_modules {
          for (name, classes) in v {
            let mut after_transform_classes = Vec::new();
            for v in classes {
              match v {
                CssModuleReference::Local { name } => {
                  after_transform_classes.push(name);
                }
                CssModuleReference::Global { name } => {
                  after_transform_classes.push(name);
                }
                CssModuleReference::Dependency { name, specifier } => {
                  let v = dynamic_import_of_composes
                    .entry(specifier.to_string())
                    .or_insert(format!("f_{}", sha256(specifier.as_bytes(), 5)));
                  after_transform_classes.push(format!("${{{}[\"{}\"]}}", v, name));
                }
              }
            }

            export_names.push((
              self.locals_conversion.transform(&name),
              after_transform_classes,
            ));
          }
        }

        export_names.sort_by_key(|e| e.0.to_string());

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
            .iter()
            .map(|(name, classes)| format!("\"{}\": `{}`", name, classes.join(" ").trim()))
            .collect::<Vec<String>>()
            .join(",")
        );

        // collapse sourcemap chain
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

        Ok(Some(PluginTransformHookResult {
          content: code,
          module_type: Some(ModuleType::Js),
          source_map: None,
          ignore_previous_source_map: true,
        }))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  fn parse(
    &self,
    param: &PluginParseHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ModuleMetaData>> {
    if matches!(param.module_type, ModuleType::Css) {
      let meta = self
        .css_processer
        .create_module_data(&param.module_id.to_string(), param.content.clone())?;

      Ok(Some(meta))
    } else {
      Ok(None)
    }
  }

  fn process_module(
    &self,
    param: &mut farmfe_core::plugin::PluginProcessModuleHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let enable_prefixer = context.config.css.prefixer.is_some();

    if enable_prefixer {
      return self
        .css_processer
        .prefixer(&mut param.meta, &context.config.css);
    }

    Ok(None)
  }

  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if param.module.module_type == ModuleType::Css {
      param
        .deps
        .extend(self.css_processer.analyze_deps(&param.module.meta, context));
    }

    Ok(None)
  }

  fn build_end(&self, context: &Arc<CompilationContext>) -> farmfe_core::error::Result<Option<()>> {
    if !context.config.mode.is_dev() || !context.config.output.target_env.is_browser() {
      return Ok(None);
    }

    // transform all css to script
    let css_modules = context
      .module_graph
      .write()
      .modules()
      .into_iter()
      .filter_map(|m| matches!(m.module_type, ModuleType::Css).then(|| m.id.clone()))
      .collect::<Vec<ModuleId>>();

    transform_css_to_script::transform_css_to_script_modules(css_modules, context)?;

    Ok(Some(()))
  }

  fn module_graph_updated(
    &self,
    param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let mut module_ids = param.updated_modules_ids.clone();
    module_ids.extend(param.added_modules_ids.clone());
    transform_css_to_script::transform_css_to_script_modules(module_ids, context)?;

    Ok(Some(()))
  }

  fn render_resource_pot(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      let module_graph = context.module_graph.read();

      let mut modules = vec![];
      let mut module_execution_order = HashMap::default();

      for module_id in resource_pot.modules() {
        let module = module_graph.module(module_id).unwrap();
        module_execution_order.insert(&module.id, module.execution_order);
        modules.push(module);
      }

      let meta =
        self
          .css_processer
          .create_resource_pot_metadata(CreateResourcePotMetadataContext {
            resource_pot,
            context,
            modules,
            module_execution_order: &module_execution_order,
            module_graph: &module_graph,
          })?;

      Ok(Some(meta))
    } else {
      Ok(None)
    }
  }

  fn generate_resources(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginGenerateResourcesHookResult>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      let (css_code, src_map) = self.css_processer.codegen(CodegenContext {
        context,
        resource_pot: &resource_pot,
      })?;

      let resource = Resource {
        name: resource_pot.name.to_string(),
        name_hash: resource_pot.modules_name_hash.clone(),
        bytes: css_code.into_bytes(),
        emitted: false,
        should_transform_output_filename: true,
        resource_type: ResourceType::Css,
        origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
        meta: Default::default(),
      };
      let mut source_map = None;

      if let Some(src_map) = src_map {
        let module_graph = context.module_graph.read();
        let sourcemap = SourceMap::from_slice(src_map.as_bytes()).unwrap();
        // trace sourcemap chain of each module
        let sourcemap = trace_module_sourcemap(sourcemap, &module_graph, &context.config.root);

        let mut chain = resource_pot
          .source_map_chain
          .iter()
          .map(|s| JsonSourceMap::from_slice(s.as_bytes()).unwrap())
          .collect::<Vec<_>>();
        chain.push(sourcemap);
        // collapse sourcemap chain
        let sourcemap = collapse_sourcemap_chain(
          chain,
          CollapseSourcemapOptions {
            inline_content: true,
            remap_source: None,
          },
        );

        let mut buf = vec![];
        sourcemap
          .to_writer(&mut buf)
          .map_err(|e| CompilationError::RenderScriptModuleError {
            id: resource_pot.id.to_string(),
            source: Some(Box::new(e)),
          })?;
        let sourcemap = String::from_utf8(buf).unwrap();
        let ty = ResourceType::SourceMap(resource_pot.id.to_string());
        source_map = Some(Resource {
          name: resource_pot.name.to_string(),
          name_hash: resource_pot.modules_name_hash.clone(),
          bytes: sourcemap.into_bytes(),
          emitted: false,
          should_transform_output_filename: true,
          resource_type: ty,
          origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
          meta: Default::default(),
        });
      }

      Ok(Some(PluginGenerateResourcesHookResult {
        resources: vec![GeneratedResource {
          resource,
          source_map,
        }],
      }))
    } else {
      Ok(None)
    }
  }

  fn write_plugin_cache(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    if !self.content_map.is_empty() || !self.sourcemap_map.lock().is_empty() {
      let cache = CssModulesCache {
        content_map: self.content_map.clone().into_iter().collect(),
        sourcemap_map: self.sourcemap_map.lock().clone(),
      };

      Ok(Some(serialize!(&cache)))
    } else {
      Ok(None)
    }
  }
}

impl FarmPluginCss {
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
      ast_map: Mutex::new(Default::default()),
      content_map: Default::default(),
      sourcemap_map: Mutex::new(Default::default()),
      locals_conversion: get_config_css_modules_local_conversion(config),
      css_processer: CssPluginProcesser {
        // adapter: CssPluginAdapter::SwcCss,
        adapter: CssPluginAdapter::LightningCss(LightningCss {}),
        ast_map: Default::default(),
      },
    }
  }

  pub fn is_path_match_css_modules(&self, path: &str) -> bool {
    self
      .css_modules_paths
      .iter()
      .any(|regex| regex.is_match(path))
  }
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

pub fn source_replace(
  stylesheet: &mut Stylesheet,
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  resources_map: &HashMap<String, Resource>,
  public_path: String,
  alias: Vec<AliasItem>,
) {
  let mut source_replacer = SourceReplacer::new(
    module_id.clone(),
    module_graph,
    resources_map,
    public_path,
    alias,
  );
  stylesheet.visit_mut_with(&mut source_replacer);
}
