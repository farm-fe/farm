#![feature(box_patterns)]

use std::sync::Arc;

use dep_analyzer::DepAnalyzer;
use farmfe_core::{
  config::{Config, CssPrefixerConfig, TargetEnv},
  context::CompilationContext,
  hashbrown::HashMap,
  module::{
    module_graph::{self, ModuleGraph},
    CssModuleMetaData, ModuleId, ModuleMetaData, ModuleType,
  },
  parking_lot::Mutex,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginGenerateResourcesHookResult, PluginHookContext,
    PluginLoadHookParam, PluginLoadHookResult, PluginParseHookParam, PluginTransformHookResult,
  },
  resource::{
    resource_pot::{CssResourcePotMetaData, ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceOrigin, ResourceType,
  },
  swc_common::{FilePathMapping, SourceMap, DUMMY_SP},
  swc_css_ast::Stylesheet,
};
use farmfe_toolkit::{
  css::{codegen_css_stylesheet, parse_css_stylesheet},
  fs::read_file_utf8,
  hash::sha256,
  regex::Regex,
  script::module_type_from_id,
  swc_atoms::JsWord,
  swc_css_modules::{compile, CssClassName, TransformConfig},
  swc_css_prefixer,
  swc_css_visit::{VisitMut, VisitMutWith, VisitWith},
};
use farmfe_utils::{parse_query, stringify_query};
use source_replacer::SourceReplacer;
use transform_resource_pot::transform_css_resource_pot;

const FARM_CSS_MODULES_SUFFIX: &str = ".FARM_CSS_MODULES";

mod dep_analyzer;
mod source_replacer;
pub mod transform_resource_pot;

pub struct FarmPluginCss {
  css_modules_paths: Vec<Regex>,
  ast_map: Mutex<HashMap<String, Stylesheet>>,
}

pub fn wrapper_style_load(code: &str, id: String, css_deps: &String) -> String {
  format!(
    r#"
const cssCode = `{}`;
const farmId = '{}';
{}
const previousStyle = document.querySelector(`style[data-farm-id="${{farmId}}"]`);
const style = document.createElement('style');
style.setAttribute('data-farm-id', farmId);
style.innerHTML = cssCode;
if (previousStyle) {{
previousStyle.replaceWith(style);
}} else {{
document.head.appendChild(style);
}}
module.meta.hot.accept();

module.onDispose(() => {{
style.remove();
}});
"#,
    code.replace('`', "'").replace('\\', "\\\\"),
    id.replace('\\', "\\\\"),
    css_deps
  )
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
    99
  }

  fn resolve(
    &self,
    param: &farmfe_core::plugin::PluginResolveHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginResolveHookResult>> {
    if is_farm_css_modules(&param.source) {
      let split = param.source.split('?').collect::<Vec<&str>>();
      let strip_query_path = split[0].to_string();
      let query = parse_query(&param.source);

      return Ok(Some(farmfe_core::plugin::PluginResolveHookResult {
        resolved_path: strip_query_path,
        query,
        ..Default::default()
      }));
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
        // return empty content as we don't need to load the module, it is already parsed and stored in the ast_map
        content: "".to_string(),
        module_type: ModuleType::Css,
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
    if matches!(param.module_type, ModuleType::Css) {
      let module_id = ModuleId::new(
        param.resolved_path,
        &stringify_query(&param.query),
        &context.config.root,
      );
      let enable_css_modules = context.config.css.modules.is_some();

      // real css modules code
      if is_farm_css_modules(param.resolved_path) {
        // if matches!(context.config.mode, farmfe_core::config::Mode::Development) {
        //   let ast = self.ast_map.lock().remove(param.resolved_path).unwrap();
        //   let (css_code, _) = codegen_css_stylesheet(&ast, None, context.config.minify);
        //   let js_code = wrapper_style_load(&css_code, module_id.to_string());

        //   return Ok(Some(PluginTransformHookResult {
        //     content: js_code,
        //     module_type: Some(ModuleType::Js),
        //     source_map: None,
        //   }));
        // } else {
        return Ok(None);
        // }
      }

      // css modules
      if enable_css_modules && self.is_path_match_css_modules(param.resolved_path) {
        let mut css_stylesheet = parse_css_stylesheet(
          &module_id.to_string(),
          &param.content,
          Arc::new(SourceMap::new(FilePathMapping::empty())),
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
            hash: sha256(module_id.to_string().as_bytes(), 8),
          },
        );

        self.ast_map.lock().insert(
          format!("{}{}", param.resolved_path, FARM_CSS_MODULES_SUFFIX),
          css_stylesheet,
        );

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
    import "{}{}?{}";
    {}
    export default {{{}}}
    "#,
          if cfg!(windows) {
            param.resolved_path.replace('\\', "\\\\")
          } else {
            param.resolved_path.to_string()
          },
          FARM_CSS_MODULES_SUFFIX,
          // add hash to avoid cache, make sure hmr works
          // TODO use updateModules hook to invalidate cache instead of hash
          sha256(param.content.replace("\r\n", "\n").as_bytes(), 8),
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

        Ok(Some(PluginTransformHookResult {
          content: code,
          module_type: Some(ModuleType::Js),
          source_map: None,
        }))
      // } else if matches!(context.config.mode, farmfe_core::config::Mode::Development) {
      //   let css_js_code = wrapper_style_load(&param.content, module_id.to_string());

      //   Ok(Some(PluginTransformHookResult {
      //     content: css_js_code,
      //     module_type: Some(ModuleType::Js),
      //     source_map: None,
      //   }))
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
      let css_stylesheet = if is_farm_css_modules(&param.resolved_path) {
        self
          .ast_map
          .lock()
          .remove(&param.resolved_path)
          .unwrap_or_else(|| panic!("ast not found {:?}", param.resolved_path))
      } else {
        parse_css_stylesheet(
          &param.module_id.to_string(),
          &param.content,
          context.meta.css.cm.clone(),
        )?
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

  fn process_resource_pots(
    &self,
    resource_pots: &mut Vec<&mut ResourcePot>,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !matches!(context.config.mode, farmfe_core::config::Mode::Development)
      || !matches!(context.config.output.target_env, TargetEnv::Browser)
    {
      return Ok(None);
    }

    let mut module_graph = context.module_graph.write();

    // transform css resource pot to js resource pot in development mode
    for resource_pot in resource_pots {
      transform_css_resource_pot(resource_pot, &mut module_graph, context)?;
    }

    Ok(Some(()))
  }

  fn render_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      let module_graph = context.module_graph.read();
      let mut merged_css_ast = Stylesheet {
        span: DUMMY_SP,
        rules: vec![],
      };

      let mut modules = vec![];

      for module_id in resource_pot.modules() {
        let module = module_graph.module(module_id).unwrap();
        modules.push(module);
      }

      modules.sort_by_key(|module| module.execution_order);
      let resources_map = context.resources_map.lock();

      for module in modules {
        let mut module_css_ast: Stylesheet = Stylesheet {
          span: DUMMY_SP,
          rules: module.meta.as_css().ast.rules.to_vec(),
        };

        source_replace(
          &mut module_css_ast,
          &module.id,
          &module_graph,
          &resources_map,
        );

        merged_css_ast.rules.extend(module_css_ast.rules);
      }

      resource_pot.meta = ResourcePotMetaData::Css(CssResourcePotMetaData {
        ast: merged_css_ast,
      });

      Ok(Some(()))
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
      let stylesheet = &resource_pot.meta.as_css().ast;
      let source_map_enabled = context.config.sourcemap.enabled();
      let module_graph = context.module_graph.read();
      let (css_code, src_map) = codegen_css_stylesheet(
        stylesheet,
        if source_map_enabled {
          Some(context.meta.css.cm.clone())
        } else {
          None
        },
        context.config.minify,
        &module_graph,
      );

      let resource = Resource {
        name: resource_pot.name.to_string(),
        bytes: css_code.as_bytes().to_vec(),
        emitted: false,
        resource_type: ResourceType::Css,
        origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
      };
      let mut source_map = None;

      if context.config.sourcemap.enabled()
        && (context.config.sourcemap.is_all() || !resource_pot.immutable)
      {
        // css_code.push_str(format!("\n/*# sourceMappingURL={} */", sourcemap_filename).as_str());
        if let Some(bytes) = src_map {
          source_map = Some(Resource {
            name: resource_pot.name.to_string(),
            bytes,
            emitted: false,
            resource_type: ResourceType::SourceMap(resource_pot.id.to_string()),
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

pub fn source_replace(
  stylesheet: &mut Stylesheet,
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  resources_map: &HashMap<String, Resource>,
) {
  let mut source_replacer = SourceReplacer::new(module_id.clone(), module_graph, resources_map);
  stylesheet.visit_mut_with(&mut source_replacer);
}
