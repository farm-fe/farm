pub use farmfe_core::module::meta_data::script::{EXPORT_DEFAULT, EXPORT_NAMESPACE};
use farmfe_core::{
  module::{meta_data::script::statement::SwcId, module_graph::ModuleGraph, ModuleId},
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, ExportSpecifier, Expr, Ident, IdentName,
    ImportSpecifier, KeyValuePatProp, KeyValueProp, MemberProp, ModuleExportName, ObjectPat,
    ObjectPatProp, Pat, Prop, PropName, PropOrSpread, SimpleAssignTarget,
  },
  HashMap,
};
use swc_atoms::Atom;
use swc_ecma_visit::{VisitMut, VisitMutWith};

#[derive(Default)]
struct TopLevelIdents {
  idents: HashMap<Atom, usize>,
}

impl TopLevelIdents {
  pub fn new() -> Self {
    let mut tli = Self {
      idents: HashMap::default(),
    };
    // should always add default export to avoid name conflicts with preserved key words
    tli.add_ident(EXPORT_DEFAULT.into());

    return tli;
  }

  fn extend(&mut self, iter: impl Iterator<Item = Atom>) {
    for ident in iter {
      if !self.idents.contains_key(&ident) {
        self.add_ident(ident);
      }
    }
  }

  pub fn add_ident(&mut self, ident: Atom) {
    let count = self.idents.entry(ident).or_insert(0);
    *count += 1;
  }

  pub fn get_unique_ident(&mut self, ident: &Atom) -> Atom {
    if let Some(mut count) = self.idents.get(ident).cloned() {
      if count > 1 {
        let mut unique_ident = self.generate_unique_ident(ident, count);
        // make sure the unique ident is not used
        while self.idents.contains_key(&unique_ident) {
          self.add_ident(unique_ident.clone());
          count += 1;
          unique_ident = self.generate_unique_ident(ident, count);
        }
        // update the count for the new unique ident
        self.add_ident(unique_ident.clone());

        return unique_ident;
      } else {
        return ident.clone();
      }
    }

    unreachable!("add_ident({ident}) should be called before get_unique_ident")
  }

  fn generate_unique_ident(&mut self, ident: &Atom, count: usize) -> Atom {
    format!("{ident}${}", count - 1).into()
  }
}

#[derive(Default)]
pub struct TopLevelIdentsRenameHandler {
  module_rename_map: HashMap<ModuleId, HashMap<SwcId, SwcId>>,
  top_level_idents: TopLevelIdents,
}

impl TopLevelIdentsRenameHandler {
  fn new(top_level_idents: TopLevelIdents) -> Self {
    Self {
      module_rename_map: HashMap::default(),
      top_level_idents,
    }
  }

  pub fn rename_ident(&mut self, module_id: ModuleId, from: SwcId, to: SwcId) {
    self
      .module_rename_map
      .entry(module_id)
      .or_insert_with(HashMap::default)
      .insert(from, to);
  }

  pub fn get_renamed_ident(&self, module_id: &ModuleId, ident: &SwcId) -> Option<SwcId> {
    self
      .module_rename_map
      .get(module_id)
      .and_then(|map| map.get(ident))
      .cloned()
  }

  pub fn get_unique_ident(&mut self, ident: &SwcId) -> Option<SwcId> {
    self.top_level_idents.add_ident(ident.sym.clone());
    let unique_ident = self.top_level_idents.get_unique_ident(&ident.sym);

    if unique_ident != *ident.sym {
      let mut cloned = ident.clone();
      cloned.sym = unique_ident;
      Some(cloned)
    } else {
      None
    }
  }

  /// rename the imported ident if there are conflicts
  pub fn rename_ident_if_conflict(&mut self, module_id: &ModuleId, ident: &SwcId) -> Option<SwcId> {
    self.get_unique_ident(ident).map(|unique_ident| {
      self.rename_ident(module_id.clone(), ident.clone(), unique_ident.clone());
      unique_ident
    })
  }
}

pub struct RenameVisitor<'a> {
  module_id: &'a ModuleId,
  source_module_id: Option<&'a ModuleId>,
  rename_handler: &'a TopLevelIdentsRenameHandler,
}

impl<'a> RenameVisitor<'a> {
  pub fn new(
    module_id: &'a ModuleId,
    source_module_id: Option<&'a ModuleId>,
    rename_handler: &'a TopLevelIdentsRenameHandler,
  ) -> Self {
    Self {
      module_id,
      source_module_id,
      rename_handler,
    }
  }

  fn get_renamed_ident(&self, ident: &Ident) -> Option<SwcId> {
    self
      .rename_handler
      .get_renamed_ident(self.module_id, &ident.to_id().into())
  }
}

impl<'a> VisitMut for RenameVisitor<'a> {
  fn visit_mut_import_decl(&mut self, node: &mut farmfe_core::swc_ecma_ast::ImportDecl) {
    if let Some(source_module_id) = self.source_module_id {
      node.src = Box::new(source_module_id.to_string().into());
    }

    node.visit_mut_children_with(self);
  }

  fn visit_mut_export_all(&mut self, node: &mut farmfe_core::swc_ecma_ast::ExportAll) {
    if let Some(source_module_id) = self.source_module_id {
      node.src = Box::new(source_module_id.to_string().into());
    }

    node.visit_mut_children_with(self);
  }

  fn visit_mut_named_export(&mut self, node: &mut farmfe_core::swc_ecma_ast::NamedExport) {
    if node.src.is_none()
      && let Some(source_module_id) = self.source_module_id
    {
      node.src = Some(Box::new(source_module_id.to_string().into()));
    }

    node.visit_mut_children_with(self);
  }

  fn visit_mut_import_specifier(&mut self, sp: &mut ImportSpecifier) {
    if let ImportSpecifier::Named(named) = sp {
      // import { a as aa } to import { a as aa1 }
      if named.imported.is_some() {
        named.local.visit_mut_with(self);
      } else if let Some(renamed_ident) = self.get_renamed_ident(&named.local) {
        named.imported = Some(ModuleExportName::Ident(Ident::new(
          named.local.sym.clone(),
          DUMMY_SP,
          named.local.ctxt,
        )));
        let ctxt = renamed_ident.ctxt();
        // import { a } to import { a as a1 }
        named.local = Ident::new(renamed_ident.sym, DUMMY_SP, ctxt);
      }
    } else {
      sp.visit_mut_children_with(self);
    }
  }

  fn visit_mut_export_specifier(&mut self, sp: &mut ExportSpecifier) {
    if let ExportSpecifier::Named(named) = sp {
      // do not rename exported ident
      named.orig.visit_mut_children_with(self);
    } else {
      sp.visit_mut_children_with(self);
    }
  }

  fn visit_mut_ident(&mut self, n: &mut Ident) {
    if let Some(renamed_ident) = self
      .rename_handler
      .get_renamed_ident(self.module_id, &n.to_id().into())
    {
      n.ctxt = renamed_ident.ctxt();
      n.sym = renamed_ident.sym;
    }
  }

  fn visit_mut_prop(&mut self, n: &mut farmfe_core::swc_ecma_ast::Prop) {
    match n {
      Prop::Shorthand(m) => {
        if let Some(new_name) = self.get_renamed_ident(m) {
          *n = Prop::KeyValue(farmfe_core::swc_ecma_ast::KeyValueProp {
            key: farmfe_core::swc_ecma_ast::PropName::Ident(IdentName {
              span: DUMMY_SP,
              sym: m.sym.as_str().into(),
            }),
            value: Box::new(farmfe_core::swc_ecma_ast::Expr::Ident(new_name.sym.into())),
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
          if let Some(new_name) = self.get_renamed_ident(ident) {
            *p = Prop::KeyValue(KeyValueProp {
              key: farmfe_core::swc_ecma_ast::PropName::Ident(IdentName {
                span: DUMMY_SP,
                sym: ident.sym.as_str().into(),
              }),
              value: Box::new(farmfe_core::swc_ecma_ast::Expr::Ident(new_name.sym.into())),
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
          if self.get_renamed_ident(&n.key).is_some() {
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

pub fn init_rename_handler(
  sorted_modules: &Vec<ModuleId>,
  module_graph: &ModuleGraph,
) -> TopLevelIdentsRenameHandler {
  let mut top_level_idents = TopLevelIdents::new();

  // init top level idents
  sorted_modules.iter().for_each(|module_id| {
    let module = module_graph.module(module_id).unwrap();
    let script_meta = module.meta.as_script();

    top_level_idents.extend(
      script_meta
        .unresolved_idents
        .iter()
        .map(|id| id.sym.clone()),
    );

    // there are name conflicts deeply in the module, for example:
    // ```
    // // xxx
    // export const a = 'a';
    //
    // // index.js
    // import { a as renamedA } from 'xxx'
    // function A() {
    //   const a = 2;
    //   console.log(renamedA);
    // }
    // ```
    // should be renamed to:
    // ```
    // const a$1 = 'a'
    // function A() {
    //   const a = 2;
    //   console.log(a$1)
    // }
    // ```
    // we have to rename a to a$1 to avoid ident conflicts
    top_level_idents.extend(
      script_meta
        .all_deeply_declared_idents
        .iter()
        .map(|id| id.clone()),
    );
  });

  let mut rename_handler = TopLevelIdentsRenameHandler::new(top_level_idents);

  // rename top level idents that are not defined in module decl statement
  sorted_modules.iter().for_each(|module_id| {
    let module = module_graph.module(module_id).unwrap();
    let script_meta = module.meta.as_script();

    for statement in &script_meta.statements {
      if statement.import_info.is_some()
        || statement.export_info.is_some()
        || statement.defined_idents.is_empty()
      {
        continue;
      }

      statement.defined_idents.iter().for_each(|ident| {
        rename_handler.rename_ident_if_conflict(module_id, ident);
      });
    }
  });

  rename_handler
}
