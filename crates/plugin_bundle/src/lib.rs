#![feature(box_patterns)]
#![feature(let_chains)]

use std::sync::Arc;

use farmfe_core::{
  config::{
    config_regex::ConfigRegex, partial_bundling::PartialBundlingEnforceResourceConfig, Config,
  },
  context::CompilationContext,
  enhanced_magic_string::bundle::Bundle,
  module::ModuleType,
  parking_lot::Mutex,
  plugin::{
    Plugin, PluginAnalyzeDepsHookResultEntry, PluginFinalizeResourcesHookParam,
    PluginGenerateResourcesHookResult, PluginHookContext, PluginLoadHookParam,
    PluginLoadHookResult, PluginResolveHookParam, PluginResolveHookResult, ResolveKind,
  },
  regex::Regex,
  relative_path::RelativePath,
  resource::{
    meta_data::{js::JsResourcePotMetaData, ResourcePotMetaData},
    resource_pot::{ResourcePot, ResourcePotType},
    ResourceOrigin, ResourceType,
  },
  swc_ecma_ast::Module,
  HashMap, HashSet,
};
use farmfe_toolkit::constant::RUNTIME_SUFFIX;
use resource_pot_to_bundle::{
  BundleGroup, GeneratorAstResult, ShareBundleOptions, SharedBundle, FARM_BUNDLE_POLYFILL_SLOT,
  FARM_BUNDLE_REFERENCE_SLOT_PREFIX,
};

pub mod resource_pot_to_bundle;

// #[derive(Default)]
// pub struct FarmPluginBundle {
//   // runtime_code: Mutex<Option<GeneratorAstResult>>,
//   // bundle_map: Mutex<HashMap<String, GeneratorAstResult>>,
//   resource_pot_id_resource_map: Mutex<HashMap<String, String>>,
// }

// impl FarmPluginBundle {
//   pub fn new() -> Self {
//     Self::default()
//   }
// }

// impl FarmPluginBundle {
//   fn should_bundle(config: &Config) -> bool {
//     config.output.target_env.is_library()
//   }
// }

// impl Plugin for FarmPluginBundle {
//   fn name(&self) -> &str {
//     "farm-plugin-bundle"
//   }

//   fn config(&self, config: &mut Config) -> farmfe_core::error::Result<Option<()>> {
//     if Self::should_bundle(&config) {
//       // push it last
//       config
//         .partial_bundling
//         .enforce_resources
//         .push(PartialBundlingEnforceResourceConfig {
//           name: "farm_runtime".to_string(),
//           test: vec![ConfigRegex::new(FARM_BUNDLE_POLYFILL_SLOT)],
//         });
//     }

//     Ok(None)
//   }

//   fn resolve(
//     &self,
//     param: &PluginResolveHookParam,
//     _context: &Arc<CompilationContext>,
//     _hook_context: &PluginHookContext,
//   ) -> farmfe_core::error::Result<Option<PluginResolveHookResult>> {
//     if param.source.starts_with(FARM_BUNDLE_POLYFILL_SLOT) {
//       return Ok(Some(PluginResolveHookResult {
//         resolved_path: FARM_BUNDLE_POLYFILL_SLOT.to_string(),
//         external: false,
//         side_effects: true,
//         query: vec![],
//         meta: Default::default(),
//       }));
//     }

//     Ok(None)
//   }

//   fn load(
//     &self,
//     param: &PluginLoadHookParam,
//     _context: &Arc<CompilationContext>,
//     _hook_context: &PluginHookContext,
//   ) -> farmfe_core::error::Result<Option<PluginLoadHookResult>> {
//     if param.resolved_path.starts_with(FARM_BUNDLE_POLYFILL_SLOT) {
//       return Ok(Some(PluginLoadHookResult {
//         // TODO: disable tree-shaking it
//         content: r#"export {}"#.to_string(),
//         module_type: ModuleType::Js,
//         source_map: None,
//       }));
//     }

//     Ok(None)
//   }

//   fn analyze_deps(
//     &self,
//     param: &mut farmfe_core::plugin::PluginAnalyzeDepsHookParam,
//     context: &Arc<CompilationContext>,
//   ) -> farmfe_core::error::Result<Option<()>> {
//     let module_graph = context.module_graph.read();

//     if Self::should_bundle(&context.config)
//       && module_graph.entries.contains_key(&param.module.id)
//       && param.module.module_type.is_script()
//       && !param.module.id.to_string().ends_with(RUNTIME_SUFFIX)
//     {
//       param.deps.push(PluginAnalyzeDepsHookResultEntry {
//         source: FARM_BUNDLE_POLYFILL_SLOT.to_string(),
//         kind: ResolveKind::Import,
//       });
//     }

//     Ok(None)
//   }

//   fn process_resource_pots(
//     &self,
//     resource_pots: &mut Vec<&mut ResourcePot>,
//     context: &Arc<CompilationContext>,
//   ) -> farmfe_core::error::Result<Option<()>> {
//     println!(
//       "process_resource_pots {} {}",
//       self.runtime_code.lock().is_some(),
//       resource_pots.len()
//     );
//     if self.runtime_code.lock().is_some() {
//       return Ok(None);
//     }

//     let module_graph = context.module_graph.read();

//     resource_pots.sort_by_key(|item| item.id.clone());

//     let r = resource_pots
//       .iter()
//       .filter(|item| {
//         context.config.output.target_env.is_library()
//           || matches!(item.resource_pot_type, ResourcePotType::Runtime)
//       })
//       .map(|item| BundleGroup::from(&**item))
//       .collect::<Vec<_>>();
//     let mut shared_bundle = SharedBundle::new(
//       r,
//       &module_graph,
//       context,
//       Some(ShareBundleOptions {
//         format: context.config.output.format,
//         ..Default::default()
//       }),
//     )?;

//     shared_bundle.render()?;

//     println!("process_resource_pots {}", resource_pots.len(),);

//     for resource_pot in resource_pots.iter() {
//       println!(
//         "bundle resource pot id {} {:?}",
//         resource_pot.id, resource_pot.resource_pot_type
//       );
//       if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime)
//         || (context.config.output.target_env.is_library()
//           && matches!(resource_pot.resource_pot_type, ResourcePotType::Js))
//       {
//         println!("bundle resource pot id {}", resource_pot.id);
//         let resource_pot_id = resource_pot.id.clone();

//         let module = shared_bundle.codegen(&resource_pot_id)?;

//         if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
//           *self.runtime_code.lock() = Some(module);
//         } else {
//           self.bundle_map.lock().insert(resource_pot_id, module);
//         }
//       }
//     }

//     Ok(None)
//   }

//   fn render_resource_pot(
//     &self,
//     resource_pot: &ResourcePot,
//     _context: &Arc<CompilationContext>,
//     _hook_context: &PluginHookContext,
//   ) -> farmfe_core::error::Result<Option<ResourcePotMetaData>> {
//     println!(
//       "render_resource_pot id {} {}",
//       resource_pot.id,
//       self.runtime_code.lock().is_some()
//     );
//     if matches!(resource_pot.resource_pot_type, ResourcePotType::Runtime) {
//       if let Some(code) = self.runtime_code.lock().as_ref() {
//         return Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
//           // ast: code.ast.clone(),
//           // comments: code.comments.clone(),
//           external_modules: Default::default(),
//           rendered_modules: Default::default(),
//         })));
//       }

//       return Ok(None);
//     } else if let Some(bundle) = self.bundle_map.lock().remove(&resource_pot.id) {
//       return Ok(Some(ResourcePotMetaData::Js(JsResourcePotMetaData {
//         // ast: bundle.ast,
//         // comments: bundle.comments,
//         external_modules: Default::default(),
//         rendered_modules: Default::default(),
//       })));
//     }

//     Ok(None)
//   }

//   fn process_generated_resources(
//     &self,
//     resources: &mut PluginGenerateResourcesHookResult,
//     _context: &Arc<CompilationContext>,
//   ) -> farmfe_core::error::Result<Option<()>> {
//     if let ResourceOrigin::ResourcePot(ref resource_pot_id) = resources.resource.origin {
//       self
//         .resource_pot_id_resource_map
//         .lock()
//         .insert(resource_pot_id.to_string(), resources.resource.name.clone());
//     }

//     Ok(None)
//   }

//   fn finalize_resources(
//     &self,
//     param: &mut PluginFinalizeResourcesHookParams,
//     context: &Arc<CompilationContext>,
//   ) -> farmfe_core::error::Result<Option<()>> {
//     if !context.config.output.target_env.is_library() {
//       return Ok(None);
//     }

//     let mut map = HashMap::default();

//     for (name, resource) in param.resources_map.iter() {
//       if let ResourceOrigin::ResourcePot(id) = &resource.origin {
//         map.insert(id.clone(), name.clone());
//       }
//     }

//     for (name, resource) in param.resources_map.iter_mut() {
//       if !matches!(
//         resource.resource_type,
//         ResourceType::Js | ResourceType::Runtime
//       ) {
//         continue;
//       }
//       let before = std::time::Instant::now();

//       let r = format!("/{}", name);
//       let relative_path = RelativePath::new(&r);

//       let mut content = String::from_utf8_lossy(&resource.bytes).to_string();

//       let reg =
//         Regex::new(format!("{}\\(\\(.+?\\)\\)", FARM_BUNDLE_REFERENCE_SLOT_PREFIX).as_str())
//           .unwrap();

//       let items = reg
//         .captures_iter(&content)
//         .flat_map(|i| {
//           i.iter()
//             .flatten()
//             .map(|i| i.as_str().to_string())
//             .collect::<Vec<_>>()
//         })
//         .map(|i| i.as_str().to_string())
//         .collect::<HashSet<_>>();

//       if items.is_empty() {
//         continue;
//       }

//       for item in items {
//         let resource_pot_id = item
//           .trim_start_matches(FARM_BUNDLE_REFERENCE_SLOT_PREFIX)
//           .trim_start_matches("((")
//           .trim_end_matches("))");
//         let resource_name = map
//           .get(resource_pot_id)
//           .expect("cannot find bundle reference, please ensure your resource cornet");

//         let r1 = format!("/{}", resource_name);

//         println!("resource pot id {} to {} ", resource_pot_id, r1);

//         let relative_resource_path = RelativePath::new(&r1);
//         content = content.replace(
//           &item,
//           &format!(
//             "./{}",
//             relative_path
//               .parent()
//               .map(|i| i.relative(relative_resource_path).to_string())
//               .unwrap()
//               .trim_start_matches("/")
//           ),
//         );
//       }

//       resource.bytes = content.into_bytes();

//       println!(
//         "resource_name {} time: {}",
//         name,
//         before.elapsed().as_secs_f32()
//       );
//     }

//     Ok(None)
//   }
// }
