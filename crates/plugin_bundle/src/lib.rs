#![feature(box_patterns)]

use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  sync::Arc,
};

use farmfe_core::{
  enhanced_magic_string::bundle::Bundle,
  parking_lot::Mutex,
  plugin::Plugin,
  regex::Regex,
  resource::{
    resource_pot::{ResourcePotMetaData, ResourcePotType},
    ResourceType,
  },
};
use resource_pot_to_bundle::{Polyfill, SharedBundle, FARM_BUNDLE_REFERENCE_SLOT_PREFIX};

pub mod resource_pot_to_bundle;

const MODULE_NEED_POLYFILLS: [Polyfill; 3] = [
  Polyfill::Wildcard,
  Polyfill::InteropRequireDefault,
  Polyfill::ExportStar,
];

#[derive(Default)]
pub struct FarmPluginBundle {
  runtime_code: Mutex<Arc<String>>,
  bundle_map: Mutex<HashMap<String, Bundle>>,
  resource_pot_id_resource_map: Mutex<HashMap<String, String>>,
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

    let r = resource_pots
      .iter()
      .filter(|item| {
        context.config.output.target_env.is_library()
          || matches!(item.resource_pot_type, ResourcePotType::Runtime)
      })
      .map(|item| &**item)
      .collect::<Vec<_>>();
    let mut shared_bundle = SharedBundle::new(r, &module_graph, context)?;

    let inject_resource_pot_id = resource_pots
      .iter()
      .find(|item| {
        (context.config.output.target_env.is_library() && item.entry_module.is_some())
          || matches!(item.resource_pot_type, ResourcePotType::Runtime)
      })
      .map(|i| i.id.clone());

    if let Some(resource_pot_id) = inject_resource_pot_id {
      let polyfill = &mut shared_bundle
        .bundle_map
        .get_mut(&resource_pot_id)
        .unwrap()
        .polyfill;

      MODULE_NEED_POLYFILLS
        .iter()
        .for_each(|item| polyfill.add(item.clone()));
    }

    shared_bundle.render()?;

    let mut defer_minify = vec![];
    for resource_pot in resource_pots.iter() {
      if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime)
        || (context.config.output.target_env.is_library()
          && resource_pot.resource_pot_type == ResourcePotType::Js)
      {
        let resource_pot_id = resource_pot.id.clone();

        let bundle = shared_bundle.codegen(&resource_pot_id)?;

        defer_minify.push(resource_pot_id.clone());

        if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
          *self.runtime_code.lock() = Arc::new(bundle.to_string());
        } else {
          self.bundle_map.lock().insert(resource_pot_id, bundle);
        }
      }
    }

    for resource_pot in resource_pots {
      if defer_minify.contains(&resource_pot.id) {
        resource_pot.defer_minify_as_resource_pot();
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
    } else if let Some(bundle) = self.bundle_map.lock().get(&resource_pot.id) {
      return Ok(Some(ResourcePotMetaData {
        // TODO
        rendered_modules: HashMap::new(),
        rendered_content: Arc::new(bundle.to_string()),
        rendered_map_chain: vec![],
        custom_data: resource_pot.meta.custom_data.clone(),
      }));
    }

    Ok(None)
  }

  fn process_generated_resources(
    &self,
    resources: &mut farmfe_core::plugin::PluginGenerateResourcesHookResult,
    _context: &Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if let Some(resource_pot_id) = resources.resource.info.as_ref().map(|i| i.id.clone()) {
      self
        .resource_pot_id_resource_map
        .lock()
        .insert(resource_pot_id, resources.resource.name.clone());
    }

    Ok(None)
  }

  fn finalize_resources(
    &self,
    param: &mut farmfe_core::plugin::PluginFinalizeResourcesHookParams,
    context: &Arc<farmfe_core::context::CompilationContext>,
  ) -> farmfe_core::error::Result<Option<()>> {
    if !context.config.output.target_env.is_library() {
      return Ok(None);
    }
    let mut map = HashMap::new();
    for (name, resource) in param.resources_map.iter() {
      map.insert(resource.info.as_ref().unwrap().id.clone(), name.clone());
    }

    for (name, resource) in param.resources_map.iter_mut() {
      if !matches!(
        resource.resource_type,
        ResourceType::Js | ResourceType::Runtime
      ) {
        continue;
      }
      let before = std::time::Instant::now();

      println!("\n\nresource name: {}", name);

      let mut content = String::from_utf8_lossy(&resource.bytes).to_string();

      let reg =
        Regex::new(format!("{}\\(\\(.+?\\)\\)", FARM_BUNDLE_REFERENCE_SLOT_PREFIX).as_str())
          .unwrap();

      let items = reg
        .captures_iter(&content)
        .into_iter()
        .flat_map(|i| {
          i.iter()
            .filter_map(|i| i)
            .map(|i| i.as_str().to_string())
            .collect::<Vec<_>>()
        })
        .map(|i| i.as_str().to_string())
        .collect::<HashSet<_>>();

      if items.is_empty() {
        continue;
      }

      for item in items {
        let resource_pot_id = item
          .trim_start_matches(FARM_BUNDLE_REFERENCE_SLOT_PREFIX)
          .trim_start_matches("((")
          .trim_end_matches("))");
        let resource_name = map
          .get(resource_pot_id)
          .expect("cannot find bundle reference, please ensure your resource cornet");

        content = content.replace(&item, resource_name);
      }

      resource.bytes = content.into_bytes();

      println!("resource_name time: {}", before.elapsed().as_secs_f32());
    }

    Ok(None)
  }
}
