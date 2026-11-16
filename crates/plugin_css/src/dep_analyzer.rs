use farmfe_core::{
  config::AliasItem,
  plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind},
  swc_css_ast::{ImportHref, Url},
};
use farmfe_toolkit::{resolve::path_start_with_alias::is_start_with_alias, swc_css_visit::Visit};

pub struct DepAnalyzer {
  pub deps: Vec<PluginAnalyzeDepsHookResultEntry>,
  alias: Vec<AliasItem>,
}

impl DepAnalyzer {
  pub fn new(alias: Vec<AliasItem>) -> Self {
    Self {
      deps: vec![],
      alias,
    }
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
    // Empty CSS URLs do not identify a dependency, even with an empty alias.
    if dep.source.trim().is_empty() {
      return false;
    }

    // ignore http and /
    if is_source_ignored(&dep.source) && !is_start_with_alias(&self.alias, &dep.source) {
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
    || source.starts_with("/")
    || source.starts_with("data:")
    || source.starts_with('#')
}

#[cfg(test)]
mod empty_url_tests {
  use super::*;
  use farmfe_toolkit::{css::parse_css_stylesheet, swc_css_visit::VisitWith};
  use std::sync::Arc;

  #[test]
  fn parsed_empty_urls_are_not_dependencies() {
    let parsed = parse_css_stylesheet(
      "empty.css",
      Arc::new(".a{a:url();b:url('');c:url(\"\");d:url('   ');e:url(  )}".to_string()),
    )
    .unwrap();
    let mut analyzer = DepAnalyzer::new(vec![]);
    parsed.ast.visit_with(&mut analyzer);
    assert!(analyzer.deps.is_empty());
  }

  #[test]
  fn empty_urls_are_not_dependencies() {
    for source in ["", " ", "\t\n"] {
      let mut analyzer = DepAnalyzer::new(vec![]);
      assert!(!analyzer.insert_dep(PluginAnalyzeDepsHookResultEntry {
        source: source.to_string(),
        kind: ResolveKind::CssUrl,
      }));
      assert!(analyzer.deps.is_empty());
    }
  }
}
