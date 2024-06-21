use std::collections::{HashMap, HashSet};

use farmfe_core::{
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, Expr, Id, KeyValuePatProp, ObjectPatProp,
    Pat, Prop, PropName, SimpleAssignTarget,
  },
};
use farmfe_toolkit::{
  swc_atoms::Atom,
  swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith},
};

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
              if let Some(box ref value) = assign_prop.value {
                value.visit_with(self);
              }
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
      Pat::Expr(box expr) => {
        expr.visit_with(self);
      }
    }
  }
}

use farmfe_core::swc_ecma_ast::Ident;

use super::Var;

#[derive(Debug, Default)]
pub struct RenameIdent<'a> {
  map: HashMap<&'a Id, &'a Var>,
}

impl<'a> RenameIdent<'a> {
  pub fn new(map: HashMap<&'a Id, &'a Var>) -> Self {
    Self { map }
  }

  fn rename(&self, ident: &Ident) -> Option<String> {
    self.map.get(&ident.to_id()).map(|var| var.render_name())
  }
}

impl<'a> VisitMut for RenameIdent<'a> {
  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    if let Some(new_name) = self.rename(ident) {
      ident.sym = Atom::from(new_name.as_str());
      ident.span = DUMMY_SP;
    }
  }

  fn visit_mut_prop(&mut self, n: &mut farmfe_core::swc_ecma_ast::Prop) {
    match n {
      Prop::Shorthand(m) => {
        if let Some(new_name) = self.rename(m) {
          *n = farmfe_core::swc_ecma_ast::Prop::KeyValue(farmfe_core::swc_ecma_ast::KeyValueProp {
            key: farmfe_core::swc_ecma_ast::PropName::Ident(Ident {
              span: DUMMY_SP,
              sym: m.sym.as_str().into(),
              optional: false,
            }),
            value: Box::new(farmfe_core::swc_ecma_ast::Expr::Ident(
              new_name.as_str().into(),
            )),
          });

          n.visit_mut_children_with(self);
        }
      }

      _ => n.visit_mut_children_with(self),
    }
  }

  fn visit_mut_object_pat(&mut self, n: &mut farmfe_core::swc_ecma_ast::ObjectPat) {
    for prop in &mut n.props {
      match prop {
        ObjectPatProp::Assign(n) => {
          // const { field = 100 } = x;
          // =>
          // const { field: field = 100 } = x;

          if self.rename(&n.key).is_some() {
            let mut new_value = if let Some(ref value) = n.value {
              Box::new(Pat::Expr(Box::new(Expr::Assign(AssignExpr {
                span: DUMMY_SP,
                op: AssignOp::Assign,
                left: AssignTarget::Simple(SimpleAssignTarget::Ident(n.key.clone())),
                right: value.clone(),
              }))))
            } else {
              Box::new(Pat::Ident(BindingIdent {
                id: n.key.id.clone(),
                type_ann: None,
              }))
            };

            new_value.visit_mut_with(self);

            *prop = ObjectPatProp::KeyValue(KeyValuePatProp {
              key: PropName::Ident(n.key.clone().into()),
              value: new_value,
            });
          } else {
            n.visit_mut_with(self);
          }
        }
        _ => prop.visit_mut_children_with(self),
      };
    }
  }
}
