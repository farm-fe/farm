use farmfe_core::{
  module::{
    meta_data::script::statement::{ExportSpecifierInfo, ImportSpecifierInfo, Statement, SwcId},
    ModuleId,
  },
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    BindingIdent, Decl, Expr, Ident, IdentName, ImportDecl, ImportDefaultSpecifier,
    ImportNamedSpecifier, ImportSpecifier, ImportStarAsSpecifier, MemberExpr, MemberProp,
    ModuleDecl, ModuleExportName, ModuleItem, Pat, Stmt, Str, VarDecl, VarDeclKind, VarDeclarator,
  },
  HashSet,
};
use swc_ecma_visit::VisitMutWith;

use super::{
  strip_module_decl::{PreservedImportDeclItem, PreservedImportDeclType, StripModuleDeclResult},
  unique_idents::{RenameVisitor, TopLevelIdentsRenameHandler},
  utils::{create_export_external_namespace_ident, replace_module_decl},
  StripModuleContext,
};

pub fn handle_external_modules(
  module_id: &ModuleId,
  source_module_id: &ModuleId,
  statement: &Statement,
  result: &mut StripModuleDeclResult,
  strip_context: &mut StripModuleContext,
) {
  if statement.import_info.is_some() {
    handle_external_import(
      module_id,
      source_module_id,
      statement,
      result,
      strip_context,
    );
  } else if statement.export_info.is_some() {
    handle_external_export(module_id, source_module_id, statement, strip_context);
  }
}

fn handle_external_import(
  module_id: &ModuleId,
  source_module_id: &ModuleId,
  statement: &Statement,
  result: &mut StripModuleDeclResult,
  strip_context: &mut StripModuleContext,
) {
  let rename_handler = strip_context.rename_handler.clone();
  let mut rename_handler = rename_handler.borrow_mut();

  let item = replace_module_decl(statement, result);

  // if the external module has been imported, we should reuse the import statement to avoid duplicate imports
  if let Some(preserved_item) = strip_context
    .preserved_import_decls
    .iter_mut()
    .find(|item| !item.is_namespace_import && item.source_module_id == *source_module_id)
  {
    for sp in &statement.import_info.as_ref().unwrap().specifiers {
      if matches!(sp, ImportSpecifierInfo::Namespace(_)) {
        push_new_preserved_import(
          module_id,
          source_module_id,
          statement,
          item,
          strip_context,
          &mut rename_handler,
        );
        break;
      } else if let Some((existing_ident, sp_ident)) =
        get_imported_external_ident(&preserved_item.import_item, sp)
      {
        // rename the imported ident to the unique name
        let renamed_ident = rename_handler
          .get_renamed_ident(source_module_id, &existing_ident)
          .unwrap_or(existing_ident);
        rename_handler.rename_ident(module_id.clone(), sp_ident, renamed_ident.clone());
      } else {
        // push the ident to existing import decl and rename it
        let import_decl = preserved_item
          .import_item
          .as_mut_module_decl()
          .unwrap()
          .as_mut_import()
          .unwrap();
        let new_sp = match sp {
          ImportSpecifierInfo::Namespace(_) => {
            unreachable!("Namespace should be in a separate import")
          }
          ImportSpecifierInfo::Named { local, imported } => {
            rename_handler.rename_ident_if_conflict(module_id, local);
            let renamed_ident = rename_handler.get_renamed_ident(module_id, local);

            if let Some(renamed_ident) = renamed_ident {
              ImportSpecifier::Named(ImportNamedSpecifier {
                span: DUMMY_SP,
                local: Ident::new(renamed_ident.sym.clone(), DUMMY_SP, renamed_ident.ctxt()),
                imported: imported
                  .as_ref()
                  .map(|i| ModuleExportName::Ident(Ident::new(i.sym.clone(), DUMMY_SP, i.ctxt())))
                  .or(Some(ModuleExportName::Ident(Ident::new(
                    local.sym.clone(),
                    DUMMY_SP,
                    local.ctxt(),
                  )))),
                is_type_only: false,
              })
            } else {
              ImportSpecifier::Named(ImportNamedSpecifier {
                span: DUMMY_SP,
                local: Ident::new(local.sym.clone(), DUMMY_SP, local.ctxt()),
                imported: imported
                  .as_ref()
                  .map(|i| ModuleExportName::Ident(Ident::new(i.sym.clone(), DUMMY_SP, i.ctxt()))),
                is_type_only: false,
              })
            }
          }
          ImportSpecifierInfo::Default(swc_id) => {
            rename_handler.rename_ident_if_conflict(module_id, swc_id);
            let renamed_ident = rename_handler
              .get_renamed_ident(module_id, swc_id)
              .unwrap_or(swc_id.clone());
            let local = Ident::new(renamed_ident.sym.clone(), DUMMY_SP, renamed_ident.ctxt());
            ImportSpecifier::Default(ImportDefaultSpecifier {
              span: DUMMY_SP,
              local,
            })
          }
        };
        import_decl.specifiers.push(new_sp);
      }
    }
  } else {
    push_new_preserved_import(
      module_id,
      source_module_id,
      statement,
      item,
      strip_context,
      &mut rename_handler,
    );
  }
}

fn push_new_preserved_import(
  module_id: &ModuleId,
  source_module_id: &ModuleId,
  statement: &Statement,
  mut item: ModuleItem,
  strip_context: &mut StripModuleContext,
  rename_handler: &mut TopLevelIdentsRenameHandler,
) {
  // rename the imported ident if there are conflicts
  for defined_ident in &statement.defined_idents {
    rename_handler.rename_ident_if_conflict(module_id, defined_ident);
  }

  let mut rename_visitor = RenameVisitor::new(module_id, &rename_handler);
  item.visit_mut_with(&mut rename_visitor);

  let is_namespace_import = if let Some(import_info) = statement.import_info.as_ref() {
    import_info
      .specifiers
      .iter()
      .any(|sp| matches!(sp, ImportSpecifierInfo::Namespace(_)))
  } else {
    false
  };

  // preserve the import statement, e.g. `import { createRequire } from 'module';`
  strip_context
    .preserved_import_decls
    .push(PreservedImportDeclItem {
      import_item: item,
      source_module_id: source_module_id.clone(),
      preserved_type: PreservedImportDeclType::ExternalOriginal,
      used_idents: HashSet::default(),
      is_namespace_import,
    });
}

fn get_imported_external_ident(
  existing_import_decl: &ModuleItem,
  current_specifier: &ImportSpecifierInfo,
) -> Option<(SwcId, SwcId)> {
  existing_import_decl
    .as_module_decl()
    .unwrap()
    .as_import()
    .unwrap()
    .specifiers
    .iter()
    .find_map(|specifier| match specifier {
      ImportSpecifier::Named(import_named_specifier) => {
        let local_ident: SwcId = import_named_specifier.local.to_id().into();

        if let ImportSpecifierInfo::Named { local, .. } = current_specifier {
          if local_ident.sym == local.sym {
            return Some((local_ident, local.clone()));
          }
        }

        None
      }
      ImportSpecifier::Default(import_default_specifier) => {
        let local_ident: SwcId = import_default_specifier.local.to_id().into();

        if let ImportSpecifierInfo::Default(default_ident) = current_specifier {
          Some((local_ident, default_ident.clone()))
        } else {
          None
        }
      }
      ImportSpecifier::Namespace(import_star_as_specifier) => {
        let local_ident: SwcId = import_star_as_specifier.local.to_id().into();

        if let ImportSpecifierInfo::Namespace(ns_ident) = current_specifier {
          Some((local_ident, ns_ident.clone()))
        } else {
          None
        }
      }
    })
}

/// Handle external export statements. Following is the example:
/// in `foo.js`:
/// ```js
/// export * from 'module'; // `module` is a external module
/// export const foo = 'foo';
/// ```
/// in `index.js`:
/// ```js
/// import { foo, baz, zoo } from './foo';
/// console.log(foo, baz, zoo);
/// ```
/// =>
///
/// in `module`:
/// ```js
/// import * as external_all_farm_internal_ from 'module';
/// var zoo = external_all_farm_internal_.zoo;
/// var bar = external_all_farm_internal_.bar;
/// export { zoo, bar as baz }
/// ```
/// full bundle:
/// ```js
/// // module
/// import * as external_all_farm_internal_ from 'module';
/// var zoo = external_all_farm_internal_.zoo;
/// var bar = external_all_farm_internal_.bar;
///
/// // foo.js
/// const foo = 'foo';
///
/// // index.js
/// console.log(foo, baz, zoo);
/// ```
fn handle_external_export(
  module_id: &ModuleId,
  source_module_id: &ModuleId,
  statement: &Statement,
  strip_context: &mut StripModuleContext,
) {
  let rename_handler = strip_context.rename_handler.clone();
  let mut rename_handler = rename_handler.borrow_mut();

  // create a unique ident for the external module
  let ident = {
    let ident = create_export_external_namespace_ident(source_module_id);
    // rename the imported ident to the unique name
    rename_handler.rename_ident_if_conflict(source_module_id, &ident.to_id().into());
    let renamed_ident = rename_handler.get_renamed_ident(source_module_id, &ident.to_id().into());

    renamed_ident
      .map(|i| Ident::new(i.sym.clone(), DUMMY_SP, i.ctxt()))
      .unwrap_or(ident)
  };

  // check if the external module has been handled
  let preserved_item =
    if let Some(item) = find_external_export_preserved_import(strip_context, source_module_id) {
      item
    } else {
      // insert `import * as external_all_farm_internal_ from 'module';`
      let import_decl = ImportDecl {
        span: DUMMY_SP,
        specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
          span: DUMMY_SP,
          local: ident.clone(),
        })],
        src: Box::new(Str {
          span: DUMMY_SP,
          value: source_module_id.to_string().into(),
          raw: None,
        }),
        type_only: false,
        with: None,
        phase: Default::default(),
      };

      // insert the generated import statement to the preserved import decls
      strip_context
        .preserved_import_decls
        .push(PreservedImportDeclItem {
          import_item: ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)),
          source_module_id: source_module_id.clone(),
          preserved_type: PreservedImportDeclType::ExternalGenerated,
          used_idents: HashSet::default(),
          is_namespace_import: true,
        });
      strip_context.preserved_import_decls.last_mut().unwrap()
    };

  let mut extra_items = vec![];

  for sp in &statement.export_info.as_ref().unwrap().specifiers {
    match sp {
      ExportSpecifierInfo::All => {
        // for `export * from 'module';`. Just remove the statement, it will be handled by when tracing the imports.
      }
      ExportSpecifierInfo::Default => unreachable!(),
      ExportSpecifierInfo::Named { local, exported } => {
        if preserved_item.used_idents.contains(local) {
          continue;
        }

        // for `export { foo, bar as baz } from 'module';`. Rename the exported ident to the unique name.
        // var foo = external_all_farm_internal_.foo;
        // var baz = external_all_farm_internal_.bar;
        let defined_ident = if let Some(exported) = exported {
          exported
        } else {
          local
        };

        // println!("defined_ident: {defined_ident:?}");
        // rename the imported ident to a unique name
        rename_handler.rename_ident_if_conflict(source_module_id, defined_ident);
        let defined_ident = rename_handler
          .get_renamed_ident(source_module_id, defined_ident)
          .unwrap_or(defined_ident.clone());
        // println!("defined_ident new: {defined_ident:?}");

        let var_decl = VarDecl {
          span: DUMMY_SP,
          ctxt: SyntaxContext::empty(),
          kind: VarDeclKind::Var,
          declare: false,
          decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(BindingIdent {
              id: Ident::new(defined_ident.sym.clone(), DUMMY_SP, defined_ident.ctxt()),
              type_ann: None,
            }),
            init: Some(Box::new(Expr::Member(MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::Ident(ident.clone())),
              prop: MemberProp::Ident(IdentName {
                span: DUMMY_SP,
                sym: local.sym.clone(),
              }),
            }))),
            definite: false,
          }],
        };

        preserved_item.used_idents.insert(local.clone());

        let var_decl_item = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(var_decl))));
        extra_items.push(var_decl_item);
      }
      ExportSpecifierInfo::Namespace(ns) => {
        // for `export * as ns from 'module';`. Rename ns to a unique name.
        rename_handler.rename_ident(module_id.clone(), ns.clone(), ident.to_id().into());
      }
    }
  }

  strip_context
    .extra_external_module_items
    .extend(extra_items);
}

fn find_external_export_preserved_import<'a>(
  strip_context: &'a mut StripModuleContext,
  source_module_id: &ModuleId,
) -> Option<&'a mut PreservedImportDeclItem> {
  strip_context
    .preserved_import_decls
    .iter_mut()
    .find(|item| {
      item.source_module_id == *source_module_id
        && matches!(
          item.preserved_type,
          PreservedImportDeclType::ExternalGenerated
        )
    })
}
