use farmfe_macro_cache_item::cache_item;
use swc_common::DUMMY_SP;
use swc_css_ast::Stylesheet;

use crate::HashMap;

use super::{custom::CustomMetaDataMap, script::CommentsMetaData};

#[cache_item]
pub struct CssModuleMetaData {
  pub ast: Stylesheet,
  pub comments: CommentsMetaData,
  pub custom: CustomMetaDataMap,
}

impl Clone for CssModuleMetaData {
  fn clone(&self) -> Self {
    let custom = if self.custom.is_empty() {
      HashMap::default()
    } else {
      let mut custom = HashMap::default();
      for (k, v) in self.custom.iter() {
        let cloned_data = v.serialize_bytes().unwrap();
        let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
        custom.insert(k.clone(), cloned_custom);
      }
      custom
    };

    Self {
      ast: self.ast.clone(),
      comments: self.comments.clone(),
      custom: CustomMetaDataMap::from(custom),
    }
  }
}

impl CssModuleMetaData {
  pub fn take_ast(&mut self) -> Stylesheet {
    std::mem::replace(
      &mut self.ast,
      Stylesheet {
        span: DUMMY_SP,
        rules: vec![],
      },
    )
  }

  pub fn set_ast(&mut self, ast: Stylesheet) {
    self.ast = ast;
  }
}
