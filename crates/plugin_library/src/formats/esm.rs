use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  plugin::{GeneratedResource, PluginHookContext},
  resource::resource_pot::ResourcePot,
};

use crate::utils::{add_format_to_generated_resources, emit_resource_pot};

pub fn emit_esm_resources(
  resource_pot: &mut ResourcePot,
  context: &Arc<CompilationContext>,
  hook_context: &PluginHookContext,
) -> farmfe_core::error::Result<Vec<GeneratedResource>> {
  let mut resources = emit_resource_pot(resource_pot, context, hook_context)?;

  add_format_to_generated_resources(&mut resources, "esm");

  Ok(resources)
}
