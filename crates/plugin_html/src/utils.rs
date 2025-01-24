use std::{borrow::Cow, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::ModuleId,
  resource::{Resource, ResourceOrigin, ResourceType},
  swc_html_ast::Element,
};
use farmfe_toolkit::fs::transform_output_filename;

use crate::deps_analyzer::{
  get_href_link_value, get_link_css_code, get_script_src_value, get_script_type_module_code,
};

pub const FARM_RESOURCE: &str = "data-farm-resource";

fn is_external_module(
  source: String,
  current_html_id: &ModuleId,
  context: &Arc<CompilationContext>,
) -> bool {
  let module_graph = context.module_graph.read();

  if let Some(id) = module_graph.get_dep_by_source_optional(current_html_id, &source, None) {
    if let Some(m) = module_graph.module(&id) {
      return m.external;
    }
  }

  false
}

pub fn is_script_src_or_type_module_code(
  element: &Element,
  current_html_id: &ModuleId,
  context: &Arc<CompilationContext>,
) -> bool {
  if let Some(v) = get_script_src_value(element) {
    !is_external_module(v, current_html_id, context)
  } else {
    get_script_type_module_code(element).is_some()
  }
}

pub fn is_link_css_or_code(
  element: &Element,
  current_html_id: &ModuleId,
  context: &Arc<CompilationContext>,
) -> bool {
  if let Some(v) = get_href_link_value(element) {
    !is_external_module(v, current_html_id, context)
  } else {
    get_link_css_code(element).is_some()
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

pub fn create_farm_runtime_output_resource(
  bytes: Cow<[u8]>,
  resource_name: &str,
  context: &Arc<CompilationContext>,
  already_inject_resources: &Vec<String>,
) -> (String, Option<Resource>) {
  let name = transform_output_filename(
    context.config.output.filename.clone(),
    resource_name,
    &bytes,
    "js", // todo: support configuring extension
          // match context.config.output.format {
          //   ModuleFormat::EsModule => "mjs",
          //   ModuleFormat::CommonJs => "cjs",
          // },
  );

  if already_inject_resources.contains(&name) {
    return (name, None);
  }

  (
    name.clone(),
    Some(Resource {
      name: name.clone(),
      bytes: bytes.to_owned().into(),
      emitted: false,
      resource_type: ResourceType::Js,
      origin: ResourceOrigin::ResourcePot(name),
      should_transform_output_filename: true,
      meta: Default::default(),
    }),
  )
}
