use farmfe_core::{
  config::{FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM},
  swc_html_ast::{Child, Document, Element},
};
use farmfe_toolkit::{
  html::{create_element_with_attrs, create_element_with_text},
  swc_html_visit::{VisitMut, VisitMutWith},
};

use crate::deps_analyzer::{is_link_href, is_script_src};

/// inject resources into the html ast
pub struct ResourcesInjector {
  runtime_code: String,
  script_resources: Vec<String>,
  css_resources: Vec<String>,
  script_entries: Vec<String>,
}

impl ResourcesInjector {
  pub fn new(
    runtime_code: String,
    script_resources: Vec<String>,
    css_resources: Vec<String>,
    script_entries: Vec<String>,
  ) -> Self {
    Self {
      runtime_code,
      css_resources,
      script_resources,
      script_entries,
    }
  }

  pub fn inject(&mut self, ast: &mut Document) {
    ast.visit_mut_with(self);
  }
}

impl VisitMut for ResourcesInjector {
  fn visit_mut_element(&mut self, element: &mut Element) {
    let mut children_to_remove = vec![];

    // remove all existing <href /> and <script /> first
    for (i, child) in element.children.iter().enumerate() {
      if let Child::Element(e) = child {
        if is_script_src(e) || is_link_href(e) {
          children_to_remove.push(i);
        }
      }
    }

    children_to_remove.into_iter().for_each(|i| {
      element.children.remove(i);
    });

    if element.tag_name.to_string() == "head" {
      // inject css <link>
      for css in &self.css_resources {
        element
          .children
          .push(Child::Element(create_element_with_attrs(
            "link",
            vec![("rel", "stylesheet"), ("href", css)],
          )));
      }

      // inject runtime <script>
      element
        .children
        .push(Child::Element(create_element_with_text(
          "script",
          &self.runtime_code,
        )));
    } else if element.tag_name.to_string() == "body" {
      for script in &self.script_resources {
        element
          .children
          .push(Child::Element(create_element_with_attrs(
            "script",
            vec![("src", script)],
          )));
      }

      for entry in &self.script_entries {
        element
          .children
          .push(Child::Element(create_element_with_text(
            "script",
            &format!(
              r#"var {} = globalThis || window || self;
              var __farm_module_system_local__ = {}.{};
              __farm_module_system_local__.bootstrap();
              __farm_module_system_local__.require("{}")"#,
              FARM_GLOBAL_THIS, FARM_GLOBAL_THIS, FARM_MODULE_SYSTEM, entry
            ),
          )));
      }
    }

    element.visit_mut_children_with(self);
  }
}
