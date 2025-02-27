use farmfe_core::{
  config::Config,
  error::CompilationError,
  plugin::Plugin,
  resource::{
    meta_data::{js::JsResourcePotMetaData, ResourcePotMetaData},
    resource_pot::ResourcePotType,
  },
};
use farmfe_toolkit::script::concatenate_modules::{
  concatenate_modules_ast, ConcatenateModulesAstResult,
};

#[derive(Default)]
pub struct FarmPluginLibrary {}

impl FarmPluginLibrary {
  pub fn new(_: &Config) -> Self {
    Self::default()
  }
}

impl Plugin for FarmPluginLibrary {
  fn name(&self) -> &str {
    "FarmPluginLibrary"
  }

  // TODO: add a hook collect resource pot import/export info before render resource pot

  fn render_resource_pot(
    &self,
    resource_pot: &farmfe_core::resource::resource_pot::ResourcePot,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
    if resource_pot.resource_pot_type != ResourcePotType::Js {
      return Ok(None);
    }

    let module_graph = context.module_graph.read();

    let ConcatenateModulesAstResult {
      ast,
      module_ids,
      external_modules,
      source_map,
      comments,
      globals,
      unresolved_mark,
      top_level_mark,
    } = concatenate_modules_ast(
      resource_pot.entry_module.as_ref().unwrap(), // TODO: support dynamic imported entry module for multiple library bundle
      &resource_pot.modules,
      &module_graph,
      context,
    )
    .map_err(|e| CompilationError::GenericError(e.to_string()))?;

    context
      .meta
      .set_resource_pot_source_map(&resource_pot.id, source_map);

    // handle import/export between resource pots
    // if let Some(entry) = &resource_pot.entry_module {
    //   let entry_module = module_graph.module(entry).unwrap();
    //   let script_meta = entry_module.meta.as_script();

    //   if !script_meta.export_ident_map.is_empty() {
    //     let export_item = script_meta.get_export_module_item();
    //     ast.body.push(export_item);
    //   }
    // }

    // TODO find exports in this resource pot

    Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
      ast,
      external_modules: external_modules
        .into_values()
        .map(|id| id.to_string())
        .collect(),
      rendered_modules: module_ids,
      comments,
    })))
  }
}
