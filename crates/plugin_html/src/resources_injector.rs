use std::{borrow::Cow, sync::Arc};

use farmfe_core::{
  config::{custom::get_config_runtime_isolate, FARM_MODULE_SYSTEM},
  context::CompilationContext,
  module::ModuleId,
  resource::Resource,
  swc_html_ast::{Child, Document, Element},
};
use farmfe_toolkit::{
  html::{create_element, get_farm_global_this},
  swc_html_visit::{VisitMut, VisitMutWith},
};

use crate::utils::{
  create_farm_runtime_output_resource, is_link_css_or_code, is_script_resource,
  is_script_src_or_type_module_code, FARM_RESOURCE,
};

pub struct ResourcesInjectorOptions {
  pub public_path: String,
  pub namespace: String,
  pub current_html_id: ModuleId,
  pub context: Arc<CompilationContext>,
}

/// inject resources into the html ast
pub struct ResourcesInjector<'a> {
  pub additional_inject_resources: Vec<Resource>,
  runtime_code: &'a str,
  script_resources: Vec<String>,
  css_resources: Vec<String>,
  script_entries: Vec<String>,
  dynamic_resources: &'a str,
  dynamic_module_resources_map: &'a str,
  options: ResourcesInjectorOptions,
  farm_global_this: String,
}
pub const FARM_RUNTIME_INJECT_RESOURCE: &str = "farm_runtime_resource";
pub const FARM_MODULE_SYSTEM_BOOTSTRAP: &str = "farm_module_system_bootstrap";

impl<'a> ResourcesInjector<'a> {
  pub fn new(
    additional_inject_resources: Vec<Resource>,
    runtime_code: &'a str,
    script_resources: Vec<String>,
    css_resources: Vec<String>,
    script_entries: Vec<String>,
    dynamic_resources: &'a str,
    dynamic_module_resources_map: &'a str,
    options: ResourcesInjectorOptions,
  ) -> Self {
    Self {
      additional_inject_resources,
      runtime_code,
      css_resources,
      script_resources,
      script_entries,
      dynamic_resources,
      dynamic_module_resources_map,
      farm_global_this: get_farm_global_this(
        &options.namespace,
        &options.context.config.output.target_env,
      ),
      options,
    }
  }

  pub fn inject(&mut self, ast: &mut Document) {
    ast.visit_mut_with(self);
  }

  // Support isolate runtime resource (https://github.com/farm-fe/farm/issues/434)
  fn inject_runtime_resources(&mut self, element: &mut Element) {
    element.children.push(Child::Element(create_element(
      "script",
      Some(self.runtime_code),
      vec![],
    )));
  }

  fn get_initial_resources_code(&self) -> String {
    let mut initial_resources = vec![];
    initial_resources.extend(self.script_resources.clone());
    initial_resources.extend(self.css_resources.clone());
    initial_resources.sort();

    let initial_resources_code = initial_resources
      .into_iter()
      .map(|path| format!("'{path}'"))
      .collect::<Vec<_>>()
      .join(",");

    format!(
      r#"{}.{}.si([{}]);"#,
      self.farm_global_this, FARM_MODULE_SYSTEM, initial_resources_code
    )
  }

  fn inject_initial_loaded_resources(&mut self, element: &mut Element) {
    let code = self.get_initial_resources_code();

    element.children.push(Child::Element(create_element(
      "script",
      Some(&code),
      vec![],
    )));
  }

  fn get_dynamic_resources_map_code(&self) -> String {
    if self.dynamic_resources.is_empty() {
      return "".to_string();
    }

    format!(
      r#"{}.{}.sd({},{});"#,
      self.farm_global_this,
      FARM_MODULE_SYSTEM,
      self.dynamic_resources,
      self.dynamic_module_resources_map
    )
  }

  fn inject_dynamic_resources_map(&mut self, element: &mut Element) {
    let final_code = self.get_dynamic_resources_map_code();

    element.children.push(Child::Element(create_element(
      "script",
      Some(&final_code),
      vec![],
    )));
  }

  fn get_global_this_code(&self) -> String {
    format!(
      r#"{FARM_GLOBAL_THIS} = {{}};{FARM_GLOBAL_THIS} = {{__FARM_TARGET_ENV__: 'browser'}};"#,
      FARM_GLOBAL_THIS = self.farm_global_this,
    )
  }

  fn inject_global_this(&mut self, element: &mut Element) {
    let code = self.get_global_this_code();

    element.children.push(Child::Element(create_element(
      "script",
      Some(&code),
      vec![],
    )));
  }

  fn inject_bootstrap(&self, element: &mut Element) {
    let code = self.get_bootstrap_code();

    element.children.push(Child::Element(create_element(
      "script",
      Some(&code),
      vec![],
    )));
  }

  fn get_bootstrap_code(&self) -> String {
    let mut final_code = String::new();
    final_code.push_str(&format!(
      r#"{}.{}.sp(['{}']);"#,
      self.farm_global_this, FARM_MODULE_SYSTEM, self.options.public_path
    ));
    final_code.push_str(&format!(
      r#"{}.{}.b();"#,
      self.farm_global_this, FARM_MODULE_SYSTEM
    ));
    for entry in &self.script_entries {
      final_code.push_str(&format!(
        r#"{}.{}.r("{}");"#,
        self.farm_global_this, FARM_MODULE_SYSTEM, entry
      ))
    }

    final_code
  }

  fn inject_additional_resource(&mut self, name: &str, code: String, element: &mut Element) {
    let resource = create_farm_runtime_output_resource(
      Cow::Owned(code.into_bytes()),
      name,
      &self.options.context,
    );

    element.children.push(Child::Element(create_element(
      "script",
      None,
      vec![("src", &format!("/{name}"))],
    )));

    self.additional_inject_resources.push(resource);
  }
}

impl VisitMut for ResourcesInjector<'_> {
  fn visit_mut_element(&mut self, element: &mut Element) {
    if element.tag_name.to_string() == "head" || element.tag_name.to_string() == "body" {
      let mut children_to_remove = vec![];

      // remove all non-http existing <href /> and <script /> first
      for (i, child) in element.children.iter().enumerate() {
        if let Child::Element(e) = child {
          if is_link_css_or_code(e, &self.options.current_html_id, &self.options.context)
            || is_script_src_or_type_module_code(
              e,
              &self.options.current_html_id,
              &self.options.context,
            )
            || is_script_resource(e)
          {
            children_to_remove.push(i);
          }
        }
      }

      // remove from the end to the beginning, so that the index is not affected
      children_to_remove.reverse();
      children_to_remove.into_iter().for_each(|i| {
        element.children.remove(i);
      });
    }

    if element.tag_name.to_string() == "head" {
      if get_config_runtime_isolate(&self.options.context) {
        let runtime_resource_code = format!("{}{}", self.get_global_this_code(), self.runtime_code);
        self.inject_additional_resource(
          FARM_RUNTIME_INJECT_RESOURCE,
          runtime_resource_code,
          element,
        );
      } else {
        // inject global this <script>
        self.inject_global_this(element);
        // inject runtime <script>
        self.inject_runtime_resources(element);
      }

      // inject css <link>
      for css in &self.css_resources {
        element.children.push(Child::Element(create_element(
          "link",
          None,
          vec![
            ("rel", "stylesheet"),
            ("href", &format!("{}{}", self.options.public_path, css)),
          ],
        )));
      }
    } else if element.tag_name == "body" {
      for script in &self.script_resources {
        element.children.push(Child::Element(create_element(
          "script",
          None,
          vec![
            ("src", &format!("{}{}", self.options.public_path, script)),
            (FARM_RESOURCE, "true"),
          ],
        )));
      }

      if get_config_runtime_isolate(&self.options.context) {
        let bootstrap_code = format!(
          "{}{}{}",
          self.get_initial_resources_code(),
          self.get_dynamic_resources_map_code(),
          self.get_bootstrap_code()
        );
        self.inject_additional_resource(FARM_MODULE_SYSTEM_BOOTSTRAP, bootstrap_code, element);
      } else {
        self.inject_initial_loaded_resources(element);
        self.inject_dynamic_resources_map(element);
        self.inject_bootstrap(element);
      }
    };

    element.visit_mut_children_with(self);
  }
}
