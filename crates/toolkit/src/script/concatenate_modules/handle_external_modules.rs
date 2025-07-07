use farmfe_core::{
  module::{
    meta_data::script::statement::{ExportSpecifierInfo, ImportSpecifierInfo, Statement, SwcId},
    ModuleId,
  },
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    Expr, Ident, IdentName, ImportDecl, ImportDefaultSpecifier, ImportNamedSpecifier,
    ImportSpecifier, ImportStarAsSpecifier, MemberExpr, MemberProp, ModuleDecl, ModuleExportName,
    ModuleItem,
  },
  HashSet,
};
use swc_ecma_visit::VisitMutWith;

use crate::script::analyze_statement::analyze_statement_info;

use super::{
  strip_module_decl::{PreservedImportDeclItem, PreservedImportDeclType, StripModuleDeclResult},
  unique_idents::{RenameVisitor, TopLevelIdentsRenameHandler},
  utils::{
    create_export_external_all_ident, create_import_external_namespace_stmt, create_var_decl_item,
    replace_module_decl,
  },
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
    handle_external_export(
      module_id,
      source_module_id,
      statement,
      result,
      strip_context,
    );
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
  let is_namespace_import = is_namespace_import_stmt(statement);

  // if the external module has been imported, we should reuse the import statement to avoid duplicate imports
  if !is_namespace_import
    && let Some(preserved_item) = strip_context
      .preserved_import_decls
      .iter_mut()
      .find(|item| !item.is_namespace_import && item.source_module_id == *source_module_id)
  {
    for sp in &statement.import_info.as_ref().unwrap().specifiers {
      if let Some((existing_ident, sp_ident)) =
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
  } else if let Some(preserved_item) = strip_context
    .preserved_import_decls
    .iter_mut()
    .find(|item| item.is_namespace_import && item.source_module_id == *source_module_id)
  {
    for sp in &statement.import_info.as_ref().unwrap().specifiers {
      if let ImportSpecifierInfo::Namespace(swc_id) = sp {
        rename_handler.rename_ident(
          module_id.clone(),
          swc_id.clone(),
          preserved_item
            .namespace_ident
            .as_ref()
            .unwrap()
            .to_id()
            .into(),
        );
      } else {
        // the import decl is not a namespace import, append the new import decl
        push_new_preserved_import(
          module_id,
          source_module_id,
          statement,
          item,
          strip_context,
          &mut rename_handler,
        );
        break;
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
  let mut renamed_ident = None;
  // rename the imported ident if there are conflicts
  for defined_ident in &statement.defined_idents {
    rename_handler.rename_ident_if_conflict(module_id, defined_ident);
    renamed_ident = rename_handler
      .get_renamed_ident(module_id, defined_ident)
      .or(Some(defined_ident.clone()));
  }

  let mut rename_visitor = RenameVisitor::new(module_id, &rename_handler);
  item.visit_mut_with(&mut rename_visitor);

  let is_namespace_import = is_namespace_import_stmt(statement);

  // preserve the import statement, e.g. `import { createRequire } from 'module';`
  strip_context
    .preserved_import_decls
    .push(PreservedImportDeclItem {
      import_item: item,
      source_module_id: source_module_id.clone(),
      preserved_type: PreservedImportDeclType::ExternalOriginal,
      used_idents: HashSet::default(),
      namespace_ident: if is_namespace_import {
        renamed_ident.map(|i| {
          let ctxt = i.ctxt();
          Ident::new(i.sym, DUMMY_SP, ctxt)
        })
      } else {
        None
      },
      is_namespace_import,
    });
}

fn is_namespace_import_stmt(statement: &Statement) -> bool {
  if let Some(import_info) = statement.import_info.as_ref() {
    import_info
      .specifiers
      .iter()
      .any(|sp| matches!(sp, ImportSpecifierInfo::Namespace(_)))
  } else {
    false
  }
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

pub fn find_or_create_preserved_import_item<'a>(
  strip_context: &'a mut StripModuleContext,
  module_id: &ModuleId,
  source_module_id: &ModuleId,
) -> &'a mut PreservedImportDeclItem {
  // check if the external module has been handled
  if let Some(index) = find_external_export_preserved_import(strip_context, source_module_id) {
    strip_context.preserved_import_decls.get_mut(index).unwrap()
  } else {
    // create a unique ident for the external module
    let ident = {
      let mut rename_handler = strip_context.rename_handler.borrow_mut();
      create_unique_external_namespace_ident(module_id, source_module_id, &mut rename_handler)
    };
    // insert `import * as external_all_farm_internal_ from 'module';`
    let import_item = create_import_external_namespace_stmt(ident.clone(), source_module_id);

    // insert the generated import statement to the preserved import decls
    strip_context
      .preserved_import_decls
      .push(PreservedImportDeclItem {
        import_item,
        source_module_id: source_module_id.clone(),
        preserved_type: PreservedImportDeclType::ExternalGenerated,
        used_idents: HashSet::default(),
        namespace_ident: Some(ident),
        is_namespace_import: true,
      });
    strip_context.preserved_import_decls.last_mut().unwrap()
  }
}

pub fn create_unique_external_namespace_ident(
  module_id: &ModuleId,
  source_module_id: &ModuleId,
  rename_handler: &mut TopLevelIdentsRenameHandler,
) -> Ident {
  let ident = create_export_external_all_ident(source_module_id);
  // rename the imported ident to the unique name
  rename_handler.rename_ident_if_conflict(source_module_id, &ident.to_id().into());
  let renamed_ident = rename_handler.get_renamed_ident(source_module_id, &ident.to_id().into());

  if let Some(renamed_ident) = &renamed_ident {
    rename_handler.rename_ident(
      module_id.clone(),
      ident.to_id().into(),
      renamed_ident.clone(),
    );
  }

  let ident = renamed_ident
    .map(|i| Ident::new(i.sym.clone(), DUMMY_SP, i.ctxt()))
    .unwrap_or(ident);

  ident
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
  result: &mut StripModuleDeclResult,
  strip_context: &mut StripModuleContext,
) {
  let mut extra_items = vec![];
  let rename_handler = strip_context.rename_handler.clone();
  let mut cached_preserved_item = None;

  for sp in &statement.export_info.as_ref().unwrap().specifiers {
    match sp {
      ExportSpecifierInfo::All => {
        // for `export * from 'module';`. Just remove the statement, it will be handled by when tracing the imports.
      }
      ExportSpecifierInfo::Default => unreachable!(),
      ExportSpecifierInfo::Named { local, exported } => {
        let preserved_item = if let Some(preserved_item) = cached_preserved_item.as_mut() {
          preserved_item
        } else {
          cached_preserved_item = Some(find_or_create_preserved_import_item(
            strip_context,
            module_id,
            source_module_id,
          ));
          cached_preserved_item.as_mut().unwrap()
        };

        // check if the external module has been handled
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

        let mut rename_handler = rename_handler.borrow_mut();

        // rename the imported ident to a unique name
        rename_handler.rename_ident_if_conflict(source_module_id, defined_ident);
        let renamed_ident = if let Some(renamed_ident) =
          rename_handler.get_renamed_ident(source_module_id, defined_ident)
        {
          rename_handler.rename_ident(
            module_id.clone(),
            defined_ident.clone(),
            renamed_ident.clone(),
          );
          renamed_ident
        } else {
          defined_ident.clone()
        };

        // rename local to the unique name
        rename_handler.rename_ident(
          source_module_id.clone(),
          local.clone(),
          renamed_ident.clone(),
        );

        let var_decl_item = create_var_decl_item(
          Ident::new(renamed_ident.sym.clone(), DUMMY_SP, renamed_ident.ctxt()),
          Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(
              preserved_item.namespace_ident.as_ref().unwrap().clone(),
            )),
            prop: MemberProp::Ident(IdentName {
              span: DUMMY_SP,
              sym: local.sym.clone(),
            }),
          })),
        );

        preserved_item.used_idents.insert(local.clone());

        extra_items.push(var_decl_item);
      }
      ExportSpecifierInfo::Namespace(ns) => {
        // transform `export * as ns from 'module';` to `import * as ns from 'module';`
        let import_item = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
          span: DUMMY_SP,
          specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
            span: DUMMY_SP,
            local: Ident::new(ns.sym.clone(), DUMMY_SP, ns.ctxt()),
          })],
          src: Box::new(
            statement
              .export_info
              .as_ref()
              .unwrap()
              .source
              .as_ref()
              .unwrap()
              .as_str()
              .into(),
          ),
          type_only: false,
          with: None,
          phase: Default::default(),
        }));

        let info = analyze_statement_info(&statement.id, &import_item);
        result.ast.body[statement.id] = import_item;

        handle_external_import(
          module_id,
          source_module_id,
          &info.into(),
          result,
          strip_context,
        );

        break;
      }
    }
  }

  strip_context
    .extra_external_module_items
    .extend(extra_items);
}

fn find_external_export_preserved_import(
  strip_context: &StripModuleContext,
  source_module_id: &ModuleId,
) -> Option<usize> {
  strip_context
    .preserved_import_decls
    .iter()
    .position(|item| {
      item.source_module_id == *source_module_id
        && matches!(
          item.preserved_type,
          PreservedImportDeclType::ExternalGenerated
        )
    })
}
