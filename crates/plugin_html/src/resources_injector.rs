use std::{borrow::Cow, sync::Arc};

use farmfe_core::{
  config::FARM_MODULE_SYSTEM,
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
  FARM_RESOURCE, create_farm_runtime_output_resource, is_link_css_or_code, is_script_resource,
  is_script_src_or_type_module_code,
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
  #[allow(clippy::too_many_arguments)]
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
      Some(&format!("(function(){{{}}})();", self.runtime_code)),
      vec![],
    )));
  }

  fn get_initial_resources_code(&self) -> String {
    let mut initial_resources = vec![];
    initial_resources.extend(self.script_resources.clone());
    initial_resources.extend(self.css_resources.clone());
    initial_resources.sort();

    if initial_resources.is_empty() {
      return "".to_string();
    }

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

  fn get_resources_info_code(&self) -> String {
    let dynamic_resources_map = self.get_dynamic_resources_map_code();

    if dynamic_resources_map.is_empty() {
      return "".to_string();
    }

    let initial_resources = self.get_initial_resources_code();
    let public_path = format!(
      r#"{}.{}.sp(['{}']);"#,
      self.farm_global_this, FARM_MODULE_SYSTEM, self.options.public_path
    );

    format!(
      "{}{}{}",
      initial_resources, dynamic_resources_map, public_path
    )
  }

  fn inject_resources_info(&mut self, element: &mut Element) {
    let final_code = self.get_resources_info_code();

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
      vec![(
        "src",
        &format!("{}{}", self.options.public_path, resource.name),
      )],
    )));

    self.additional_inject_resources.push(resource);
  }
}

impl<'a> VisitMut for ResourcesInjector<'a> {
  fn visit_mut_element(&mut self, element: &mut Element) {
    if element.tag_name == "head" || element.tag_name == "body" {
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

    if element.tag_name == "head" {
      if self.options.context.config.runtime.isolate {
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

      if self.options.context.config.runtime.isolate {
        let bootstrap_code = format!(
          "{}{}",
          self.get_resources_info_code(),
          self.get_bootstrap_code()
        );
        // Entry-specific bootstrap bytes must not share a name when output.filename
        // omits [hash]. Keep the stable HTML identity even for nested entries.
        let bootstrap_name = format!(
          "{}-{}",
          FARM_MODULE_SYSTEM_BOOTSTRAP,
          self.options.current_html_id.hash()
        );
        self.inject_additional_resource(&bootstrap_name, bootstrap_code, element);
      } else {
        self.inject_resources_info(element);
        self.inject_bootstrap(element);
      }
    };

    element.visit_mut_children_with(self);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::config::Config;

  #[test]
  fn isolated_bootstraps_keep_each_html_entry_distinct() {
    for filename in ["[name].[ext]", "[name]-[hash].[ext]"] {
      let mut config = Config::default();
      config.output.filename = filename.into();
      config.runtime.isolate = true;
      let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
      let mut resources = std::collections::HashMap::new();
      let mut references = vec![];
      for entry in ["index", "about"] {
        let mut injector = ResourcesInjector::new(
          vec![],
          "",
          vec![format!("{entry}.js")],
          vec![],
          vec![format!("{entry}-entry")],
          "",
          "",
          ResourcesInjectorOptions {
            public_path: "/".into(),
            namespace: "test".into(),
            current_html_id: ModuleId::from(format!("{entry}.html").as_str()),
            context: context.clone(),
          },
        );
        let mut body = create_element("body", None, vec![]);
        injector.visit_mut_element(&mut body);
        assert_eq!(injector.additional_inject_resources.len(), 1);
        let resource = injector.additional_inject_resources.pop().unwrap();
        let Child::Element(script) = body.children.last().unwrap() else {
          panic!("Expected bootstrap script");
        };
        let src = script
          .attributes
          .iter()
          .find(|attr| attr.name == "src")
          .unwrap();
        assert_eq!(
          src.value.as_ref().unwrap().as_ref(),
          format!("/{}", resource.name)
        );
        references.push((resource.name.clone(), format!("{entry}-entry")));
        resources.insert(resource.name.clone(), resource.bytes);
      }
      assert_eq!(resources.len(), 2, "Each HTML page needs its own bootstrap");
      for (name, entry) in references {
        let code = std::str::from_utf8(&resources[&name]).unwrap();
        assert!(code.contains(&format!("r(\"{entry}\")")));
      }
    }
  }
}
