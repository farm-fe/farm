use crate::swc_ecma_visit::{Visit, VisitWith};
use farmfe_core::swc_common::Mark;
use farmfe_core::HashSet;
use farmfe_core::{
  module::meta_data::script::statement::SwcId,
  swc_ecma_ast::{ObjectPatProp, Pat},
};

#[derive(Default)]
pub struct DefinedIdentsCollector<'a> {
  pub defined_idents: HashSet<SwcId>,
  assign_callback: Option<&'a mut dyn FnMut(SwcId, SwcId)>,
  current_assign_left_idents: HashSet<SwcId>,
}

impl<'a> DefinedIdentsCollector<'a> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_callback(cb: &'a mut dyn FnMut(SwcId, SwcId)) -> Self {
    Self {
      defined_idents: HashSet::default(),
      assign_callback: Some(cb),
      current_assign_left_idents: HashSet::default(),
    }
  }
}

impl Visit for DefinedIdentsCollector<'_> {
  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    if let Some(callback) = &mut self.assign_callback {
      for ident in &self.current_assign_left_idents {
        callback(ident.clone(), n.to_id().into());
      }
    }
  }

  fn visit_pat(&mut self, pat: &Pat) {
    match pat {
      Pat::Ident(bi) => {
        self.defined_idents.insert(bi.id.to_id().into());
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
              self.defined_idents.insert(assign_prop.key.to_id().into());
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

pub struct UnresolvedIdentCollector {
  pub unresolved_idents: HashSet<SwcId>,
  unresolved_mark: Mark,
}

impl UnresolvedIdentCollector {
  pub fn new(unresolved_mark: Mark) -> Self {
    Self {
      unresolved_idents: HashSet::default(),
      unresolved_mark,
    }
  }
}

impl Visit for UnresolvedIdentCollector {
  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    if n.ctxt.outer() == self.unresolved_mark {
      self.unresolved_idents.insert(n.to_id().into());
    }
  }
}
