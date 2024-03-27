use std::{collections::HashMap, fmt::Display, sync::Arc};
use url::Url;

use farmfe_core::{
  config::{Mode, FARM_MODULE_SYSTEM},
  context::CompilationContext,
  module::ModuleId,
  resource::ResourceType,
  serde_json,
  swc_html_ast::{Child, Document, Element},
};
use farmfe_toolkit::{
  get_dynamic_resources_map::get_dynamic_resources_code,
  html::{create_element, get_farm_global_this},
  swc_html_visit::{VisitMut, VisitMutWith},
};

use crate::utils::{
  is_link_css_or_code, is_script_entry, is_script_resource, is_script_src_or_type_module_code,
  FARM_ENTRY, FARM_RESOURCE,
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
  pub define: std::collections::HashMap<String, serde_json::Value>,
  pub namespace: String,
  pub current_html_id: ModuleId,
  pub context: Arc<CompilationContext>,
  pub preload: Vec<PreloadResource>,
  pub prefetch: Vec<PrefetchResource>,
  pub dynamic_prefetch: Vec<DynamicPrefetchResource>,
}

/// inject resources into the html ast
pub struct ResourcesInjector {
  runtime_code: String,
  script_resources: Vec<String>,
  css_resources: Vec<String>,
  script_entries: Vec<String>,
  dynamic_resources_map: HashMap<ModuleId, Vec<(String, ResourceType)>>,
  options: ResourcesInjectorOptions,
  farm_global_this: String,
}

impl ResourcesInjector {
  pub fn new(
    runtime_code: String,
    script_resources: Vec<String>,
    css_resources: Vec<String>,
    script_entries: Vec<String>,
    dynamic_resources_map: HashMap<ModuleId, Vec<(String, ResourceType)>>,
    options: ResourcesInjectorOptions,
  ) -> Self {
    Self {
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

  fn inject_dynamic_resources_map(&self, element: &mut Element) {
    let dynamic_resources_code =
      get_dynamic_resources_code(&self.dynamic_resources_map, self.options.mode.clone());

    element.children.push(Child::Element(create_element(
      "script",
      Some(&format!(
        r#"{}.{}.setDynamicModuleResourcesMap({});"#,
        self.farm_global_this, FARM_MODULE_SYSTEM, dynamic_resources_code
      )),
      vec![(FARM_ENTRY, "true")],
    )));
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

      // inject global define
      self.inject_global_this(element);

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
          vec![
            ("src", &format!("{}{}", self.options.public_path, script)),
            (FARM_RESOURCE, "true"),
          ],
        )));
      }

      self.inject_initial_loaded_resources(element);
      self.inject_dynamic_resources_map(element);

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

    element.visit_mut_children_with(self);
  }
}
