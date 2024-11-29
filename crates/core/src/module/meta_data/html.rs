use std::collections::HashMap;

use farmfe_macro_cache_item::cache_item;
use swc_html_ast::Document;

use super::custom::CustomMetaDataMap;

#[cache_item]
pub struct HtmlModuleMetaData {
  pub ast: Document,
  pub custom: CustomMetaDataMap,
}

impl Clone for HtmlModuleMetaData {
  fn clone(&self) -> Self {
    let custom = if self.custom.is_empty() {
      HashMap::new()
    } else {
      let mut custom = HashMap::new();
      for (k, v) in self.custom.iter() {
        let cloned_data = v.serialize_bytes().unwrap();
        let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
        custom.insert(k.clone(), cloned_custom);
      }
      custom
    };

    Self {
      ast: self.ast.clone(),
      custom: CustomMetaDataMap::from(custom),
    }
  }
}
