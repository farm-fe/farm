use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext, parking_lot::RwLock, plugin::Plugin,
  resource::resource_pot_graph::ResourcePotGraph,
};

/// Compiling and generate runtime resources, it will generate a executable runtime bootstrap code and inject the code into the entries.
/// The runtime supports html entry and script(js/jsx/ts/tsx) entry, when entry is html, the runtime will be injected as a inline <script /> tag in the <head /> tag;
/// when entry is script, the runtime will be injected into the entry module's front, make sure the runtime execute before all other code.
///
/// Note: If no compiled runtime file provided, a minimal runtime is used, the Farm runtime is pre-compiled with this minimal runtime. The minimal runtime only support script as entry.
/// ```
pub struct FarmPluginRuntime {}

impl Plugin for FarmPluginRuntime {
  fn name(&self) -> &str {
    "FarmPluginRuntime"
  }

  fn process_resource_pot_graph(
    &self,
    resource_pot_graph: &RwLock<ResourcePotGraph>,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if matches!(context.config.runtime.path, None) {
      let rpg = resource_pot_graph.read();

      if rpg.resources().len() != 1 {
        panic!("default runtime only works with single resource");
      }

      Ok(Some(()))
    } else {
      Ok(None)
    }
  }

  fn render_resource_pot(
    &self,
    _resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if matches!(context.config.runtime.path, None) {
      // Strip out the rendered module and wrap it with the minimal runtime
      Ok(Some(()))
    } else {
      Ok(None)
    }
  }
}
