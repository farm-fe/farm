use farmfe_core::{
  module::{meta_data::script::CommentsMetaData, ModuleId},
  swc_ecma_ast::Module as SwcModule,
};

pub struct RenderModuleResult {
  pub module_id: ModuleId,
  pub rendered_ast: SwcModule,
  pub comments: CommentsMetaData,
  pub external_modules: Vec<ModuleId>,
}
