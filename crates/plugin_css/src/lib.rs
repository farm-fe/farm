#![feature(box_patterns)]

use std::sync::Arc;

use dep_analyzer::DepAnalyzer;
use farmfe_core::config::css::NameConversion;
use farmfe_core::config::custom::get_config_css_modules_local_conversion;
use farmfe_core::config::AliasItem;
use farmfe_core::module::meta_data::css::CssModuleMetaData;
use farmfe_core::module::meta_data::script::CommentsMetaData;
use farmfe_core::plugin::GeneratedResource;
use farmfe_core::resource::meta_data::css::CssResourcePotMetaData;
use farmfe_core::resource::meta_data::ResourcePotMetaData;
use farmfe_core::swc_common::DUMMY_SP;
use farmfe_core::HashMap;
use farmfe_core::{
  config::{Config, CssPrefixerConfig, TargetEnv},
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
  rayon::prelude::*,
  resource::{
    resource_pot::{ResourcePot, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  serde_json, serialize,
  swc_css_ast::Stylesheet,
};
use farmfe_macro_cache_item::cache_item;
use farmfe_toolkit::css::{merge_css_sourcemap, ParseCssModuleResult};
use farmfe_toolkit::lazy_static::lazy_static;
use farmfe_toolkit::resolve::DYNAMIC_EXTENSION_PRIORITY;
use farmfe_toolkit::script::swc_try_with::try_with;
use farmfe_toolkit::sourcemap::load_source_original_sourcemap;
use farmfe_toolkit::sourcemap::{trace_module_sourcemap, SourceMap};
use farmfe_toolkit::{
  css::{codegen_css_stylesheet, parse_css_stylesheet},
  fs::read_file_utf8,
  hash::sha256,
  regex::Regex,
  script::module_type_from_id,
  sourcemap::SourceMap as JsonSourceMap,
  swc_atoms::JsWord,
  swc_css_modules::{compile, CssClassName, TransformConfig},
  swc_css_prefixer,
  swc_css_visit::{VisitMut, VisitMutWith, VisitWith},
};
use farmfe_utils::{parse_query, relative, stringify_query};
use rkyv::Deserialize;
use source_replacer::SourceReplacer;

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
  content_map: HashMap<String, String>,
  sourcemap_map: HashMap<String, String>,
}

pub struct FarmPluginCss {
  css_modules_paths: Vec<Regex>,
  ast_map: Mutex<HashMap<String, (Stylesheet, CommentsMetaData)>>,
  content_map: Mutex<HashMap<String, String>>,
  sourcemap_map: Mutex<HashMap<String, String>>,
  locals_conversion: NameConversion,
}

fn prefixer(stylesheet: &mut Stylesheet, css_prefixer_config: &CssPrefixerConfig) {
  let mut prefixer = swc_css_prefixer::prefixer(swc_css_prefixer::options::Options {
    env: css_prefixer_config.targets.clone(),
  });
  prefixer.visit_mut_stylesheet(stylesheet);
}

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
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
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
        let ParseCssModuleResult {
          ast: mut css_stylesheet,
          comments,
          source_map,
        } = parse_css_stylesheet(
          &css_modules_module_id.to_string(),
          Arc::new(param.content.clone()),
        )?;
        context
          .meta
          .set_module_source_map(&css_modules_module_id, source_map);

        // js code for css modules
        // next, get ident from ast and export through JS
        let stylesheet = compile(
          &mut css_stylesheet,
          CssModuleRename {
            indent_name: context
              .config
              .css
              .modules
              .as_ref()
              .unwrap()
              .indent_name
              .clone(),
            hash: sha256(css_modules_module_id.to_string().as_bytes(), 8),
          },
        );

        // we can not use css_modules_resolved_path here because of the compatibility of windows. eg: \\ vs \\\\
        let cache_id = css_modules_module_id.to_string();
        self.ast_map.lock().insert(
          cache_id.clone(),
          (css_stylesheet, CommentsMetaData::from(comments)),
        );

        // for composes dynamic import (eg: composes: action from "./action.css")
        let mut dynamic_import_of_composes = HashMap::default();
        let mut export_names = Vec::new();

        for (name, classes) in stylesheet.renamed.iter() {
          let mut after_transform_classes = Vec::new();
          for v in classes {
            match v {
              CssClassName::Local { name } => {
                after_transform_classes.push(name.value.to_string());
              }
              CssClassName::Global { name } => {
                after_transform_classes.push(name.value.to_string());
              }
              CssClassName::Import { name, from } => {
                let v = dynamic_import_of_composes
                  .entry(from)
                  .or_insert(format!("f_{}", sha256(from.as_bytes(), 5)));
                after_transform_classes.push(format!("${{{}[\"{}\"]}}", v, name.value));
              }
            }
          }
          export_names.push((
            self.locals_conversion.transform(&name),
            after_transform_classes,
          ));
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
          // context.cache_manager.write

          // let mut module_graph = context.module_graph.write();
          // let module = module_graph.module_mut(&css_modules_module_id).unwrap();

          // module.meta.write("content", param.content.clone());
          // module.meta.write("sourcemap", map);
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
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ModuleMetaData>> {
    if matches!(param.module_type, ModuleType::Css) {
      let (css_stylesheet, comments) = if is_farm_css_modules(&param.module_id.to_string()) {
        self
          .ast_map
          .lock()
          .remove(&param.module_id.to_string())
          .unwrap_or_else(|| panic!("ast not found {:?}", param.module_id.to_string()))
      } else {
        // swc_css_parser does not support
        let ParseCssModuleResult {
          ast,
          comments,
          source_map,
        } = parse_css_stylesheet(&param.module_id.to_string(), param.content.clone())?;
        context
          .meta
          .set_module_source_map(&param.module_id, source_map);

        (ast, CommentsMetaData::from(comments))
      };

      let meta = ModuleMetaData::Css(CssModuleMetaData {
        ast: css_stylesheet,
        comments,
        custom: Default::default(),
      });

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
    let css_stylesheet = match &mut param.meta {
      ModuleMetaData::Css(meta) => &mut meta.ast,
      _ => return Ok(None),
    };

    if enable_prefixer {
      // css prefixer
      prefixer(
        css_stylesheet,
        context.config.css.prefixer.as_ref().unwrap(),
      );

      return Ok(Some(()));
    }

    Ok(None)
  }

  fn analyze_deps(
    &self,
    param: &mut PluginAnalyzeDepsHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if param.module.module_type == ModuleType::Css {
      let stylesheet = &param.module.meta.as_css().ast;
      // analyze dependencies:
      // 1. @import './xxx.css'
      // 2. url()
      let mut dep_analyzer = DepAnalyzer::new(context.config.resolve.alias.clone());
      stylesheet.visit_with(&mut dep_analyzer);
      param.deps.extend(dep_analyzer.deps);
    }

    Ok(None)
  }

  fn build_end(&self, context: &Arc<CompilationContext>) -> farmfe_core::error::Result<Option<()>> {
    if !matches!(context.config.mode, farmfe_core::config::Mode::Development)
      || !matches!(context.config.output.target_env, TargetEnv::Browser)
    {
      return Ok(None);
    }

    // transform all css to script
    let css_modules = context
      .module_graph
      .write()
      .modules()
      .into_iter()
      .filter_map(|m| {
        if matches!(m.module_type, ModuleType::Css) {
          Some(m.id.clone())
        } else {
          None
        }
      })
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

      let resources_map = context.resources_map.lock();

      let rendered_modules = Mutex::new(Vec::with_capacity(modules.len()));
      modules.into_par_iter().try_for_each(|module| {
        let cm = context.meta.get_module_source_map(&module.id);
        let mut css_stylesheet = module.meta.as_css().ast.clone();

        try_with(cm, &context.meta.css.globals, || {
          source_replace(
            &mut css_stylesheet,
            &module.id,
            &module_graph,
            &resources_map,
            context.config.output.public_path.clone(),
            context.config.resolve.alias.clone(),
          );
        })?;

        rendered_modules
          .lock()
          .push((module.id.clone(), css_stylesheet));

        Ok::<(), CompilationError>(())
      })?;

      let mut rendered_modules = rendered_modules.into_inner();

      rendered_modules.sort_by_key(|module| module_execution_order[&module.0]);

      let mut stylesheet = Stylesheet {
        span: DUMMY_SP,
        rules: vec![],
      };

      let source_map = merge_css_sourcemap(&mut rendered_modules, context);
      context
        .meta
        .set_resource_pot_source_map(&resource_pot.id, source_map);

      for (_, rendered_module_ast) in rendered_modules {
        stylesheet.rules.extend(rendered_module_ast.rules);
      }

      Ok(Some(ResourcePotMetaData::Css(CssResourcePotMetaData {
        ast: stylesheet,
        custom: Default::default(),
      })))
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
      let css_stylesheet = &resource_pot.meta.as_css().ast;
      let source_map_enabled = context.config.sourcemap.enabled(resource_pot.immutable);

      let (css_code, src_map) = codegen_css_stylesheet(
        css_stylesheet,
        context.config.minify.enabled(),
        if source_map_enabled {
          Some(context.meta.get_resource_pot_source_map(&resource_pot.id))
        } else {
          None
        },
      );

      let resource = Resource {
        name: resource_pot.name.to_string(),
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
        let sourcemap = trace_module_sourcemap(sourcemap, &module_graph);

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
          name: format!("{}.{}", resource_pot.name, ty.to_ext()),
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
    if !self.content_map.lock().is_empty() || !self.sourcemap_map.lock().is_empty() {
      let cache = CssModulesCache {
        content_map: self.content_map.lock().clone(),
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
      content_map: Mutex::new(Default::default()),
      sourcemap_map: Mutex::new(Default::default()),
      locals_conversion: get_config_css_modules_local_conversion(config),
    }
  }

  pub fn is_path_match_css_modules(&self, path: &str) -> bool {
    self
      .css_modules_paths
      .iter()
      .any(|regex| regex.is_match(path))
  }
}

struct CssModuleRename {
  indent_name: String,
  hash: String,
}

impl TransformConfig for CssModuleRename {
  fn new_name_for(&self, local: &JsWord) -> JsWord {
    let name = local.to_string();
    let r: HashMap<String, &String> = [("name".into(), &name), ("hash".into(), &self.hash)]
      .into_iter()
      .collect();
    transform_css_module_indent_name(self.indent_name.clone(), r).into()
  }
}

fn transform_css_module_indent_name(
  indent_name: String,
  context: HashMap<String, &String>,
) -> String {
  context.iter().fold(indent_name, |acc, (key, value)| {
    acc.replace(&format!("[{key}]"), value)
  })
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
