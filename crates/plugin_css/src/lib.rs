use std::{
  path::PathBuf,
  sync::Arc,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  hashbrown::HashMap,
  module::{CssModuleMetaData, ModuleId, ModuleMetaData, ModuleType},
  parking_lot::Mutex,
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginParseHookParam, PluginTransformHookResult,
  },
  resource::{
    resource_pot::{CssResourcePotMetaData, ResourcePot, ResourcePotMetaData, ResourcePotType},
    Resource, ResourceType,
  },
  swc_common::DUMMY_SP,
  swc_css_ast::Stylesheet,
};
use farmfe_toolkit::{
  base64::engine::{general_purpose, Engine},
  css::{codegen_css_stylesheet, parse_css_stylesheet},
  fs::{read_file_utf8, transform_output_filename},
  script::module_type_from_id,
  swc_atoms::JsWord,
  swc_css_modules::TransformConfig,
};
use farmfe_toolkit::{
  hash,
  swc_css_modules::{compile, CssClassName},
};
use farmfe_utils::stringify_query;

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

pub struct FarmPluginCss {
  ast_map: Mutex<HashMap<String, Stylesheet>>,
}

fn wrapper_style_load(code: &String, id: String) -> String {
  format!(
    r#"
const cssCode = `{}`;
const farmId = '{}';
const previousStyle = document.querySelector(`style[data-farm-id="${{farmId}}"]`);
const style = document.createElement('style');
style.setAttribute('data-farm-id', farmId);
style.innerHTML = cssCode;
if (previousStyle) {{
previousStyle.replaceWith(style);
}} else {{
document.head.appendChild(style);
}}

module.onDispose(() => {{
style.remove();
}});
"#,
    code, id
  )
}

impl Plugin for FarmPluginCss {
  fn name(&self) -> &str {
    "FarmPluginCss"
  }
  /// This plugin should be executed at last
  fn priority(&self) -> i32 {
    99
  }

  fn load(
    &self,
    param: &PluginLoadHookParam,
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
    let module_type = module_type_from_id(param.resolved_path);

    if let Some(module_type) = module_type {
      if matches!(module_type, ModuleType::Css) {
        let content = read_file_utf8(param.resolved_path)?;

        Ok(Some(PluginLoadHookResult {
          content,
          module_type,
        }))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  fn transform(
    &self,
    param: &farmfe_core::plugin::PluginTransformHookParam,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
    if matches!(param.module_type, ModuleType::Css) {
      let is_modules = context.config.css.modules;

      let module_id = ModuleId::new(
        param.resolved_path,
        &stringify_query(&param.query),
        &context.config.root,
      );

      let enabled_sourcemap = context.config.sourcemap.enabled();

      // css modules
      if is_modules {
        let query = param.query.iter().fold(HashMap::new(), |mut acc, (k, v)| {
          acc.insert(k.to_string(), v.to_string());
          acc
        });

        let is_modules_file = query
          .get("modules")
          .and_then(|is_module| Some(is_module == "true"))
          .is_some();

        let is_production = matches!(context.config.mode, farmfe_core::config::Mode::Production);

        // real css code
        if is_modules_file {
          if matches!(context.config.mode, farmfe_core::config::Mode::Development) {
            let ast = self
              .ast_map
              .lock()
              .remove(module_id.relative_path())
              .expect("receive an valid css modules file");

            let (mut content, src_map) = codegen_css_stylesheet(
              &ast,
              Some(context.meta.css.cm.clone()),
              context.config.minify,
            );

            if enabled_sourcemap {
              inline_source_map(&mut content, src_map);
            }

            let js_code = wrapper_style_load(&content, module_id.to_string());

            return Ok(Some(PluginTransformHookResult {
              content: js_code,
              module_type: Some(ModuleType::Js),
              source_map: None,
            }));
          } else {
            return Ok(Some(PluginTransformHookResult {
              content: "".to_string(),
              module_type: Some(ModuleType::Css),
              source_map: None,
            }));
          }
        }

        // next, get ident from ast and export through JS
        let mut css_stylesheet = parse_css_stylesheet(
          &module_id.to_string(),
          &param.content,
          context.meta.css.cm.clone(),
        )?;

        let stylesheet = compile(
          &mut css_stylesheet,
          CssModuleRename {
            indent_name: context.config.css.indent_name.clone(),
            hash: hash::sha256(param.resolved_path.as_bytes(), 8),
          },
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
                  .or_insert(format!("f_{}", hash::sha256(from.as_bytes(), 5)));
                after_transform_classes.push(format!("${{{}[\"{}\"]}}", v, name.value));
              }
            }
          }
          export_names.push((name, after_transform_classes));
        }

        let hash: String = if is_production {
          "".into()
        } else {
          Duration::from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
            .as_micros()
            .to_string()
        };

        let code = format!(
          r#"
    import "./{}?modules=true&lang=css&hash={}"
    {}
    export default {{{}}}
    "#,
          PathBuf::from(param.resolved_path)
            .file_name()
            .unwrap()
            .to_string_lossy(),
          hash,
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

        self
          .ast_map
          .lock()
          .insert(module_id.to_string(), css_stylesheet);

        return Ok(Some(PluginTransformHookResult {
          content: code,
          module_type: Some(ModuleType::Js),
          source_map: None,
        }));
      }

      // original css
      if matches!(context.config.mode, farmfe_core::config::Mode::Development) {
        let content_with_sourcemap = if enabled_sourcemap {
          let stylesheet = parse_css_stylesheet(
            &module_id.to_string(),
            &param.content,
            context.meta.css.cm.clone(),
          )?;

          let (mut css_code, src_map) = codegen_css_stylesheet(
            &stylesheet,
            Some(context.meta.css.cm.clone()),
            context.config.minify,
          );

          inline_source_map(&mut css_code, src_map);

          Some(css_code)
        } else {
          None
        };

        let css_js_code = wrapper_style_load(
          content_with_sourcemap.as_ref().unwrap_or(&param.content),
          module_id.to_string(),
        );

        Ok(Some(PluginTransformHookResult {
          content: css_js_code,
          module_type: Some(ModuleType::Js),
          source_map: None,
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
      let is_css_modules = param
        .query
        .iter()
        .any(|(k, v)| k == "modules" && v == "true")
        && context.config.css.modules;

      let css_stylesheet = if is_css_modules {
        self
          .ast_map
          .lock()
          .remove(param.module_id.relative_path())
          .expect("invalid css module")
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

  fn analyze_deps(
    &self,
    _param: &mut PluginAnalyzeDepsHookParam,
    _context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    Ok(None)
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

      for module_id in resource_pot.modules() {
        let module = module_graph.module(module_id).unwrap();
        let module_css_ast: &Stylesheet = &module.meta.as_css().ast;
        merged_css_ast.rules.extend(module_css_ast.rules.to_vec());
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
  ) -> farmfe_core::error::Result<Option<Vec<Resource>>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      let stylesheet = &resource_pot.meta.as_css().ast;

      let source_map_enabled = context.config.sourcemap.enabled();

      let (mut css_code, src_map) = codegen_css_stylesheet(
        &stylesheet,
        if source_map_enabled {
          Some(context.meta.css.cm.clone())
        } else {
          None
        },
        context.config.minify,
      );

      let filename = transform_output_filename(
        context.config.output.filename.clone(),
        resource_pot.id.to_string().as_str(),
        css_code.as_bytes(),
        ResourceType::Css.to_ext().as_str(),
      );

      let sourcemap_filename = format!("{filename}.map");

      let mut resources = vec![];

      if context.config.sourcemap.enabled()
        && (context.config.sourcemap.is_all() || !resource_pot.immutable)
      {
        css_code.push_str(format!("\n/*# sourceMappingURL={} */", sourcemap_filename).as_str());

        resources.push(Resource {
          name: sourcemap_filename,
          bytes: src_map.unwrap(),
          emitted: false,
          resource_type: ResourceType::SourceMap,
          resource_pot: resource_pot.id.clone(),
          preserve_name: true,
        })
      }

      resources.push(Resource {
        name: filename.clone(),
        bytes: css_code.as_bytes().to_vec(),
        emitted: false,
        resource_type: ResourceType::Css,
        resource_pot: resource_pot.id.clone(),
        preserve_name: true,
      });

      Ok(Some(resources))
    } else {
      Ok(None)
    }
  }
}

impl FarmPluginCss {
  pub fn new(_: &Config) -> Self {
    Self {
      ast_map: Mutex::new(HashMap::new()),
    }
  }
}

fn inline_source_map(css_code: &mut String, src_map: Option<Vec<u8>>) {
  let base64 = general_purpose::STANDARD.encode(src_map.expect("failed to generate source map"));

  css_code.push_str(
    format!("\n/*# sourceMappingURL=data:application/json;charset=utf-8;base64,{base64}*/")
      .as_str(),
  );
}
