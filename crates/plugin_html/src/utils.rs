use std::sync::Arc;

use farmfe_core::{context::CompilationContext, module::ModuleId, swc_html_ast::Element};

use crate::deps_analyzer::{get_href_link_value, get_script_src_value};

pub const FARM_ENTRY: &str = "data-farm-entry-script";
pub const FARM_RESOURCE: &str = "data-farm-resource";

fn is_external_module(
  source: String,
  current_html_id: &ModuleId,
  context: &Arc<CompilationContext>,
) -> bool {
  let module_graph = context.module_graph.read();

  if let Some(id) = module_graph.get_dep_by_source_optional(current_html_id, &source) {
    if let Some(m) = module_graph.module(&id) {
      return m.external;
    }
  }

  false
}

pub fn is_script_src(
  element: &Element,
  current_html_id: &ModuleId,
  context: &Arc<CompilationContext>,
) -> bool {
  if let Some(v) = get_script_src_value(element) {
    !is_external_module(v, current_html_id, context)
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

pub fn is_link_href(
  element: &Element,
  current_html_id: &ModuleId,
  context: &Arc<CompilationContext>,
) -> bool {
  if let Some(v) = get_href_link_value(element) {
    !is_external_module(v, current_html_id, context)
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
