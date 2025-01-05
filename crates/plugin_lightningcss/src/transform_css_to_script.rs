use std::sync::Arc;
use farmfe_core::{context::CompilationContext, module::{ModuleId, ModuleMetaData, ModuleType}, rayon::iter::{IntoParallelIterator, ParallelIterator}, serde_json};
use lightningcss::stylesheet::StyleSheet;

use crate::LightingCssModuleMetaData;



pub fn transform_css_stylesheet(module_id:&ModuleId, context: &Arc<CompilationContext>)->StyleSheet {

  let mut module_graph = context.module_graph.write();
  let mut css_stylesheet = match &mut module_graph.module_mut(module_id).unwrap().meta {
    ModuleMetaData::Custom(meta) => {
      let meta = meta.downcast_ref::<LightingCssModuleMetaData>().unwrap();
      let ast: StyleSheet = serde_json::from_str(&meta.ast).unwrap();
      ast
    }
    _ => return Ok(None),
  };
  
}