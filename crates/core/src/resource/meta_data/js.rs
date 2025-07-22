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
  pub top_level_mark: u32,
  pub unresolved_mark: u32,
}

impl JsResourcePotMetaData {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn take_comments(&mut self) -> CommentsMetaData {
    std::mem::take(&mut self.comments)
  }

  pub fn set_comments(&mut self, comments: CommentsMetaData) {
    self.comments = comments;
  }

  pub fn take_ast(&mut self) -> SwcModule {
    std::mem::take(&mut self.ast)
  }

  pub fn set_ast(&mut self, ast: SwcModule) {
    self.ast = ast;
  }
}
