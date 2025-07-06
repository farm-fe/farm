use farmfe_core::{
  config::AliasItem,
  plugin::{PluginAnalyzeDepsHookResultEntry, ResolveKind},
  swc_css_ast::{ImportHref, Url},
};
use farmfe_toolkit::{resolve::path_start_with_alias::is_start_with_alias, swc_css_visit::Visit};
use lightningcss::{
  rules::{import, CssRule},
  visitor::{Visit as _, VisitTypes},
};

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

impl<'a> lightningcss::visitor::Visitor<'a> for DepAnalyzer {
  type Error = String;

  fn visit_types(&self) -> lightningcss::visitor::VisitTypes {
    VisitTypes::URLS | VisitTypes::RULES
  }

  fn visit_rule(&mut self, rule: &mut lightningcss::rules::CssRule<'a>) -> Result<(), Self::Error> {
    match rule {
      CssRule::Import(import) => {
        self.insert_dep(PluginAnalyzeDepsHookResultEntry {
          source: import.url.to_string(),
          kind: ResolveKind::CssAtImport,
        });
      }
      _ => rule.visit_children(self)?,
    }

    Ok(())
  }

  fn visit_url(
    &mut self,
    _url: &mut lightningcss::values::url::Url<'a>,
  ) -> Result<(), Self::Error> {
    self.insert_dep(PluginAnalyzeDepsHookResultEntry {
      source: _url.url.to_string(),
      kind: ResolveKind::CssUrl,
    });

    Ok(())
  }
}

pub fn is_source_ignored(source: &str) -> bool {
  source.starts_with("http://")
    || source.starts_with("https://")
    || source.starts_with("/")
    || source.starts_with("data:")
    || source.starts_with('#')
}
