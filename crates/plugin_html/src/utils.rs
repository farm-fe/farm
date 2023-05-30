use farmfe_core::swc_html_ast::Element;

use crate::deps_analyzer::{get_href_link_value, get_script_src_value};

pub const FARM_ENTRY: &str = "data-farm-entry-script";
pub const FARM_RESOURCE: &str = "data-farm-resource";

pub fn is_script_src(element: &Element) -> bool {
  if let Some(v) = get_script_src_value(element) {
    if !v.starts_with("http") {
      true
    } else {
      false
    }
  } else {
    false
  }
}

pub fn is_script_entry(element: &Element) -> bool {
  if element.tag_name.to_string() == "script" {
    let src_attr = element
      .attributes
      .iter()
      .find(|&attr| attr.name.to_string() == FARM_ENTRY);

    return src_attr.is_some();
  }

  false
}

pub fn is_link_href(element: &Element) -> bool {
  if let Some(v) = get_href_link_value(element) {
    if !v.starts_with("http") {
      true
    } else {
      false
    }
  } else {
    false
  }
}

pub fn is_script_resource(element: &Element) -> bool {
  if element.tag_name.to_string() == "script" {
    let src_attr = element
      .attributes
      .iter()
      .find(|&attr| attr.name.to_string() == FARM_RESOURCE);

    return src_attr.is_some();
  }

  false
}
