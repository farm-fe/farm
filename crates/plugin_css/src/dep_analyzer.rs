use farmfe_core::{
  plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind},
  swc_css_ast::{ImportHref, Url},
};
use farmfe_toolkit::swc_css_visit::Visit;

pub struct DepAnalyzer {
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
}

impl DepAnalyzer {
  pub fn new() -> Self {
    Self { deps: vec![] }
  }

  fn deal_url(&mut self, url: &Url, kind: ResolveKind) {
    if let Some(name) = &url.name.raw {
      if name == "url" {
        if let Some(value) = &url.value {
          match value {
            box farmfe_core::swc_css_ast::UrlValue::Str(str) => {
              self.insert_dep(PluginAnalyzeDepsHookResultEntry {
                source: str.value.to_string(),
                kind,
              });
            }
            box farmfe_core::swc_css_ast::UrlValue::Raw(raw) => {
              self.insert_dep(PluginAnalyzeDepsHookResultEntry {
                source: raw.value.to_string(),
                kind,
              });
            }
          }
        }
      }
    }
  }

  fn insert_dep(&mut self, dep: PluginAnalyzeDepsHookResultEntry) -> bool {
    // ignore http and /
    if is_source_ignored(&dep.source) {
      return false;
    }

    self.deps.push(dep);
    true
  }
}

impl Visit for DepAnalyzer {
  fn visit_import_href(&mut self, import: &ImportHref) {
    match import {
      ImportHref::Url(url) => {
        self.deal_url(url, ResolveKind::CssAtImport);
      }
      ImportHref::Str(str) => {
        self.insert_dep(PluginAnalyzeDepsHookResultEntry {
          source: str.value.to_string(),
          kind: ResolveKind::CssAtImport,
        });
      }
    }
  }

  fn visit_url(&mut self, url: &Url) {
    self.deal_url(url, ResolveKind::CssUrl);
  }
}

pub fn is_source_ignored(source: &str) -> bool {
  source.starts_with("http://")
    || source.starts_with("https://")
    || source.starts_with('/')
    || source.starts_with("data:")
    || source.starts_with('#')
}
