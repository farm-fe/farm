use farmfe_macro_cache_item::cache_item;
use swc_ecma_ast::Module as SwcModule;

use crate::{
  module::{meta_data::script::CommentsMetaData, ModuleId},
  HashSet,
};

#[cache_item]
#[derive(Clone)]
pub struct RenderModuleResult {
  pub module_id: ModuleId,
  pub rendered_ast: SwcModule,
  pub hoisted_module_ids: Vec<ModuleId>,
  pub comments: CommentsMetaData,
  pub external_modules: Vec<ModuleId>,
}

#[cache_item]
#[derive(Clone, Default)]
pub struct JsResourcePotMetaData {
  pub ast: SwcModule,
  pub external_modules: HashSet<String>,
  pub rendered_modules: Vec<ModuleId>,
  pub comments: CommentsMetaData,
}

impl JsResourcePotMetaData {
  pub fn new() -> Self {
    Self::default()
  }
}
