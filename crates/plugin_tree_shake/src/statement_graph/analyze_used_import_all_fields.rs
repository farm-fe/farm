use farmfe_core::module::meta_data::script::statement::SwcId;
use farmfe_core::swc_ecma_ast::{Expr, Lit, ModuleItem};
use farmfe_core::{HashMap, HashSet};
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

use super::{ImportSpecifierInfo, StatementGraph, StatementGraphEdge, StatementId};

pub fn update_used_import_all_fields_of_edges(
  module_item: &ModuleItem,
  stmt_graph: &StatementGraph,
  mut deps: HashMap<StatementId, StatementGraphEdge>,
) -> HashMap<StatementId, StatementGraphEdge> {
  for (dep_stmt_id, edge_weight) in &mut deps {
    let dep_stmt = stmt_graph.stmt(dep_stmt_id);
    // only for import * as a from 'a' statement, update used_import_all_fields
    if let Some(import_info) = &dep_stmt.import_info {
      let import_all_specifier = import_info
        .specifiers
        .iter()
        .find(|i| matches!(i, ImportSpecifierInfo::Namespace(_)));

      if let Some(import_all_specifier) = import_all_specifier {
        if let ImportSpecifierInfo::Namespace(ns) = import_all_specifier {
          let mut used_import_all_fields_collector = UsedImportAllCollector::new(ns);
          module_item.visit_with(&mut used_import_all_fields_collector);
          let used_import_all_fields = used_import_all_fields_collector.used_import_all_fields;

          edge_weight
            .used_import_all_fields
            .entry(ns.clone())
            .and_modify(|e| e.extend(used_import_all_fields.clone()))
            .or_insert(used_import_all_fields);
        }
      }
    }
  }

  deps
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub enum UsedImportAllFields {
  /// Used all fields of the import statement
  #[default]
  All,
  /// example:
  /// ```js
  /// import * as a from 'a';
  /// a.foo();
  /// ```
  Ident(String),
  /// example:
  /// ```js
  /// import * as a from 'a';
  /// a['foo']();
  /// ```
  LiteralComputed(String),
}

pub struct UsedImportAllCollector<'a> {
  namespace_ident: &'a SwcId,
  pub used_import_all_fields: HashSet<UsedImportAllFields>,
}

impl<'a> UsedImportAllCollector<'a> {
  pub fn new(namespace_ident: &'a SwcId) -> Self {
    Self {
      namespace_ident,
      used_import_all_fields: HashSet::default(),
    }
  }
}

impl Visit for UsedImportAllCollector<'_> {
  fn visit_member_expr(&mut self, n: &farmfe_core::swc_ecma_ast::MemberExpr) {
    // if obj is namespace_ident, then add the member to used_import_all_fields
    if let Expr::Ident(ident) = &*n.obj {
      let id: SwcId = ident.to_id().into();
      if id == *self.namespace_ident {
        match &n.prop {
          farmfe_core::swc_ecma_ast::MemberProp::Ident(ident) => {
            self
              .used_import_all_fields
              .insert(UsedImportAllFields::Ident(ident.sym.to_string()));
          }
          farmfe_core::swc_ecma_ast::MemberProp::PrivateName(private_name) => {
            self
              .used_import_all_fields
              .insert(UsedImportAllFields::LiteralComputed(
                private_name.name.to_string(),
              ));
          }
          farmfe_core::swc_ecma_ast::MemberProp::Computed(computed_prop_name) => {
            if let Expr::Lit(Lit::Str(str)) = &*computed_prop_name.expr {
              self
                .used_import_all_fields
                .insert(UsedImportAllFields::LiteralComputed(str.value.to_string()));
            }
          }
        }
        // do not visit children if the obj is namespace_ident
        return;
      }
    }

    n.visit_children_with(self);
  }

  /// If the ident is not used as member expr, then it should be treated as All
  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    let id: SwcId = n.to_id().into();
    if id == *self.namespace_ident {
      self.used_import_all_fields.insert(UsedImportAllFields::All);
    }
  }
}
