use farmfe_macro_cache_item::cache_item;
use swc_html_ast::Document;

use super::custom::CustomMetaDataMap;
use crate::HashMap;

#[cache_item]
#[derive(Clone)]
pub struct HtmlModuleMetaData {
  pub ast: Document,
  pub custom: CustomMetaDataMap,
}
