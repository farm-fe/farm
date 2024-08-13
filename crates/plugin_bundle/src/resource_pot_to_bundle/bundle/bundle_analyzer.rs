use std::{
  cell::RefCell,
  cmp::Ordering,
  collections::{HashMap, HashSet},
  rc::Rc,
  sync::Arc,
};

use farmfe_core::{
  config::{external::ExternalConfig, Config, Mode, ModuleFormat, TargetEnv},
  context::CompilationContext,
  enhanced_magic_string::{
    bundle::{Bundle, BundleOptions},
    magic_string::{MagicString, MagicStringOptions},
  },
  error::{CompilationError, Result},
  farm_profile_function, farm_profile_scope,
  module::{module_graph::ModuleGraph, ModuleId, ModuleSystem, ModuleType},
  resource::resource_pot::{ResourcePot, ResourcePotType},
  swc_common::{comments::SingleThreadedComments, util::take::Take},
};
use farmfe_toolkit::{
  common::build_source_map,
  script::{codegen_module, swc_try_with::try_with, CodeGenCommentsConfig},
  swc_ecma_transforms::fixer,
  swc_ecma_visit::VisitMutWith,
};

use crate::resource_pot_to_bundle::{
  bundle::bundle_reference::{ReferenceBuilder, ReferenceKind},
  common::OptionToResult,
  modules_analyzer::module_analyzer::{
    ExportSpecifierInfo, ImportSpecifierInfo, StmtAction, Variable,
  },
  polyfill::SimplePolyfill,
  targets::generate::{
    generate_bundle_import_by_bundle_reference, generate_export_by_reference_export,
  },
  uniq_name::{BundleVariable, FindModuleExportResult},
};

use super::{bundle_reference::BundleReferenceManager, BundleReference, ModuleAnalyzerManager};

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

  // pub bundle_reference: BundleReference,
  pub polyfill: SimplePolyfill,
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
      // bundle_reference: BundleReference::new(),
      // bundle level polyfill
      polyfill: SimplePolyfill::default(),
    }
  }

  pub fn set_namespace(&mut self, resource_pot_id: &str) {
    self
      .bundle_variable
      .borrow_mut()
      .set_namespace(resource_pot_id.to_string());
  }

  // step: 1 toposort fetch modules
  pub fn build_module_order(&mut self, order_index_map: &HashMap<ModuleId, usize>) {
    farm_profile_function!();
    let mut resource_pot_modules = self.resource_pot.modules();

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

        for index in variables {
          // used_name is global variable, we need rename same used_name in cjs
          if is_commonjs && !(self.bundle_variable.borrow().is_in_used_name(index)) {
            continue;
          }

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
    farm_profile_function!("");
    for module_id in &self.ordered_modules {
      farm_profile_scope!(format!("bundle strip module: {}", module_id.to_string()));
      let mut stmt_action = HashSet::new();
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
  // TODO: refactor bundle_reference logic
  pub fn link_module_relation(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    bundle_reference_manager: &mut BundleReferenceManager,
  ) -> Result<()> {
    farm_profile_function!("");

    let bundle_reference = bundle_reference_manager.reference_mut(&self.resource_pot.id);
    let mut bundle_reference = bundle_reference.borrow_mut();

    let is_format_to_commonjs = self.context.config.output.format == ModuleFormat::CommonJs;

    for module_id in &self.ordered_modules {
      farm_profile_scope!(format!(
        "bundle analyzer module relation: {}",
        module_id.to_string()
      ));

      if let Some(module_analyzer) = module_analyzer_manager.module_analyzer(module_id) {
        let resource_pot_id = module_analyzer.resource_pot_id.clone();
        let module_system = module_analyzer.module_system.clone();
        let is_reference_by_another = module_analyzer.entry || {
          let importer = self.module_graph.dependents_ids(module_id);
          importer.iter().any(|importer| {
            module_analyzer_manager
              .module_analyzer(importer)
              .is_some_and(|i| i.resource_pot_id != resource_pot_id)
          })
        };

        println!(
          "\n\nmodule_analyzer: {}\nresource_pot_id: {}\nis_reference_by_another: {}",
          module_analyzer.module_id.to_string(),
          resource_pot_id,
          is_reference_by_another
        );

        let mut is_contain_export = false;

        for statement in &module_analyzer.statements {
          if let Some(import) = &statement.import {
            for specify in &import.specifiers {
              match specify {
                // import * as person from "person"
                ImportSpecifierInfo::Namespace(ns) => {
                  let target = self.bundle_variable.borrow().find_ident_by_index(
                    *ns,
                    &import.source,
                    module_analyzer_manager,
                    resource_pot_id.clone(),
                    false,
                    true,
                  );

                  if let Some(target) = target {
                    let is_common_js = target.is_common_js();

                    match target {
                      FindModuleExportResult::Local(_, target_module_id, _) => {
                        if let Some(mut uniq_ns) = module_analyzer_manager
                          .module_global_uniq_name
                          .namespace_name(&target_module_id)
                        {
                          if is_common_js {
                            uniq_ns = bundle_reference.add_declare_commonjs_import(
                              &ImportSpecifierInfo::Namespace(uniq_ns),
                              target_module_id.into(),
                              &self.bundle_variable.borrow(),
                            )?;
                          }

                          self
                            .bundle_variable
                            .borrow_mut()
                            .set_rename_from_other_render_name(*ns, uniq_ns);
                        }
                      }

                      FindModuleExportResult::External(_, _, _) => {
                        bundle_reference.add_import(
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
                          .set_rename_from_other_render_name(*ns, rename);
                      }

                      // TODO: bundle
                      FindModuleExportResult::Bundle(_, bundle_name, _, _) => {
                        let import_rename = bundle_reference.add_import(
                          specify,
                          bundle_name.into(),
                          &self.bundle_variable.borrow(),
                        )?;

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_rename_from_other_render_name(*ns, import_rename);
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
                    resource_pot_id.clone(),
                    self.bundle_variable.borrow().name(imported) == "default",
                    false,
                  );

                  if let Some(target) = target {
                    let is_common_js = target.is_common_js();
                    let is_reexport = target.is_reexport();
                    match target {
                      FindModuleExportResult::Local(mut index, target_source, _) => {
                        if is_common_js {
                          index = bundle_reference.add_declare_commonjs_import(
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

                      FindModuleExportResult::External(_, target, _) => {
                        let rename = bundle_reference.add_import(
                          specify,
                          target.into(),
                          &self.bundle_variable.borrow(),
                        )?;

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_uniq_name_both(rename, *local);
                      }

                      FindModuleExportResult::Bundle(_, target, _, _) => {
                        // let br = bundle_reference_manager.reference_mut(&target);
                        let rename = bundle_reference.add_import(
                          specify,
                          target.into(),
                          &self.bundle_variable.borrow(),
                        )?;
                        // br.borrow_mut().add_local_export(target, module_system);

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_uniq_name_both(rename, *local);
                      }
                    }
                  }
                }

                // import person from "person"
                ImportSpecifierInfo::Default(default) => {
                  let target = self.bundle_variable.borrow().find_ident_by_index(
                    *default,
                    &import.source,
                    module_analyzer_manager,
                    resource_pot_id.clone(),
                    true,
                    false,
                  );

                  if let Some(target) = target {
                    let is_common_js = target.is_common_js();

                    println!("import default target: {:#?}", target);
                    match target {
                      FindModuleExportResult::Local(mut index, target_source, _) => {
                        if is_common_js {
                          index = bundle_reference.add_declare_commonjs_import(
                            specify,
                            target_source.into(),
                            &self.bundle_variable.borrow(),
                          )?;
                        }

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_uniq_name_both(index, *default);
                      }

                      FindModuleExportResult::External(_, target, _) => {
                        let rename = bundle_reference.add_import(
                          specify,
                          target.into(),
                          &self.bundle_variable.borrow(),
                        )?;

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_uniq_name_both(rename, *default);
                      }

                      FindModuleExportResult::Bundle(i, target, _, _) => {
                        let rename: usize = bundle_reference.add_import(
                          specify,
                          target.into(),
                          &self.bundle_variable.borrow(),
                        )?;

                        println!("target import default: {:#?}", bundle_reference);

                        self
                          .bundle_variable
                          .borrow_mut()
                          .set_uniq_name_both(rename, *default);
                      }
                    }
                  };
                }
              }
            }
          }

          if let Some(export) = &statement.export {
            is_contain_export = true;
            if module_analyzer_manager.is_commonjs(module_id) && !is_reference_by_another {
              continue;
            }

            for specify in &export.specifiers {
              match specify {
                // export * from 'person'
                ExportSpecifierInfo::All(_) => {
                  let Some(source) = &export.source else {
                    unreachable!("export all should have source")
                  };

                  bundle_reference.add_reexport_all(ReferenceBuilder {
                    is_reference_by_another_bundle: is_reference_by_another,
                    module_analyzer_manager: &module_analyzer_manager,
                    module_analyzer,
                    bundle_variable: &mut self.bundle_variable.borrow_mut(),
                    source,
                    module_system: module_system.clone(),
                    config: &self.context.config,
                    module_id,
                  })?;
                }

                // export { name as personName }
                // export { name as personName } from './person';
                ExportSpecifierInfo::Named(variable) => {
                  if let Some(source) = &export.source {
                    let is_find_default =
                      self.bundle_variable.borrow().name(variable.local()) == "default";
                    let target = self.bundle_variable.borrow_mut().find_ident_by_index(
                      variable.local(),
                      source,
                      module_analyzer_manager,
                      resource_pot_id.clone(),
                      is_find_default,
                      false,
                    );

                    if let Some(target) = target {
                      let is_common_js = target.is_common_js();
                      let mut is_confirmed_import = false;
                      let target_source = target.target_source();
                      let module_system = module_system
                        .merge(target.module_system().unwrap_or(module_system.clone()));

                      match target {
                        FindModuleExportResult::Local(local, target_source, _) => {
                          is_confirmed_import = true;
                          let is_default_key = self.bundle_variable.borrow().is_default_key(local);

                          let name = if is_default_key {
                            module_analyzer_manager
                              .module_global_uniq_name
                              .default_name_result(&target_source)?
                          } else {
                            local
                          };

                          if is_common_js {
                            self.bundle_variable.borrow_mut().set_var_uniq_rename(local);

                            bundle_reference.add_declare_commonjs_import(
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
                              &self.bundle_variable.borrow(),
                            )?;
                          }

                          if is_reference_by_another {
                            bundle_reference.add_local_export(
                              &ExportSpecifierInfo::Named(Variable(
                                name,
                                Some(variable.export_as()),
                              )),
                              module_system,
                            );
                          }
                        }
                        FindModuleExportResult::External(_, target_source, _) => {
                          if is_reference_by_another {
                            bundle_reference.add_reference_export(
                              specify,
                              target_source.into(),
                              module_system,
                            );
                            is_confirmed_import = true;
                          }
                        }

                        FindModuleExportResult::Bundle(_, _, _, _) => {
                          is_confirmed_import = true;
                          // TODO: bundle impl
                        }
                      }

                      if !is_confirmed_import {
                        bundle_reference.add_empty_import(target_source)
                      }
                    }
                  } else {
                    self
                      .bundle_variable
                      .borrow_mut()
                      .set_var_uniq_rename(variable.local());

                    if is_reference_by_another {
                      if module_analyzer_manager.is_commonjs(module_id) {
                        let is_default_key = self
                          .bundle_variable
                          .borrow()
                          .is_default_key(variable.local());

                        bundle_reference.add_declare_commonjs_import(
                          &ImportSpecifierInfo::Named {
                            local: if is_default_key {
                              module_analyzer_manager
                                .module_global_uniq_name
                                .default_name_result(*module_id)?
                            } else {
                              variable.local()
                            },
                            imported: Some(variable.export_as()),
                          },
                          ReferenceKind::Module((*module_id).clone()),
                          &self.bundle_variable.borrow(),
                        )?;
                      }

                      bundle_reference.add_local_export(specify, module_system.clone());
                    }
                  }
                }

                // export default n, Default(n)
                // export default 1 + 1, Default("default")
                ExportSpecifierInfo::Default(var) => {
                  let default_name = || {
                    module_analyzer_manager
                      .module_global_uniq_name
                      .default_name_result(*module_id)
                  };

                  if self.bundle_variable.borrow().name(*var) == "default" {
                    let rendered_name = self.bundle_variable.borrow().render_name(default_name()?);

                    self
                      .bundle_variable
                      .borrow_mut()
                      .set_rename(*var, rendered_name);
                  } else {
                    self.bundle_variable.borrow_mut().set_var_uniq_rename(*var);
                  }

                  if is_reference_by_another {
                    if module_analyzer_manager.is_commonjs(module_id) {
                      bundle_reference.add_declare_commonjs_import(
                        &ImportSpecifierInfo::Default(default_name()?),
                        ReferenceKind::Module((*module_id).clone()),
                        &self.bundle_variable.borrow(),
                      )?;
                    }

                    bundle_reference.add_local_export(specify, module_system.clone());
                  }
                }

                // export * as ns from 'person'
                ExportSpecifierInfo::Namespace(ns) => {
                  let source = export
                    .source
                    .as_ref()
                    .to_result("namespace should have source, but not found")?;

                  let local_var = module_analyzer_manager
                    .module_global_uniq_name
                    .namespace_name(source)
                    .to_result(format!("not found module {source:?} namespace named"))?;

                  let local_name = self.bundle_variable.borrow().render_name(local_var);

                  self
                    .bundle_variable
                    .borrow_mut()
                    .set_rename(*ns, local_name);

                  let target = self.bundle_variable.borrow().find_ident_by_index(
                    local_var,
                    source,
                    module_analyzer_manager,
                    resource_pot_id.clone(),
                    false,
                    true,
                  );

                  if let Some(target) = target {
                    let mut is_confirmed_import = false;
                    let target_source = target.target_source();
                    let module_system =
                      module_system.merge(target.module_system().unwrap_or(module_system.clone()));
                    match target {
                      FindModuleExportResult::Local(_, _, _) => {
                        is_confirmed_import = true;
                        if module_analyzer_manager.is_commonjs(source) {
                          bundle_reference.add_declare_commonjs_import(
                            &ImportSpecifierInfo::Namespace(local_var),
                            source.clone().into(),
                            &self.bundle_variable.borrow(),
                          )?;
                        }

                        if is_reference_by_another {
                          bundle_reference.add_local_export(
                            &ExportSpecifierInfo::Named((local_var, Some(*ns)).into()),
                            module_system,
                          );
                        }
                      }

                      FindModuleExportResult::External(_, _, _) => {
                        if is_format_to_commonjs {
                          is_confirmed_import = true;
                          bundle_reference.add_import(
                            &ImportSpecifierInfo::Namespace(*ns),
                            source.clone().into(),
                            &self.bundle_variable.borrow(),
                          )?;
                        }

                        if is_reference_by_another {
                          is_confirmed_import = true;
                          bundle_reference.add_reference_export(
                            specify,
                            source.clone().into(),
                            module_system,
                          );
                        }
                      }

                      FindModuleExportResult::Bundle(_, _, _, _) => {
                        // bundle
                        // export * as ns from './other_bundle_module'
                        // bundle_reference.sync_export(
                        //   &ExportSpecifierInfo::Named((*ns).into()),
                        //   Some(module_analyzer.resource_pot_id.clone().into()),
                        //   false,
                        // );
                      }
                    }

                    if !is_confirmed_import {
                      bundle_reference.add_empty_import(target_source);
                    }
                  }
                }
              }
            }
          }
        }

        println!(
          "\nis_contain_export: {:?}\nis_commonjs: {:?}\nentry: {:?}",
          is_contain_export,
          module_analyzer_manager.is_commonjs(module_id),
          module_analyzer.entry
        );

        if !is_contain_export && module_analyzer_manager.is_commonjs(module_id) {
          if module_analyzer.entry {
            let reference_kind = ReferenceKind::Module((*module_id).clone());
            bundle_reference.execute_module_for_cjs(reference_kind);
          } else if is_reference_by_another {
            bundle_reference.add_local_export(
              &ExportSpecifierInfo::Named(
                module_analyzer_manager
                  .module_global_uniq_name
                  .commonjs_name_result(*module_id)?
                  .into(),
              ),
              ModuleSystem::CommonJs,
            );
          }
        }
      }
    }

    Ok(())
  }

  // 3. start process bundle
  pub fn render(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    order_index_map: &HashMap<ModuleId, usize>,
    bundle_reference: &mut BundleReferenceManager,
  ) -> Result<()> {
    self.module_conflict_name(module_analyzer_manager);

    self.strip_module(module_analyzer_manager)?;

    self.link_module_relation(module_analyzer_manager, bundle_reference)?;

    self.patch_ast(module_analyzer_manager, order_index_map, bundle_reference)?;

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
  ) -> Result<()> {
    farm_profile_function!("");

    let bundle_reference = bundle_reference_manager.reference_mut(&self.resource_pot.id);

    let mut commonjs_import_executed: HashSet<ModuleId> = HashSet::new();
    let external_config = ExternalConfig::from(self.context.config.as_ref());

    for module_id in &self.ordered_modules {
      farm_profile_scope!(format!(
        "bundle patch ast module: {}",
        module_id.to_string()
      ));
      module_analyzer_manager.patch_module_analyzer_ast(
        module_id,
        &self.context,
        self.module_graph,
        &mut self.bundle_variable.borrow_mut(),
        &mut bundle_reference_manager
          .reference_mut(&self.resource_pot.id)
          .borrow_mut(),
        &mut commonjs_import_executed,
        order_index_map,
        &mut self.polyfill,
        &external_config,
      )?;
    }

    let is_runtime_bundle = matches!(
      self.resource_pot.resource_pot_type,
      ResourcePotType::Runtime
    );

    let mut patch_export_to_module = vec![];
    let mut patch_import_to_module = vec![];

    // runtime bundle cannot export
    // 1. if import by other bundle or entry export, should reexport some variable
    if !is_runtime_bundle {
      patch_export_to_module.extend(generate_export_by_reference_export(
        &self.resource_pot.id,
        &self.bundle_variable.borrow(),
        &mut bundle_reference.borrow_mut(),
        module_analyzer_manager,
        &self.context,
        &mut self.polyfill,
      )?);
    }

    // 2. maybe import external、other bundle, should generate import
    patch_import_to_module.extend(generate_bundle_import_by_bundle_reference(
      &self.context.config.output.format,
      &self.bundle_variable.borrow(),
      &bundle_reference.borrow(),
      module_analyzer_manager,
      &mut self.polyfill,
      &self.resource_pot.id,
    )?);

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

  // step: 4 generate bundle code
  pub fn codegen(
    &mut self,
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    config: &Config,
  ) -> Result<Bundle> {
    let mut bundle = Bundle::new(BundleOptions {
      separator: Some('\n'),
      intro: None,
      trace_source_map_chain: Some(false),
    });

    for module_id in &self.ordered_modules {
      let module = self
        .module_graph
        .module(module_id)
        .unwrap_or_else(|| panic!("Module not found: {module_id:?}"));
      let module_analyzer = module_analyzer_manager.module_analyzer_mut_unchecked(module_id);

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
          "failed to convert code bytes to string, origin error: {err}"
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

        source_map_chain.clone_from(&module.source_map_chain);
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

    // in browser, should avoid naming pollution
    if matches!(self.context.config.output.target_env, TargetEnv::Browser)
      && matches!(
        self.resource_pot.resource_pot_type,
        ResourcePotType::Runtime
      )
    {
      bundle.prepend("((function(){");
      bundle.append("})());", None);
    };

    let injectable_resource_pot = (config.output.target_env.is_library()
      && self.resource_pot.entry_module.is_some())
      || matches!(
        self.resource_pot.resource_pot_type,
        ResourcePotType::Runtime
      );

    if !self.polyfill.is_empty() && injectable_resource_pot {
      for item in self.polyfill.to_str() {
        bundle.prepend(&item);
      }
    }

    Ok(bundle)
  }
}
