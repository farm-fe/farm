use farmfe_core::{
  module::{
    meta_data::script::statement::SwcId, module_graph::ModuleGraph, ModuleId, ModuleSystem,
  },
  plugin::ResolveKind,
  swc_ecma_ast::{
    self, ImportDecl, ImportSpecifier, ModuleDecl, ModuleExportName, ModuleItem, Stmt,
  },
};
use farmfe_core::{HashMap, HashSet};
use farmfe_toolkit::{
  script::{analyze_statement::analyze_statements, idents_collector::DefinedIdentsCollector},
  swc_ecma_visit::{VisitMut, VisitMutWith, VisitWith},
};

use crate::module::TreeShakeModule;

pub fn remove_useless_stmts(
  tree_shake_module_id: &ModuleId,
  module_graph: &mut ModuleGraph,
  tree_shake_modules_map: &HashMap<ModuleId, TreeShakeModule>,
) -> Vec<ModuleId> {
  farmfe_core::farm_profile_function!(format!(
    "remove_useless_stmts {:?}",
    tree_shake_module_id.to_string()
  ));

  let tree_shake_module = tree_shake_modules_map.get(tree_shake_module_id).unwrap();
  // if the module is not esm, we should keep all the statements
  if tree_shake_module.module_system != ModuleSystem::EsModule {
    return vec![];
  }
  let is_export_all_source_module_empty =
    get_export_all_source_module_is_empty(tree_shake_module_id, module_graph);

  let module = module_graph.module_mut(tree_shake_module_id).unwrap();
  let meta = module.meta.as_script_mut();
  let swc_module = &mut meta.ast;

  let mut modules_to_remove = vec![];
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
          ModuleDecl::ExportAll(export_all) => {
            if let Some((source_module_id, is_empty)) =
              is_export_all_source_module_empty.get(&export_all.src.value.to_string())
            {
              if *is_empty {
                modules_to_remove.push(source_module_id.clone());
                stmts_to_remove.push(index);
              }
            }
          }
          _ => { /* ignore other module decl statement */ }
        },
        ModuleItem::Stmt(Stmt::Decl(swc_ecma_ast::Decl::Var(var_decl))) => {
          useless_specifier_remover.visit_mut_var_decl(var_decl);
        }
        _ => { /* ignore other statement */ }
      }
    }
  }

  stmts_to_remove.reverse();

  for index in stmts_to_remove {
    swc_module.body.remove(index);
  }

  // update statements in module_graph
  meta.statements = analyze_statements(&swc_module);

  modules_to_remove
}

/// For export all statement, return whether the source module is empty
/// e.g. export * from './foo'; if foo is empty, the return value should be:
/// {
///  './foo': true
/// }
fn get_export_all_source_module_is_empty(
  tree_shake_module_id: &ModuleId,
  module_graph: &ModuleGraph,
) -> HashMap<String, (ModuleId, bool)> {
  let mut source_module_is_empty = HashMap::default();

  let tree_shake_module = module_graph.module(tree_shake_module_id).unwrap();
  let meta = tree_shake_module.meta.as_script();

  for item in meta.ast.body.iter() {
    if let ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export_all)) = item {
      let source = export_all.src.value.to_string();
      let source_module_id = module_graph.get_dep_by_source_optional(
        tree_shake_module_id,
        &source,
        Some(ResolveKind::ExportFrom),
      );

      if let Some(source_module_id) = source_module_id {
        let source_module = module_graph.module(&source_module_id).unwrap();
        let is_empty = if source_module.module_type.is_script() && !source_module.external {
          source_module.meta.as_script().ast.body.is_empty()
        } else {
          false
        };
        source_module_is_empty.insert(source, (source_module_id, is_empty));
      }
    }
  }

  source_module_is_empty
}

struct UselessSpecifierRemover<'a> {
  used_defined_idents: &'a HashSet<SwcId>,
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

      if !self.used_defined_idents.contains(&id.into()) {
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

      if !self.used_defined_idents.contains(&id.into()) {
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
    // skip remove unused var decl if self.used_defined_idents is empty
    // when self.used_defined_idents is empty, it means this statement is preserved when tracing dependents side effects statements
    // Farm do not handle this case for now, it may be optimized in the future
    if self.used_defined_idents.is_empty() {
      return;
    }

    let mut decls_to_remove = vec![];

    for (index, decl) in n.decls.iter_mut().enumerate() {
      let mut defined_idents_collector = DefinedIdentsCollector::new();
      decl.name.visit_with(&mut defined_idents_collector);

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
