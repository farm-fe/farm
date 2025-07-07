use farmfe_macro_cache_item::cache_item;
use swc_css_ast::Stylesheet;

use crate::module::{meta_data::script::CommentsMetaData, CustomMetaDataMap};

#[cache_item]
#[derive(Clone)]
pub struct CssResourcePotMetaData {
  pub ast: Stylesheet,
  pub comments: CommentsMetaData,
  pub custom: CustomMetaDataMap,
}
