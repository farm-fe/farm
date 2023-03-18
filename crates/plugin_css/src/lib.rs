use std::sync::Arc;

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{CssModuleMetaData, ModuleId, ModuleMetaData, ModuleType},
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
  css::{codegen_css_stylesheet, parse_css_stylesheet},
  fs::read_file_utf8,
  script::module_type_from_id,
};
use farmfe_utils::stringify_query;

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx files to js chunks
pub struct FarmPluginCss {}

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
    if matches!(param.module_type, ModuleType::Css)
      && matches!(context.config.mode, farmfe_core::config::Mode::Development)
    {
      let rp = param.resolved_path.to_string() + &stringify_query(&param.query);
      let module_id = ModuleId::new(&rp, &context.config.root);

      let css_js_code = format!(
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
"#,
        param.content,
        module_id.to_string()
      );

      Ok(Some(PluginTransformHookResult {
        content: css_js_code,
        module_type: Some(ModuleType::Js),
        source_map: None,
      }))
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
      let css_stylesheet = parse_css_stylesheet(
        &param.module_id.to_string(),
        &param.content,
        context.meta.css.cm.clone(),
      )?;

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
    _context: &Arc<CompilationContext>,
    _hook_context: &PluginHookContext,
  ) -> farmfe_core::error::Result<Option<Vec<Resource>>> {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      let stylesheet = &resource_pot.meta.as_css().ast;

      let css_code = codegen_css_stylesheet(&stylesheet);

      Ok(Some(vec![Resource {
        name: resource_pot.id.to_string(),
        bytes: css_code.as_bytes().to_vec(),
        emitted: false,
        resource_type: ResourceType::Css,
        resource_pot: resource_pot.id.clone(),
        preserve_name: false,
      }]))
    } else {
      Ok(None)
    }
  }
}

impl FarmPluginCss {
  pub fn new(_: &Config) -> Self {
    Self {}
  }
}
