use farmfe_macro_cache_item::cache_item;
use swc_css_ast::Stylesheet;

use crate::module::CustomMetaDataMap;

#[cache_item]
#[derive(Debug, Clone)]
pub struct CssResourcePotMetaData {
  pub ast: Stylesheet,
  pub custom: CustomMetaDataMap,
}
