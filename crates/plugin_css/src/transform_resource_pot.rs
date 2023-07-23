use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{
    module_graph::ModuleGraph, ModuleId, ModuleMetaData, ModuleSystem, ModuleType,
    ScriptModuleMetaData,
  },
  resource::resource_pot::{ResourcePot, ResourcePotType},
  swc_common::Mark,
  swc_css_ast::Stylesheet,
  swc_ecma_ast::EsVersion,
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::{
  css::codegen_css_stylesheet,
  script::{parse_module, swc_try_with::try_with},
  swc_ecma_transforms_base::resolver,
  swc_ecma_visit::VisitMutWith,
};

use crate::{source_replace, wrapper_style_load};

/// transform css resource pot to script resource pot
pub fn transform_css_resource_pot(
  resource_pot: &mut ResourcePot,
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) -> farmfe_core::error::Result<()> {
  for module_id in resource_pot.modules() {
    if !matches!(
      module_graph.module(module_id).unwrap().module_type,
      ModuleType::Css
    ) {
      continue;
    }

    let stylesheet = transform_css_stylesheet(module_id, module_graph, context);
    let css_deps = transform_css_deps(module_id, module_graph, context);

    let module = module_graph.module_mut(module_id).unwrap();
    let source_map_enabled = context.config.sourcemap.enabled();
    let (css_code, _src_map) = codegen_css_stylesheet(
      &stylesheet,
      if source_map_enabled {
        Some(context.meta.css.cm.clone())
      } else {
        None
      },
      context.config.minify,
    );
    // TODO: support source map
    try_with(
      context.meta.script.cm.clone(),
      &context.meta.script.globals,
      || {
        let css_code = wrapper_style_load(&css_code, module.id.to_string(), &css_deps);
        let mut ast = parse_module(
          &module.id.to_string(),
          &css_code,
          Syntax::default(),
          EsVersion::default(),
          context.meta.script.cm.clone(),
        )
        .unwrap();
        let top_level_mark = Mark::new();
        let unresolved_mark = Mark::new();

        ast.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

        module.meta = ModuleMetaData::Script(ScriptModuleMetaData {
          ast,
          top_level_mark: top_level_mark.as_u32(),
          unresolved_mark: unresolved_mark.as_u32(),
          module_system: ModuleSystem::EsModule,
          hmr_accepted: true,
        });

        module.module_type = ModuleType::Js;
      },
    )?;
  }

  if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
    resource_pot.resource_pot_type = ResourcePotType::Js;
  }

  Ok(())
}

pub fn transform_css_stylesheet(
  module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Stylesheet {
  let mut stylesheet = {
    let module = module_graph.module_mut(module_id).unwrap();
    module.meta.as_css_mut().take_ast()
  };

  let resources_map = context.resources_map.lock();
  source_replace(&mut stylesheet, module_id, module_graph, &resources_map);

  stylesheet
}

pub fn transform_css_deps(
  module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) -> String {
  let mut load_statements = Vec::new();
  let dep_modules = module_graph.dependencies(module_id);
  for (module, _) in dep_modules {
    let relative_path = module.id(context.config.mode.clone()).to_string();
    let load_statement = format!(
      "farmRequire(\"{}\");",
      if cfg!(windows) {
        relative_path.replace('\\', "\\\\")
      } else {
        relative_path.to_string()
      }
    );
    load_statements.push(load_statement);
  }
  load_statements.join(" ")
}
