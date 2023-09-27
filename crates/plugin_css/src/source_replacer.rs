use farmfe_core::{
  hashbrown::HashMap,
  module::{module_graph::ModuleGraph, ModuleId},
  resource::{Resource, ResourceOrigin},
  swc_common::DUMMY_SP,
  swc_css_ast::{AtRulePrelude, ImportHref, Rule, Str, Stylesheet, Url, UrlValue},
};
use farmfe_toolkit::swc_css_visit::{VisitMut, VisitMutWith};

use crate::dep_analyzer::is_source_ignored;

pub struct SourceReplacer<'a> {
  module_id: ModuleId,
  module_graph: &'a ModuleGraph,
  resources_map: &'a HashMap<String, Resource>,
}

impl<'a> SourceReplacer<'a> {
  pub fn new(
    module_id: ModuleId,
    module_graph: &'a ModuleGraph,
    resources_map: &'a HashMap<String, Resource>,
  ) -> Self {
    Self {
      module_id,
      module_graph,
      resources_map,
    }
  }
}

impl<'a> VisitMut for SourceReplacer<'a> {
  fn visit_mut_url(&mut self, url: &mut Url) {
    if let Some(name) = &url.name.raw {
      if name == "url" {
        if let Some(value) = &mut url.value {
          let deal_url_value = |source: &str| -> String {
            if is_source_ignored(source) {
              return source.to_string();
            }

            let dep_module = self.module_graph.get_dep_by_source(&self.module_id, source);

            for resource in self.resources_map.values() {
              if let ResourceOrigin::Module(m_id) = &resource.origin {
                if &dep_module == m_id {
                  return resource.name.clone();
                }
              }
            }

            panic!(
              "can not find resource: resolving {:?} for {:?}",
              source, self.module_id
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
            value: resource_name.as_str().into(),
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
                .get_dep_by_source_optional(&self.module_id, &source)
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
