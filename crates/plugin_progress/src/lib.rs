// use farmfe_core::{
//   cache_item, config::Config, context::CompilationContext, custom_meta_data, error::Result,
//   parking_lot::Mutex, plugin::Plugin,
// };
// use indicatif::{ProgressBar, ProgressStyle};
// use std::{sync::Arc, time::Duration};

// pub struct FarmPluginProgress {
//   module_count: Arc<Mutex<u32>>,
//   progress_bar: ProgressBar,
//   first_build: Mutex<bool>,
// }

// impl FarmPluginProgress {
//   pub fn new(_config: &Config) -> Self {
//     let spinner_style =
//       ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.green} {wide_msg}")
//         .unwrap()
//         .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

//     let progress_bar = ProgressBar::new(1);
//     progress_bar.set_style(spinner_style.clone());
//     progress_bar.set_prefix("[ building ]");

//     // tick every 200ms
//     progress_bar.enable_steady_tick(Duration::from_millis(200));

//     Self {
//       module_count: Arc::new(Mutex::new(0)),
//       progress_bar,
//       first_build: Mutex::new(true),
//     }
//   }

//   pub fn increment_module_count(&self) {
//     let mut count = self.module_count.lock();
//     *count += 1;
//   }

//   pub fn reset_module_count(&self) {
//     let mut count = self.module_count.lock();
//     *count = 0;
//   }

//   pub fn get_module_count(&self) -> u32 {
//     let count = self.module_count.lock();
//     *count
//   }
// }

// impl Plugin for FarmPluginProgress {
//   fn name(&self) -> &'static str {
//     "FarmPluginProgress"
//   }

//   fn priority(&self) -> i32 {
//     i32::MAX
//   }

//   fn update_modules(
//     &self,
//     _params: &mut farmfe_core::plugin::PluginUpdateModulesHookParams,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<()>> {
//     self.progress_bar.reset();
//     self.reset_module_count();
//     Ok(None)
//   }

//   fn build_start(&self, _context: &Arc<CompilationContext>) -> Result<Option<()>> {
//     self.reset_module_count();
//     Ok(None)
//   }

//   fn transform(
//     &self,
//     param: &farmfe_core::plugin::PluginTransformHookParam,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<farmfe_core::plugin::PluginTransformHookResult>> {
//     self.increment_module_count();
//     let count = self.get_module_count();
//     let module = &param.module_id;
//     self
//       .progress_bar
//       .set_message(format!("transform ({count}) {module}"));
//     self.progress_bar.inc(1);

//     Ok(None)
//   }

//   fn handle_persistent_cached_module(
//     &self,
//     module: &farmfe_core::module::Module,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<bool>> {
//     self.increment_module_count();
//     let count = self.get_module_count();
//     let module = &module.id;
//     self.progress_bar.set_message(format!(
//       "load cached module({count}) {}",
//       module.to_string()
//     ));
//     self.progress_bar.inc(1);

//     Ok(None)
//   }

//   fn module_graph_updated(
//     &self,
//     _param: &farmfe_core::plugin::PluginModuleGraphUpdatedHookParams,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<()>> {
//     let first_build = self.first_build.lock();

//     if !*first_build {
//       self.progress_bar.finish_and_clear();
//     }

//     Ok(None)
//   }

//   fn optimize_module_graph(
//     &self,
//     _module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<()>> {
//     let first_build = self.first_build.lock();

//     if *first_build {
//       self.progress_bar.set_message("optimize module graph");
//       self.progress_bar.inc(1);
//     }

//     Ok(None)
//   }

//   fn render_resource_pot_modules(
//     &self,
//     resource_pot: &farmfe_core::resource::resource_pot::ResourcePot,
//     _context: &Arc<CompilationContext>,
//     _hook_context: &farmfe_core::plugin::PluginHookContext,
//   ) -> Result<Option<farmfe_core::resource::resource_pot::ResourcePotMetaData>> {
//     let first_build = self.first_build.lock();

//     if *first_build {
//       self
//         .progress_bar
//         .set_message(format!("render resource pot modules {}", resource_pot.name));
//       self.progress_bar.inc(1);
//     }

//     Ok(None)
//   }

//   fn render_resource_pot(
//     &self,
//     param: &farmfe_core::plugin::PluginRenderResourcePotHookParam,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<farmfe_core::plugin::PluginRenderResourcePotHookResult>> {
//     let first_build = self.first_build.lock();

//     if *first_build {
//       let name: &String = &param.resource_pot_info.name.clone();
//       self.progress_bar.set_message(format!("render {name}"));
//       self.progress_bar.inc(1);
//     }

//     Ok(None)
//   }

//   fn optimize_resource_pot(
//     &self,
//     resource: &mut farmfe_core::resource::resource_pot::ResourcePot,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<()>> {
//     let first_build = self.first_build.lock();

//     if *first_build {
//       self
//         .progress_bar
//         .set_message(format!("optimize resource pot {}", resource.name));
//       self.progress_bar.inc(1);
//     }

//     Ok(None)
//   }

//   fn generate_resources(
//     &self,
//     resource_pot: &mut farmfe_core::resource::resource_pot::ResourcePot,
//     _context: &Arc<CompilationContext>,
//     _hook_context: &farmfe_core::plugin::PluginHookContext,
//   ) -> Result<Option<farmfe_core::plugin::PluginGenerateResourcesHookResult>> {
//     let first_build = self.first_build.lock();

//     if *first_build {
//       self.progress_bar.set_message(format!(
//         "generate resources for resource pot {}",
//         resource_pot.name
//       ));
//       self.progress_bar.inc(1);
//     }

//     Ok(None)
//   }

//   fn finalize_resources(
//     &self,
//     _param: &mut farmfe_core::plugin::PluginFinalizeResourcesHookParams,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<()>> {
//     let first_build = self.first_build.lock();

//     if *first_build {
//       self.progress_bar.set_message("finalize resources...");
//       self.progress_bar.inc(1);
//     }

//     Ok(None)
//   }

//   fn finish(
//     &self,
//     _stat: &farmfe_core::stats::Stats,
//     _context: &Arc<CompilationContext>,
//   ) -> Result<Option<()>> {
//     let mut first_build = self.first_build.lock();

//     if *first_build {
//       self.progress_bar.finish_and_clear();

//       *first_build = false;
//     }

//     Ok(None)
//   }
// }

use farmfe_core::custom_meta_data;
use farmfe_core::module::{
  CustomModuleMetaData, Module, ModuleMetaData, SerializeCustomModuleMetaData,
};
use rkyv::*;
use rkyv_dyn::archive_dyn;
use rkyv_typename::TypeName;

#[archive_dyn(deserialize)]
trait ExampleTrait: CustomModuleMetaData {
  fn value(&self) -> String;
}

#[derive(Archive, Serialize, Deserialize)]
#[archive_attr(derive(TypeName))]
struct StringStruct(String);

#[archive_dyn(deserialize)]
impl ExampleTrait for StringStruct {
  fn value(&self) -> String {
    self.0.clone()
  }
}

impl ExampleTrait for Archived<StringStruct> {
  fn value(&self) -> String {
    self.0.as_str().to_string()
  }
}

impl CustomModuleMetaData for ArchivedStringStruct {}
impl CustomModuleMetaData for StringStruct {}
impl SerializeCustomModuleMetaData for StringStruct {}
