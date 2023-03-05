use rkyv::{Archive, Deserialize, Serialize};

use farmfe_macro_cache_item::cache_item;

use self::resource_pot::ResourcePotId;

pub mod resource_pot;
pub mod resource_pot_map;

#[cache_item]
#[derive(Debug, Clone)]
pub enum ResourceType {
  Runtime,
  Js,
  Css,
  Html,
  SourceMap,
  Asset(String),
  Custom(String),
}

impl ResourceType {
  pub fn to_ext(&self) -> String {
    match self {
      ResourceType::Asset(str) => str.to_string(),
      ResourceType::Custom(str) => str.to_string(),
      ResourceType::Runtime => "js".to_string(),
      ResourceType::Js => "js".to_string(),
      ResourceType::Css => "css".to_string(),
      ResourceType::Html => "html".to_string(),
      ResourceType::SourceMap => "map".to_string(),
    }
  }

  pub fn to_html_tag(&self) -> String {
    match self {
      ResourceType::Asset(str) => str.to_string(),
      ResourceType::Custom(str) => str.to_string(),
      ResourceType::Runtime => "script".to_string(),
      ResourceType::Js => "script".to_string(),
      ResourceType::Css => "link".to_string(),
      ResourceType::Html => "html".to_string(),
      ResourceType::SourceMap => "map".to_string(),
    }
  }
}

#[cache_item]
pub struct Resource {
  pub name: String,
  pub bytes: Vec<u8>,
  /// whether this resource emitted, true means this resource will not present in the final production
  pub emitted: bool,
  pub resource_type: ResourceType,
  /// the resource pot this [Resource] generated from
  pub resource_pot: ResourcePotId,
  /// true means this resource's name should not be changed according to the [Config].
  pub preserve_name: bool,
}
