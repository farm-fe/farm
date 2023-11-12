#![feature(box_patterns)]

use std::{path::PathBuf, sync::Arc};

use dep_analyzer::DepAnalyzer;
use farmfe_core::{
  config::{Config, CssPrefixerConfig, TargetEnv},
  context::CompilationContext,
  deserialize,
  enhanced_magic_string::{
    bundle::{Bundle, BundleOptions},
    collapse_sourcemap::{collapse_sourcemap_chain, CollapseSourcemapOptions},
    magic_string::{MagicString, MagicStringOptions},
    types::SourceMapOptions,
  },
  error::CompilationError,
  hashbrown::HashMap,
  module::{module_graph::ModuleGraph, CssModuleMetaData, ModuleId, ModuleMetaData, ModuleType},
  parking_lot::Mutex,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginGenerateResourcesHookResult, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginParseHookParam, PluginResolveHookParam,
    PluginTransformHookResult, ResolveKind,
  },
  rayon::prelude::*,
  resource::{
    resource_pot::{RenderedModule, ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  serialize,
  swc_css_ast::Stylesheet,
};
use farmfe_macro_cache_item::cache_item;
use farmfe_toolkit::{
  common::Source,
  css::{codegen_css_stylesheet, parse_css_stylesheet},
  fs::read_file_utf8,
  hash::sha256,
  regex::Regex,
  script::module_type_from_id,
  sourcemap::SourceMap,
  swc_atoms::JsWord,
  swc_css_modules::{compile, CssClassName, TransformConfig},
  swc_css_prefixer,
  swc_css_visit::{VisitMut, VisitMutWith, VisitWith},
};
use farmfe_utils::{parse_query, relative};
use rkyv::Deserialize;
use source_replacer::SourceReplacer;

const FARM_CSS_MODULES_SUFFIX: &str = ".FARM_CSS_MODULES";

mod dep_analyzer;
mod source_replacer;
pub mod transform_css_to_script;

#[cache_item]
struct CssModulesCache {
  content_map: HashMap<String, String>,
  sourcemap_map: HashMap<String, String>,
}

pub struct FarmPluginCss {
  css_modules_paths: Vec<Regex>,
  ast_map: Mutex<HashMap<String, Stylesheet>>,
  content_map: Mutex<HashMap<String, String>>,
  sourcemap_map: Mutex<HashMap<String, String>>,
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

  fn plugin_cache_loaded(
    &self,
    cache: &Vec<u8>,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let cache = deserialize!(cache, CssModulesCache);
    self.content_map.lock().extend(cache.content_map);
    self.sourcemap_map.lock().extend(cache.sourcemap_map);
    Ok(Some(()))
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
      // otherwise it's always relative
      let source = if let Some(striped_source) = param.source.strip_suffix('~') {
        striped_source.to_string()
      } else if !param.source.starts_with('.') {
        format!("./{}", param.source)
      } else {
        param.source.clone()
      };

      return context.plugin_driver.resolve(
        &PluginResolveHookParam {
          source,
          ..param.clone()
        },
        context,
        &PluginHookContext {
          caller: Some("FarmPluginCss".to_string()),
          meta: Default::default(),
        },
      );
    }

    Ok(None)
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    if is_farm_css_modules(param.resolved_path) {
      return Ok(Some(PluginLoadHookResult {
        content: self
          .content_map
          .lock()
          .get(param.resolved_path)
          .unwrap()
          .clone(),
        module_type: ModuleType::Custom(FARM_CSS_MODULES_SUFFIX.to_string()),
      }));
    };

    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if matches!(module_type, ModuleType::Css) {
        let content = read_file_utf8(param.resolved_path)?;

        return Ok(Some(PluginLoadHookResult {
          content,
          module_type,
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
        source_map: self.sourcemap_map.lock().get(param.resolved_path).cloned(),
        ignore_previous_source_map: false,
      }));
    }

    if matches!(param.module_type, ModuleType::Css) {
      let enable_css_modules = context.config.css.modules.is_some();

      // css modules
      if enable_css_modules && self.is_path_match_css_modules(param.resolved_path) {
        // add hash to avoid cache, make sure hmr works
        // TODO use updateModules hook to invalidate cache instead of hash.
        let query_string = format!(
          "?{}",
          sha256(param.content.replace("\r\n", "\n").as_bytes(), 8)
        );
        let css_modules_resolved_path = format!(
          "{}{}",
          if cfg!(windows) {
            param.resolved_path.replace('\\', "\\\\")
          } else {
            param.resolved_path.to_string()
          },
          FARM_CSS_MODULES_SUFFIX,
        );
        let css_modules_module_id = ModuleId::new(
          &css_modules_resolved_path,
          &query_string,
          &context.config.root,
        );
        let mut css_stylesheet = parse_css_stylesheet(
          &css_modules_module_id.to_string(),
          Arc::new(param.content.clone()),
        )?;

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
        let cache_id = format!("{}{}", param.resolved_path, FARM_CSS_MODULES_SUFFIX);
        self.ast_map.lock().insert(cache_id.clone(), css_stylesheet);
        self
          .content_map
          .lock()
          .insert(cache_id, param.content.clone());

        // for composes dynamic import (eg: composes: action from "./action.css")
        let mut dynamic_import_of_composes = HashMap::new();
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
          export_names.push((name, after_transform_classes));
        }

        let code = format!(
          r#"
    import "{}{}";
    {}
    export default {{{}}}
    "#,
          css_modules_resolved_path,
          query_string,
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
        if param.source_map_chain.len() > 0 {
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
            .insert(css_modules_resolved_path, map);
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
      let css_stylesheet = if is_farm_css_modules(&param.resolved_path) {
        self
          .ast_map
          .lock()
          .remove(&param.resolved_path)
          .unwrap_or_else(|| panic!("ast not found {:?}", param.resolved_path))
      } else {
        parse_css_stylesheet(&param.module_id.to_string(), param.content.clone())?
      };

      let meta = ModuleMetaData::Css(CssModuleMetaData {
        ast: css_stylesheet,
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
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if param.module.module_type == ModuleType::Css {
      let stylesheet = &param.module.meta.as_css().ast;
      // analyze dependencies:
      // 1. @import './xxx.css'
      // 2. url()
      let mut dep_analyzer = DepAnalyzer::new();
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

  fn render_resource_pot_modules(
    &self,
    resource_pot: &ResourcePot,
    context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      let source_map_enabled = context.config.sourcemap.enabled(resource_pot.immutable);
      let module_graph = context.module_graph.read();

      let mut modules = vec![];

      for module_id in resource_pot.modules() {
        let module = module_graph.module(module_id).unwrap();
        modules.push(module);
      }

      modules.sort_by_key(|module| module.execution_order);
      let resources_map = context.resources_map.lock();

      let rendered_modules = modules
        .into_par_iter()
        .map(|module| {
          let mut css_stylesheet = module.meta.as_css().ast.clone();

          source_replace(
            &mut css_stylesheet,
            &module.id,
            &module_graph,
            &resources_map,
          );

          let (css_code, src_map) = codegen_css_stylesheet(
            &css_stylesheet,
            if source_map_enabled {
              Some(Source {
                path: PathBuf::from(&module.id.resolved_path_with_query(&context.config.root)),
                content: module.content.clone(),
              })
            } else {
              None
            },
            false,
          );

          RenderedModule {
            id: module.id.clone(),
            rendered_length: css_code.len(),
            original_length: module.size,
            rendered_content: Arc::new(css_code),
            rendered_map: src_map.map(|s| Arc::new(s)),
          }
        })
        .collect::<Vec<_>>();

      let mut bundle = Bundle::new(BundleOptions {
        trace_source_map_chain: Some(true),
        ..Default::default()
      });

      for rendered in &rendered_modules {
        let mut source_map_chain = vec![];

        if source_map_enabled {
          let module = module_graph.module(&rendered.id).unwrap();
          source_map_chain = module.source_map_chain.clone();

          if let Some(map) = &rendered.rendered_map {
            source_map_chain.push(map.clone());
          }
        }

        let magic_module = MagicString::new(
          rendered.rendered_content.as_str(),
          Some(MagicStringOptions {
            source_map_chain,
            filename: Some(rendered.id.resolved_path_with_query(&context.config.root)),
            ..Default::default()
          }),
        );
        bundle.add_source(magic_module, None).map_err(|e| {
          CompilationError::GenericError(format!("failed to add source to bundle: {:?}", e))
        })?;
      }

      let rendered_content = Arc::new(bundle.to_string());
      let rendered_map = if source_map_enabled {
        let root = context.config.root.clone();
        Some(
          bundle
            .generate_map(SourceMapOptions {
              include_content: Some(true),
              remap_source: Some(Box::new(move |src| format!("/{}", relative(&root, src)))),
              ..Default::default()
            })
            .map_err(|e| {
              CompilationError::GenericError(format!("failed to generate source map: {:?}", e))
            })?,
        )
        .map(|v| {
          let mut buf = vec![];
          v.to_writer(&mut buf).unwrap();
          Arc::new(String::from_utf8(buf).unwrap())
        })
      } else {
        None
      };

      Ok(Some(ResourcePotMetaData {
        rendered_modules: rendered_modules
          .into_iter()
          .map(|m| (m.id.clone(), m))
          .collect(),
        rendered_content,
        rendered_map_chain: rendered_map.map(|v| vec![v]).unwrap_or(vec![]),
      }))
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
      let resource = Resource {
        name: resource_pot.name.to_string(),
        bytes: resource_pot.meta.rendered_content.as_bytes().to_vec(),
        emitted: false,
        resource_type: ResourceType::Css,
        origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
      };
      let mut source_map = None;

      if context.config.sourcemap.enabled(resource_pot.immutable) {
        // css_code.push_str(format!("\n/*# sourceMappingURL={} */", sourcemap_filename).as_str());
        if !resource_pot.meta.rendered_map_chain.is_empty() {
          let collapsed_sourcemap = collapse_sourcemap_chain(
            resource_pot
              .meta
              .rendered_map_chain
              .iter()
              .map(|m| SourceMap::from_slice(m.as_bytes()).unwrap())
              .collect(),
            Default::default(),
          );
          let mut buf = vec![];
          collapsed_sourcemap
            .to_writer(&mut buf)
            .expect("failed to write sourcemap");
          let resource_type = ResourceType::SourceMap(resource_pot.id.to_string());
          source_map = Some(Resource {
            name: format!("{}.{}", resource_pot.name, resource_type.to_ext()),
            bytes: buf,
            emitted: false,
            resource_type,
            origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
          });
        }
      }

      Ok(Some(PluginGenerateResourcesHookResult {
        resource,
        source_map,
      }))
    } else {
      Ok(None)
    }
  }

  fn write_plugin_cache(
    &self,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<Vec<u8>>> {
    let cache = CssModulesCache {
      content_map: self.content_map.lock().clone(),
      sourcemap_map: self.sourcemap_map.lock().clone(),
    };

    Ok(Some(serialize!(&cache)))
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
    acc.replace(&format!("[{}]", key), value)
  })
}

fn is_farm_css_modules(path: &str) -> bool {
  path
    .split('?')
    .next()
    .unwrap()
    .ends_with(FARM_CSS_MODULES_SUFFIX)
}

fn is_farm_css_modules_type(module_type: &ModuleType) -> bool {
  if let ModuleType::Custom(c) = module_type {
    return c.as_str() == FARM_CSS_MODULES_SUFFIX;
  }

  false
}

pub fn source_replace(
  stylesheet: &mut Stylesheet,
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  resources_map: &HashMap<String, Resource>,
) {
  let mut source_replacer = SourceReplacer::new(module_id.clone(), module_graph, resources_map);
  stylesheet.visit_mut_with(&mut source_replacer);
}
