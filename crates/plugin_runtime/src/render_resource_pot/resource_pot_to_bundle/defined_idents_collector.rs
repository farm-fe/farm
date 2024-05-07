use std::collections::{HashMap, HashSet};

use farmfe_core::{
  swc_common::DUMMY_SP,
  swc_ecma_ast::{Id, ObjectPatProp, Pat},
};
use farmfe_toolkit::{
  swc_atoms::Atom,
  swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith},
};

pub struct DefinedIdentsCollector {
  pub defined_idents: HashSet<Id>,
  pub used_idents: HashSet<Id>,
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

use farmfe_core::swc_ecma_ast::Ident;

use super::Var;

pub struct UsedIdentsCollector {
  pub used_idents: HashSet<Id>,
}

impl UsedIdentsCollector {
  pub fn new() -> Self {
    Self {
      used_idents: HashSet::new(),
    }
  }
}

impl Visit for UsedIdentsCollector {
  fn visit_ident(&mut self, ident: &Ident) {
    self.used_idents.insert(ident.to_id());
  }
}

#[derive(Debug, Default)]
pub struct RenameIdent<'a> {
  map: HashMap<&'a Id, &'a Var>,
}

impl<'a> RenameIdent<'a> {
  pub fn new(map: HashMap<&'a Id, &'a Var>) -> Self {
    Self {
      map,
      ..Default::default()
    }
  }
}

impl<'a> VisitMut for RenameIdent<'a> {
  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    if let Some(Some(new_name)) = self.map.get(&ident.to_id()).map(|var| var.rename.as_ref()) {
      ident.sym = Atom::from(new_name.as_str());
      ident.span = DUMMY_SP;
    }
  }

  fn visit_mut_prop(&mut self, n: &mut farmfe_core::swc_ecma_ast::Prop) {
    match n {
      farmfe_core::swc_ecma_ast::Prop::Shorthand(m) => {
        if self
          .map
          .get(&m.to_id())
          .is_some_and(|var| var.rename.is_some())
        {
          *n = farmfe_core::swc_ecma_ast::Prop::KeyValue(farmfe_core::swc_ecma_ast::KeyValueProp {
            key: farmfe_core::swc_ecma_ast::PropName::Ident(Ident {
              span: DUMMY_SP,
              sym: m.sym.as_str().into(),
              optional: false,
            }),
            value: Box::new(farmfe_core::swc_ecma_ast::Expr::Ident(m.clone())),
          });

          n.visit_mut_children_with(self);
        }
      }
      _ => n.visit_mut_children_with(self),
    }
  }
}
