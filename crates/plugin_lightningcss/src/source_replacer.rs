use std::collections::HashMap;

use farmfe_core::config::AliasItem;
use farmfe_core::module::{module_graph::ModuleGraph, ModuleId};
use farmfe_core::plugin::ResolveKind;
use farmfe_core::resource::{Resource, ResourceOrigin};
use lightningcss::values::url::Url;
use lightningcss::visit_types;
use lightningcss::visitor::{VisitTypes, Visitor};
use rkyv::Infallible;

use crate::is_source_ignored;

use farmfe_toolkit::resolve::path_start_with_alias::is_start_with_alias;

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

impl<'i> Visitor<'i> for SourceReplacer<'i> {
  type Error = Infallible;

  fn visit_types(&self) -> VisitTypes {
    visit_types!(URLS | LENGTHS)
  }

  fn visit_url(&mut self, url: &mut Url<'i>) -> Result<(), Self::Error> {
    let deal_url_value = |source: &str| -> String {
      if is_source_ignored(source) && !is_start_with_alias(&self.alias, source) {
        return source.to_string();
      }
      let dep_module =
        self
          .module_graph
          .get_dep_by_source(&self.module_id, source, Some(ResolveKind::CssUrl));

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
    Ok(())
  }
}
