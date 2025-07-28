use crate::utils::{get_reexport_named_local, is_reexport_all};
use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, ImportSpecifierInfo},
      ModuleExportIdent, ModuleExportIdentType, AMBIGUOUS_EXPORT_ALL, EXPORT_DEFAULT,
    },
    Module, ModuleId, ModuleSystem,
  },
  parking_lot::Mutex,
  plugin::ResolveKind,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    EmptyStmt, ExportNamedSpecifier, ExportSpecifier, Ident, ModuleDecl, ModuleExportName,
    ModuleItem, NamedExport, Stmt,
  },
  HashMap, HashSet,
};
use farmfe_toolkit::script::{
  analyze_statement::analyze_statements, concatenate_modules::EXPORT_NAMESPACE,
};

pub fn mangle_exports(
  can_not_be_mangled: &HashSet<ModuleExportIdent>,
  module_graph: &farmfe_core::module::module_graph::ModuleGraph,
) -> HashMap<ModuleId, HashMap<String, String>> {
  let mangled_ident_map: Mutex<HashMap<ModuleId, HashMap<String, String>>> =
    Mutex::new(HashMap::default());
  let exclude_exports = [EXPORT_DEFAULT, AMBIGUOUS_EXPORT_ALL, EXPORT_NAMESPACE];
  let generator_map: Mutex<HashMap<ModuleId, crate::ident_generator::MinifiedIdentsGenerator>> =
    Mutex::new(HashMap::default());

  let should_not_mangle_export =
    |module: &Module, export_ident: &ModuleExportIdent, export: &str| {
      let internal_ident = export_ident.as_internal();
      export.len() <= 2
        || module.is_entry
        || module.is_dynamic_entry
        || exclude_exports.contains(&export)
        || can_not_be_mangled.contains(export_ident)
        || !matches!(
          internal_ident.export_type,
          ModuleExportIdentType::Declaration
        )
    };

  // mangle exports of module_graph in parallel
  module_graph.modules().into_par_iter().for_each(|module| {
    if module.module_type.is_script() {
      let meta = module.meta.as_script();

      let mut ident_generator =
        crate::ident_generator::MinifiedIdentsGenerator::new(HashSet::default());

      // add all top level idents and all deeply declared idents and unresolved idents to ident_generator
      for ident in meta.top_level_idents.iter() {
        ident_generator.add_used_ident(&ident.sym);
      }
      for ident in meta.all_deeply_declared_idents.iter() {
        ident_generator.add_used_ident(&ident);
      }
      for ident in meta.unresolved_idents.iter() {
        ident_generator.add_used_ident(&ident.sym);
      }

      let mut module_mangled_ident_map: HashMap<String, String> = HashMap::default();

      let mut exports = meta.export_ident_map.keys().collect::<Vec<_>>();
      exports.sort();

      // mangle exports
      for export in exports {
        let ident = meta.export_ident_map.get(export).unwrap();

        // only mangle exports defined in current module
        if module.id != ident.as_internal().module_id
          || should_not_mangle_export(module, ident, export)
        {
          continue;
        }

        let mangled_ident = ident_generator.generate();

        module_mangled_ident_map.insert(export.clone(), mangled_ident);
      }

      mangled_ident_map
        .lock()
        .entry(module.id.clone())
        .or_default()
        .extend(module_mangled_ident_map);
      generator_map
        .lock()
        .insert(module.id.clone(), ident_generator);
    }
  });

  let mut mangled_ident_map = mangled_ident_map.into_inner();
  let new_mangled_ident_map: Mutex<HashMap<ModuleId, HashMap<String, String>>> =
    Mutex::new(HashMap::default());

  // update parent module's mangled_ident_map
  module_graph.modules().into_par_iter().for_each(|module| {
    if module.module_type.is_script() {
      let meta = module.meta.as_script();
      let mut ident_generator = generator_map.lock().remove(&module.id).unwrap();

      let mut module_mangled_ident_map: HashMap<String, String> = HashMap::default();

      // mangle exports
      for (export, export_ident) in meta.export_ident_map.iter() {
        let ident = export_ident.as_internal();
        // only mangle exports defined in current module
        if module.id != ident.module_id && !should_not_mangle_export(module, export_ident, export) {
          if is_reexport_all(&meta.reexport_ident_map, export) {
            // for export * from, to avoid name conflict of reexported mangled ident, we should transform export * from to export { xxx }
            let mangled_ident = ident_generator.generate();
            module_mangled_ident_map.insert(export.clone(), mangled_ident.clone());
          } else if get_reexport_named_local(&meta.reexport_ident_map, export).is_some() {
            // should not minify default export
            if export != EXPORT_DEFAULT {
              let mangled_ident = ident_generator.generate();
              module_mangled_ident_map.insert(export.clone(), mangled_ident.clone());
            }
          }
        }
      }

      new_mangled_ident_map
        .lock()
        .entry(module.id.clone())
        .or_default()
        .extend(module_mangled_ident_map);
    }
  });

  // merge new_mangled_ident_map and mangled_ident_map
  new_mangled_ident_map.into_inner().into_iter().for_each(
    |(module_id, module_mangled_ident_map)| {
      if let Some(mangled_ident_map) = mangled_ident_map.get_mut(&module_id) {
        mangled_ident_map.extend(module_mangled_ident_map);
      } else {
        mangled_ident_map.insert(module_id, module_mangled_ident_map);
      }
    },
  );

  mangled_ident_map
}

pub fn find_imports_to_rename(
  full_mangled_ident_map: &HashMap<ModuleId, HashMap<String, String>>,
  module_graph: &farmfe_core::module::module_graph::ModuleGraph,
) -> HashMap<ModuleId, HashMap<(String, String), String>> {
  let imports_to_rename: Mutex<HashMap<ModuleId, HashMap<(String, String), String>>> =
    Mutex::new(HashMap::default());

  module_graph.modules().into_par_iter().for_each(|module| {
    if module.module_type.is_script() {
      let meta = module.meta.as_script();
      let mut module_imports_to_rename = HashMap::default();

      for stmt in meta.statements.iter() {
        if let Some(import_info) = &stmt.import_info {
          let source_module_id = module_graph.get_dep_by_source(
            &module.id,
            &import_info.source,
            Some(ResolveKind::Import),
          );
          let source_module = module_graph.module(&source_module_id).unwrap();

          if source_module.external || !source_module.module_type.is_script() {
            continue;
          }

          if let Some(mangled_ident_map) = full_mangled_ident_map.get(&source_module_id) {
            for specifier in &import_info.specifiers {
              match specifier {
                ImportSpecifierInfo::Named { local, imported } => {
                  let imported_str = if let Some(imported) = imported {
                    &imported.sym
                  } else {
                    &local.sym
                  };

                  if let Some(mangled_ident) = mangled_ident_map.get(imported_str.as_str()) {
                    module_imports_to_rename.insert(
                      (imported_str.to_string(), import_info.source.clone()),
                      mangled_ident.clone(),
                    );
                  }
                }
                _ => {
                  // do nothing
                }
              }
            }
          }
        } else if let Some(export_info) = &stmt.export_info {
          if let Some(source) = &export_info.source {
            let source_module_id =
              module_graph.get_dep_by_source(&module.id, source, Some(ResolveKind::ExportFrom));
            let source_module = module_graph.module(&source_module_id).unwrap();

            if source_module.external || !source_module.module_type.is_script() {
              continue;
            }

            if let Some(mangled_ident_map) = full_mangled_ident_map.get(&source_module_id) {
              for specifier in &export_info.specifiers {
                match specifier {
                  ExportSpecifierInfo::Named { local, .. } => {
                    if let Some(mangled_ident) = mangled_ident_map.get(local.sym.as_str()) {
                      module_imports_to_rename.insert(
                        (local.sym.to_string(), source.clone()),
                        mangled_ident.clone(),
                      );
                    }
                  }
                  ExportSpecifierInfo::All => {
                    for (export, _) in &source_module.meta.as_script().export_ident_map {
                      if let Some(mangled_ident) = mangled_ident_map.get(export) {
                        module_imports_to_rename
                          .insert((export.to_string(), source.clone()), mangled_ident.clone());
                      }
                    }
                  }

                  _ => {
                    // do nothing
                  }
                }
              }
            }
          }
        }
      }

      imports_to_rename
        .lock()
        .entry(module.id.clone())
        .or_default()
        .extend(module_imports_to_rename);
    }
  });

  imports_to_rename.into_inner()
}

pub fn update_exports_meta_and_module_decl(
  full_mangled_ident_map: &HashMap<ModuleId, HashMap<String, String>>,
  full_imports_to_rename: &HashMap<ModuleId, HashMap<(String, String), String>>,
  module_graph: &mut farmfe_core::module::module_graph::ModuleGraph,
) {
  macro_rules! get_mangled_ident_map {
    ($ident_map:expr, $mangled_ident_map:expr) => {{
      let mut export_ident_map = HashMap::default();

      for (export, ident) in $ident_map.drain() {
        if let Some(mangled_ident) = $mangled_ident_map.get(&export) {
          export_ident_map.insert(mangled_ident.clone(), ident);
        } else {
          export_ident_map.insert(export, ident);
        }
      }

      export_ident_map
    }};
  }
  module_graph
    .modules_mut()
    .into_par_iter()
    .for_each(|module| {
      if module.module_type.is_script() {
        let default_map = HashMap::default();
        let mangled_ident_map = full_mangled_ident_map
          .get(&module.id)
          .unwrap_or_else(|| &default_map);

        let import_default_map = HashMap::default();
        let imports_to_rename = full_imports_to_rename
          .get(&module.id)
          .unwrap_or_else(|| &import_default_map);
        let meta = module.meta.as_script_mut();

        // remove original exports and insert mangled exports

        meta.export_ident_map = get_mangled_ident_map!(meta.export_ident_map, mangled_ident_map);
        meta.reexport_ident_map =
          get_mangled_ident_map!(meta.reexport_ident_map, mangled_ident_map);
        meta.ambiguous_export_ident_map =
          get_mangled_ident_map!(meta.ambiguous_export_ident_map, mangled_ident_map);

        let mut extra_export_specifiers = vec![];

        // exports
        // ```
        // 1. case 1
        // export const hello = 1;
        // =>
        // const hello = 1;
        // export { hello as a };
        //
        // 2. case 2
        // export { hello as hello1 } from './hello';
        // export { world };
        // =>
        // export { a as b } from './hello';
        // export { world as c };
        //
        // 3. case 3
        // import { hello as hello1 } from './hello';
        // =>
        // import { a as hello1 } from './hello';
        // ```
        for (i, item) in meta.ast.body.iter_mut().enumerate() {
          if let Some(module_decl) = item.as_mut_module_decl() {
            match module_decl {
              farmfe_core::swc_ecma_ast::ModuleDecl::ExportDecl(_) => {
                let stmt = &meta.statements[i].clone();

                for defined_ident in stmt.defined_idents.iter() {
                  extra_export_specifiers.push(ExportSpecifier::Named(ExportNamedSpecifier {
                    span: DUMMY_SP,
                    is_type_only: false,
                    orig: ModuleExportName::Ident(Ident::new(
                      defined_ident.sym.clone(),
                      DUMMY_SP,
                      defined_ident.ctxt(),
                    )),
                    exported: mangled_ident_map.get(defined_ident.sym.as_str()).map(
                      |mangled_ident| {
                        ModuleExportName::Ident(Ident::new(
                          mangled_ident.as_str().into(),
                          DUMMY_SP,
                          defined_ident.ctxt(),
                        ))
                      },
                    ),
                  }));
                }

                let mut empty = ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
                std::mem::swap(&mut empty, item);
                let decl = empty.expect_module_decl().expect_export_decl();
                *item = ModuleItem::Stmt(Stmt::Decl(decl.decl));
              }
              farmfe_core::swc_ecma_ast::ModuleDecl::ExportNamed(NamedExport {
                specifiers,
                src,
                ..
              }) => {
                for specifier in specifiers.iter_mut() {
                  match specifier {
                    ExportSpecifier::Named(named) => {
                      let exported_str = named
                        .exported
                        .as_ref()
                        .unwrap_or(&named.orig)
                        .atom()
                        .to_string();

                      if src.is_some() {
                        // rename export { hello as hello1 } from './hello'; to export { a as hello1 } from './hello';
                        if let Some(mangled_ident) = imports_to_rename.get(&(
                          named.orig.atom().to_string(),
                          src.as_ref().unwrap().value.to_string(),
                        )) {
                          match &mut named.orig {
                            ModuleExportName::Ident(ident) => {
                              ident.sym = mangled_ident.as_str().into();
                            }
                            ModuleExportName::Str(_) => unreachable!(),
                          }
                        }
                      }

                      // rename export { a as hello1 } from './hello'; to export { a as b } from './hello';
                      if let Some(mangled_ident) = mangled_ident_map.get(&exported_str) {
                        if let Some(exported) = named.exported.as_mut() {
                          match exported {
                            ModuleExportName::Ident(ident) => {
                              ident.sym = mangled_ident.as_str().into();
                            }
                            ModuleExportName::Str(_) => unreachable!(),
                          }
                        } else {
                          named.exported = Some(ModuleExportName::Ident(Ident::new(
                            mangled_ident.as_str().into(),
                            DUMMY_SP,
                            SyntaxContext::empty(),
                          )));
                        }
                      }
                    }
                    ExportSpecifier::Namespace(export_namespace_specifier) => {
                      if let Some(mangled_ident) =
                        mangled_ident_map.get(export_namespace_specifier.name.atom().as_str())
                      {
                        match &mut export_namespace_specifier.name {
                          ModuleExportName::Ident(ident) => {
                            ident.sym = mangled_ident.as_str().into();
                          }
                          ModuleExportName::Str(_) => unreachable!(),
                        }
                      }
                    }
                    ExportSpecifier::Default(_) => {
                      // do nothing
                    }
                  }
                }
              }
              farmfe_core::swc_ecma_ast::ModuleDecl::Import(import_decl) => {
                for specifier in import_decl.specifiers.iter_mut() {
                  match specifier {
                    farmfe_core::swc_ecma_ast::ImportSpecifier::Named(import_named_specifier) => {
                      let imported_str = import_named_specifier
                        .imported
                        .as_ref()
                        .map(|i| i.atom().to_string())
                        .unwrap_or(import_named_specifier.local.sym.to_string());

                      if let Some(mangled_ident) =
                        imports_to_rename.get(&(imported_str, import_decl.src.value.to_string()))
                      {
                        match &mut import_named_specifier.imported {
                          Some(imported) => match imported {
                            ModuleExportName::Ident(ident) => {
                              ident.sym = mangled_ident.as_str().into();
                            }
                            ModuleExportName::Str(_) => unreachable!(),
                          },
                          None => {
                            import_named_specifier.imported =
                              Some(ModuleExportName::Ident(Ident::new(
                                mangled_ident.as_str().into(),
                                DUMMY_SP,
                                SyntaxContext::empty(),
                              )))
                          }
                        }
                      }
                    }
                    _ => {
                      // do nothing
                    }
                  }
                }
              }
              _ => {
                // do nothing
              }
            }
          }
        }

        if extra_export_specifiers.len() > 0 {
          // add extra export specifiers
          meta
            .ast
            .body
            .push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
              NamedExport {
                span: DUMMY_SP,
                src: None,
                specifiers: extra_export_specifiers,
                type_only: false,
                with: None,
              },
            )));
        }

        // re-analyze statement info
        meta.statements = analyze_statements(&meta.ast);
      }
    });
}

pub fn find_idents_can_not_be_mangled(
  module_graph: &farmfe_core::module::module_graph::ModuleGraph,
) -> HashSet<ModuleExportIdent> {
  let can_not_be_mangled = Mutex::new(HashSet::default());

  // find all imported idents and filter out the ones that can not be mangled
  module_graph.modules().into_par_iter().for_each(|module| {
    if module.module_type.is_script() {
      let meta = module.meta.as_script();

      let mut ident_can_not_be_mangled = HashSet::default();

      // if module is not a es module or the dep is dynamically imported, all idents of its dependencies can not be mangled
      for (dep_id, edge_info) in module_graph.dependencies(&module.id) {
        if meta.module_system != ModuleSystem::EsModule || edge_info.contains_dynamic_import() {
          let dep_module = module_graph.module(&dep_id).unwrap();

          if !dep_module.module_type.is_script() {
            continue;
          }

          let dep_meta = dep_module.meta.as_script();

          dep_meta.export_ident_map.iter().for_each(|(_, ident)| {
            ident_can_not_be_mangled.insert(ident.clone());
          });
        }
      }

      // filter out the ones that can not be mangled
      for stmt in meta.statements.iter() {
        if let Some(import_info) = &stmt.import_info {
          let source_module_id = module_graph.get_dep_by_source(
            &module.id,
            &import_info.source,
            Some(ResolveKind::Import),
          );
          let source_module = module_graph.module(&source_module_id).unwrap();

          if !source_module.module_type.is_script() {
            continue;
          }

          let source_meta = source_module.meta.as_script();

          for specifier in &import_info.specifiers {
            match specifier {
              ImportSpecifierInfo::Namespace(_) => {
                // namespace import can not be mangled
                source_meta.export_ident_map.iter().for_each(|(_, ident)| {
                  ident_can_not_be_mangled.insert(ident.clone());
                });
              }
              _ => {
                // do nothing
              }
            }
          }
        } else if let Some(export_info) = &stmt.export_info {
          if let Some(source) = &export_info.source {
            let source_module_id =
              module_graph.get_dep_by_source(&module.id, source, Some(ResolveKind::ExportFrom));
            let source_module = module_graph.module(&source_module_id).unwrap();

            if !source_module.module_type.is_script() {
              continue;
            }

            let source_meta = source_module.meta.as_script();

            for specifier in &export_info.specifiers {
              match specifier {
                ExportSpecifierInfo::Namespace(_) => {
                  source_meta.export_ident_map.iter().for_each(|(_, ident)| {
                    ident_can_not_be_mangled.insert(ident.clone());
                  });
                }
                _ => {
                  // do nothing
                }
              }
            }
          }
        }
      }

      can_not_be_mangled.lock().extend(ident_can_not_be_mangled);
    }
  });

  can_not_be_mangled.into_inner()
}
