use std::{borrow::Cow, rc::Rc, sync::Arc};

use farmfe_core::{
  config::{custom::get_config_runtime_isolate, Mode, FARM_MODULE_SYSTEM},
  context::CompilationContext,
  module::ModuleId,
  resource::{Resource, ResourceType},
  swc_html_ast::{Child, Document, Element},
  HashMap,
};
use farmfe_toolkit::{
  get_dynamic_resources_map::get_dynamic_resources_code,
  html::{create_element, get_farm_global_this},
  swc_html_visit::{VisitMut, VisitMutWith},
};

use crate::utils::{
  create_farm_runtime_output_resource, is_link_css_or_code, is_script_resource,
  is_script_src_or_type_module_code, FARM_RESOURCE,
};

pub struct ResourcesInjectorOptions {
  pub mode: Mode,
  pub public_path: String,
  pub namespace: String,
  pub current_html_id: ModuleId,
  pub context: Arc<CompilationContext>,
}

/// inject resources into the html ast
pub struct ResourcesInjector<'a> {
  additional_inject_resources: Vec<Resource>,
  runtime_code: Rc<String>,
  script_resources: Vec<String>,
  css_resources: Vec<String>,
  script_entries: Vec<String>,
  dynamic_resources_map: HashMap<ModuleId, Vec<(String, ResourceType)>>,
  options: ResourcesInjectorOptions,
  farm_global_this: String,
  already_injected_resources: &'a mut Vec<String>,
}
pub const FARM_RUNTIME_INJECT_RESOURCE: &str = "farm_runtime_resource";
pub const FARM_MODULE_SYSTEM_RESOURCE: &str = "farm_module_system";
pub const FARM_DYNAMIC_RESOURCES_MAP_RESOURCE: &str = "farm_dynamic_resources_map";

impl<'a> ResourcesInjector<'a> {
  pub fn new(
    additional_inject_resources: Vec<Resource>,
    runtime_code: Rc<String>,
    script_resources: Vec<String>,
    css_resources: Vec<String>,
    script_entries: Vec<String>,
    dynamic_resources_map: HashMap<ModuleId, Vec<(String, ResourceType)>>,
    options: ResourcesInjectorOptions,
    already_injected_resources: &'a mut Vec<String>,
  ) -> Self {
    Self {
      additional_inject_resources,
      runtime_code,
      css_resources,
      script_resources,
      script_entries,
      dynamic_resources_map,
      farm_global_this: get_farm_global_this(
        &options.namespace,
        &options.context.config.output.target_env,
      ),
      options,
      already_injected_resources,
    }
  }

  pub fn inject(&mut self, ast: &mut Document) {
    ast.visit_mut_with(self);
  }

  // insert the runtime and other resources that need to be inject in the resource_map.
  pub fn update_resource(self, resources_map: &mut HashMap<String, Resource>) {
    for resource in self.additional_inject_resources {
      resources_map.insert(resource.name.clone(), resource.clone());
      self.already_injected_resources.push(resource.name);
    }
  }

  // Support isolate runtime resource (https://github.com/farm-fe/farm/issues/434)
  fn inject_runtime_resources(&mut self, element: &mut Element) {
    if get_config_runtime_isolate(&self.options.context) {
      let (name, resource) = create_farm_runtime_output_resource(
        Cow::Borrowed(self.runtime_code.as_bytes()),
        FARM_RUNTIME_INJECT_RESOURCE,
        &self.options.context,
        &self.already_injected_resources,
      );

      let script_element = create_element("script", None, vec![("src", &format!("/{name}"))]);
      element.children.push(Child::Element(script_element));

      if let Some(resource) = resource {
        self.additional_inject_resources.push(resource);
      }
    } else {
      element.children.push(Child::Element(create_element(
        "script",
        Some(&self.runtime_code),
        vec![],
      )));
    }
  }

  fn inject_initial_loaded_resources(&self, element: &mut Element) {
    let mut initial_resources = vec![];
    initial_resources.extend(self.script_resources.clone());
    initial_resources.extend(self.css_resources.clone());
    initial_resources.sort();

    let initial_resources_code = initial_resources
      .into_iter()
      .map(|path| format!("'{path}'"))
      .collect::<Vec<_>>()
      .join(",");

    element.children.push(Child::Element(create_element(
      "script",
      Some(&format!(
        r#"{}.{}.setInitialLoadedResources([{}]);"#,
        self.farm_global_this, FARM_MODULE_SYSTEM, initial_resources_code
      )),
      vec![],
    )));
  }

  fn inject_dynamic_resources_map(&mut self, element: &mut Element) {
    let (dynamic_resources, dynamic_module_resources_map) =
      get_dynamic_resources_code(&self.dynamic_resources_map, self.options.mode.clone());

    if dynamic_resources.is_empty() {
      return;
    }

    let finalize_code = format!(
      r#"{}.{}.setDynamicModuleResourcesMap({},{});"#,
      self.farm_global_this, FARM_MODULE_SYSTEM, dynamic_resources, dynamic_module_resources_map
    );

    if get_config_runtime_isolate(&self.options.context) {
      let (name, resource) = create_farm_runtime_output_resource(
        Cow::Owned(finalize_code.into_bytes()),
        FARM_DYNAMIC_RESOURCES_MAP_RESOURCE,
        &self.options.context,
        &self.already_injected_resources,
      );

      element.children.push(Child::Element(create_element(
        "script",
        None,
        vec![("src", &format!("/{name}"))],
      )));

      if let Some(resource) = resource {
        self.additional_inject_resources.push(resource);
      }
    } else {
      element.children.push(Child::Element(create_element(
        "script",
        Some(&finalize_code),
        vec![],
      )));
    }
  }

  fn inject_global_this(&self, element: &mut Element) {
    let code = format!(
      r#"
{FARM_GLOBAL_THIS} = {{}};
{FARM_GLOBAL_THIS} = {{
  __FARM_TARGET_ENV__: 'browser',
}};"#,
      FARM_GLOBAL_THIS = self.farm_global_this,
    );

    element.children.push(Child::Element(create_element(
      "script",
      Some(&code),
      vec![],
    )));
  }

  fn inject_other_entry_file(&self, element: &mut Element) {
    element.children.push(Child::Element(create_element(
      "script",
      Some(&format!(
        r#"{}.{}.setPublicPaths(['{}']);"#,
        self.farm_global_this, FARM_MODULE_SYSTEM, self.options.public_path
      )),
      vec![],
    )));

    element.children.push(Child::Element(create_element(
      "script",
      Some(&format!(
        r#"{}.{}.bootstrap();"#,
        self.farm_global_this, FARM_MODULE_SYSTEM
      )),
      vec![],
    )));

    for entry in &self.script_entries {
      element.children.push(Child::Element(create_element(
        "script",
        Some(&format!(
          r#"{}.{}.require("{}")"#,
          self.farm_global_this, FARM_MODULE_SYSTEM, entry
        )),
        vec![],
      )));
    }
  }

  fn inject_resource_separate_file(&mut self, element: &mut Element) {
    let mut finalize_code = String::new();
    finalize_code.push_str(&format!(
      r#"{}.{}.setPublicPaths(['{}']);"#,
      self.farm_global_this, FARM_MODULE_SYSTEM, self.options.public_path
    ));
    finalize_code.push_str(&format!(
      r#"{}.{}.bootstrap();"#,
      self.farm_global_this, FARM_MODULE_SYSTEM
    ));
    for entry in &self.script_entries {
      finalize_code.push_str(&format!(
        r#"{}.{}.require("{}");"#,
        self.farm_global_this, FARM_MODULE_SYSTEM, entry
      ))
    }
    // create resource
    let (name, resource) = create_farm_runtime_output_resource(
      Cow::Owned(finalize_code.into_bytes()),
      FARM_MODULE_SYSTEM_RESOURCE,
      &self.options.context,
      &self.already_injected_resources,
    );
    // inject script
    element.children.push(Child::Element(create_element(
      "script",
      None,
      vec![("src", &format!("/{name}"))],
    )));

    if let Some(resource) = resource {
      self.additional_inject_resources.push(resource);
    }
  }
}

impl<'a> VisitMut for ResourcesInjector<'a> {
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
      // inject global this
      self.inject_global_this(element);

      // inject runtime <script>
      self.inject_runtime_resources(element);

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
    } else if element.tag_name.to_string() == "body" {
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

      self.inject_initial_loaded_resources(element);
      self.inject_dynamic_resources_map(element);

      if get_config_runtime_isolate(&self.options.context) {
        self.inject_resource_separate_file(element);
      } else {
        self.inject_other_entry_file(element);
      }
    };

    element.visit_mut_children_with(self);
  }
}
