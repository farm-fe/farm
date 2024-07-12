use std::collections::HashSet;

use crate::{
  swc_ecma_utils::ident::IdentLike,
  swc_ecma_visit::{Visit, VisitWith},
};
use farmfe_core::swc_ecma_ast::{Id, ObjectPatProp, Pat};

pub struct DefinedIdentsCollector<'a> {
  pub defined_idents: HashSet<Id>,
  assign_callback: Option<Box<&'a mut dyn FnMut(Id, Id)>>,
  current_assign_left_idents: HashSet<Id>,
}

impl<'a> DefinedIdentsCollector<'a> {
  pub fn new() -> Self {
    Self {
      defined_idents: HashSet::new(),
      assign_callback: None,
      current_assign_left_idents: HashSet::new(),
    }
  }

  pub fn from_callback(cb: Box<&'a mut dyn FnMut(Id, Id)>) -> Self {
    Self {
      defined_idents: HashSet::new(),
      assign_callback: Some(cb),
      current_assign_left_idents: HashSet::new(),
    }
  }
}

impl<'a> Visit for DefinedIdentsCollector<'a> {
  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    if let Some(callback) = &mut self.assign_callback {
      for ident in &self.current_assign_left_idents {
        callback(ident.to_id(), n.to_id());
      }
    }
  }

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
        assign_pat.left.visit_with(self);

        // collect defined idents for assign right
        let mut collector = DefinedIdentsCollector::new();
        assign_pat.left.visit_with(&mut collector);
        self.current_assign_left_idents = collector.defined_idents.clone();

        assign_pat.right.visit_with(self);
        self.current_assign_left_idents.clear();
      }
      Pat::Invalid(_) => {}
      Pat::Expr(_) => {}
    }
  }
}
