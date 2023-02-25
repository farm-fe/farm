use farmfe_core::{
  config::{Mode, FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM},
  hashbrown::HashMap,
  module::ModuleId,
  resource::ResourceType,
  swc_html_ast::{Child, Document, Element},
};
use farmfe_toolkit::{
  html::create_element,
  swc_html_visit::{VisitMut, VisitMutWith},
};

use crate::deps_analyzer::{is_link_href, is_script_entry, is_script_src, FARM_ENTRY};

/// inject resources into the html ast
pub struct ResourcesInjector {
  runtime_code: String,
  script_resources: Vec<String>,
  css_resources: Vec<String>,
  script_entries: Vec<String>,
  dynamic_resources_map: HashMap<ModuleId, Vec<(String, ResourceType)>>,
  mode: Mode,
  public_path: String,
}

impl ResourcesInjector {
  pub fn new(
    runtime_code: String,
    script_resources: Vec<String>,
    css_resources: Vec<String>,
    script_entries: Vec<String>,
    dynamic_resources_map: HashMap<ModuleId, Vec<(String, ResourceType)>>,
    mode: Mode,
    public_path: String,
  ) -> Self {
    Self {
      runtime_code,
      css_resources,
      script_resources,
      script_entries,
      dynamic_resources_map,
      mode,
      public_path,
    }
  }

  pub fn inject(&mut self, ast: &mut Document) {
    ast.visit_mut_with(self);
  }
}

impl VisitMut for ResourcesInjector {
  fn visit_mut_element(&mut self, element: &mut Element) {
    let mut children_to_remove = vec![];

    // remove all non-http existing <href /> and <script /> first
    for (i, child) in element.children.iter().enumerate() {
      if let Child::Element(e) = child {
        if is_script_src(e) || is_script_entry(e) || is_link_href(e) {
          children_to_remove.push(i);
        }
      }
    }
    // remove from the end to the beginning, so that the index is not affected
    children_to_remove.reverse();
    children_to_remove.into_iter().for_each(|i| {
      element.children.remove(i);
    });

    if element.tag_name.to_string() == "head" {
      // inject css <link>
      for css in &self.css_resources {
        element.children.push(Child::Element(create_element(
          "link",
          None,
          vec![("rel", "stylesheet"), ("href", css)],
        )));
      }

      // inject runtime <script>
      let script_element = create_element(
        "script",
        Some(&self.runtime_code),
        vec![(FARM_ENTRY, "true")],
      );
      element.children.push(Child::Element(script_element));
    } else if element.tag_name.to_string() == "body" {
      for script in &self.script_resources {
        element.children.push(Child::Element(create_element(
          "script",
          None,
          vec![("src", script)],
        )));
      }

      let mut dynamic_resources_code = String::new();

      // inject dynamic resources
      for (module_id, resources) in &self.dynamic_resources_map {
        let mut resources_code = String::new();

        for (resource_name, resource_type) in resources {
          match resource_type {
            ResourceType::Js => {
              resources_code += &format!(r#"{{ path: '{}', type: 'script' }},"#, resource_name)
            }
            ResourceType::Css => {
              resources_code += &format!(r#"{{ path: '{}', type: 'link' }},"#, resource_name)
            }
            _ => {
              panic!(
                "unknown supported type ({:?}) when injecting dynamic resources",
                resource_type
              )
            }
          }
        }

        dynamic_resources_code += &format!(
          r#"'{}': [{}],"#,
          module_id.id(self.mode.clone()),
          resources_code
        );
      }

      dynamic_resources_code = format!("{{ {} }}", dynamic_resources_code);

      element.children.push(Child::Element(create_element(
        "script",
        Some(&format!(
          r#"var {} = globalThis || window || self;
            var __farm_module_system_local__ = {}.{};
            __farm_module_system_local__.setDynamicModuleResourcesMap({});"#,
          FARM_GLOBAL_THIS, FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM, dynamic_resources_code
        )),
        vec![(FARM_ENTRY, "true")],
      )));
      element.children.push(Child::Element(create_element(
        "script",
        Some(&format!(
          r#"var {} = globalThis || window || self;
            var __farm_module_system_local__ = {}.{};
            __farm_module_system_local__.setPublicPaths(['{}']);"#,
          FARM_GLOBAL_THIS, FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM, self.public_path
        )),
        vec![(FARM_ENTRY, "true")],
      )));

      element.children.push(Child::Element(create_element(
        "script",
        Some(&format!(
          r#"var {} = globalThis || window || self;
            var __farm_module_system_local__ = {}.{};
            __farm_module_system_local__.bootstrap();"#,
          FARM_GLOBAL_THIS, FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM
        )),
        vec![(FARM_ENTRY, "true")],
      )));

      for entry in &self.script_entries {
        element.children.push(Child::Element(create_element(
          "script",
          Some(&format!(
            r#"var {} = globalThis || window || self;
              var __farm_module_system_local__ = {}.{};
              __farm_module_system_local__.require("{}")"#,
            FARM_GLOBAL_THIS, FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM, entry
          )),
          vec![(FARM_ENTRY, "true")],
        )));
      }
    }

    element.visit_mut_children_with(self);
  }
}
