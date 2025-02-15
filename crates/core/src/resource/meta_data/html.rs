use farmfe_macro_cache_item::cache_item;

use crate::module::CustomMetaDataMap;

#[cache_item]
#[derive(Debug, Clone)]
pub struct HtmlResourcePotMetaData {
  pub custom: CustomMetaDataMap,
}
