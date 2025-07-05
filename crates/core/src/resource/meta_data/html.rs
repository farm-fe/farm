use farmfe_macro_cache_item::cache_item;
use swc_html_ast::Document;

use crate::module::CustomMetaDataMap;

#[cache_item]
#[derive(Debug, Clone)]
pub struct HtmlResourcePotMetaData {
  pub ast: Document,
  pub custom: CustomMetaDataMap,
}
