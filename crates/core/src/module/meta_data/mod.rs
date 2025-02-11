use std::collections::HashMap;

use css::CssModuleMetaData;
use custom::CustomMetaDataMap;
use farmfe_macro_cache_item::cache_item;
use html::HtmlModuleMetaData;
use script::ScriptModuleMetaData;

use crate::Cacheable;

pub mod css;
pub mod custom;
pub mod html;
pub mod script;

/// Module meta data shared by core plugins through the compilation
/// Meta data which is not shared by core plugins should be stored in [ModuleMetaData::Custom]
#[cache_item]
pub enum ModuleMetaData {
  Script(Box<ScriptModuleMetaData>),
  Css(Box<CssModuleMetaData>),
  Html(Box<HtmlModuleMetaData>),
  Custom(CustomMetaDataMap),
}

impl Default for ModuleMetaData {
  fn default() -> Self {
    Self::Custom(CustomMetaDataMap::default())
  }
}

impl ToString for ModuleMetaData {
  fn to_string(&self) -> String {
    match self {
      Self::Script(_) => "script".to_string(),
      Self::Css(_) => "css".to_string(),
      Self::Html(_) => "html".to_string(),
      Self::Custom(_) => "custom".to_string(),
    }
  }
}

impl Clone for ModuleMetaData {
  fn clone(&self) -> Self {
    match self {
      Self::Script(script) => Self::Script(script.clone()),
      Self::Css(css) => Self::Css(css.clone()),
      Self::Html(html) => Self::Html(html.clone()),
      Self::Custom(custom) => {
        let mut custom_new = HashMap::default();
        for (k, v) in custom.iter() {
          let cloned_data = v.serialize_bytes().unwrap();
          let cloned_custom = v.deserialize_bytes(cloned_data).unwrap();
          custom_new.insert(k.clone(), cloned_custom);
        }
        Self::Custom(CustomMetaDataMap::from(custom_new))
      }
    }
  }
}

impl ModuleMetaData {
  pub fn as_script_mut(&mut self) -> &mut ScriptModuleMetaData {
    if let Self::Script(script) = self {
      script
    } else {
      panic!("ModuleMetaData is not Script")
    }
  }

  pub fn as_script(&self) -> &ScriptModuleMetaData {
    if let Self::Script(script) = self {
      script
    } else {
      panic!("ModuleMetaData is not Script but {:?}", self.to_string())
    }
  }

  pub fn as_css(&self) -> &CssModuleMetaData {
    if let Self::Css(css) = self {
      css
    } else {
      panic!("ModuleMetaData is not css")
    }
  }

  pub fn as_css_mut(&mut self) -> &mut CssModuleMetaData {
    if let Self::Css(css) = self {
      css
    } else {
      panic!("ModuleMetaData is not css")
    }
  }

  pub fn as_html(&self) -> &HtmlModuleMetaData {
    if let Self::Html(html) = self {
      html
    } else {
      panic!("ModuleMetaData is not html")
    }
  }

  pub fn as_html_mut(&mut self) -> &mut HtmlModuleMetaData {
    if let Self::Html(html) = self {
      html
    } else {
      panic!("ModuleMetaData is not html")
    }
  }

  /// get custom meta data by key
  pub fn get_custom_mut<T: Cacheable + Default>(&mut self, key: &str) -> &mut T {
    if let Self::Custom(custom) = self {
      custom.get_mut(key).unwrap()
    } else {
      panic!("ModuleMetaData is not Custom")
    }
  }
}
