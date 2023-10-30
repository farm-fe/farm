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
  // TODO parallelize here

  if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
    resource_pot.resource_pot_type = ResourcePotType::Js;
  }

  Ok(())
}
