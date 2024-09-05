use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
};

use farmfe_core::{
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, Expr, Id, IdentName, KeyValuePatProp,
    KeyValueProp, MemberProp, ObjectPat, ObjectPatProp, Pat, Prop, PropName, PropOrSpread,
    SimpleAssignTarget,
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

use super::{
  bundle::ModuleAnalyzerManager, modules_analyzer::module_analyzer::VarRefKey,
  uniq_name::BundleVariable,
};

type RenameMap<'a> = HashMap<VarRefKey<'a>, usize>;

// #[derive(Debug)]
pub struct RenameIdent<'a> {
  map: RenameMap<'a>,
  bundle_variable: &'a BundleVariable,
  module_analyzer_manager: &'a ModuleAnalyzerManager<'a>,
}

impl<'a> RenameIdent<'a> {
  pub fn new(
    map: RenameMap<'a>,
    bundle_variable: &'a BundleVariable,
    module_analyzer_manager: &'a ModuleAnalyzerManager<'a>,
  ) -> Self {
    Self {
      map,
      bundle_variable,
      module_analyzer_manager,
    }
  }

  fn rename(&self, ident: &Ident) -> Option<String> {
    let r = RefCell::new(ident.to_id());
    let v = self.map.get(&(r.borrow().into())).map(|var| {
      return self.bundle_variable.render_name(*var);
      let (var, root) = self.bundle_variable.var(*var);

      let Some(root) = root else {
        return var.render_name();
      };

      let var_module_id = self.bundle_variable.module_id_by_var(&var);
      let root_module_id = self.bundle_variable.module_id_by_var(&root);

      let (Some(var_module_id), Some(root_module_id)) = (var_module_id, root_module_id) else {
        return root.render_name();
      };

      if self
        .module_analyzer_manager
        .is_same_bundle(var_module_id, root_module_id)
      {
        return root.render_name();
      }

      var.render_name()
    });
    v
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
          *n = Prop::KeyValue(farmfe_core::swc_ecma_ast::KeyValueProp {
            key: farmfe_core::swc_ecma_ast::PropName::Ident(IdentName {
              span: DUMMY_SP,
              sym: m.sym.as_str().into(),
            }),
            value: Box::new(farmfe_core::swc_ecma_ast::Expr::Ident(
              new_name.as_str().into(),
            )),
          });
          return;
        }
      }

      _ => {}
    }

    n.visit_mut_children_with(self);
  }

  fn visit_mut_prop_or_spread(&mut self, n: &mut PropOrSpread) {
    match n {
      PropOrSpread::Prop(box p) => match p {
        Prop::Shorthand(ident) => {
          if let Some(new_name) = self.rename(ident) {
            *p = Prop::KeyValue(KeyValueProp {
              key: farmfe_core::swc_ecma_ast::PropName::Ident(IdentName {
                span: DUMMY_SP,
                sym: ident.sym.as_str().into(),
              }),
              value: Box::new(farmfe_core::swc_ecma_ast::Expr::Ident(
                new_name.as_str().into(),
              )),
            });
          } else {
            p.visit_mut_with(self);
          }
        }
        Prop::KeyValue(key_value_prop) => {
          key_value_prop.visit_mut_with(self);
        }
        _ => {
          p.visit_mut_with(self);
        }
      },
      PropOrSpread::Spread(s) => {
        s.visit_mut_with(self);
      }
    }
  }

  fn visit_mut_key_value_prop(&mut self, n: &mut KeyValueProp) {
    //
    // skip it
    // ```js
    // {
    //   key: value,
    // }
    // ```
    //
    if let farmfe_core::swc_ecma_ast::PropName::Ident(_) = n.key {
    } else {
      n.key.visit_mut_with(self);
    }

    n.value.visit_mut_with(self);
  }

  fn visit_mut_key_value_pat_prop(&mut self, n: &mut KeyValuePatProp) {
    if let PropName::Ident(_) = n.key {
    } else {
      n.key.visit_mut_with(self);
    }

    n.value.visit_mut_with(self);
  }

  fn visit_mut_object_pat(&mut self, n: &mut ObjectPat) {
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
        ObjectPatProp::KeyValue(n) => {
          n.visit_mut_with(self);
        }
        _ => prop.visit_mut_children_with(self),
      };
    }
  }

  fn visit_mut_member_prop(&mut self, n: &mut MemberProp) {
    // ns.default, skip
    if let MemberProp::Ident(_) = n {
      return;
    }

    n.visit_mut_children_with(self);
  }
}
