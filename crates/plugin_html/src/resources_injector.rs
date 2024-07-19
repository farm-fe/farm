use std::{collections::HashMap, fmt::Display, sync::Arc};
use url::Url;

use farmfe_core::{
  config::{custom::get_config_runtime_isolate, Mode, FARM_MODULE_SYSTEM},
  context::CompilationContext,
  module::ModuleId,
  resource::{Resource, ResourceType},
  swc_html_ast::{Child, Document, Element},
};
use farmfe_toolkit::{
  get_dynamic_resources_map::get_dynamic_resources_code,
  html::{create_element, get_farm_global_this},
  swc_html_visit::{VisitMut, VisitMutWith},
};

use crate::utils::{
  create_farm_runtime_output_resource, is_link_css_or_code, is_script_entry, is_script_resource,
  is_script_src_or_type_module_code, FARM_ENTRY, FARM_RESOURCE,
};

#[derive(Debug, Clone)]
pub struct PreloadResource {
    pub href: String,
    pub as_: String,
    pub crossorigin: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrefetchResource {
    pub href: String,
    pub as_: Option<String>,
    pub crossorigin: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DynamicPrefetchResource {
    pub module_id: ModuleId,
    pub href: String,
    pub as_: Option<String>,
    pub crossorigin: Option<String>,
}

pub struct ResourcesInjectorOptions {
  pub mode: Mode,
  pub public_path: String,
  pub namespace: String,
  pub current_html_id: ModuleId,
  pub context: Arc<CompilationContext>,
  pub preload: Vec<PreloadResource>,
  pub prefetch: Vec<PrefetchResource>,
  pub dynamic_prefetch: Vec<DynamicPrefetchResource>,
}

/// inject resources into the html ast
pub struct ResourcesInjector {
  additional_inject_resources: Vec<Resource>,
  runtime_code: String,
  script_resources: Vec<String>,
  css_resources: Vec<String>,
  script_entries: Vec<String>,
  dynamic_resources_map: HashMap<ModuleId, Vec<(String, ResourceType)>>,
  options: ResourcesInjectorOptions,
  farm_global_this: String,
}
pub const FARM_RUNTIME_INJECT_RESOURCE: &str = "farm_runtime_resource";

impl ResourcesInjector {
  pub fn new(
    additional_inject_resources: Vec<Resource>,
    runtime_code: String,
    script_resources: Vec<String>,
    css_resources: Vec<String>,
    script_entries: Vec<String>,
    dynamic_resources_map: HashMap<ModuleId, Vec<(String, ResourceType)>>,
    options: ResourcesInjectorOptions,
  ) -> Self {
    Self {
      additional_inject_resources,
      runtime_code,
      css_resources,
      script_resources,
      script_entries,
      dynamic_resources_map,
      farm_global_this: get_farm_global_this(&options.namespace),
      options,
    }
  }

  fn inject_preload_and_prefetch(&self, element: &mut Element) {
    // Inject preload links
    for resource in &self.options.preload {
        let mut attrs = vec![
            ("rel", "preload"),
            ("href", resource.href.as_str()),
            ("as", resource.as_.as_str()),
        ];
        if let Some(crossorigin) = &resource.crossorigin {
            attrs.push(("crossorigin", crossorigin.as_str()));
        }
        element.children.push(Child::Element(create_element(
            "link",
            None,
            attrs,
        )));
    }

    // Inject prefetch links
    for resource in &self.options.prefetch {
        let mut attrs = vec![
            ("rel", "prefetch"),
            ("href", resource.href.as_str()),
        ];
        if let Some(as_) = &resource.as_ {
            attrs.push(("as", as_.as_str()));
        }
        if let Some(crossorigin) = &resource.crossorigin {
            attrs.push(("crossorigin", crossorigin.as_str()));
        }
        element.children.push(Child::Element(create_element(
            "link",
            None,
            attrs,
        )));
    }
}

fn inject_dynamic_prefetch(&self, element: &mut Element) {
    // Inject dynamic prefetch links
    for resource in &self.options.dynamic_prefetch {
        let mut attrs = vec![
            ("rel", "prefetch"),
            ("href", resource.href.as_str()),
        ];
        if let Some(as_) = &resource.as_ {
            attrs.push(("as", as_.as_str()));
        }
        if let Some(crossorigin) = &resource.crossorigin {
            attrs.push(("crossorigin", crossorigin.as_str()));
        }
        let onload = format!(
            r#"
            const moduleId = "{}";
            const href = this.href;
            const as_ = this.as;
            const crossorigin = this.crossorigin;
            const link = document.createElement("link");
            link.rel = "modulepreload";
            link.href = href;
            link.as = as_;
            link.crossorigin = crossorigin;
            link.onload = () => {{
              const dynamicResources = {}.{}.getDynamicModuleResourcesMap()[moduleId];
              for (const [resource, type_] of dynamicResources) {{
                if (type_ === "dynamic-import" && !resource.starts_with(href)) {{
                  const prefetchLink = document.createElement("link");
                  prefetchLink.rel = "prefetch";
                  prefetchLink.href = resource;
                  prefetchLink.as = "fetch";
                  document.head.appendChild(prefetchLink);
                }}
              }}
            }};
            document.head.appendChild(link);
            "#,
            resource.module_id, self.farm_global_this, FARM_MODULE_SYSTEM
        );
        attrs.push(("onload", onload.as_str()));
        element.children.push(Child::Element(create_element(
            "link",
            None,
            attrs,
        )));
    }
}

  pub fn inject(&mut self, ast: &mut Document) {
    ast.visit_mut_with(self);
  }

  // insert the runtime and other resources that need to be inject in the resource_map.
  pub fn update_resource(&mut self, resources_map: &mut HashMap<String, Resource>) {
    for resource in &self.additional_inject_resources {
      resources_map.insert(resource.name.clone(), resource.clone());
    }
  }

  // Support isolate runtime resource (https://github.com/farm-fe/farm/issues/434)
  fn inject_runtime_resources(&mut self, element: &mut Element) {
    if get_config_runtime_isolate(&self.options.context) {
      let resource = create_farm_runtime_output_resource(
        self.runtime_code.clone().into_bytes(),
        FARM_RUNTIME_INJECT_RESOURCE,
        &self.options.context,
      );
      let script_element = create_element(
        "script",
        None,
        vec![
          (FARM_ENTRY, "true"),
          ("src", &format!("/{}", resource.name)),
        ],
      );
      element.children.push(Child::Element(script_element));
      self.additional_inject_resources.push(resource.clone());
    } else {
      element.children.push(Child::Element(create_element(
        "script",
        Some(&self.runtime_code),
        vec![(FARM_ENTRY, "true")],
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
      .map(|path| format!("'{}'", path))
      .collect::<Vec<_>>()
      .join(",");

    element.children.push(Child::Element(create_element(
      "script",
      Some(&format!(
        r#"{}.{}.setInitialLoadedResources([{}]);"#,
        self.farm_global_this, FARM_MODULE_SYSTEM, initial_resources_code
      )),
      vec![(FARM_ENTRY, "true")],
    )));
  }

  fn inject_dynamic_resources_map(&mut self, element: &mut Element) {
    let dynamic_resources_code =
      get_dynamic_resources_code(&self.dynamic_resources_map, self.options.mode.clone());

    let finalize_code = format!(
      r#"{}.{}.setDynamicModuleResourcesMap({});"#,
      self.farm_global_this, FARM_MODULE_SYSTEM, dynamic_resources_code
    );

    if get_config_runtime_isolate(&self.options.context) {
      let resource = create_farm_runtime_output_resource(
        finalize_code.into_bytes(),
        "dynamic_resources_map",
        &self.options.context,
      );
      element.children.push(Child::Element(create_element(
        "script",
        None,
        vec![
          ("FARM_ENTRY", "true"),
          ("src", &format!("/{}", resource.name)),
        ],
      )));
      self.additional_inject_resources.push(resource);
    } else {
      element.children.push(Child::Element(create_element(
        "script",
        Some(&finalize_code),
        vec![(FARM_ENTRY, "true")],
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
      vec![(FARM_ENTRY, "true")],
    )));
  }

  fn inject_other_entry_file(&self, element: &mut Element) {
    element.children.push(Child::Element(create_element(
      "script",
      Some(&format!(
        r#"{}.{}.setPublicPaths(['{}']);"#,
        self.farm_global_this, FARM_MODULE_SYSTEM, self.options.public_path
      )),
      vec![(FARM_ENTRY, "true")],
    )));

    element.children.push(Child::Element(create_element(
      "script",
      Some(&format!(
        r#"{}.{}.bootstrap();"#,
        self.farm_global_this, FARM_MODULE_SYSTEM
      )),
      vec![(FARM_ENTRY, "true")],
    )));

    for entry in &self.script_entries {
      element.children.push(Child::Element(create_element(
        "script",
        Some(&format!(
          r#"{}.{}.require("{}")"#,
          self.farm_global_this, FARM_MODULE_SYSTEM, entry
        )),
        vec![(FARM_ENTRY, "true")],
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
    let resource = create_farm_runtime_output_resource(
      finalize_code.into_bytes(),
      "farm_module_system",
      &self.options.context,
    );
    // inject script
    element.children.push(Child::Element(create_element(
      "script",
      None,
      vec![
        (FARM_ENTRY, "true"),
        ("src", &format!("/{}", resource.name)),
      ],
    )));
    self.additional_inject_resources.push(resource);
  }
}

impl VisitMut for ResourcesInjector {
  fn visit_mut_element(&mut self, element: &mut Element) {
    if element.tag_name.to_string() == "head" || element.tag_name.to_string() == "body" {
      let mut children_to_remove = vec![];

      // remove all non-http existing <href /> and <script /> first
      for (i, child) in element.children.iter().enumerate() {
        if let Child::Element(e) = child {
          if is_link_css_or_code(e, &self.options.current_html_id, &self.options.context)
            || is_script_entry(e)
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
        // Inject preload and prefetch links
        self.inject_preload_and_prefetch(element);

        // Inject dynamic prefetch links
        self.inject_dynamic_prefetch(element);

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

      // inject global this
      self.inject_global_this(element);

      // inject runtime <script>
      self.inject_runtime_resources(element);
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
