use farmfe_core::{
  config::AliasItem,
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  resource::{Resource, ResourceOrigin},
  swc_common::DUMMY_SP,
  swc_css_ast::{AtRulePrelude, ImportHref, Rule, Str, Stylesheet, Url, UrlValue},
  HashMap,
};
use farmfe_toolkit::{
  resolve::path_start_with_alias::is_start_with_alias,
  swc_css_visit::{VisitMut, VisitMutWith},
};

use crate::dep_analyzer::is_source_ignored;

pub struct SourceReplacer<'a> {
  module_id: ModuleId,
  module_graph: &'a ModuleGraph,
  resources_map: &'a HashMap<String, Resource>,
  public_path: String,
  alias: Vec<AliasItem>,
}

impl<'a> SourceReplacer<'a> {
  pub fn new(
    module_id: ModuleId,
    module_graph: &'a ModuleGraph,
    resources_map: &'a HashMap<String, Resource>,
    public_path: String,
    alias: Vec<AliasItem>,
  ) -> Self {
    Self {
      module_id,
      module_graph,
      resources_map,
      public_path,
      alias,
    }
  }
}

impl VisitMut for SourceReplacer<'_> {
  fn visit_mut_url(&mut self, url: &mut Url) {
    if let Some(name) = &url.name.raw {
      if name == "url" {
        if let Some(value) = &mut url.value {
          let deal_url_value = |source: &str| -> String {
            if is_source_ignored(source) && !is_start_with_alias(&self.alias, source) {
              return source.to_string();
            }
            let dep_module = self.module_graph.get_dep_by_source(
              &self.module_id,
              source,
              Some(ResolveKind::CssUrl),
            );

            for resource in self.resources_map.values() {
              if let ResourceOrigin::Module(m_id) = &resource.origin {
                if &dep_module == m_id {
                  // fix #1076. url prefixed by publicPath
                  let normalized_public_path = if self.public_path.is_empty() {
                    ""
                  } else {
                    self.public_path.trim_end_matches('/')
                  };

                  let normalized_public_path = if normalized_public_path.is_empty() {
                    "/".to_string()
                  } else {
                    format!("{normalized_public_path}/")
                  };

                  return format!("{normalized_public_path}{}", resource.name);
                }
              }
            }

            panic!(
              "can not find resource: resolving {:?} for {:?}. dep: {:?}",
              source, self.module_id, dep_module
            );
          };

          let resource_name = match &mut **value {
            farmfe_core::swc_css_ast::UrlValue::Str(str) => {
              deal_url_value(str.value.to_string().as_str())
            }
            farmfe_core::swc_css_ast::UrlValue::Raw(raw) => {
              deal_url_value(raw.value.to_string().as_str())
            }
          };

          *value = Box::new(UrlValue::Str(Str {
            span: DUMMY_SP,
            value: resource_name.into(),
            raw: None,
          }));
        }
      }
    }
  }

  fn visit_mut_stylesheet(&mut self, stylesheet: &mut Stylesheet) {
    let mut rules_to_remove = vec![];
    // remove all at rule that resolves to a module
    for (i, rule) in stylesheet.rules.iter().enumerate() {
      if let Rule::AtRule(box at_rule) = rule {
        if let Some(box AtRulePrelude::ImportPrelude(import)) = &at_rule.prelude {
          let source = match &import.href {
            box ImportHref::Url(url) => url.value.as_ref().map(|value| match &**value {
              UrlValue::Str(str) => str.value.to_string(),
              UrlValue::Raw(raw) => raw.value.to_string(),
            }),
            box ImportHref::Str(str) => Some(str.value.to_string()),
          };

          if let Some(source) = source {
            if !is_source_ignored(&source)
              && self
                .module_graph
                .get_dep_by_source_optional(
                  &self.module_id,
                  &source,
                  Some(ResolveKind::CssAtImport),
                )
                .is_some()
            {
              rules_to_remove.push(i);
            }
          }
        }
      }
    }

    rules_to_remove.reverse();

    rules_to_remove.into_iter().for_each(|rtr| {
      stylesheet.rules.remove(rtr);
    });

    stylesheet.visit_mut_children_with(self);
  }
}
