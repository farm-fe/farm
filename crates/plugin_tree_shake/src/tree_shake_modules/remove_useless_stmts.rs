use std::collections::{HashMap, HashSet};

use farmfe_core::{
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  swc_ecma_ast::{
    self, Id, ImportDecl, ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem, Stmt,
  },
};
use farmfe_toolkit::swc_ecma_visit::{VisitMut, VisitMutWith, VisitWith};

use crate::{
  module::TreeShakeModule, statement_graph::defined_idents_collector::DefinedIdentsCollector,
};

pub fn remove_useless_stmts(
  tree_shake_module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  tree_shake_modules_map: &HashMap<ModuleId, TreeShakeModule>,
) {
  farmfe_core::farm_profile_function!(format!(
    "remove_useless_stmts {:?}",
    tree_shake_module.id.to_string()
  ));

  let tree_shake_module = tree_shake_modules_map.get(tree_shake_module_id).unwrap();
  // if the module contains side effects, we should keep all the statements
  if tree_shake_module.side_effects {
    return;
  }

  let module = module_graph.module_mut(tree_shake_module_id).unwrap();
  let swc_module = &mut module.meta.as_script_mut().ast;

  let mut stmts_to_remove = vec![];

  for (index, item) in swc_module.body.iter_mut().enumerate() {
    if !tree_shake_module.stmt_graph.used_stmts().contains(&index) {
      stmts_to_remove.push(index);
    } else {
      let mut useless_specifier_remover = UselessSpecifierRemover {
        used_defined_idents: &tree_shake_module
          .stmt_graph
          .stmt(&index)
          .used_defined_idents,
      };

      // remove unused import / export / var decl
      match item {
        ModuleItem::ModuleDecl(decl) => match decl {
          ModuleDecl::Import(_) | ModuleDecl::ExportDecl(_) | ModuleDecl::ExportNamed(_) => {
            decl.visit_mut_with(&mut useless_specifier_remover);
          }
          _ => { /* ignore other module decl statement */ }
        },
        ModuleItem::Stmt(Stmt::Decl(decl)) => {
          if let swc_ecma_ast::Decl::Var(var_decl) = decl {
            useless_specifier_remover.visit_mut_var_decl(var_decl);
          }
        }
        _ => { /* ignore other statement */ }
      }
    }
  }

  // for import or export from statement, if the source module contains side effects statement, we should keep the statement
  let mut pending_import_export_from_check = vec![];

  for index in &stmts_to_remove {
    if let ModuleItem::ModuleDecl(module_decl) = &mut swc_module.body[*index] {
      match module_decl {
        ModuleDecl::Import(import_decl) => {
          pending_import_export_from_check.push((
            *index,
            import_decl.src.value.to_string(),
            ResolveKind::Import,
          ));
        }
        ModuleDecl::ExportNamed(export_decl) => {
          if let Some(src) = &export_decl.src {
            pending_import_export_from_check.push((
              *index,
              src.value.to_string(),
              ResolveKind::ExportFrom,
            ));
          }
        }
        ModuleDecl::ExportAll(export_all) => {
          pending_import_export_from_check.push((
            *index,
            export_all.src.value.to_string(),
            ResolveKind::ExportFrom,
          ));
        }
        _ => {}
      }
    }
  }

  let mut preserved_import_export_from_stmts = vec![];

  for (index, src, kind) in pending_import_export_from_check {
    let dep_module_id = module_graph.get_dep_by_source(tree_shake_module_id, &src, Some(kind));
    let dep_module = module_graph.module(&dep_module_id).unwrap();
    let dep_tree_shake_module = tree_shake_modules_map.get(&dep_module_id);
    // if dep tree shake module is not found, it means the dep module is not tree shakable, so we should keep the import / export from statement
    // and preserve import / export from statement if the source module contains side effects statement
    if dep_module.external
      || dep_module.side_effects
      || dep_tree_shake_module.is_none()
      || dep_tree_shake_module.unwrap().contains_self_executed_stmt
    {
      preserved_import_export_from_stmts.push(index);
    }
  }

  let module = module_graph.module_mut(tree_shake_module_id).unwrap();
  let swc_module = &mut module.meta.as_script_mut().ast;
  stmts_to_remove.reverse();

  for index in stmts_to_remove {
    if !preserved_import_export_from_stmts.contains(&index) {
      swc_module.body.remove(index);
    } else {
      // remove all the specifiers in the import / export from statement
      let mut useless_specifier_remover = UselessSpecifierRemover {
        used_defined_idents: &HashSet::new(),
      };
      if let ModuleItem::ModuleDecl(module_decl) = &mut swc_module.body[index] {
        if module_decl.is_import() {
          if let ModuleDecl::Import(import_decl) = module_decl {
            useless_specifier_remover.visit_mut_import_decl(import_decl);
          }
        } else if module_decl.is_export_named() {
          if let ModuleDecl::ExportNamed(export_decl) = module_decl {
            useless_specifier_remover.visit_mut_export_specifiers(&mut export_decl.specifiers);
          }
        }
      }
    }
  }
}

struct UselessSpecifierRemover<'a> {
  used_defined_idents: &'a HashSet<Id>,
}

impl<'a> VisitMut for UselessSpecifierRemover<'a> {
  fn visit_mut_import_decl(&mut self, import_decl: &mut ImportDecl) {
    let mut specifiers_to_remove = vec![];

    for (index, specifier) in import_decl.specifiers.iter().enumerate() {
      let id = match specifier {
        ImportSpecifier::Named(named_specifier) => named_specifier.local.to_id(),
        ImportSpecifier::Default(default) => default.local.to_id(),
        ImportSpecifier::Namespace(ns) => ns.local.to_id(),
      };

      if !self.used_defined_idents.contains(&id) {
        specifiers_to_remove.push(index);
      }
    }

    specifiers_to_remove.reverse();

    for index in specifiers_to_remove {
      import_decl.specifiers.remove(index);
    }
  }

  fn visit_mut_export_specifiers(
    &mut self,
    specifiers: &mut Vec<farmfe_core::swc_ecma_ast::ExportSpecifier>,
  ) {
    let mut specifiers_to_remove = vec![];

    for (index, specifier) in specifiers.iter().enumerate() {
      let id = match specifier {
        farmfe_core::swc_ecma_ast::ExportSpecifier::Named(named_specifier) => {
          match &named_specifier.orig {
            ModuleExportName::Ident(ident) => ident.to_id(),
            _ => unreachable!(),
          }
        }
        farmfe_core::swc_ecma_ast::ExportSpecifier::Namespace(ns) => match &ns.name {
          ModuleExportName::Ident(ident) => ident.to_id(),
          _ => unreachable!(),
        },
        farmfe_core::swc_ecma_ast::ExportSpecifier::Default(default) => default.exported.to_id(),
      };

      if !self.used_defined_idents.contains(&id) {
        specifiers_to_remove.push(index);
      }
    }

    specifiers_to_remove.reverse();

    for index in specifiers_to_remove {
      specifiers.remove(index);
    }
  }

  fn visit_mut_export_decl(&mut self, n: &mut swc_ecma_ast::ExportDecl) {
    // only remove unused var decl
    if let swc_ecma_ast::Decl::Var(var_decl) = &mut n.decl {
      self.visit_mut_var_decl(var_decl);
    }
  }

  fn visit_mut_var_decl(&mut self, n: &mut swc_ecma_ast::VarDecl) {
    let mut decls_to_remove = vec![];

    for (index, decl) in n.decls.iter_mut().enumerate() {
      let mut defined_idents_collector = DefinedIdentsCollector::new();
      decl.visit_with(&mut defined_idents_collector);

      if self
        .used_defined_idents
        .is_disjoint(&defined_idents_collector.defined_idents)
      {
        decls_to_remove.push(index);
      }
    }

    decls_to_remove.reverse();

    for index in decls_to_remove {
      n.decls.remove(index);
    }
  }
}
