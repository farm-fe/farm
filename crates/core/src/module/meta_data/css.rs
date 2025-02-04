use farmfe_macro_cache_item::cache_item;
use swc_common::DUMMY_SP;
use swc_css_ast::Stylesheet;

use crate::{Cacheable, HashMap};

use super::{custom::CustomMetaDataMap, script::CommentsMetaData};

#[cache_item]
#[derive(Clone)]

pub struct CssModuleMetaData {
  pub ast: Stylesheet,
  pub comments: CommentsMetaData,
  pub custom: CustomMetaDataMap,
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
