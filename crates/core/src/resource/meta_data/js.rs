use farmfe_macro_cache_item::cache_item;
use swc_ecma_ast::Module as SwcModule;

use crate::{
  module::{meta_data::script::CommentsMetaData, ModuleId},
  HashSet,
};

#[cache_item]
#[derive(Clone, Default)]
pub struct JsResourcePotMetaData {
  pub ast: SwcModule,
  pub comments: CommentsMetaData,
  pub external_modules: HashSet<String>,
  pub rendered_modules: Vec<ModuleId>,
}

impl JsResourcePotMetaData {
  pub fn new() -> Self {
    Self::default()
  }
}
