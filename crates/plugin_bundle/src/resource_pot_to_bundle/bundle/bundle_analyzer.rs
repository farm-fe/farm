use std::{cell::RefCell, cmp::Ordering, rc::Rc, sync::Arc};

use farmfe_core::{
  config::{external::ExternalConfig, ModuleFormat},
  context::{get_swc_sourcemap_filename, CompilationContext},
  error::Result,
  farm_profile_function, farm_profile_scope,
  module::{
    meta_data::script::CommentsMetaData, module_graph::ModuleGraph, ModuleId, ModuleSystem,
  },
  plugin::ResolveKind,
  rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator},
  resource::resource_pot::ResourcePotType,
  swc_common::{
    comments::{Comments, SingleThreadedComments},
    util::take::Take,
    SourceMap, DUMMY_SP,
  },
  swc_ecma_ast::Module,
  HashMap, HashSet,
};
use farmfe_toolkit::{
  script::{
    sourcemap::{merge_sourcemap, SpanUpdater},
    swc_try_with::try_with,
  },
  swc_ecma_transforms::fixer,
  swc_ecma_visit::VisitMutWith,
};

use crate::resource_pot_to_bundle::{
  bundle::bundle_reference::{
    try_reexport_entry_module, CommonJsImportMap, ReferenceBuilder, ReferenceKind,
  },
  common::OptionToResult,
  modules_analyzer::module_analyzer::{
    ExportSpecifierInfo, ImportSpecifierInfo, StmtAction, Variable,
  },
  polyfill::SimplePolyfill,
  targets::{
    cjs::CjsModuleAnalyzer,
    generate::{generate_bundle_import_by_bundle_reference, generate_export_by_reference_export},
  },
  uniq_name::{BundleVariable, FindModuleExportResult},
  BundleGroup, ShareBundleContext, FARM_BUNDLE_POLYFILL_SLOT,
};

use super::{
  bundle_reference::{BundleReferenceManager, CombineBundleReference},
  ModuleAnalyzerManager,
};

#[derive(Debug)]
#[allow(dead_code)]
enum NamespaceExportType {
  External,
  Bundle,
  Entry(ModuleId),
}

pub struct GeneratorAstResult {
  pub ast: Module,
  pub comments: CommentsMetaData,
  pub rendered_modules: Vec<ModuleId>,
}

pub struct BundleAnalyzer<'a> {
  pub group: BundleGroup<'a>,
  pub ordered_modules: Vec<&'a ModuleId>,
  pub bundle_variable: Rc<RefCell<BundleVariable>>,

  module_graph: &'a ModuleGraph,
  context: Arc<CompilationContext>,

  pub polyfill: SimplePolyfill,
}

impl<'a> BundleAnalyzer<'a> {
  pub fn new(
    resource_pot: BundleGroup<'a>,
    module_graph: &'a ModuleGraph,
    context: &Arc<CompilationContext>,
    bundle_variable: Rc<RefCell<BundleVariable>>,
  ) -> Self {
    Self {
      bundle_variable,
      group: resource_pot,
      ordered_modules: vec![],
      module_graph,
      context: context.clone(),
      // bundle_reference: BundleReference::new(),
      // bundle level polyfill
      polyfill: SimplePolyfill::default(),
    }
  }

  pub fn set_namespace(&mut self, group_id: &str) {
    self
      .bundle_variable
      .borrow_mut()
      .set_namespace(group_id.to_string());
  }

  // step: 1 toposort fetch modules
  pub fn build_module_order(&mut self, order_index_map: &HashMap<ModuleId, usize>) {
    farm_profile_function!();
    let mut resource_pot_modules = self.group.modules.clone();

    resource_pot_modules.sort_by(|a, b| {
      if !order_index_map.contains_key(a) || !order_index_map.contains_key(b) {
        return Ordering::Greater;
      }

      order_index_map[*b].cmp(&order_index_map[*a])
    });

    self.ordered_modules = resource_pot_modules;
  }

  // 3-1. uniq declare variable name
  pub fn module_conflict_name(&mut self, module_analyzer_manager: &mut ModuleAnalyzerManager) {
    farm_profile_function!("");

    for module_id in &self.ordered_modules {
      farm_profile_scope!(format!(
        "bundle module conflict name: {}",
        module_id.to_string()
      ));
      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer_mut(module_id) {
        let is_commonjs = module_analyzer.is_commonjs();
        let mut variables = module_analyzer.variables().into_iter().collect::<Vec<_>>();

        variables.sort();
        let mut bundle_variable = self.bundle_variable.borrow_mut();

        for index in variables {
          // used_name is global variable, we need rename same used_name in cjs
          if is_commonjs && !(bundle_variable.is_in_used_name(index)) {
            continue;
          }

          bundle_variable.set_var_uniq_rename(index);
        }
      };
    }
  }

  // 3-2. collect need remove module import/export
  pub fn strip_module(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
  ) -> Result<()> {
    farm_profile_function!("");
    for module_id in &self.ordered_modules {
      farm_profile_scope!(format!("bundle strip module: {}", module_id.to_string()));
      let mut stmt_action = HashSet::default();
      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer(module_id) {
        for statement in &module_analyzer.statements {
          // import
          if let Some(import) = statement.import.as_ref() {
            if module_analyzer_manager.is_commonjs(&import.source) {
              stmt_action.insert(StmtAction::StripCjsImport(
                statement.id,
                if import.specifiers.is_empty() {
                  Some(import.source.clone())
                } else {
                  None
                },
              ));
            } else {
              stmt_action.insert(StmtAction::RemoveImport(statement.id));
            }
          }

          // export
          if let Some(export) = statement.export.as_ref() {
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
      }

      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer_mut(module_id) {
        module_analyzer.statement_actions.extend(stmt_action);
      }
    }

    Ok(())
  }

  // 3-3 find module relation and link local variable
  // TODO:
  //  1. refactor bundle_reference import/export logic
  //  2. split BundleReference collection and reuse it
  pub fn link_module_relation(
    &mut self,
    module_id: &ModuleId,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    bundle_reference_manager: &mut BundleReferenceManager,
    options: &ShareBundleContext,
  ) -> Result<()> {
    let is_format_to_commonjs = matches!(options.options.format, ModuleFormat::CommonJs);

    farm_profile_scope!(format!(
      "bundle analyzer module relation: {}",
      module_id.to_string()
    ));

    if let Some(module_analyzer) = module_analyzer_manager.module_analyzer(module_id) {
      let group_id = module_analyzer.bundle_group_id.clone();
      let module_system = module_analyzer.module_system.clone();
      let is_entry = module_analyzer.entry;

      let is_reference_by_another = is_entry
        || module_analyzer.is_reference_by_another(|| {
          let importer = self.module_graph.dependents_ids(module_id);
          importer.iter().any(|importer| {
            if module_analyzer_manager.contain(importer) {
              module_analyzer_manager
                .group_id(importer)
                .is_some_and(|i| i != &group_id)
            } else {
              // partial ShareBundle should reexport raw
              true
            }
          })
        });

      let bundle_reference1 = bundle_reference_manager.reference_mut(&group_id);
      let mut bundle_reference1 = bundle_reference1.borrow_mut();

      // reexport as namespace
      if module_analyzer_manager
        .namespace_modules
        .contains(module_id)
        && !module_analyzer_manager.is_commonjs(module_id)
      {
        let ns = module_analyzer_manager
          .module_global_uniq_name
          .namespace_name_result(module_id)?;

        bundle_reference1.add_local_export(
          &ExportSpecifierInfo::Namespace(ns),
          module_system.clone(),
          is_entry,
        );
      }

      for statement in &module_analyzer.statements {
        if let Some(import) = &statement.import {
          if import.specifiers.is_empty() {
            if module_analyzer_manager.is_same_bundle(module_id, &import.source) {
              continue;
            }

            let reference_kind = ReferenceKind::Module(import.source.clone());

            // import 'module';
            if module_analyzer_manager.is_commonjs(&import.source)
              && !module_analyzer_manager.is_external(module_id)
            {
              if let Some(name) = module_analyzer_manager
                .module_global_uniq_name
                .commonjs_name(&import.source)
              {
                bundle_reference1.add_import(
                  &ImportSpecifierInfo::Named {
                    local: name,
                    imported: None,
                  },
                  import.source.clone().into(),
                  &self.bundle_variable.borrow_mut(),
                )?;
              }
            } else {
              bundle_reference1.add_execute_module(reference_kind);
            }

            continue;
          }

          for specify in &import.specifiers {
            match specify {
              // import * as person from "person"
              ImportSpecifierInfo::Namespace(ns) => {
                let target = self.bundle_variable.borrow().find_ident_by_index(
                  *ns,
                  &import.source,
                  module_analyzer_manager,
                  group_id.clone(),
                  false,
                  true,
                );

                if let Some(target) = target {
                  let is_common_js = target.is_common_js();

                  match target {
                    FindModuleExportResult::Local {
                      source: target_module_id,
                      ..
                    } => {
                      if let Some(local) = module_analyzer_manager
                        .module_global_uniq_name
                        .namespace_name(&target_module_id)
                      {
                        if is_common_js {
                          bundle_reference1.add_declare_commonjs_import(
                            &ImportSpecifierInfo::Namespace(local),
                            target_module_id.into(),
                            &self.bundle_variable.borrow(),
                          )?;
                        }

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename_from_other_render_name(*ns, local);
                      }
                    }

                    FindModuleExportResult::External(_, _, _) => {
                      bundle_reference1.add_import(
                        specify,
                        import.source.clone().into(),
                        &self.bundle_variable.borrow(),
                      )?;

                      let rename = module_analyzer_manager
                        .module_global_uniq_name
                        .namespace_name(&import.source)
                        .to_result(format!(
                          "not found module {:?} namespace named",
                          import.source
                        ))?;

                      self
                        .bundle_variable
                        .borrow_mut()
                        .set_rename_from_other_name(*ns, rename);
                    }

                    // TODO: bundle
                    FindModuleExportResult::Bundle(_, target_id, _, _) => {
                      let namespace = module_analyzer_manager
                        .module_global_uniq_name
                        .namespace_name_result(&target_id)?;

                      let mut bundle_variable = self.bundle_variable.borrow_mut();
                      bundle_variable.set_var_root(*ns, namespace);

                      let is_same_bundle = bundle_variable.is_same_bundle_by_root(
                        *ns,
                        &group_id,
                        module_analyzer_manager,
                      );

                      let mut import_rename = namespace;

                      if !is_same_bundle {
                        // let other_bundle_reference =
                        //   bundle_reference_manager.reference1_mut(&target_id);
                        let other_bundle_reference = bundle_reference_manager
                          .reference_mut_by_module(&target_id, &module_analyzer_manager);
                        let mut other_bundle_reference = other_bundle_reference.borrow_mut();

                        other_bundle_reference.add_local_export(
                          &ExportSpecifierInfo::Named((namespace).into()),
                          module_system.clone(),
                          is_entry,
                        );

                        import_rename = bundle_reference1.add_import(
                          &ImportSpecifierInfo::Named {
                            local: namespace,
                            imported: None,
                          },
                          target_id.into(),
                          &bundle_variable,
                        )?;
                      }

                      bundle_variable.set_rename_from_other_render_name(*ns, import_rename);
                    }
                  }
                }
              }

              // import { name, age } from "person";
              ImportSpecifierInfo::Named { local, imported } => {
                let imported = imported.unwrap_or(*local);

                let target = self.bundle_variable.borrow().find_ident_by_index(
                  imported,
                  &import.source,
                  module_analyzer_manager,
                  group_id.clone(),
                  self.bundle_variable.borrow().name(imported) == "default",
                  false,
                );

                if let Some(target) = target {
                  let is_common_js = target.is_common_js();
                  match target {
                    FindModuleExportResult::Local {
                      mut index,
                      source: target_source,
                      dynamic_reference,
                      ..
                    } => {
                      if is_common_js || dynamic_reference {
                        index = bundle_reference1.add_declare_commonjs_import(
                          specify,
                          target_source.clone().into(),
                          &self.bundle_variable.borrow(),
                        )?;
                      }

                      self
                        .bundle_variable
                        .borrow_mut()
                        .set_uniq_name_both(index, *local);
                    }

                    FindModuleExportResult::External(index, target, _) => {
                      let mut bundle_variable = self.bundle_variable.borrow_mut();

                      let rename = bundle_reference1.add_import(
                        &ImportSpecifierInfo::Named {
                          local: *local,
                          imported: Some(index),
                        },
                        target.into(),
                        &bundle_variable,
                      )?;

                      bundle_variable.set_uniq_name_both(rename, *local);
                    }

                    FindModuleExportResult::Bundle(index, target_id, _, _) => {
                      let mut bundle_variable = self.bundle_variable.borrow_mut();
                      let is_same_bundle = if is_common_js {
                        module_analyzer_manager.is_same_bundle(&module_id, &target_id)
                      } else {
                        bundle_variable.set_var_root(*local, index);
                        bundle_variable.is_same_bundle_by_root(
                          *local,
                          &group_id,
                          module_analyzer_manager,
                        )
                      };
                      let mut rename = index;

                      if !is_same_bundle {
                        if is_common_js {
                          bundle_reference1.add_import(
                            &ImportSpecifierInfo::Named {
                              local: module_analyzer_manager
                                .module_global_uniq_name
                                .commonjs_name_result(&target_id)?,
                              imported: None,
                            },
                            target_id.clone().into(),
                            &bundle_variable,
                          )?;
                        } else {
                          rename = bundle_reference1.add_import(
                            &ImportSpecifierInfo::Named {
                              local: index,
                              imported: None,
                            },
                            bundle_variable
                              .module_id_by_var_index(index)
                              .unwrap()
                              .into(),
                            &bundle_variable,
                          )?;
                        }
                      }
                      if is_common_js {
                        bundle_reference1.add_declare_commonjs_import(
                          &ImportSpecifierInfo::Named {
                            local: *local,
                            imported: None,
                          },
                          target_id.clone().into(),
                          &bundle_variable,
                        )?;
                      }
                      bundle_variable.set_uniq_name_for_cross_bundle(
                        rename,
                        *local,
                        &target_id,
                        &module_id,
                        module_analyzer_manager,
                      );
                    }
                  }
                }
              }

              // import person from "person"
              ImportSpecifierInfo::Default(default) => {
                let mut bundle_variable = self.bundle_variable.borrow_mut();
                let target = bundle_variable.find_ident_by_index(
                  *default,
                  &import.source,
                  module_analyzer_manager,
                  group_id.clone(),
                  true,
                  false,
                );

                if let Some(target) = target {
                  let is_common_js = target.is_common_js();
                  match target {
                    FindModuleExportResult::Local {
                      mut index,
                      source: target_source,
                      ..
                    } => {
                      if is_common_js {
                        index = bundle_reference1.add_declare_commonjs_import(
                          specify,
                          target_source.into(),
                          &bundle_variable,
                        )?;
                      }
                      bundle_variable.set_uniq_name_both(index, *default);
                    }

                    FindModuleExportResult::External(index, target, _) => {
                      let mut rename = index;

                      if target == import.source {
                        rename = bundle_reference1.add_import(
                          &ImportSpecifierInfo::Default(index),
                          target.into(),
                          &bundle_variable,
                        )?;
                      }

                      bundle_variable.set_uniq_name_both(rename, *default);
                    }

                    FindModuleExportResult::Bundle(target_default_index, target_id, _, _) => {
                      let mut name = target_default_index;

                      if is_common_js {
                        name = module_analyzer_manager
                          .module_global_uniq_name
                          .commonjs_name_result(&target_id)?;
                      } else if bundle_variable.name(target_default_index) == "default" {
                        name = module_analyzer_manager
                          .module_global_uniq_name
                          .default_name_result(&target_id)?;
                      }

                      let is_same_bundle = if is_common_js {
                        module_analyzer_manager.is_same_bundle(module_id, &target_id)
                      } else {
                        bundle_variable.set_var_root(*default, name);
                        bundle_variable.is_same_bundle_by_root(
                          *default,
                          &group_id,
                          module_analyzer_manager,
                        )
                      };

                      if !is_same_bundle {
                        if is_common_js {
                          bundle_reference1.add_import(
                            &ImportSpecifierInfo::Named {
                              local: module_analyzer_manager
                                .module_global_uniq_name
                                .commonjs_name_result(&target_id)?
                                .into(),
                              imported: None,
                            },
                            target_id.clone().into(),
                            &bundle_variable,
                          )?;
                        } else {
                          bundle_reference1.add_import(
                            &ImportSpecifierInfo::Default(*default),
                            target_id.clone().into(),
                            &bundle_variable,
                          )?;
                        }
                      };

                      if is_common_js {
                        bundle_reference1.add_declare_commonjs_import(
                          &ImportSpecifierInfo::Default(*default),
                          target_id.clone().into(),
                          &bundle_variable,
                        )?;
                        bundle_variable.set_var_uniq_rename(*default);
                      } else {
                        bundle_variable.set_uniq_name_for_cross_bundle(
                          name,
                          *default,
                          &target_id,
                          module_id,
                          module_analyzer_manager,
                        );
                      }
                    }
                  }
                };
              }
            }
          }
        }

        if let Some(export) = &statement.export {
          if module_analyzer_manager.is_commonjs(module_id) && !is_reference_by_another {
            continue;
          }

          if module_analyzer.is_dynamic && is_reference_by_another {
            if let Some(x) = module_analyzer_manager
              .module_global_uniq_name
              .namespace_name(module_id)
            {
              bundle_reference1.add_local_export(
                &ExportSpecifierInfo::Named(x.into()),
                module_system.clone(),
                is_entry,
              );
            }
          }

          for specify in &export.specifiers {
            match specify {
              // export * from 'person'
              ExportSpecifierInfo::All(_) => {
                let Some(source) = &export.source else {
                  unreachable!("export all should have source")
                };

                bundle_reference1.add_reexport_all(ReferenceBuilder {
                  is_reference_by_another_bundle: is_reference_by_another,
                  module_analyzer_manager,
                  module_analyzer,
                  bundle_variable: &mut self.bundle_variable.borrow_mut(),
                  source,
                  module_system: module_system.clone(),
                  config: &options,
                  module_id: &module_id,
                  is_entry,
                })?;
              }

              // export { name as personName }
              // export { name as personName } from './person';
              ExportSpecifierInfo::Named(variable) => {
                let mut bundle_variable = self.bundle_variable.borrow_mut();
                if let Some(source) = &export.source {
                  let is_find_default = bundle_variable.is_default_key(variable.local());
                  let target = bundle_variable.find_ident_by_index(
                    variable.local(),
                    source,
                    module_analyzer_manager,
                    group_id.clone(),
                    is_find_default,
                    false,
                  );
                  if let Some(target) = target {
                    let is_common_js = target.is_common_js();
                    let mut is_confirmed_import = false;
                    let target_source = target.target_source();
                    let module_system =
                      module_system.merge(target.module_system().unwrap_or(module_system.clone()));

                    match target {
                      FindModuleExportResult::Local {
                        index: local,
                        source: target_source,
                        dynamic_reference,
                        ..
                      } => {
                        is_confirmed_import = true;
                        let is_default_key = bundle_variable.is_default_key(local);

                        let name = if is_default_key {
                          module_analyzer_manager
                            .module_global_uniq_name
                            .default_name_result(&target_source)?
                        } else {
                          local
                        };

                        if is_common_js || dynamic_reference {
                          bundle_variable.set_var_uniq_rename(local);

                          bundle_reference1.add_declare_commonjs_import(
                            &if is_default_key {
                              ImportSpecifierInfo::Named {
                                local: name,
                                imported: Some(local),
                              }
                            } else {
                              ImportSpecifierInfo::Named {
                                local,
                                imported: None,
                              }
                            },
                            target_source.into(),
                            &bundle_variable,
                          )?;
                        }

                        if is_reference_by_another {
                          bundle_reference1.add_local_export(
                            &ExportSpecifierInfo::Named(Variable(name, Some(variable.export_as()))),
                            module_system,
                            is_entry,
                          );
                        }
                      }

                      FindModuleExportResult::External(_, target_source, _) => {
                        // TODO: should used in
                        let is_reference_by_self_bundle = self
                          .module_graph
                          .dependents(&target_source)
                          .iter()
                          .any(|(m, edge)| {
                            module_analyzer_manager.is_same_bundle(module_id, m)
                              && edge.iter().any(|edge| {
                                matches!(
                                  edge.kind,
                                  ResolveKind::DynamicImport
                                    | ResolveKind::Require
                                    | ResolveKind::Import
                                    | ResolveKind::ExportFrom
                                )
                              })
                          });
                        is_confirmed_import = true;
                        if is_reference_by_another {
                          bundle_reference1.add_reference_export(
                            specify,
                            target_source.clone().into(),
                            module_system,
                            is_entry,
                          );
                        }

                        {
                          if !bundle_variable.is_default_key(variable.export_from()) {
                            bundle_variable
                              .set_uniq_name_both(variable.export_from(), variable.export_as());
                          }

                          if is_reference_by_self_bundle {
                            bundle_reference1.add_import(
                              &ImportSpecifierInfo::Named {
                                local: variable.export_as(),
                                imported: Some(variable.export_from()),
                              },
                              target_source.into(),
                              &bundle_variable,
                            )?;
                          }
                        }
                      }

                      FindModuleExportResult::Bundle(index, target_id, _, _) => {
                        is_confirmed_import = true;
                        let is_same_bundle = if is_common_js {
                          module_analyzer_manager.is_same_bundle(module_id, &target_id)
                        } else {
                          bundle_variable.set_var_root(variable.local(), index);
                          bundle_variable.is_same_bundle_by_root(
                            index,
                            &group_id,
                            module_analyzer_manager,
                          )
                        };

                        if is_same_bundle {
                          bundle_reference1.add_local_export(
                            &ExportSpecifierInfo::Named((variable.local()).into()),
                            module_system,
                            is_entry,
                          );
                        } else {
                          bundle_reference1.add_import(
                            &ImportSpecifierInfo::Named {
                              local: index,
                              imported: None,
                            },
                            target_id.clone().into(),
                            &bundle_variable,
                          )?;

                          bundle_reference1.add_local_export(
                            &ExportSpecifierInfo::Named((variable.local()).into()),
                            module_system,
                            is_entry,
                          );
                        }
                      }
                    }

                    if !is_confirmed_import {
                      bundle_reference1.add_empty_import(target_source)
                    }
                  }
                } else {
                  bundle_variable.set_var_uniq_rename(variable.local());

                  if is_reference_by_another {
                    if module_analyzer_manager.is_commonjs(module_id) {
                      let is_default_key = bundle_variable.is_default_key(variable.local());

                      bundle_reference1.add_declare_commonjs_import(
                        &ImportSpecifierInfo::Named {
                          local: if is_default_key {
                            module_analyzer_manager
                              .module_global_uniq_name
                              .default_name_result(module_id)?
                          } else {
                            variable.local()
                          },
                          imported: Some(variable.export_as()),
                        },
                        ReferenceKind::Module((*module_id).clone()),
                        &bundle_variable,
                      )?;
                    }

                    bundle_reference1.add_local_export(specify, module_system.clone(), is_entry);
                  }
                }
              }

              // export default n, Default(n)
              // export default 1 + 1, Default("default")
              ExportSpecifierInfo::Default(var) => {
                let default_name = || {
                  module_analyzer_manager
                    .module_global_uniq_name
                    .default_name_result(module_id)
                };
                let mut bundle_variable = self.bundle_variable.borrow_mut();
                let is_default_key = bundle_variable.is_default_key(*var);

                if is_default_key {
                  let default_name = default_name()?;
                  let rendered_name = bundle_variable.render_name(default_name);

                  bundle_variable.set_var_root(*var, default_name);
                  bundle_variable.set_rename(*var, rendered_name);
                } else {
                  bundle_variable.set_var_uniq_rename(*var);
                }

                if is_reference_by_another {
                  if module_analyzer_manager.is_commonjs(module_id) {
                    bundle_reference1.add_declare_commonjs_import(
                      &ImportSpecifierInfo::Default(default_name()?),
                      ReferenceKind::Module((*module_id).clone()),
                      &bundle_variable,
                    )?;
                  }

                  if is_entry {
                    bundle_reference1.add_local_export(specify, module_system.clone(), is_entry);
                  } else {
                    bundle_reference1.add_local_export(
                      &ExportSpecifierInfo::Named((*var).into()),
                      module_system.clone(),
                      is_entry,
                    );
                  }
                }
              }

              // export * as ns from 'person'
              ExportSpecifierInfo::Namespace(ns) => {
                let mut bundle_variable = self.bundle_variable.borrow_mut();
                let source = export
                  .source
                  .as_ref()
                  .to_result("namespace should have source, but not found")?;

                let local_var = module_analyzer_manager
                  .module_global_uniq_name
                  .namespace_name(source)
                  .to_result(format!("not found module {:?} namespace named", source))?;

                let target = bundle_variable.find_ident_by_index(
                  local_var,
                  source,
                  module_analyzer_manager,
                  group_id.clone(),
                  false,
                  true,
                );

                if let Some(target) = target {
                  let mut is_confirmed_import = false;
                  let target_source = target.target_source();
                  let module_system =
                    module_system.merge(target.module_system().unwrap_or(module_system.clone()));
                  match target {
                    FindModuleExportResult::Local { .. } => {
                      let local_name = bundle_variable.render_name(local_var);

                      bundle_variable.set_rename(*ns, local_name);

                      is_confirmed_import = true;
                      if module_analyzer_manager.is_commonjs(source) {
                        bundle_reference1.add_declare_commonjs_import(
                          &ImportSpecifierInfo::Namespace(local_var),
                          source.clone().into(),
                          &bundle_variable,
                        )?;
                      }

                      if is_reference_by_another {
                        bundle_reference1.add_local_export(
                          &ExportSpecifierInfo::Named((local_var, Some(*ns)).into()),
                          module_system,
                          is_entry,
                        );
                      }
                    }

                    FindModuleExportResult::External(_, _, _) => {
                      let local_name = bundle_variable.render_name(local_var);

                      bundle_variable.set_rename(*ns, local_name);

                      if is_format_to_commonjs {
                        is_confirmed_import = true;
                        bundle_reference1.add_import(
                          &ImportSpecifierInfo::Namespace(*ns),
                          source.clone().into(),
                          &bundle_variable,
                        )?;
                      }

                      if is_reference_by_another {
                        is_confirmed_import = true;
                        bundle_reference1.add_reference_export(
                          specify,
                          source.clone().into(),
                          module_system,
                          is_entry,
                        );
                      }
                    }

                    FindModuleExportResult::Bundle(index, target_id, _, _) => {
                      is_confirmed_import = true;

                      let is_commonjs = module_analyzer_manager.is_commonjs(&target_id);

                      let is_same_bundle = if is_commonjs {
                        module_analyzer_manager.is_same_bundle(module_id, &target_id)
                      } else {
                        bundle_variable.set_var_root(
                          *ns,
                          module_analyzer_manager
                            .module_global_uniq_name
                            .namespace_name_result(&target_id)?,
                        );

                        bundle_variable.is_same_bundle_by_root(
                          index,
                          &group_id,
                          module_analyzer_manager,
                        )
                      };

                      if !is_same_bundle {
                        bundle_reference1.add_import(
                          &ImportSpecifierInfo::Named {
                            local: if is_commonjs {
                              module_analyzer_manager
                                .module_global_uniq_name
                                .commonjs_name_result(&target_id)?
                            } else {
                              module_analyzer_manager
                                .module_global_uniq_name
                                .namespace_name_result(&target_id)?
                            }
                            .into(),
                            imported: None,
                          },
                          target_id.clone().into(),
                          &bundle_variable,
                        )?;

                        if is_commonjs {
                          bundle_reference1.add_declare_commonjs_import(
                            &ImportSpecifierInfo::Namespace(*ns),
                            target_id.clone().into(),
                            &bundle_variable,
                          )?;
                        }

                        if is_reference_by_another {
                          let target_reference = bundle_reference_manager
                            .reference_mut_by_module(&target_id, &module_analyzer_manager);
                          let mut target_reference = target_reference.borrow_mut();

                          target_reference.add_local_export(
                            &ExportSpecifierInfo::Named(
                              if is_commonjs {
                                module_analyzer_manager
                                  .module_global_uniq_name
                                  .commonjs_name_result(target_id)?
                              } else {
                                module_analyzer_manager
                                  .module_global_uniq_name
                                  .namespace_name_result(target_id)?
                              }
                              .into(),
                            ),
                            module_system.clone(),
                            is_entry,
                          );
                        }
                      }

                      bundle_reference1.add_local_export(
                        &ExportSpecifierInfo::Named((*ns).into()),
                        module_system,
                        is_entry,
                      );
                    }
                  }

                  if !is_confirmed_import {
                    bundle_reference1.add_empty_import(target_source);
                  }
                }
              }
            }
          }
        }
      }

      if module_analyzer_manager.is_commonjs(module_id) {
        if module_analyzer.entry {
          try_reexport_entry_module(
            self.group.group_type.clone(),
            &mut bundle_reference1,
            module_id,
            module_system.clone(),
            is_entry,
          )?;
        }
        // fix multiple bundle reexport
        else if is_reference_by_another
          && let Some(ns) = module_analyzer_manager
            .module_global_uniq_name
            .commonjs_name(module_id)
        {
          bundle_reference1.add_local_export(
            &ExportSpecifierInfo::Named(ns.into()),
            module_system,
            is_entry,
          );
        }
      }
    }

    Ok(())
  }

  // 3. start process bundle
  pub fn render(&mut self, module_analyzer_manager: &mut ModuleAnalyzerManager) -> Result<()> {
    self.strip_module(module_analyzer_manager)?;
    Ok(())
  }

  /// 3-4
  /// 1. strip or remove import/export
  /// 2. generate import/export, e.g: module from external or other bundle
  pub fn patch_ast(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    order_index_map: &HashMap<ModuleId, usize>,
    bundle_reference_manager: &mut BundleReferenceManager,
    ctx: &ShareBundleContext,
  ) -> Result<()> {
    farm_profile_function!("");

    let mut commonjs_import_executed: HashSet<ModuleId> = HashSet::default();
    let external_config = ExternalConfig::from(self.context.config.as_ref());

    let mut patch_export_to_module = vec![];
    let mut patch_import_to_module = vec![];
    let mut patch_after_import_to_module = vec![];

    let mut is_polyfilled_es_module_flag = false;
    let mut already_redeclare: HashSet<ReferenceKind> = HashSet::default();

    // sort by order
    // 1. sort commonjs declaration to top
    // 2. commonjs import declaration should use after declaration or first module use
    for module_id in self.ordered_modules.iter() {
      // let bundle_reference = bundle_reference_manager.reference1_mut(&module_id);
      // let mut bundle_reference = bundle_reference.borrow_mut();
      let bundle_reference =
        bundle_reference_manager.reference_mut_by_module(&module_id, &module_analyzer_manager);
      let mut bundle_reference = bundle_reference.borrow_mut();
      // let module_of_bundle_reference =
      //   bundle_reference.fetch_mut(module_id, module_analyzer_manager);

      module_analyzer_manager.patch_module_analyzer_ast(
        module_id,
        &self.context,
        &mut self.bundle_variable.borrow_mut(),
        &mut bundle_reference,
        &mut commonjs_import_executed,
        order_index_map,
        &mut self.polyfill,
        &external_config,
        ctx,
      )?;

      let reference_kind = (*module_id).into();

      let result = if let Some(map) = bundle_reference
        .redeclare_commonjs_import
        .get(&reference_kind)
      {
        already_redeclare.insert(reference_kind.clone());
        let map = HashMap::from_iter([(reference_kind, map.clone())]);
        CjsModuleAnalyzer::redeclare_commonjs_export(
          &self.bundle_variable.borrow(),
          &map,
          &module_analyzer_manager.module_global_uniq_name,
          &mut self.polyfill,
          ctx,
        )?
      } else {
        vec![]
      };

      let module_analyzer = module_analyzer_manager.module_analyzer_mut_unchecked(&module_id);

      module_analyzer.ast.body.extend(result);
    }

    let bundle_commonjs_declare_map: CommonJsImportMap = CommonJsImportMap::default();

    let bundle_reference = bundle_reference_manager.reference_mut(&self.group.id);
    let mut bundle_reference = bundle_reference.borrow_mut();

    let map = bundle_reference
      .redeclare_commonjs_import
      .clone()
      .into_iter()
      .filter(|(key, _)| !already_redeclare.contains(&key))
      .collect();

    patch_after_import_to_module.extend(CjsModuleAnalyzer::redeclare_commonjs_export(
      &self.bundle_variable.borrow(),
      &map,
      &module_analyzer_manager.module_global_uniq_name,
      &mut self.polyfill,
      ctx,
    )?);

    for module_id in &self.ordered_modules {
      module_analyzer_manager.patch_rename(
        module_id,
        &self.context,
        &mut self.bundle_variable.borrow_mut(),
        &bundle_commonjs_declare_map,
      );
    }

    let is_runtime_bundle = matches!(self.group.group_type, ResourcePotType::Runtime);

    if !is_runtime_bundle {
      patch_export_to_module.extend(generate_export_by_reference_export(
        &self.group.id,
        true,
        &self.bundle_variable.borrow(),
        &mut bundle_reference.reexport_raw,
        module_analyzer_manager,
        &self.context,
        &mut self.polyfill,
        &mut is_polyfilled_es_module_flag,
        ctx,
      )?);

      patch_export_to_module.extend(generate_export_by_reference_export(
        &self.group.id,
        false,
        &self.bundle_variable.borrow(),
        &mut bundle_reference.bundle_reference1,
        module_analyzer_manager,
        &self.context,
        &mut self.polyfill,
        &mut is_polyfilled_es_module_flag,
        ctx,
      )?);
    }

    // 2. maybe import external、other bundle, should generate import
    patch_import_to_module.extend(generate_bundle_import_by_bundle_reference(
      &ctx.options.format,
      &self.bundle_variable.borrow(),
      &bundle_reference,
      module_analyzer_manager,
      &mut self.polyfill,
      &self.group.id,
      ctx,
    )?);

    // patch_import_to_module.extend(generate_bundle_import_by_bundle_reference(
    //   &ctx.options.format,
    //   &self.bundle_variable.borrow(),
    //   &bundle_reference,
    //   module_analyzer_manager,
    //   &mut self.polyfill,
    //   &self.group.id,
    //   ctx,
    // )?);

    patch_import_to_module.extend(patch_after_import_to_module);

    if !patch_import_to_module.is_empty() {
      if let Some(module_analyzer) = self
        .ordered_modules
        .first()
        .and_then(|item| module_analyzer_manager.module_analyzer_mut(item))
      {
        let ast = &mut module_analyzer.ast;

        ast.body = patch_import_to_module
          .into_iter()
          .chain(ast.body.take())
          .collect();
      };
    }

    if !patch_export_to_module.is_empty() {
      if let Some(module_analyzer) = self
        .ordered_modules
        .last()
        .and_then(|id| module_analyzer_manager.module_analyzer_mut(id))
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

  pub fn patch_polyfill_for_bundle(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    context: &ShareBundleContext,
  ) -> Result<()> {
    if let Some(module_id) = self.ordered_modules.first() {
      let module_analyzer = module_analyzer_manager.module_analyzer_mut_unchecked(module_id);

      let mut bundle_reference = CombineBundleReference::new();

      let bundle_variable = self.bundle_variable.borrow_mut();

      for name in self.polyfill.to_export() {
        if let Some(index) = &bundle_variable.polyfill_index_map.get(&name) {
          bundle_reference.add_import(
            &ImportSpecifierInfo::Named {
              local: **index,
              imported: None,
            },
            ModuleId::from(FARM_BUNDLE_POLYFILL_SLOT).into(),
            &bundle_variable,
          )?;
        }
      }

      let mut ast = module_analyzer.ast.body.take();

      let stmts = generate_bundle_import_by_bundle_reference(
        &context.options.format,
        &bundle_variable,
        &bundle_reference,
        &module_analyzer_manager,
        &mut self.polyfill,
        &self.group.id,
        context,
      )?;

      ast = stmts.into_iter().chain(ast.take()).collect();

      module_analyzer_manager.set_ast_body(&module_id, ast);
    }

    Ok(())
  }

  // TODO: for partial ShareBundle
  pub fn patch_polyfill_inline(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
  ) -> Result<()> {
    if let Some(module_id) = self.ordered_modules.first() {
      let module_analyzer = module_analyzer_manager.module_analyzer_mut_unchecked(module_id);

      let mut ast = module_analyzer.ast.body.take();

      ast = self
        .polyfill
        .to_ast()?
        .into_iter()
        .chain(ast.take())
        .collect();

      module_analyzer_manager.set_ast_body(&module_id, ast);
    }

    Ok(())
  }

  pub fn patch_polyfill(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    polyfill: SimplePolyfill,
    context: &ShareBundleContext,
  ) -> Result<()> {
    let module_id = ModuleId::from(FARM_BUNDLE_POLYFILL_SLOT);
    let module_analyzer = module_analyzer_manager
      .module_analyzer_mut(&module_id)
      .to_result(format!("cannot found module {:?}", module_id))?;

    let mut ast = module_analyzer.ast.body.take();

    ast = [
      polyfill.to_ast().into_iter().flatten().collect::<Vec<_>>(),
      ast,
    ]
    .concat();

    let mut bundle_reference = CombineBundleReference::new();

    for name in polyfill.to_export() {
      if let Some(index) = &self.bundle_variable.borrow().polyfill_index_map.get(&name) {
        bundle_reference.add_local_export(
          &ExportSpecifierInfo::Named((**index).into()),
          module_analyzer.module_system.clone(),
          false,
        );
      }
    }

    let group_id = module_analyzer.bundle_group_id.clone();

    let stmts = generate_export_by_reference_export(
      &group_id,
      false,
      &self.bundle_variable.borrow(),
      &mut bundle_reference.bundle_reference1,
      &module_analyzer_manager,
      &self.context,
      &mut SimplePolyfill::default(),
      &mut false,
      context,
    )?;

    ast = ast.into_iter().chain(stmts).collect();

    module_analyzer_manager.set_ast_body(&module_id, ast);

    Ok(())
  }

  // step: 4 generate bundle code
  // pub fn codegen(
  //   &mut self,
  //   module_analyzer_manager: &mut ModuleAnalyzerManager,
  //   config: &Config,
  // ) -> Result<Bundle> {
  //   let mut bundle = Bundle::new(BundleOptions {
  //     separator: Some('\n'),
  //     intro: None,
  //     trace_source_map_chain: Some(false),
  //   });

  //   for module_id in &self.ordered_modules {
  //     let module = self
  //       .module_graph
  //       .module(module_id)
  //       .unwrap_or_else(|| panic!("Module not found: {module_id:?}"));
  //     let module_analyzer = module_analyzer_manager.module_analyzer_mut_unchecked(module_id);

  //     let comments: SingleThreadedComments = module.meta.as_script().comments.clone().into();

  //     let sourcemap_enabled = self.context.config.sourcemap.enabled(module.immutable);

  //     try_with(
  //       module_analyzer.cm.clone(),
  //       &self.context.meta.script.globals,
  //       || {
  //         module_analyzer
  //           .ast
  //           .visit_mut_with(&mut fixer(Some(&comments)));
  //       },
  //     )?;

  //     let mut mappings = vec![];
  //     let code_bytes = codegen_module(
  //       &module_analyzer.ast,
  //       self.context.config.script.target,
  //       module_analyzer.cm.clone(),
  //       if sourcemap_enabled {
  //         Some(&mut mappings)
  //       } else {
  //         None
  //       },
  //       false,
  //       Some(CodeGenCommentsConfig {
  //         comments: &comments,
  //         config: &self.context.config.comments,
  //       }),
  //     )
  //     .map_err(|err| CompilationError::RenderScriptModuleError {
  //       id: module_analyzer.module_id.to_string(),
  //       source: Some(Box::new(err)),
  //     })?;

  //     let code = String::from_utf8(code_bytes).map_err(|err| {
  //       CompilationError::GenericError(format!(
  //         "failed to convert code bytes to string, origin error: {err}"
  //       ))
  //     })?;

  //     let mut source_map_chain = vec![];

  //     if sourcemap_enabled {
  //       let sourcemap = build_source_map(module_analyzer.cm.clone(), &mappings);
  //       let mut buf = vec![];
  //       sourcemap
  //         .to_writer(&mut buf)
  //         .map_err(|e| CompilationError::RenderScriptModuleError {
  //           id: module_id.to_string(),
  //           source: Some(Box::new(e)),
  //         })?;
  //       let map = Arc::new(String::from_utf8(buf).unwrap());

  //       source_map_chain.clone_from(&module.source_map_chain);
  //       source_map_chain.push(map);
  //     }

  //     let mut module = MagicString::new(
  //       &code,
  //       Some(MagicStringOptions {
  //         filename: Some(module_id.resolved_path_with_query(&self.context.config.root)),
  //         source_map_chain,
  //         ..Default::default()
  //       }),
  //     );

  //     if matches!(self.context.config.mode, Mode::Development) {
  //       // debug info
  //       module.prepend(&format!("// module_id: {}\n", module_id.to_string()));
  //     }

  //     bundle.add_source(module, None).unwrap();
  //   }

  //   // in browser, should avoid naming pollution
  //   if matches!(self.context.config.output.target_env, TargetEnv::Browser)
  //     && matches!(self.group.group_type, ResourcePotType::Runtime)
  //   {
  //     bundle.prepend(";((function(){");
  //     bundle.append("})());", None);
  //   };

  //   Ok(bundle)
  // }

  pub fn gen_ast(
    self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
  ) -> Result<GeneratorAstResult> {
    let cm = self
      .context
      .meta
      .merge_modules_source_mpa(&self.ordered_modules, self.module_graph);

    let get_start_pos = |module_id: &ModuleId| {
      let filename = get_swc_sourcemap_filename(module_id);
      let Some(source_file) = cm.get_source_file(&filename) else {
        panic!("no source file found for {:?}", module_id);
      };
      source_file.start_pos
    };

    let modules = self
      .ordered_modules
      .iter()
      .map(|module_id| {
        let Some(module) = self.module_graph.module(module_id) else {
          panic!("Module not found: {module_id:?}")
        };
        let module_analyzer = module_analyzer_manager.module_analyzer_mut_unchecked(module_id);

        let comments = module.meta.as_script().comments.clone();

        let module = module_analyzer.ast.take();

        (module_id, (module, comments))
      })
      .par_bridge()
      .map(|(module_id, (mut module, data))| {
        module.visit_mut_with(&mut SpanUpdater {
          start_pos: get_start_pos(module_id),
        });

        (module_id, module, data)
      })
      .collect::<Vec<_>>();

    let (ast, comments) = modules.into_iter().fold(
      (
        Module {
          span: DUMMY_SP,
          body: vec![],
          shebang: None,
        },
        SingleThreadedComments::default(),
      ),
      |(mut module, comments), (module_id, ast, data)| {
        module.body.extend(ast.body);

        let start_pos = get_start_pos(module_id);

        for item in data.leading {
          let byte_pos = start_pos + item.byte_pos;

          for comment in item.comment {
            comments.add_leading(byte_pos, comment);
          }
        }

        for item in data.trailing {
          let byte_pos = start_pos + item.byte_pos;

          for comment in item.comment {
            comments.add_trailing(byte_pos, comment);
          }
        }

        (module, comments)
      },
    );

    Ok(GeneratorAstResult {
      ast,
      comments: comments.into(),
      rendered_modules: self.ordered_modules.into_iter().cloned().collect(),
    })
  }
}
