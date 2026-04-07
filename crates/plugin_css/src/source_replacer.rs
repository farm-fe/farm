use farmfe_core::{
  config::AliasItem,
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  resource::{Resource, ResourceOrigin, RESOURCE_META_PRIMARY_KEY},
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
  alias: Vec<AliasItem>,
  normalized_public_path: String,
  resources_map: &'a HashMap<String, Resource>,
}

impl<'a> SourceReplacer<'a> {
  pub fn new(
    module_id: ModuleId,
    module_graph: &'a ModuleGraph,
    resources_map: &'a HashMap<String, Resource>,
    public_path: String,
    alias: Vec<AliasItem>,
  ) -> Self {
    let normalized_public_path = if public_path.is_empty() {
      "/".to_string()
    } else {
      let trimmed = public_path.trim_end_matches('/');
      if trimmed.is_empty() {
        "/".to_string()
      } else {
        format!("{}/", trimmed)
      }
    };

    Self {
      module_id,
      module_graph,
      alias,
      normalized_public_path,
      resources_map,
    }
  }

  fn resolve_url(&self, source: &str) -> Option<String> {
    if source.trim().is_empty() {
      return None;
    }

    if is_source_ignored(source) && !is_start_with_alias(&self.alias, source) {
      return Some(source.to_string());
    }
    
    let dep_module = self.module_graph.get_dep_by_source_optional(
      &self.module_id,
      source,
      Some(ResolveKind::CssUrl),
    )?;

    let matching_resources: Vec<&Resource> = self
      .resources_map
      .values()
      .filter(|r| matches!(&r.origin, ResourceOrigin::Module(m_id) if m_id == &dep_module))
      .collect();

    if matching_resources.is_empty() {
      return None;
    }

    let resource = if matching_resources.len() > 1 {
      matching_resources
        .iter()
        .find(|res| {
          res
            .meta
            .get(RESOURCE_META_PRIMARY_KEY)
            .map(|v| v == "true")
            .unwrap_or(false)
        })
        .unwrap_or(&matching_resources[0])
    } else {
      &matching_resources[0]
    };

    Some(format!("{}{}", self.normalized_public_path, resource.name))
  }
}

impl<'a> VisitMut for SourceReplacer<'a> {
  fn visit_mut_url(&mut self, url: &mut Url) {
    if let Some(name) = &url.name.raw {
      if name == "url" {
        if let Some(value) = &mut url.value {
          let resource_name = match &mut **value {
            farmfe_core::swc_css_ast::UrlValue::Str(str) => self.resolve_url(&str.value),
            farmfe_core::swc_css_ast::UrlValue::Raw(raw) => self.resolve_url(&raw.value),
          };

          // Only update the value if we got a valid resource name
          if let Some(name) = resource_name {
            *value = Box::new(UrlValue::Str(Str {
              span: DUMMY_SP,
              value: name.into(),
              raw: None,
            }));
          }
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
