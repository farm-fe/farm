use css::CssResourcePotMetaData;
use farmfe_macro_cache_item::cache_item;
use html::HtmlResourcePotMetaData;
use js::JsResourcePotMetaData;

use crate::{module::{meta_data::custom::{CustomMetaDataMapRef, CustomMetaDataMapRefMut}, CustomMetaDataMap}, Cacheable};

pub mod css;
pub mod html;
pub mod js;

/// Info data which is not shared by core plugins should be stored in [ResourcePotInfo::Custom]
#[cache_item]
#[derive(Clone)]
pub enum ResourcePotMetaData {
  Js(JsResourcePotMetaData),
  Css(CssResourcePotMetaData),
  Html(HtmlResourcePotMetaData),
  Custom(CustomMetaDataMap),
}

impl Default for ResourcePotMetaData {
  fn default() -> Self {
    Self::Custom(CustomMetaDataMap::default())
  }
}

impl ResourcePotMetaData {
  pub fn as_js(&self) -> &JsResourcePotMetaData {
    match self {
      Self::Js(info) => info,
      _ => panic!("ResourcePotInfo is not ResourcePotInfo::Script"),
    }
  }

  pub fn as_css(&self) -> &CssResourcePotMetaData {
    match self {
      Self::Css(info) => info,
      _ => panic!("ResourcePotInfo is not ResourcePotInfo::Css"),
    }
  }

  pub fn as_html(&self) -> &HtmlResourcePotMetaData {
    match self {
      Self::Html(info) => info,
      _ => panic!("ResourcePotInfo is not ResourcePotInfo::Html"),
    }
  }

  pub fn as_html_mut(&mut self) -> &mut HtmlResourcePotMetaData {
    match self {
      Self::Html(info) => info,
      _ => panic!("ResourcePotInfo is not ResourcePotInfo::Html"),
    }
  }

  /// get custom meta data by key
  pub fn get_custom_mut<T: Cacheable + Default>(&self, key: &str) -> CustomMetaDataMapRefMut<T> {
    if let Self::Custom(custom) = self {
      custom.get_mut(key).unwrap()
    } else {
      panic!("ModuleMetaData is not Custom")
    }
  }

  pub fn get_custom<T: Cacheable + Default>(&self, key: &str) -> CustomMetaDataMapRef<T> {
    if let Self::Custom(custom) = self {
      custom.get(key).unwrap()
    } else {
      panic!("ModuleMetaData is not Custom")
    }
  }
}
