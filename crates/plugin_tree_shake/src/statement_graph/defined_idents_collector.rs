use std::collections::HashSet;

use farmfe_core::swc_ecma_ast::{Id, ObjectPatProp, Pat};
use farmfe_toolkit::swc_ecma_visit::Visit;

pub struct DefinedIdentsCollector {
  pub defined_idents: HashSet<Id>,
}

impl DefinedIdentsCollector {
  pub fn new() -> Self {
    Self {
      defined_idents: HashSet::new(),
    }
  }
}

impl Visit for DefinedIdentsCollector {
  fn visit_pat(&mut self, pat: &Pat) {
    match pat {
      Pat::Ident(bi) => {
        self.defined_idents.insert(bi.id.to_id());
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
              self.defined_idents.insert(assign_prop.key.to_id());
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
