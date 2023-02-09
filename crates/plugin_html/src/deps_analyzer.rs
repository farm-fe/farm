use farmfe_core::{
  plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind},
  swc_html_ast::{Document, Element},
};
use farmfe_toolkit::swc_html_visit::{Visit, VisitWith};

pub struct DepsAnalyzer {
  pub deps: Option<Vec<PluginAnalyzeDepsHookResultEntry>>,
}

pub const FARM_ENTRY: &str = "data-farm-entry-script";

impl DepsAnalyzer {
  pub fn new() -> Self {
    Self { deps: None }
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
}

impl Visit for DepsAnalyzer {
  fn visit_element(&mut self, element: &Element) {
    if let Some(value) = get_script_src_value(element) {
      self.insert_dep(PluginAnalyzeDepsHookResultEntry {
        kind: ResolveKind::ScriptSrc,
        source: value,
      })
    } else if let Some(value) = get_href_link_value(element) {
      self.insert_dep(PluginAnalyzeDepsHookResultEntry {
        kind: ResolveKind::LinkHref,
        source: value,
      })
    }

    element.visit_children_with(self);
  }
}

pub fn get_script_src_value(element: &Element) -> Option<String> {
  if element.tag_name.to_string() == "script" {
    let src_attr = element
      .attributes
      .iter()
      .find(|&attr| attr.name.to_string() == "src");

    if let Some(src_attr) = src_attr {
      if let Some(value) = &src_attr.value {
        Some(value.to_string())
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  }
}

pub fn get_href_link_value(element: &Element) -> Option<String> {
  if element.tag_name.to_string() == "link" {
    let src_attr = element
      .attributes
      .iter()
      .find(|&attr| attr.name.to_string() == "href");

    if let Some(src_attr) = src_attr {
      if let Some(value) = &src_attr.value {
        Some(value.to_string())
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  }
}

pub fn is_script_src(element: &Element) -> bool {
  if let Some(v) = get_script_src_value(element) {
    if !v.starts_with("http") {
      true
    } else {
      false
    }
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

pub fn is_link_href(element: &Element) -> bool {
  if let Some(v) = get_href_link_value(element) {
    if !v.starts_with("http") {
      true
    } else {
      false
    }
  } else {
    false
  }
}
