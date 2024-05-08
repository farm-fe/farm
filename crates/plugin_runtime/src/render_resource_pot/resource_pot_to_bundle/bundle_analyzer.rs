use std::{
  cell::RefCell,
  cmp::Ordering,
  collections::{HashMap, HashSet},
  rc::Rc,
  sync::Arc,
};

use farmfe_core::{
  config::Mode,
  context::CompilationContext,
  enhanced_magic_string::{
    bundle::{Bundle, BundleOptions},
    magic_string::{MagicString, MagicStringOptions},
  },
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, ModuleId, ModuleSystem},
  resource::resource_pot::{ResourcePot, ResourcePotType},
  swc_common::{comments::SingleThreadedComments, util::take::Take, DUMMY_SP},
  swc_ecma_ast::{
    ExportAll, ExportDefaultExpr, ExportNamedSpecifier, ExportNamespaceSpecifier, Expr, ImportDecl,
    ImportDefaultSpecifier, ImportNamedSpecifier, ImportStarAsSpecifier, ModuleDecl, ModuleItem,
    NamedExport, Str,
  },
};
use farmfe_toolkit::{
  common::build_source_map,
  script::{codegen_module, swc_try_with::try_with, CodeGenCommentsConfig},
  swc_ecma_transforms::fixer,
  swc_ecma_visit::VisitMutWith,
};

use crate::resource_pot_to_bundle::modules_analyzer::module_analyzer::ExportSpecifierInfo;

use super::{
  bundle::ModuleAnalyzerManager,
  bundle_external::{BundleReference, ExternalReferenceExport, ReferenceKind},
  common,
  modules_analyzer::module_analyzer::{ImportSpecifierInfo, StmtAction},
  uniq_name::{BundleVariable, FindModuleExportResult},
};

fn generate_export(
  source: Option<&ModuleId>,
  export: &ExternalReferenceExport,
  bundle_variable: &BundleVariable,
) -> Vec<ModuleItem> {
  let mut stmts = vec![];

  let mut specifiers = vec![];

  let source = source.map(|source| source.relative_path());

  let mut ordered_keys = export.named.keys().collect::<Vec<_>>();

  ordered_keys.sort_by(|a, b| bundle_variable.name(**a).cmp(&bundle_variable.name(**b)));

  for exported in ordered_keys {
    let local = &export.named[exported];
    if bundle_variable.var_by_index(*local).removed {
      continue;
    }

    let named_render_name = bundle_variable.render_name(*local);
    let exported_name = bundle_variable.name(*exported);

    let exported_name = if named_render_name == exported_name {
      None
    } else {
      Some(exported_name.as_str().into())
    };

    specifiers.push(farmfe_core::swc_ecma_ast::ExportSpecifier::Named(
      ExportNamedSpecifier {
        span: DUMMY_SP,
        orig: farmfe_core::swc_ecma_ast::ModuleExportName::Ident(named_render_name.as_str().into()),
        exported: exported_name.map(farmfe_core::swc_ecma_ast::ModuleExportName::Ident),
        is_type_only: false,
      },
    ));
  }

  if let Some(namespace) = export.namespace.as_ref() {
    specifiers.push(farmfe_core::swc_ecma_ast::ExportSpecifier::Namespace(
      ExportNamespaceSpecifier {
        span: DUMMY_SP,
        name: farmfe_core::swc_ecma_ast::ModuleExportName::Ident(
          bundle_variable.name(*namespace).as_str().into(),
        ),
      },
    ));
  }

  if let Some(source) = source {
    if export.all {
      stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportAll(ExportAll {
        span: DUMMY_SP,
        src: Box::new(source.into()),
        type_only: false,
        with: None,
      })));
    }
  }

  if !specifiers.is_empty() {
    stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(
      NamedExport {
        span: DUMMY_SP,
        specifiers,
        src: source.map(|source| Box::new(source.into())),
        type_only: false,
        with: None,
      },
    )));
  }

  if let Some(default) = export.default.as_ref() {
    stmts.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
      ExportDefaultExpr {
        span: DUMMY_SP,
        expr: Box::new(Expr::Ident(
          bundle_variable.render_name(*default).as_str().into(),
        )),
      },
    )));
  }

  stmts
}

#[derive(Debug)]
#[allow(dead_code)]
enum NamespaceExportType {
  External,
  Bundle,
  Entry(ModuleId),
}

pub struct BundleAnalyzer<'a> {
  pub resource_pot: &'a ResourcePot,
  pub ordered_modules: Vec<&'a ModuleId>,
  pub bundle_variable: Rc<RefCell<BundleVariable>>,

  module_graph: &'a ModuleGraph,
  context: Arc<CompilationContext>,

  pub bundle_reference: BundleReference,
}

impl<'a> BundleAnalyzer<'a> {
  pub fn new(
    resource_pot: &'a ResourcePot,
    module_graph: &'a ModuleGraph,
    context: &Arc<CompilationContext>,
    bundle_variable: Rc<RefCell<BundleVariable>>,
  ) -> Self {
    Self {
      bundle_variable,
      resource_pot,
      ordered_modules: vec![],
      module_graph,
      context: context.clone(),
      bundle_reference: BundleReference::new(),
    }
  }

  // step: 1 toposort fetch modules
  pub fn build_module_order(&mut self, order_index_map: &HashMap<ModuleId, usize>) {
    let mut resource_pot_modules = self.resource_pot.modules();

    resource_pot_modules.sort_by(|a, b| {
      if !order_index_map.contains_key(a) || !order_index_map.contains_key(b) {
        return Ordering::Greater;
      }

      return order_index_map[*b].cmp(&order_index_map[*a]);
    });

    self.ordered_modules = resource_pot_modules;
  }

  // 3-1. uniq declare variable name
  pub fn module_conflict_name(&mut self, module_analyzer_manager: &mut ModuleAnalyzerManager) {
    for module_id in &self.ordered_modules {
      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer_mut(&module_id) {
        if matches!(
          module_analyzer.module_system,
          ModuleSystem::CommonJs | ModuleSystem::Hybrid
        ) {
          continue;
        }

        let variables = module_analyzer.variables();

        for index in variables {
          self.bundle_variable.borrow_mut().set_var_uniq_rename(index);
        }
      };
    }
  }

  // 3-2. collect need remove module import/export
  pub fn strip_module(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
  ) -> Result<()> {
    for module_id in &self.ordered_modules {
      let mut stmt_action = HashSet::new();
      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer(module_id) {
        for statement in &module_analyzer.statements {
          if let Some(import) = statement.import.as_ref() {
            if module_analyzer_manager.is_commonjs(&import.source) {
              stmt_action.insert(StmtAction::ReplaceCjsImport(
                statement.id,
                import.source.clone(),
              ));
            } else {
              stmt_action.insert(StmtAction::RemoveImport(statement.id));
            }
          }

          if let Some(export) = statement.export.as_ref() {
            if let Some(source) = &export.source {
              stmt_action.insert(StmtAction::StripExport(statement.id));

              if module_analyzer_manager.is_commonjs(source) {
                stmt_action.insert(StmtAction::ReplaceCjsExport(source.clone()));
              }
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
      }

      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer_mut(module_id) {
        module_analyzer.statement_actions.extend(stmt_action);
      }
    }

    Ok(())
  }

  // 3-3 analyze module relation and link local variable
  pub fn analyzer_module_relation(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
  ) -> Result<()> {
    for module_id in &self.ordered_modules {
      // if other bundle or entry import namespace, should export it
      // if module_analyzer_manager.is_contain_namespace(module_id) {
      //   if let Some((local, named_as)) =
      //     module_analyzer_manager.namespace_uniq_named.get(&module_id)
      //   {
      //     let module_analyzer = module_analyzer_manager.module_analyzer(module_id).unwrap();

      //     let importer = self.module_graph.dependents_ids(module_id);

      //     let target = {
      //       if module_analyzer.entry {
      //         Some(NamespaceExportType::Entry(
      //           module_analyzer.module_id.clone(),
      //         ))
      //       } else {
      //         importer.iter().find_map(|item| {
      //           if let Some(m) = module_analyzer_manager.module_analyzer(item) {
      //             if m.resource_pot_id != module_analyzer.resource_pot_id {
      //               return Some(NamespaceExportType::Bundle);
      //             }
      //           }

      //           return None;
      //         })
      //       }
      //     };

      //     if let Some(result) = target {
      //       match result {
      //         NamespaceExportType::External => {
      //           self
      //             .bundle_external_reference
      //             .sync_export(&ExportSpecifierInfo::Named(local.clone().into()), &None);
      //         }

      //         NamespaceExportType::Bundle => {
      //           self
      //             .bundle_external_reference
      //             .sync_export(&ExportSpecifierInfo::Named(local.clone().into()), &None);
      //         }

      //         NamespaceExportType::Entry(importer) => {
      //           self.bundle_external_reference.sync_export(
      //             &ExportSpecifierInfo::Named(
      //               (
      //                 *local,
      //                 Some(*common::otr!(
      //                   named_as.get(&importer),
      //                   CompilationError::GenericError(format!(
      //                     "not found module {:?} namespace named",
      //                     importer
      //                   ))
      //                 )?),
      //               )
      //                 .into(),
      //             ),
      //             &None,
      //           );
      //         }
      //       }
      //     }
      //   };
      // }
      let mut commonjs_export_map: HashMap<ReferenceKind, ExternalReferenceExport> =
        HashMap::default();

      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer(module_id) {
        let resource_pot_id = module_analyzer.resource_pot_id.clone();

        for statement in &module_analyzer.statements {
          if let Some(import) = &statement.import {
            for specify in &import.specifiers {
              match specify {
                // import * as fs from "node:fs"
                ImportSpecifierInfo::Namespace(ns) => {
                  let target = self.bundle_variable.borrow().find_ident_by_index(
                    *ns,
                    &import.source,
                    &module_analyzer_manager.module_map,
                    resource_pot_id.clone(),
                    &self.module_graph,
                    false,
                    true,
                  );

                  if let Some(target) = target {
                    let is_common_js = target.is_common_js();
                    match target {
                      FindModuleExportResult::Local(_, target_module_id, _) => {
                        if let Some(local) = module_analyzer_manager
                          .module_global_uniq_name
                          .namespace_name(&target_module_id)
                        {
                          let rendered_name = self.bundle_variable.borrow().render_name(local);

                          self
                            .bundle_variable
                            .borrow_mut()
                            .set_rename(*ns, rendered_name);
                          if is_common_js {
                            self.bundle_reference.sync_import(
                              target_module_id.into(),
                              specify,
                              &self.bundle_variable.borrow(),
                              is_common_js,
                            )?;
                          }
                        }
                      }

                      FindModuleExportResult::External(_, _, _) => {
                        self.bundle_reference.sync_import(
                          ReferenceKind::Module(import.source.clone()),
                          specify,
                          &self.bundle_variable.borrow(),
                          false,
                        )?;

                        let rename = self.bundle_variable.borrow().name(common::otr!(
                          module_analyzer_manager
                            .module_global_uniq_name
                            .namespace_name(&import.source),
                          CompilationError::GenericError(format!(
                            "not found module {:?} namespace named",
                            import.source
                          ))
                        )?);

                        self.bundle_variable.borrow_mut().set_rename(*ns, rename);
                      }

                      FindModuleExportResult::Bundle(_, bundle_name, _) => {
                        let common_import_rename = self.bundle_reference.sync_import(
                          bundle_name.into(),
                          specify,
                          &self.bundle_variable.borrow(),
                          false,
                        )?;

                        let rendered_name = self
                          .bundle_variable
                          .borrow()
                          .render_name(common_import_rename);

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename(*ns, rendered_name);
                      }
                    }
                  }
                }

                ImportSpecifierInfo::Named { local, imported } => {
                  let imported = imported.unwrap_or(*local);
                  let target = self.bundle_variable.borrow().find_ident_by_index(
                    imported,
                    &import.source,
                    &module_analyzer_manager.module_map,
                    resource_pot_id.clone(),
                    self.module_graph,
                    self.bundle_variable.borrow().name(imported) == "default",
                    false,
                  );

                  if let Some(target) = target {
                    let is_common_js = target.is_common_js();
                    match target {
                      FindModuleExportResult::Local(index, target_source, _) => {
                        self.bundle_variable.borrow_mut().set_var_uniq_rename(index);

                        let rendered_name = self.bundle_variable.borrow().render_name(index);
                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename(*local, rendered_name);
                        if is_common_js {
                          self.bundle_reference.sync_import(
                            target_source.into(),
                            specify,
                            &self.bundle_variable.borrow(),
                            is_common_js,
                          )?;
                        }
                      }

                      FindModuleExportResult::External(_, target, _) => {
                        let rename = self.bundle_reference.sync_import(
                          target.into(),
                          specify,
                          &self.bundle_variable.borrow(),
                          false,
                        )?;

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_var_uniq_rename(rename);

                        // external
                        let rendered_name = self.bundle_variable.borrow().render_name(rename);
                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename(*local, rendered_name);
                      }

                      FindModuleExportResult::Bundle(_, target, _) => {
                        let rename = self.bundle_reference.sync_import(
                          target.into(),
                          specify,
                          &self.bundle_variable.borrow(),
                          false,
                        )?;

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_var_uniq_rename(rename);

                        let rendered_name = self.bundle_variable.borrow().render_name(rename);
                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename(*local, rendered_name);
                      }
                    }
                  }
                }

                ImportSpecifierInfo::Default(default) => {
                  let target = self.bundle_variable.borrow().find_ident_by_index(
                    *default,
                    &import.source,
                    &module_analyzer_manager.module_map,
                    resource_pot_id.clone(),
                    self.module_graph,
                    true,
                    false,
                  );

                  if let Some(target) = target {
                    let is_common_js = target.is_common_js();
                    match target {
                      FindModuleExportResult::Local(index, target_source, _) => {
                        self.bundle_variable.borrow_mut().set_var_uniq_rename(index);
                        let rendered_name = self.bundle_variable.borrow().render_name(index);
                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename(*default, rendered_name);
                        if is_common_js {
                          self.bundle_reference.sync_import(
                            target_source.into(),
                            specify,
                            &self.bundle_variable.borrow(),
                            is_common_js,
                          )?;
                        }
                      }

                      FindModuleExportResult::External(_, target, _) => {
                        let rename = self.bundle_reference.sync_import(
                          target.into(),
                          specify,
                          &self.bundle_variable.borrow(),
                          false,
                        )?;

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_var_uniq_rename(rename);

                        // external
                        let rendered_name = self.bundle_variable.borrow().render_name(rename);
                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename(*default, rendered_name);
                      }

                      FindModuleExportResult::Bundle(_, target, _) => {
                        let rename = self.bundle_reference.sync_import(
                          target.into(),
                          specify,
                          &self.bundle_variable.borrow(),
                          false,
                        )?;

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_var_uniq_rename(rename);

                        let rendered_name = self.bundle_variable.borrow().render_name(rename);
                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename(*default, rendered_name);
                      }
                    }
                  };
                }
              }
            }
          }

          if let Some(export) = &statement.export {
            for specify in &export.specifiers {
              match specify {
                // export * from 'module'
                ExportSpecifierInfo::All(_) => {
                  if let Some(source) = &export.source {
                    // entry | other bundle
                    if module_analyzer.entry || {
                      let importer = self.module_graph.dependents_ids(module_id);

                      importer.iter().any(|importer| {
                        if let Some(m) = module_analyzer_manager.module_analyzer(importer) {
                          return m.resource_pot_id != resource_pot_id;
                        };

                        false
                      })
                    } {
                      let exports = module_analyzer_manager
                        .export_names(source, &self.bundle_variable.borrow());

                      for (export, export_source) in exports {
                        let is_in_self_bundle = self.ordered_modules.contains(&&export_source);

                        for specify in export.specifiers {
                          let t = Some(export.source.clone().unwrap_or(export_source.clone()));
                          self.bundle_reference.sync_export(
                            &specify,
                            if is_in_self_bundle {
                              export.source.as_ref().map(|item| item.clone().into())
                            } else {
                              t.map(|item| item.into())
                            },
                            None,
                          );
                        }
                      }
                    }
                  } else {
                    unreachable!("export all should have source")
                  }
                }

                // export { a as b }
                // export { a as b } from './module';
                ExportSpecifierInfo::Named(variables) => {
                  if let Some(source) = &export.source {
                    let is_find_default =
                      self.bundle_variable.borrow().name(variables.local()) == "default";
                    let target = self.bundle_variable.borrow_mut().find_ident_by_index(
                      variables.local(),
                      source,
                      &module_analyzer_manager.module_map,
                      resource_pot_id.clone(),
                      self.module_graph,
                      is_find_default,
                      false,
                    );

                    if let Some(target) = target {
                      let is_common_js = target.is_common_js();
                      match target {
                        FindModuleExportResult::Local(local, target_source, _) => {
                          let importers = self.module_graph.dependents_ids(module_id);

                          if module_analyzer.entry
                            || importers.iter().any(|importer| {
                              if let Some(m) = module_analyzer_manager.module_analyzer(importer) {
                                return m.resource_pot_id != resource_pot_id;
                              };
                              false
                            })
                            || is_common_js
                          {
                            self.bundle_reference.sync_export(
                              &ExportSpecifierInfo::Named((local).into()),
                              if is_common_js {
                                Some(target_source.into())
                              } else {
                                None
                              },
                              if is_common_js {
                                Some(&mut commonjs_export_map)
                              } else {
                                None
                              },
                            );
                          }
                        }
                        FindModuleExportResult::External(_, target_source, _) => {
                          self.bundle_reference.sync_export(
                            specify,
                            Some(target_source.into()),
                            None,
                          );
                        }
                        FindModuleExportResult::Bundle(_, _, _) => {}
                      }
                    }
                  } else {
                    let importers = self.module_graph.dependents_ids(module_id);

                    if module_analyzer.entry
                      || importers.iter().any(|importer| {
                        if let Some(m) = module_analyzer_manager.module_analyzer(importer) {
                          return m.resource_pot_id != resource_pot_id;
                        };
                        false
                      })
                    {
                      self.bundle_reference.sync_export(specify, None, None);
                    }
                  }
                }

                // export default n, Default(n)
                // export default 1 + 1, Default("default")
                ExportSpecifierInfo::Default(var) => {
                  if self.bundle_variable.borrow().name(*var) == "default" {
                    let rendered_name = common::otr!(
                      module_analyzer_manager
                        .module_global_uniq_name
                        .default_name(module_id),
                      CompilationError::GenericError(format!(
                        "not found module {:?} default name",
                        module_id
                      ))
                    )?;

                    let rendered_name = self.bundle_variable.borrow().render_name(rendered_name);

                    self
                      .bundle_variable
                      .borrow_mut()
                      .set_rename(*var, rendered_name);
                  } else {
                    self.bundle_variable.borrow_mut().set_var_uniq_rename(*var);
                  }

                  if module_analyzer.entry || {
                    let importers = module_analyzer_manager.module_analyzer(module_id);

                    importers
                      .iter()
                      .any(|importer| importer.resource_pot_id != resource_pot_id)
                  } {
                    self.bundle_reference.sync_export(
                      &ExportSpecifierInfo::Default(*var),
                      None,
                      None,
                    );
                  }
                }

                // export * as ns from 'module'
                ExportSpecifierInfo::Namespace(ns) => {
                  let source = common::otr!(
                    export.source.as_ref(),
                    CompilationError::GenericError(
                      "namespace should have source, but not found".to_string()
                    )
                  )?;

                  let local_var = common::otr!(
                    module_analyzer_manager
                      .module_global_uniq_name
                      .namespace_name(source),
                    CompilationError::GenericError(format!(
                      "not found module {:?} namespace named",
                      source
                    ))
                  )?;

                  let local_name = self.bundle_variable.borrow().render_name(local_var);

                  self
                    .bundle_variable
                    .borrow_mut()
                    .set_rename(*ns, local_name);

                  // bundle
                  if let Some(module_analyzer) = module_analyzer_manager.module_analyzer(source) {
                    // export * from './other_bundle_module'
                    if module_analyzer.resource_pot_id != resource_pot_id {
                      // export { } from './other_bundle_module'
                      self.bundle_reference.sync_export(
                        &ExportSpecifierInfo::Named((*ns).into()),
                        Some(module_analyzer.resource_pot_id.clone().into()),
                        None,
                      );
                    }
                  }
                  // external
                  else {
                    // export * as fs from "node:fs" => import * as node_fs from "node:fs"
                    self.bundle_reference.sync_import(
                      source.clone().into(),
                      &ImportSpecifierInfo::Namespace(*ns),
                      &self.bundle_variable.borrow(),
                      false,
                    )?;
                  }
                }
              }
            }
          }
        }
      }

      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer_mut(module_id) {
        module_analyzer.cjs_module_analyzer.commonjs_export = commonjs_export_map;
      }
    }

    Ok(())
  }

  // 3. start process bundle
  pub fn render(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    order_index_map: &HashMap<ModuleId, usize>,
  ) -> Result<()> {
    self.module_conflict_name(module_analyzer_manager);

    self.strip_module(module_analyzer_manager)?;

    self.analyzer_module_relation(module_analyzer_manager)?;

    self.patch_ast(module_analyzer_manager, order_index_map)?;

    Ok(())
  }

  /// 3-4
  /// 1. strip or remove import/export
  /// 2. generate import/export, eg module from external or other bundle
  pub fn patch_ast(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    order_index_map: &HashMap<ModuleId, usize>,
  ) -> Result<()> {
    for module_id in &self.ordered_modules {
      module_analyzer_manager.patch_module_analyzer_ast(
        &module_id,
        &self.context,
        &self.module_graph,
        &mut self.bundle_variable.borrow_mut(),
        &self.bundle_reference,
      )?;
    }

    let polyfill_asts = module_analyzer_manager.polyfill.to_ast()?;

    let is_runtime_bundle = matches!(
      self.resource_pot.resource_pot_type,
      ResourcePotType::Runtime
    );

    let mut patch_export_to_module = vec![];
    let mut patch_import_to_module = vec![];

    // runtime bundle should not have export
    if !is_runtime_bundle {
      let mut ordered_external_export = self
        .bundle_reference
        .external_export_map
        .keys()
        .collect::<Vec<_>>();

      ordered_external_export.sort_by(|a, b| {
        order_index_map[&a.to_module_id()].cmp(&order_index_map[&b.to_module_id()])
      });

      for source in ordered_external_export {
        let export = &self.bundle_reference.external_export_map[source];

        patch_export_to_module.extend(generate_export(
          Some(&source.to_module_id()),
          export,
          &self.bundle_variable.borrow(),
        ));
      }

      if let Some(export) = self.bundle_reference.export.as_ref() {
        patch_export_to_module.extend(generate_export(
          None,
          export,
          &self.bundle_variable.borrow(),
        ));
      }
    }

    let mut ordered_import = self.bundle_reference.import_map.keys().collect::<Vec<_>>();
    ordered_import.sort_by(|a, b| a.cmp(b));

    // TODO: cjs
    for source in ordered_import {
      let import = &self.bundle_reference.import_map[source];

      if import.named.is_empty() && import.namespace.is_none() && import.default.is_none() {
        continue;
      }

      let mut specifiers = vec![];

      let mut ordered_named_keys = import.named.keys().collect::<Vec<_>>();
      ordered_named_keys.sort();
      for imported in ordered_named_keys {
        let local = &import.named[imported];
        let local_named = self.bundle_variable.borrow().render_name(*local);

        specifiers.push(farmfe_core::swc_ecma_ast::ImportSpecifier::Named(
          ImportNamedSpecifier {
            span: DUMMY_SP,
            local: local_named.as_str().into(),
            imported: if imported == &local_named {
              None
            } else {
              Some(farmfe_core::swc_ecma_ast::ModuleExportName::Ident(
                imported.as_str().into(),
              ))
            },
            is_type_only: false,
          },
        ));
      }

      if let Some(namespace) = import.namespace.as_ref() {
        specifiers.push(farmfe_core::swc_ecma_ast::ImportSpecifier::Namespace(
          ImportStarAsSpecifier {
            span: DUMMY_SP,
            local: self
              .bundle_variable
              .borrow()
              .render_name(*namespace)
              .as_str()
              .into(),
          },
        ));
      }

      if let Some(default) = import.default.as_ref() {
        specifiers.push(farmfe_core::swc_ecma_ast::ImportSpecifier::Default(
          ImportDefaultSpecifier {
            span: DUMMY_SP,
            local: self
              .bundle_variable
              .borrow()
              .render_name(*default)
              .as_str()
              .into(),
          },
        ));
      }

      patch_import_to_module.push(ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
        span: DUMMY_SP,
        specifiers,
        src: Box::new(Str {
          span: DUMMY_SP,
          value: source.to_string().as_str().into(),
          raw: None,
        }),
        type_only: false,
        with: None,
        phase: farmfe_core::swc_ecma_ast::ImportPhase::Evaluation,
      })));
    }

    if !patch_import_to_module.is_empty() || !polyfill_asts.is_empty() {
      if let Some(module_analyzer) = self
        .ordered_modules
        .first()
        .map(|item| module_analyzer_manager.module_analyzer_mut(item))
        .flatten()
      {
        let ast = &mut module_analyzer.ast;

        ast.body = patch_import_to_module
          .into_iter()
          .chain(polyfill_asts)
          .chain(ast.body.take().into_iter())
          .collect();
      };
    }

    if !patch_export_to_module.is_empty() {
      if let Some(module_analyzer) = self
        .ordered_modules
        .last()
        .map(|id| module_analyzer_manager.module_analyzer_mut(id))
        .flatten()
      {
        let ast = &mut module_analyzer.ast;

        ast.body = ast
          .body
          .take()
          .into_iter()
          .chain(patch_export_to_module)
          .collect();
      };
    }

    Ok(())
  }

  // step: 4 generate bundle code
  pub fn codegen(&mut self, module_analyzer_manager: &mut ModuleAnalyzerManager) -> Result<Bundle> {
    let mut bundle = Bundle::new(BundleOptions {
      separator: Some('\n'),
      intro: None,
      trace_source_map_chain: Some(false),
    });

    for module_id in &self.ordered_modules {
      let module = self
        .module_graph
        .module(&module_id)
        .unwrap_or_else(|| panic!("Module not found: {:?}", module_id));
      let module_analyzer = module_analyzer_manager
        .module_analyzer_mut(module_id)
        .unwrap();

      let comments: SingleThreadedComments = module.meta.as_script().comments.clone().into();

      let sourcemap_enabled = self.context.config.sourcemap.enabled(module.immutable);

      try_with(
        module_analyzer.cm.clone(),
        &self.context.meta.script.globals,
        || {
          module_analyzer
            .ast
            .visit_mut_with(&mut fixer(Some(&comments)));
        },
      )?;

      let mut mappings = vec![];
      let code_bytes = codegen_module(
        &module_analyzer.ast,
        self.context.config.script.target,
        module_analyzer.cm.clone(),
        if sourcemap_enabled {
          Some(&mut mappings)
        } else {
          None
        },
        false,
        Some(CodeGenCommentsConfig {
          comments: &comments,
          config: &self.context.config.comments,
        }),
      )
      .map_err(|err| CompilationError::RenderScriptModuleError {
        id: module_analyzer.module_id.to_string(),
        source: Some(Box::new(err)),
      })?;

      let code = String::from_utf8(code_bytes).map_err(|err| {
        CompilationError::GenericError(format!(
          "failed to convert code bytes to string, origin error: {}",
          err.to_string()
        ))
      })?;

      let mut source_map_chain = vec![];

      if sourcemap_enabled {
        let sourcemap = build_source_map(module_analyzer.cm.clone(), &mappings);
        let mut buf = vec![];
        sourcemap
          .to_writer(&mut buf)
          .map_err(|e| CompilationError::RenderScriptModuleError {
            id: module_id.to_string(),
            source: Some(Box::new(e)),
          })?;
        let map = Arc::new(String::from_utf8(buf).unwrap());

        source_map_chain = module.source_map_chain.clone();
        source_map_chain.push(map);
      }

      let mut module = MagicString::new(
        &code,
        Some(MagicStringOptions {
          filename: Some(module_id.resolved_path_with_query(&self.context.config.root)),
          source_map_chain,
          ..Default::default()
        }),
      );

      if matches!(self.context.config.mode, Mode::Development) {
        // debug info
        module.prepend(&format!("// module_id: {}\n", module_id.to_string()));
      }

      bundle.add_source(module, None).unwrap();
    }

    Ok(bundle)
  }
}
