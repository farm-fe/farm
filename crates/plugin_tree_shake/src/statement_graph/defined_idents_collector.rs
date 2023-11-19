use std::collections::HashSet;

use farmfe_core::swc_ecma_ast::{ObjectPatProp, Pat};
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

use super::used_idents_collector::UsedIdentsCollector;

pub struct DefinedIdentsCollector {
  pub defined_idents: HashSet<String>,
  pub used_idents: HashSet<String>,
}

impl DefinedIdentsCollector {
  pub fn new() -> Self {
    Self {
      defined_idents: HashSet::new(),
      used_idents: HashSet::new(),
    }
  }
}

impl Visit for DefinedIdentsCollector {
  fn visit_pat(&mut self, pat: &Pat) {
    match pat {
      Pat::Ident(bi) => {
        self.defined_idents.insert(bi.id.to_string());
      }
      Pat::Array(array_pat) => {
        for elem in array_pat.elems.iter().flatten() {
          self.visit_pat(elem);
        }
      }
      Pat::Rest(rest_pat) => {
        self.visit_pat(&rest_pat.arg);
      }
      Pat::Object(obj_pat) => {
        for prop in &obj_pat.props {
          match prop {
            ObjectPatProp::KeyValue(kv_prop) => {
              self.visit_pat(&kv_prop.value);
            }
            ObjectPatProp::Assign(assign_prop) => {
              self.defined_idents.insert(assign_prop.key.to_string());

              let mut used_idents_collector = UsedIdentsCollector::new();
              assign_prop.value.visit_with(&mut used_idents_collector);

              self.used_idents.extend(used_idents_collector.used_idents);
            }
            ObjectPatProp::Rest(rest_prop) => {
              self.visit_pat(&rest_prop.arg);
            }
          }
        }
      }
      Pat::Assign(assign_pat) => {
        self.visit_pat(&assign_pat.left);
      }
      Pat::Invalid(_) => {}
      Pat::Expr(_) => {}
    }
  }
}
