use farmfe_core::{
  farm_profile_function, farm_profile_scope,
  module::{
    meta_data::script::statement::{ExportSpecifierInfo, ImportSpecifierInfo, StatementId},
    module_graph::ModuleGraph,
    ModuleId,
  },
  plugin::ResolveKind,
  swc_ecma_ast::Module as SwcModule,
  HashSet,
};

pub fn strip_module_decl(module_id: &ModuleId, module_graph: &ModuleGraph) -> SwcModule {
  let module = module_graph.module(module_id).unwrap();
  let script_meta = module.meta.as_script();
  let mut ast = script_meta.ast.clone();

  for statement in &script_meta.statements {
    if let Some(import_info) = &statement.import_info {
      let source_module_id = module_graph.get_dep_by_source(module_id, &import_info.source, None);
      let source_module = module_graph.module(&source_module_id).unwrap();
      let source_module_script_meta = source_module.meta.as_script();

      for specifier in &import_info.specifiers {
        match specifier {
          // import { foo, bar as baz } from './module';
          // =>
          // if module is esm: var foo = foo, baz = bar;
          // if module is cjs: var module_cjs = module_default(), foo = module_cjs.foo, baz = module_cjs.bar;
          ImportSpecifierInfo::Named { local, imported } => {}

          // import foo from './module';
          // =>
          // if module is esm: var foo = module_default;
          // if module is cjs: var foo = _interopRequireDefault(module_default()).default;
          ImportSpecifierInfo::Default(_) => {}

          // import * as ns from './module';
          // =>
          // if module is esm: var ns = module_ns;
          // if module is cjs: var ns = module_default();
          ImportSpecifierInfo::Namespace(_) => {
            // remove
          }
        }
      }
    } else if let Some(export_info) = &statement.export_info {
      if let Some(source) = &export_info.source {
        let source_module_id = module_graph.get_dep_by_source(module_id, source, None);
        let source_module = module_graph.module(&source_module_id).unwrap();
        let source_module_script_meta = source_module.meta.as_script();

        let mut stmts_to_remove = vec![];

        for specifier in &export_info.specifiers {
          match specifier {
            // export * from './module';
            // =>
            // remove. cause the reexport is handled when expanding exports
            ExportSpecifierInfo::All => {
              stmts_to_remove.push(statement.id);
            }

            // export default 'expression';
            // =>
            // var module_default = 'expression';
            //
            // export default const foo = 1;
            // =>
            // const foo = 1; var module_default = foo;
            ExportSpecifierInfo::Default => {}

            // export { foo, bar as baz } from './module';
            // =>
            // if module is esm: var foo = foo, var baz = bar;
            // if module is cjs: var module_cjs = module_default(), var foo = module_cjs.foo, var baz = module_cjs.bar;
            // note that `baz` might be default
            ExportSpecifierInfo::Named { .. } => {
              stmts_to_remove.push(statement.id);
            }

            // export * as ns from './module';
            // =>
            // if module is esm: var ns = module_ns;
            // if module is cjs: var ns = module_default();
            ExportSpecifierInfo::Namespace(_) => {
              stmts_to_remove.push(statement.id);
            }
          }
        }
      } else {
      }
    }
  }
  ast
}

pub fn analyze_module_strip_action(
  topo_sorted_module_ids: &Vec<ModuleId>,
  module_graph: &ModuleGraph,
) -> farmfe_core::error::Result<()> {
  farm_profile_function!("strip module start");

  for module_id in topo_sorted_module_ids {
    farm_profile_scope!(format!("strip module: {}", module_id.to_string()));

    let module = module_graph.module(module_id).unwrap();
    let script_meta = module.meta.as_script();

    let mut stmt_action = HashSet::default();

    for statement in &script_meta.statements {
      // import
      if let Some(import) = statement.import_info.as_ref() {
        if script_meta.module_system.contains_commonjs() {
          let source_module_id =
            module_graph.get_dep_by_source(module_id, &import.source, Some(ResolveKind::Import));
          stmt_action.insert(StmtAction::StripCjsImport(
            statement.id,
            if import.specifiers.is_empty() {
              Some(source_module_id)
            } else {
              None
            },
          ));
        } else {
          stmt_action.insert(StmtAction::RemoveImport(statement.id));
        }
      }

      // export
      if let Some(export) = statement.export_info.as_ref() {
        if module_analyzer.is_commonjs() {
          continue;
        }

        if export.specifiers.is_empty() {
          stmt_action.insert(StmtAction::StripExport(statement.id));
          continue;
        }

        if export.source.is_some() {
          stmt_action.insert(StmtAction::StripExport(statement.id));
        } else {
          for specify in &export.specifiers {
            match specify {
              ExportSpecifierInfo::All(_) | ExportSpecifierInfo::Named { .. } => {
                stmt_action.insert(StmtAction::StripExport(statement.id));
              }

              ExportSpecifierInfo::Default(default) => {
                if self.bundle_variable.borrow().name(*default) == "default" {
                  stmt_action.insert(StmtAction::DeclDefaultExpr(statement.id, *default));
                } else {
                  stmt_action.insert(StmtAction::StripDefaultExport(statement.id, *default));
                }
              }

              ExportSpecifierInfo::Namespace(_) => {
                unreachable!("unsupported namespace have't source")
              }
            }
          }
        }
      }
    }

    if let Some(module_analyzer) = module_analyzer_manager.module_analyzer_mut(module_id) {
      module_analyzer.statement_actions.extend(stmt_action);
    }
  }

  Ok(())
}
