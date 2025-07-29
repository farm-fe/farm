use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, ImportSpecifierInfo, Statement, SwcId},
      ModuleExportIdentType, AMBIGUOUS_EXPORT_ALL, EXPORT_NAMESPACE, FARM_RUNTIME_MODULE_HELPER_ID,
    },
    module_graph::ModuleGraph,
    ModuleId,
  },
  plugin::ResolveKind,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    BinaryOp, Ident, ImportDecl, ImportDefaultSpecifier, ImportNamedSpecifier, ImportSpecifier,
    ImportStarAsSpecifier, ModuleDecl, ModuleExportName, ModuleItem,
  },
  HashSet,
};
use swc_ecma_visit::VisitMutWith;

use crate::script::{
  analyze_statement::{analyze_statement_info, AnalyzedStatementInfo},
  concatenate_modules::{
    strip_module_decl::{is_module_external, StripModuleDeclStatementParams},
    utils::{
      create_bin_expr, create_define_export_star_item, create_export_all_item,
      create_import_farm_define_export_helper_stmt, create_member_expr, create_var_decl_item,
      get_filename,
    },
  },
  create_ambiguous_export_all_ident,
};

use super::{
  strip_module_decl::{PreservedImportDeclItem, PreservedImportDeclType, StripModuleDeclResult},
  unique_idents::{RenameVisitor, TopLevelIdentsRenameHandler},
  utils::{create_import_external_namespace_stmt, replace_module_decl},
  StripModuleContext,
};

pub struct HandleExternalModuleOptions<'a> {
  pub module_id: &'a ModuleId,
  pub source_module_id: &'a ModuleId,
  pub is_entry_module: bool,
  pub statement: &'a Statement,
  pub result: &'a mut StripModuleDeclResult,
  pub strip_context: &'a mut StripModuleContext,
  pub module_graph: &'a ModuleGraph,
  pub module_ids: &'a HashSet<ModuleId>,
}

impl<'a> HandleExternalModuleOptions<'a> {
  pub fn from(
    strip_param: &'a mut StripModuleDeclStatementParams,
    source_module_id: &'a ModuleId,
    statement: &'a Statement,
  ) -> Self {
    Self {
      module_id: strip_param.module_id,
      source_module_id,
      is_entry_module: strip_param.is_entry_module,
      statement,
      result: strip_param.result,
      strip_context: strip_param.strip_context,
      module_graph: strip_param.module_graph,
      module_ids: strip_param.module_ids,
    }
  }
}

pub struct HandleExternalModuleResult {
  pub ambiguous_export_all_idents: Vec<SwcId>,
}

pub fn handle_external_module(options: HandleExternalModuleOptions) -> HandleExternalModuleResult {
  let mut result = HandleExternalModuleResult {
    ambiguous_export_all_idents: vec![],
  };

  if options.statement.import_info.is_some() {
    handle_external_import(options);
  } else if options.statement.export_info.is_some() {
    result
      .ambiguous_export_all_idents
      .extend(handle_external_export(options));
  }

  result
}

fn handle_external_import(options: HandleExternalModuleOptions) {
  let HandleExternalModuleOptions {
    module_id,
    source_module_id,
    statement,
    result,
    strip_context,
    ..
  } = options;
  let rename_handler = strip_context.rename_handler.clone();
  let mut rename_handler = rename_handler.borrow_mut();

  let item = replace_module_decl(statement, result);
  let is_namespace_import = is_namespace_import_stmt(statement);

  // if the external module has been imported, we should reuse the import statement to avoid duplicate imports
  if !is_namespace_import
    && let Some(preserved_item) = strip_context
      .preserved_import_decls
      .iter_mut()
      .find(|item| {
        !item.is_namespace_import
          && item.source_module_id == *source_module_id
          && item // make sure it's a import statement
            .import_item
            .as_module_decl()
            .and_then(|m| m.as_import())
            .is_some()
      })
  {
    for sp in &statement.import_info.as_ref().unwrap().specifiers {
      if let Some((existing_ident, sp_ident)) =
        get_imported_external_ident(&preserved_item.import_item, sp)
      {
        // rename the imported ident to the unique name
        let renamed_ident = rename_handler
          .get_renamed_ident(source_module_id, &existing_ident)
          .unwrap_or(existing_ident.clone());

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
  let module_decl = existing_import_decl.as_module_decl().unwrap();
  let import_decl = module_decl.as_import();

  import_decl.and_then(|import_decl| {
    import_decl
      .specifiers
      .iter()
      .find_map(|specifier| match specifier {
        ImportSpecifier::Named(import_named_specifier) => {
          let local_ident: SwcId = import_named_specifier.local.to_id().into();
          let local_imported_atom = import_named_specifier
            .imported
            .as_ref()
            .map(|i| i.atom())
            .unwrap_or(&local_ident.sym);

          if let ImportSpecifierInfo::Named { local, imported } = current_specifier {
            let imported_ident = imported.as_ref().unwrap_or(local);

            if *local_imported_atom == imported_ident.sym {
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
  })
}

fn find_or_create_preserved_import_item<'a>(
  strip_context: &'a mut StripModuleContext,
  module_id: &ModuleId,
  source_module_id: &ModuleId,
) -> &'a mut PreservedImportDeclItem {
  // check if the external module has been handled
  if let Some(index) = find_external_export_preserved_import(strip_context, source_module_id) {
    let result = strip_context.preserved_import_decls.get(index).unwrap();

    if result.is_namespace_import {
      return strip_context.preserved_import_decls.get_mut(index).unwrap();
    }
  }

  // create a unique ident for the external module
  let ident = {
    let mut rename_handler = strip_context.rename_handler.borrow_mut();
    create_unique_ambiguous_export_all_ident(module_id, source_module_id, &mut rename_handler)
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
      namespace_ident: Some(ident),
      is_namespace_import: true,
    });
  strip_context.preserved_import_decls.last_mut().unwrap()
}

fn create_unique_ambiguous_export_all_ident(
  module_id: &ModuleId,
  source_module_id: &ModuleId,
  rename_handler: &mut TopLevelIdentsRenameHandler,
) -> Ident {
  let ident = create_ambiguous_export_all_ident(source_module_id);
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
fn handle_external_export(options: HandleExternalModuleOptions) -> Vec<SwcId> {
  let HandleExternalModuleOptions {
    module_id,
    source_module_id,
    is_entry_module,
    statement,
    result,
    strip_context,
    module_graph,
    module_ids,
  } = options;

  let module = module_graph.module(module_id).unwrap();

  if module.external || !module.module_type.is_script() {
    return vec![];
  }

  let mut ambiguous_export_all_idents = vec![];

  let mut import_specifiers = vec![];
  let mut idents_to_rename = vec![];

  for sp in &statement.export_info.as_ref().unwrap().specifiers {
    match sp {
      ExportSpecifierInfo::All => {
        // for `export * from 'module';` where module is external module.
        if let Some(source) = statement
          .export_info
          .as_ref()
          .and_then(|i| i.source.as_ref())
        {
          ambiguous_export_all_idents.extend(handle_ambiguous_export_all(
            HandleAmbiguousExportAllOptions {
              module_id,
              is_entry_module,
              result,
              strip_context,
              module_graph,
              module_ids,
              source,
            },
          ));
        }
      }
      ExportSpecifierInfo::Default => unreachable!(),
      ExportSpecifierInfo::Named { local, exported } => {
        let local = local.clone();
        let exported = exported.clone();

        idents_to_rename.push((local.clone(), exported.clone().unwrap_or(local.clone())));

        // `export { foo, bar as baz } from 'module';`
        // =>
        // import { foo, bar as baz } from 'module';`
        let (import_local, imported) = if let Some(exported) = exported {
          (exported.into(), Some(local.into()))
        } else {
          (local.into(), None)
        };

        import_specifiers.push(ImportSpecifier::Named(ImportNamedSpecifier {
          local: import_local,
          imported,
          span: DUMMY_SP,
          is_type_only: false,
        }));
      }
      ExportSpecifierInfo::Namespace(ns) => {
        import_specifiers.push(ImportSpecifier::Namespace(ImportStarAsSpecifier {
          span: DUMMY_SP,
          local: Ident::new(ns.sym.clone(), DUMMY_SP, ns.ctxt()),
        }));

        break;
      }
    }
  }

  if !import_specifiers.is_empty() {
    let (info, import_item) = transform_export_stmt_to_import_stmt(statement, import_specifiers);
    result.ast.body[statement.id] = import_item;
    // reuse the preserved import decl
    handle_external_import(HandleExternalModuleOptions {
      module_id,
      source_module_id,
      is_entry_module,
      statement: &info.into(),
      result,
      strip_context,
      module_graph,
      module_ids,
    });

    let rename_handler = strip_context.rename_handler.clone();
    let mut rename_handler = rename_handler.borrow_mut();

    for (export_to_rename, import_to_get) in idents_to_rename {
      let renamed_import_ident = rename_handler
        .get_renamed_ident(module_id, &import_to_get)
        .unwrap_or(import_to_get.clone());

      if let Some(export_ident) = module
        .meta
        .as_script()
        .export_ident_map
        .get(import_to_get.sym.as_str())
      {
        let export_ident = export_ident.as_internal();
        rename_handler.rename_ident(
          export_ident.module_id.clone(),
          export_ident.ident.clone(),
          renamed_import_ident,
        );
      } else {
        rename_handler.rename_ident(
          source_module_id.clone(),
          export_to_rename,
          renamed_import_ident,
        );
      }
    }
  }

  ambiguous_export_all_idents
}

fn transform_export_stmt_to_import_stmt(
  statement: &Statement,
  specifiers: Vec<ImportSpecifier>,
) -> (AnalyzedStatementInfo, ModuleItem) {
  // transform `export * as ns from 'module';` to `import * as ns from 'module';`
  let import_item = ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
    span: DUMMY_SP,
    specifiers,
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

  (
    analyze_statement_info(&statement.id, &import_item),
    import_item,
  )
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

pub struct HandleAmbiguousExportAllOptions<'a> {
  pub module_id: &'a ModuleId,
  pub is_entry_module: bool,
  pub result: &'a mut StripModuleDeclResult,
  pub strip_context: &'a mut StripModuleContext,
  pub module_graph: &'a ModuleGraph,
  pub module_ids: &'a HashSet<ModuleId>,
  pub source: &'a str,
}

pub fn handle_ambiguous_export_all(options: HandleAmbiguousExportAllOptions) -> Vec<SwcId> {
  let HandleAmbiguousExportAllOptions {
    module_id,
    is_entry_module,
    result,
    strip_context,
    module_graph,
    module_ids,
    source,
  } = options;

  let mut ns_idents = vec![];

  let source_module_id =
    module_graph.get_dep_by_source(module_id, source, Some(ResolveKind::ExportFrom));

  let contains_ambiguous_export_ident = module_graph
    .module(module_id)
    .map(|module| {
      module
        .meta
        .as_script()
        .ambiguous_export_ident_map
        .iter()
        .filter(|(export_str, export_idents)| {
          *export_str != AMBIGUOUS_EXPORT_ALL
            && export_idents.iter().any(|export_ident| {
              // the ident is external or it's an ambiguous ident
              !module_ids.contains(&export_ident.as_internal().module_id)
                || matches!(
                  export_ident.as_internal().export_type,
                  ModuleExportIdentType::ExternalExportAll
                    | ModuleExportIdentType::UnresolvedExportAll
                )
            })
        })
        .count()
        > 0
    })
    .unwrap_or(false);
  let should_add_namespace_ident = strip_context.should_add_namespace_ident.contains(module_id);

  let source_module = module_graph.module(&source_module_id).unwrap();

  // if module and source_module are both in module ids
  if should_add_namespace_ident
    && module_ids.contains(module_id)
    && module_ids.contains(&source_module_id)
    && !source_module.external
    && source_module.module_type.is_script()
    && let Some(ns_ident) = source_module
      .meta
      .as_script()
      .export_ident_map
      .get(EXPORT_NAMESPACE)
  {
    let renamed_ident = strip_context
      .rename_handler
      .borrow()
      .get_renamed_ident(&source_module_id, &ns_ident.as_internal().ident)
      .unwrap_or(ns_ident.as_internal().ident.clone());

    // append defineExportStar
    result.items_to_append.push(create_define_export_star_item(
      &module_id,
      renamed_ident.clone().into(),
    ));

    ns_idents.push(renamed_ident)
  }
  // 1. add export star only namespace ident is enabled
  // 2. add import if there are ambiguous export all idents
  else if contains_ambiguous_export_ident || should_add_namespace_ident {
    if source_module.external || !source_module.module_type.is_script() {
      ns_idents.push(add_external_export_all_helper(
        strip_context,
        module_id,
        &source_module_id,
        result,
      ));
    } else {
      let source_module_meta = source_module.meta.as_script();

      if let Some(export_all_idents) = source_module_meta
        .ambiguous_export_ident_map
        .get(AMBIGUOUS_EXPORT_ALL)
      {
        for export_all_ident in export_all_idents {
          ns_idents.push(add_external_export_all_helper(
            strip_context,
            module_id,
            &export_all_ident.as_internal().module_id,
            result,
          ));
        }
      }
    };
  }

  // for entry module, we should add all export * from for all external module
  if is_entry_module {
    let source_module = module_graph.module(&source_module_id).unwrap();

    if !source_module.external && source_module.module_type.is_script() {
      let source_module_meta = source_module.meta.as_script();

      if let Some(export_all_idents) = source_module_meta
        .ambiguous_export_ident_map
        .get(AMBIGUOUS_EXPORT_ALL)
      {
        for export_all_ident in export_all_idents {
          let export_all_module = module_graph
            .module(&export_all_ident.as_internal().module_id)
            .unwrap();

          if is_module_external(export_all_module, module_ids) {
            // add `export * from 'xxx`
            strip_context
              .preserved_import_decls
              .push(PreservedImportDeclItem {
                import_item: create_export_all_item(&export_all_module.id),
                source_module_id: export_all_module.id.clone(),
                preserved_type: PreservedImportDeclType::ExternalGenerated,
                namespace_ident: None,
                is_namespace_import: false,
              });
          }
        }
      }
    }
  }

  ns_idents
}

fn add_external_export_all_helper(
  strip_context: &mut StripModuleContext,
  module_id: &ModuleId,
  source_module_id: &ModuleId,
  result: &mut StripModuleDeclResult,
) -> SwcId {
  let should_add_namespace_ident = strip_context.should_add_namespace_ident.contains(module_id);
  let preserved_helper_module_id: ModuleId = FARM_RUNTIME_MODULE_HELPER_ID.into();

  if should_add_namespace_ident
    && find_external_export_preserved_import(strip_context, &preserved_helper_module_id).is_none()
  {
    // add `import { defineExportStar } from '@farm-runtime/module-helper` to the top
    strip_context
      .preserved_import_decls
      .push(PreservedImportDeclItem {
        import_item: create_import_farm_define_export_helper_stmt(),
        source_module_id: preserved_helper_module_id,
        preserved_type: PreservedImportDeclType::ExternalGenerated,
        namespace_ident: None,
        is_namespace_import: false,
      });
  }

  // add import * xxx from the external module
  let preserved_item =
    find_or_create_preserved_import_item(strip_context, module_id, &source_module_id);

  // append defineExportStar
  if should_add_namespace_ident {
    result.items_to_append.push(create_define_export_star_item(
      &module_id,
      preserved_item.namespace_ident.clone().unwrap().into(),
    ));
  }

  preserved_item.namespace_ident.clone().unwrap().into()
}

pub fn add_ambiguous_ident_decl(
  module_graph: &ModuleGraph,
  module_id: &ModuleId,
  ns_idents: Vec<SwcId>,
  strip_context: &mut StripModuleContext,
) {
  if ns_idents.is_empty() {
    return;
  }

  // var export_str = xxxx.export_str
  let module = module_graph.module(module_id).unwrap();
  let module_meta = module.meta.as_script();
  let mut rename_handler = strip_context.rename_handler.borrow_mut();

  // sort export ident map to make sure the order is deterministic
  let mut exports = module_meta.export_ident_map.keys().collect::<Vec<_>>();
  exports.sort();

  for export_str in exports {
    let module_export_ident = module_meta.export_ident_map.get(export_str).unwrap();
    let module_export_ident = module_export_ident.as_internal();

    // The ident is not a normal esm export ident an it's not renamed yet. For example, other rename handler like external rename will rename it, we should not override it.
    if matches!(
      module_export_ident.export_type,
      ModuleExportIdentType::UnresolvedExportAll | ModuleExportIdentType::ExternalExportAll
    ) {
      let extra_var_decl = strip_context
        .extra_var_decls
        .iter_mut()
        .find(|(id, _, _)| *id == module_export_ident.ident);

      let filename = get_filename(&module_export_ident.module_id);
      let final_export_str = format!("{filename}_{export_str}");
      let export_ident = SwcId::from(final_export_str.as_str());

      let export_ident = extra_var_decl
        .as_ref()
        .and_then(|(_, export_ident, _)| Some(export_ident.clone()))
        .or_else(|| rename_handler.get_unique_ident(&export_ident))
        .unwrap_or(export_ident);

      let decl_item = create_var_decl_item(
        export_ident.clone().into(),
        // xxx1.export_str || xxx2.export_str
        Box::new(create_bin_expr(
          ns_idents
            .iter()
            .map(|ns_ident| create_member_expr(ns_ident, export_str.as_str()))
            .collect(),
          BinaryOp::LogicalOr,
        )),
      );

      // use use the outer decl item if there are multiple export * in the chain
      if let Some(extra_var_decl) = extra_var_decl {
        extra_var_decl.2 = decl_item;
      } else {
        rename_handler.rename_ident(
          module_export_ident.module_id.clone(),
          module_export_ident.ident.clone(),
          export_ident.clone(),
        );
        strip_context.extra_var_decls.push((
          module_export_ident.ident.clone(),
          export_ident,
          decl_item,
        ));
      }
    }
  }
}
