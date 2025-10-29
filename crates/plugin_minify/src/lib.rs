use std::sync::Arc;

use farmfe_core::{
  config::{minify::MinifyOptions, Config},
  context::CompilationContext,
  plugin::Plugin,
  resource::resource_pot::{ResourcePot, ResourcePotType},
};
use minify_resource_pot::{minify_css, minify_js};

mod minify_resource_pot;

pub struct FarmPluginMinify {
  minify_options: MinifyOptions,
}

impl FarmPluginMinify {
  pub fn new(config: &Config) -> Self {
    Self {
      minify_options: config
        .minify
        .clone()
        .map(|val| MinifyOptions::from(val))
        .unwrap_or_default(),
    }
  }
}

impl Plugin for FarmPluginMinify {
  fn name(&self) -> &'static str {
    "FarmPluginMinify"
  }

  fn optimize_resource_pot(
    &self,
    resource_pot: &mut ResourcePot,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    let enable_minify = context.config.minify.enabled();

    if !enable_minify {
      return Ok(None);
    }

    // For library, minify will be handled specially after the module format transformation, see [emit_resource_pot] in plugin_library
    if matches!(
      resource_pot.resource_pot_type,
      ResourcePotType::DynamicEntryJs
    ) || (!context.config.output.target_env.is_library()
      && matches!(resource_pot.resource_pot_type, ResourcePotType::Js))
    {
      minify_js(resource_pot, &self.minify_options, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Css) {
      minify_css(resource_pot, context)?;
    } else if matches!(resource_pot.resource_pot_type, ResourcePotType::Html) {
      // html minify is handled in plugin html after all resources are injected in finalize_resources hook
    }

    Ok(None)
  }
}
