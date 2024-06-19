#![feature(box_patterns)]

use std::{collections::HashMap, sync::Arc};

use farmfe_core::{
  parking_lot::Mutex,
  plugin::Plugin,
  resource::resource_pot::{ResourcePotMetaData, ResourcePotType},
};
use resource_pot_to_bundle::{Polyfill, SharedBundle};

pub mod resource_pot_to_bundle;

const MODULE_NEED_POLYFILLS: [Polyfill; 3] = [
  Polyfill::Wildcard,
  Polyfill::InteropRequireDefault,
  Polyfill::ExportStar,
];

#[derive(Default)]
pub struct FarmPluginBundle {
  runtime_code: Mutex<Arc<String>>,
}

impl FarmPluginBundle {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Plugin for FarmPluginBundle {
  fn name(&self) -> &str {
    "farm-plugin-bundle"
  }

  fn process_resource_pots(
    &self,
    resource_pots: &mut Vec<&mut farmfe_core::resource::resource_pot::ResourcePot>,
    context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !self.runtime_code.lock().is_empty() {
      return Ok(None);
    }
    let module_graph = context.module_graph.read();

    resource_pots.sort_by_key(|item| item.id.clone());

    for resource_pot in resource_pots {
      if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
        let mut shared_bundle = SharedBundle::new(vec![&resource_pot], &module_graph, context)?;

        let polyfill = &mut shared_bundle
          .bundle_map
          .get_mut(&resource_pot.id)
          .unwrap()
          .polyfill;

        MODULE_NEED_POLYFILLS
          .iter()
          .for_each(|item| polyfill.add(item.clone()));
        shared_bundle.render()?;

        let resource_pot_id = resource_pot.id.clone();

        let bundle = shared_bundle.codegen(&resource_pot_id)?;

        resource_pot.defer_minify_as_resource_pot();

        *self.runtime_code.lock() = Arc::new(bundle.to_string());
        break;
      }
    }

    Ok(None)
  }

  fn render_resource_pot_modules(
    &self,
    resource_pot: &farmfe_core::resource::resource_pot::ResourcePot,
    _context: &Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::resource::resource_pot::ResourcePotMetaData>>
  {
    if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
      return Ok(Some(ResourcePotMetaData {
        rendered_modules: HashMap::new(),
        rendered_content: self.runtime_code.lock().clone(),
        rendered_map_chain: vec![],
        custom_data: resource_pot.meta.custom_data.clone(),
      }));
    }

    Ok(None)
  }
}
