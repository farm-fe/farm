use std::collections::HashMap;

use farmfe_core::{
  cache_item,
  module::{ModuleId, ModuleType},
  plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind},
  swc_html_ast::{Child, Document, Element},
};
use farmfe_toolkit::swc_html_visit::{Visit, VisitWith};

pub const HTML_INLINE_ID_PREFIX: &str = "virtual:html-inline:";

pub struct DepsAnalyzer {
  deps: Option<Vec<PluginAnalyzeDepsHookResultEntry>>,
  html_id: ModuleId,
  /// key: inline id, value: inline source code
  pub inline_deps_map: HashMap<String, HtmlInlineModule>,
}

impl DepsAnalyzer {
  pub fn new(html_id: ModuleId) -> Self {
    Self {
      deps: None,
      html_id,
      inline_deps_map: HashMap::new(),
    }
  }

  pub fn analyze_deps(&mut self, document: &Document) -> Vec<PluginAnalyzeDepsHookResultEntry> {
    document.visit_with(self);

    self.deps.take().unwrap_or(vec![])
  }

  fn insert_dep(&mut self, dep: PluginAnalyzeDepsHookResultEntry) {
    if let Some(deps) = &mut self.deps {
      deps.push(dep);
    } else {
      self.deps.replace(vec![dep]);
    }
  }

  fn generate_virtual_id(&mut self, code: String, module_type: ModuleType) -> String {
    let id = self.html_id.to_string();
    let size = self.inline_deps_map.len();
    let virtual_id = format!("{HTML_INLINE_ID_PREFIX}{id}_{size}");

    self.inline_deps_map.insert(
      virtual_id.clone(),
      HtmlInlineModule {
        html_id: self.html_id.clone(),
        id: virtual_id.clone(),
        code,
        module_type,
      },
    );

    virtual_id
  }
}

impl Visit for DepsAnalyzer {
  fn visit_element(&mut self, element: &Element) {
    if let Some(value) = get_script_src_or_code(Some(self), element) {
      self.insert_dep(PluginAnalyzeDepsHookResultEntry {
        kind: ResolveKind::ScriptSrc,
        source: value,
      })
    } else if let Some(value) = get_href_link_or_code(Some(self), element) {
      self.insert_dep(PluginAnalyzeDepsHookResultEntry {
        kind: ResolveKind::LinkHref,
        source: value,
      })
    }

    element.visit_children_with(self);
  }
}

pub fn get_script_type_module_code(element: &Element) -> Option<String> {
  if element.tag_name.to_string() == "script" {
    // check if it's a module script
    let src_attr = element
      .attributes
      .iter()
      .find(|&attr| attr.name.to_string() == "type");

    let mut is_module = false;

    if let Some(src_attr) = src_attr {
      if let Some(value) = &src_attr.value {
        let value = value.to_string();
        is_module = value == "module";
      }
    }

    if !is_module {
      return None;
    }

    for child in &element.children {
      if let Child::Text(text) = child {
        // generate a virtual id for inline script
        let code = text.data.to_string();
        return Some(code);
      }
    }
  }

  None
}

fn get_script_src_or_code(
  analyzer: Option<&mut DepsAnalyzer>,
  element: &Element,
) -> Option<String> {
  if element.tag_name.to_string() == "script" {
    let src_attr = element
      .attributes
      .iter()
      .find(|&attr| attr.name.to_string() == "src");

    if let Some(src_attr) = src_attr {
      if let Some(value) = &src_attr.value {
        let value = value.to_string();
        // the dependencies of html should be relative path and should not start with http or /
        if should_ignore_source(&value) {
          return None;
        }

        Some(value)
      } else {
        None
      }
    } else {
      if let Some(analyzer) = analyzer {
        if let Some(code) = get_script_type_module_code(element) {
          return Some(analyzer.generate_virtual_id(code, ModuleType::Js));
        }
      }

      None
    }
  } else {
    None
  }
}

pub fn get_script_src_value(element: &Element) -> Option<String> {
  get_script_src_or_code(None, element)
}

pub fn is_link_css(element: &Element) -> bool {
  element.tag_name.to_string() == "link"
    && element
      .attributes
      .iter()
      .any(|attr| attr.name.to_string() == "rel" && attr.value.as_deref() == Some("stylesheet"))
}

pub fn get_link_css_code(element: &Element) -> Option<String> {
  if is_link_css(element) {
    for child in &element.children {
      if let Child::Text(text) = child {
        // generate a virtual id for inline css
        let code = text.data.to_string();
        return Some(code);
      }
    }
  }

  None
}

fn get_href_link_or_code(analyzer: Option<&mut DepsAnalyzer>, element: &Element) -> Option<String> {
  if is_link_css(element) {
    let src_attr = element
      .attributes
      .iter()
      .find(|&attr| attr.name.to_string() == "href");

    if let Some(src_attr) = src_attr {
      if let Some(value) = &src_attr.value {
        let value = value.to_string();
        // the dependencies of html should be relative path and should not start with http or /
        if should_ignore_source(&value) {
          return None;
        }

        Some(value)
      } else {
        None
      }
    } else {
      if let Some(analyzer) = analyzer {
        if let Some(code) = get_link_css_code(element) {
          return Some(analyzer.generate_virtual_id(code, ModuleType::Css));
        }
      }

      None
    }
  } else {
    None
  }
}

pub fn get_href_link_value(element: &Element) -> Option<String> {
  get_href_link_or_code(None, element)
}

pub fn should_ignore_source(source: &str) -> bool {
  source.starts_with("http")
    // || source.starts_with('/')
    || source.starts_with('#')
    || source.starts_with('?')
    || source.starts_with("data:")
}

#[cache_item(farmfe_core)]
#[derive(Debug, Clone)]
pub struct HtmlInlineModule {
  pub html_id: ModuleId,
  pub id: String,
  pub code: String,
  pub module_type: ModuleType,
}
