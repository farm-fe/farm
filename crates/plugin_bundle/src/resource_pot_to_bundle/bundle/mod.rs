use std::{
  collections::{HashMap, HashSet},
  mem::{self, replace},
  sync::{Arc, Mutex, RwLock},
};

use bundle_external::BundleReference;
use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, MapCompletionError, Result},
  farm_profile_function, farm_profile_scope,
  module::{module_graph::ModuleGraph, ModuleId, ModuleSystem},
  plugin::ResolveKind,
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  swc_common::{util::take::Take, DUMMY_SP},
  swc_ecma_ast::{
    self, BindingIdent, CallExpr, ClassDecl, Decl, EmptyStmt, Expr, ExprStmt, FnDecl, Ident,
    ModuleDecl, ModuleItem, Stmt, VarDecl, VarDeclarator,
  },
};
use farmfe_toolkit::{script::swc_try_with::try_with, swc_ecma_visit::VisitMutWith};

pub mod bundle_analyzer;
pub mod bundle_external;
pub mod bundle_variable;
pub mod reference;
use crate::resource_pot_to_bundle::targets::{
  cjs::patch::CjsPatch, dynamic_import::replace_dynamic_import,
};

use self::reference::ReferenceMap;

use super::{
  defined_idents_collector::RenameIdent,
  modules_analyzer::module_analyzer::{
    ExportSpecifierInfo, ImportSpecifierInfo, ModuleAnalyzer, StmtAction,
  },
  polyfill::SimplePolyfill,
  targets::generate::generate_namespace_by_reference_map,
  uniq_name::BundleVariable,
};
pub struct ModuleAnalyzerManager<'a> {
  pub module_map: HashMap<ModuleId, ModuleAnalyzer>,
  pub namespace_modules: HashSet<ModuleId>,

  ///
  /// ```js
  /// // namespace/moduleA.js
  /// export const foo = 'foo';
  /// export const bar = 'bar'
  ///
  /// // importA.js
  /// export * as moduleA from './moduleA.js';
  ///
  /// // importB.js
  /// export * as moduleB from './moduleA.js';
  /// ```
  ///
  ///
  /// {
  ///   "namespace/moduleA.js": (moduleA, {
  ///       "importA.js": "moduleA",
  ///       "importB.js": "moduleB"
  ///   })
  /// }
  ///
  ///
  pub module_global_uniq_name: ModuleGlobalUniqName,
  module_graph: &'a ModuleGraph,
}

#[derive(Debug)]
struct ModuleGlobalName {
  namespace: Option<usize>,
  default: Option<usize>,
  commonjs: Option<usize>,
}

impl ModuleGlobalName {
  fn new() -> Self {
    Self {
      namespace: None,
      default: None,
      commonjs: None,
    }
  }
}

#[derive(Debug, Default)]
pub struct ModuleGlobalUniqName {
  module_map: HashMap<ModuleId, ModuleGlobalName>,
}

impl ModuleGlobalUniqName {
  fn new() -> Self {
    Self {
      module_map: HashMap::default(),
    }
  }

  pub fn namespace_name(&self, module_id: &ModuleId) -> Option<usize> {
    self
      .module_map
      .get(module_id)
      .and_then(|item| item.namespace)
  }

  pub fn default_name(&self, module_id: &ModuleId) -> Option<usize> {
    self.module_map.get(module_id).and_then(|item| item.default)
  }

  pub fn commonjs_name(&self, module_id: &ModuleId) -> Option<usize> {
    self
      .module_map
      .get(module_id)
      .and_then(|item| item.commonjs)
  }

  fn entry_module(&mut self, module_id: &ModuleId) -> &mut ModuleGlobalName {
    if !self.module_map.contains_key(module_id) {
      self
        .module_map
        .insert(module_id.clone(), ModuleGlobalName::new());
    }

    self.module_map.get_mut(module_id).unwrap()
  }

  fn add_namespace<F: FnOnce(&str) -> usize>(&mut self, module_id: &ModuleId, v: F) {
    let m = self.entry_module(module_id);

    if m.namespace.is_none() {
      m.namespace = Some(v("_ns"));
    }
  }

  fn add_default<F: FnOnce(&str) -> usize>(&mut self, module_id: &ModuleId, v: F) {
    let m = self.entry_module(module_id);

    if m.default.is_none() {
      m.default = Some(v("_default"));
    }
  }

  fn add_commonjs<F: FnOnce(&str) -> usize>(&mut self, module_id: &ModuleId, v: F) {
    let m = self.entry_module(module_id);

    if m.commonjs.is_none() {
      m.commonjs = Some(v("_cjs"));
    }
  }
}

impl Take for ModuleGlobalUniqName {
  fn dummy() -> Self {
    Default::default()
  }
}

impl<'a> ModuleAnalyzerManager<'a> {
  pub fn new(module_map: HashMap<ModuleId, ModuleAnalyzer>, module_graph: &'a ModuleGraph) -> Self {
    Self {
      module_map,
      namespace_modules: HashSet::new(),
      module_global_uniq_name: ModuleGlobalUniqName::new(),
      module_graph,
    }
  }

  pub fn extract_modules_statements(
    &mut self,
    modules: &Vec<&ModuleId>,
    context: &Arc<CompilationContext>,
    module_graph: &ModuleGraph,
    bundle_variable: &mut BundleVariable,
  ) -> Result<()> {
    farm_profile_function!();

    let data = Mutex::new(vec![]);
    let module_map = {
      mem::take(&mut self.module_map)
        .into_iter()
        .map(|(key, val)| (key, Arc::new(RwLock::new(val))))
        .collect::<HashMap<ModuleId, Arc<RwLock<ModuleAnalyzer>>>>()
    };

    modules.into_par_iter().try_for_each(|module_id| {
      let mut module_analyzer = module_map
        .get(module_id)
        .map(|item| item.write().unwrap())
        .unwrap();
      let mut new_bundle_variable = bundle_variable.branch();
      new_bundle_variable.set_namespace(module_analyzer.resource_pot_id.clone());
      farm_profile_scope!(format!(
        "extract module statement: {:?}",
        module_id.to_string()
      ));

      module_analyzer.extract_statement(module_graph, context, &mut new_bundle_variable)?;

      data.lock().map_c_error()?.push((
        module_analyzer.cjs_module_analyzer.require_modules.clone(),
        new_bundle_variable,
      ));
      Ok::<(), CompilationError>(())
    })?;

    let mut map = HashMap::new();

    for (key, val) in module_map {
      map.insert(
        key,
        Arc::try_unwrap(val).unwrap().into_inner().map_c_error()?,
      );
    }

    let _ = mem::replace(&mut self.module_map, map);

    for (require_modules, inner_bundle_variable) in data.into_inner().map_c_error()? {
      self.namespace_modules.extend(require_modules);

      bundle_variable.merge(inner_bundle_variable);
    }

    Ok(())
  }

  pub fn is_commonjs(&self, module_id: &ModuleId) -> bool {
    self
      .module_map
      .get(module_id)
      .map(|item| item.is_commonjs())
      .unwrap_or(false)
  }

  pub fn module_system(&self, module_id: &ModuleId) -> ModuleSystem {
    self
      .module_map
      .get(module_id)
      .map(|item| item.module_system.clone())
      .unwrap()
  }

  pub fn is_hybrid_or_esm(&self, module_id: &ModuleId) -> bool {
    self
      .module_map
      .get(module_id)
      .map(|item| item.is_hybrid_esm())
      .unwrap_or(false)
  }

  #[inline]
  pub fn get_export_names(&self, module_id: &ModuleId) -> Arc<ReferenceMap> {
    self
      .module_map
      .get(module_id)
      .map(|m| m.export_names())
      .unwrap_or_else(|| Arc::new(ReferenceMap::new(self.module_system(module_id))))
  }

  #[inline]
  pub fn module_analyzer(&self, module_id: &ModuleId) -> Option<&ModuleAnalyzer> {
    self.module_map.get(module_id)
  }

  #[inline]
  pub fn module_analyzer_unchecked(&self, module_id: &ModuleId) -> &ModuleAnalyzer {
    &self.module_map[module_id]
  }

  #[inline]
  pub fn is_external(&self, module_id: &ModuleId) -> bool {
    self
      .module_graph
      .module(module_id)
      .is_some_and(|m| m.external)
  }

  #[inline]
  pub fn module_analyzer_mut(&mut self, module_id: &ModuleId) -> Option<&mut ModuleAnalyzer> {
    self.module_map.get_mut(module_id)
  }

  #[inline]
  pub fn module_analyzer_mut_unchecked(&mut self, module_id: &ModuleId) -> &mut ModuleAnalyzer {
    self.module_map.get_mut(module_id).unwrap()
  }

  pub fn module_analyzer_by_source(
    &self,
    module_id: &ModuleId,
    source: &str,
  ) -> Option<&ModuleAnalyzer> {
    if let Some(m) = self.module_graph.get_dep_by_source_optional(
      module_id,
      source,
      Some(ResolveKind::DynamicImport),
    ) {
      return self.module_map.get(&m);
    }

    None
  }

  pub fn is_same_bundle(&self, a: &ModuleId, b: &ModuleId) -> bool {
    match (self.module_analyzer(a), self.module_analyzer(b)) {
      (Some(a), Some(b)) => a.resource_pot_id == b.resource_pot_id,
      _ => false,
    }
  }

  #[inline]
  pub fn is_contain_namespace(&self, module_id: &ModuleId) -> bool {
    self
      .module_global_uniq_name
      .namespace_name(module_id)
      .is_some()
  }

  /// ---
  /// 1. all export continue to search
  /// 2. named export need filter
  ///   2-1. has source continue search with filter
  ///   2-2. no source collect
  /// 3. namespace export collect and skip
  /// 4. default export skip
  ///
  /// if export all, should skip default export
  pub fn build_export_names(
    &mut self,
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
  ) -> Arc<ReferenceMap> {
    let mut map = ReferenceMap::new(self.module_system(module_id));

    let exports_stmts = if let Some(module_analyzer) = self.module_analyzer(module_id) {
      if let Some(export_names) = &module_analyzer.export_names {
        return export_names.clone();
      }

      module_analyzer
        .exports_stmts()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>()
    } else {
      vec![]
    };

    for export in exports_stmts {
      for specify in export.specifiers.iter() {
        if let Some(ref source) = export.source {
          if self.is_external(source) {
            map.add_reference(source, specify);
            continue;
          }

          if self.is_commonjs(source) {
            map.add_commonjs(source, specify);
            continue;
          }

          match specify {
            ExportSpecifierInfo::All(_) => {
              let result = self.build_export_names(source, bundle_variable);
              map.extends(&result);
            }

            ExportSpecifierInfo::Named(export) => {
              let export_map = self.build_export_names(source, bundle_variable);
              if let Some(i) = export_map.query(export.export_from(), bundle_variable) {
                map.add_local(&ExportSpecifierInfo::Named(
                  (i, Some(export.export_as())).into(),
                ))
              };
            }

            ExportSpecifierInfo::Default(_) | ExportSpecifierInfo::Namespace(_) => {
              map.add_local(specify);
            }
          }
        } else {
          match specify {
            ExportSpecifierInfo::All(_) => {
              unreachable!("export All source should not be None")
            }

            ExportSpecifierInfo::Namespace(_) => {
              unreachable!("export namespace source should not be None")
            }

            _ => {
              map.add_local(specify);
            }
          }
        }
      }
    }

    if let Some(m) = self.module_analyzer_mut(module_id) {
      m.export_names = Some(Arc::new(map));
    }

    self.get_export_names(module_id)
  }

  pub fn patch_module_analyzer_ast(
    &mut self,
    module_id: &ModuleId,
    context: &Arc<CompilationContext>,
    module_graph: &ModuleGraph,
    bundle_variable: &mut BundleVariable,
    bundle_reference: &mut BundleReference,
    commonjs_import_executed: &mut HashSet<ModuleId>,
    order_index_map: &HashMap<ModuleId, usize>,
    polyfill: &mut SimplePolyfill,
  ) -> Result<()> {
    farm_profile_function!(format!(
      "patch module analyzer ast: {}",
      module_id.to_string()
    ));

    let namespace = self.module_global_uniq_name.namespace_name(module_id);

    self.patch_module(
      module_id,
      context,
      bundle_variable,
      module_graph,
      namespace,
      bundle_reference,
      commonjs_import_executed,
      order_index_map,
      polyfill,
    )?;

    Ok(())
  }

  fn patch_namespace(
    &mut self,
    module_id: &ModuleId,
    namespace: Option<usize>,
    bundle_variable: &BundleVariable,
    bundle_reference: &mut BundleReference,
    patch_asts: &mut Vec<ModuleItem>,
    order_index_map: &HashMap<ModuleId, usize>,
    polyfill: &mut SimplePolyfill,
  ) -> Result<()> {
    if self.is_commonjs(module_id) {
      return Ok(());
    }

    if let Some(ns) = namespace {
      patch_asts.extend(generate_namespace_by_reference_map(
        module_id,
        ns,
        bundle_variable,
        bundle_reference,
        &self.build_export_names(module_id, bundle_variable),
        self,
        order_index_map,
        polyfill,
      )?);
    }

    Ok(())
  }

  fn strip_module_decl(
    &mut self,
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
    commonjs_import_executed: &mut HashSet<ModuleId>,
  ) {
    let module_analyzer = self.module_analyzer_mut_unchecked(module_id);
    let mut stmt_actions = module_analyzer
      .statement_actions
      .clone()
      .into_iter()
      .collect::<Vec<_>>();
    stmt_actions.sort_by_key(|a| std::cmp::Reverse(a.index()));
    let mut ast = module_analyzer.ast.take();

    stmt_actions.iter().for_each(|action| {
            let mut replace_ast_item = |index: usize| {
              replace(&mut ast.body[index], ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP })))
            };

            match action {
              StmtAction::StripExport(index) => {
                if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) = replace_ast_item(*index) {
                  ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(export_decl.decl))
                }
              },

              StmtAction::StripDefaultExport(index, rename) => {
                if let ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl)) = replace_ast_item(*index) {
                  let rendered_name = bundle_variable.render_name(*rename);
                  ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(match export_decl.decl {
                    swc_ecma_ast::DefaultDecl::Class(class) => {
                      Decl::Class(
                        ClassDecl {
                          ident: Ident::from(rendered_name.as_str()),
                          declare: false,
                          class: class.class,
                        },
                      )
                    },
                    swc_ecma_ast::DefaultDecl::Fn(f) => {
                      Decl::Fn(FnDecl {
                        ident: Ident::from(rendered_name.as_str()),
                        declare: false,
                        function: f.function,
                      })
                    },
                    _ => {
                      unreachable!(
                        "export_default_decl.decl should not be anything clone() other than a class, function"
                      )
                    },
                  }));
                }
              }

              StmtAction::DeclDefaultExpr(index, var) => {
                match replace_ast_item(*index) {
                  ModuleItem::ModuleDecl(decl) => {
                    match decl {
                        ModuleDecl::ExportDefaultDecl(export) => {
                          let default_name = bundle_variable.render_name(self.module_global_uniq_name.default_name(module_id).unwrap());
                          match export.decl {
                            swc_ecma_ast::DefaultDecl::Class(class) => {
                              ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
                                ident: Ident::from(default_name.as_str()),
                                declare: false,
                                class: class.class,
                              })));
                            },
                            swc_ecma_ast::DefaultDecl::Fn(f) => {
                              ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
                                ident: Ident::from(default_name.as_str()),
                                declare: false,
                                function: f.function,
                              })));
                            },
                            _ => {
                              unreachable!("ExportDefault should not be anything other than a class, function")
                            }
                          }
                        },
                        ModuleDecl::ExportDefaultExpr(export_default_decl) => {
                          ast.body[*index] =
                          ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                            span: DUMMY_SP,
                            kind: swc_ecma_ast::VarDeclKind::Var,
                            declare: false,
                            decls: vec![VarDeclarator {
                              span: DUMMY_SP,
                              name: swc_ecma_ast::Pat::Ident(BindingIdent {
                                id: Ident::from(bundle_variable.render_name(*var).as_str()),
                                type_ann: None,
                              }),
                              init: Some(export_default_decl.expr),
                              definite: false,
                            }],
                          }))));
                        },
                        _ => {}
                    }
                  },
                  _ => {
                    unreachable!("ExportDefault should not be anything other than a class, function");
                  }
                }
              }

              StmtAction::StripCjsImport(index, import_execute_module) => {
                replace_ast_item(*index);
                if let Some(source) = import_execute_module {
                  if !commonjs_import_executed.contains(source) {
                    ast.body[*index] = ModuleItem::Stmt(Stmt::Expr(
                      ExprStmt { span: DUMMY_SP, expr: Box::new(Expr::Call(
                        CallExpr { span: DUMMY_SP, callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(bundle_variable.name(self.module_global_uniq_name.commonjs_name(source).unwrap()).as_str().into()))), args: vec![], type_args: None }
                      )) }
                    ));
                    commonjs_import_executed.insert(source.clone());
                  }
                }
              }

              StmtAction::RemoveImport(index) => {
                replace_ast_item(*index);
              }
            }

        });

    let module_analyzer = self.module_analyzer_mut_unchecked(module_id);
    module_analyzer.ast = ast;
  }

  fn patch_module(
    &mut self,
    module_id: &ModuleId,
    context: &Arc<CompilationContext>,
    bundle_variable: &mut BundleVariable,
    module_graph: &ModuleGraph,
    namespace: Option<usize>,
    bundle_reference: &mut BundleReference,
    commonjs_import_executed: &mut HashSet<ModuleId>,
    order_index_map: &HashMap<ModuleId, usize>,
    polyfill: &mut SimplePolyfill,
  ) -> Result<()> {
    farm_profile_function!("");

    if self.module_analyzer(module_id).is_none() {
      return Ok(());
    }

    let module_analyzer = self.module_analyzer_mut(module_id).unwrap();

    let cm = module_analyzer.cm.clone();

    try_with(cm, &context.meta.script.globals, || {
      // 1. strip/remove export
      self.strip_module_decl(module_id, bundle_variable, commonjs_import_executed);

      let mut patch_asts = vec![];

      // TODO: exact generate namespace in used
      // 2. generate namespace for module
      self
        .patch_namespace(
          module_id,
          namespace,
          bundle_variable,
          bundle_reference,
          &mut patch_asts,
          order_index_map,
          polyfill,
        )
        .unwrap();

      // 3. process hybrid module or commonjs
      CjsPatch::patch_cjs_module(
        self,
        module_id,
        module_graph,
        context,
        &mut patch_asts,
        bundle_variable,
        bundle_reference,
        polyfill,
      );

      // 1. append ast
      // 2. replace commonjs require
      // 3. rename
      {
        let module_analyzer = self.module_analyzer_mut_unchecked(module_id);
        let mut ast = module_analyzer.ast.take();

        let rename_map = module_analyzer.build_rename_map(bundle_variable);

        ast.body.extend(patch_asts);

        if matches!(
          module_analyzer.module_system,
          ModuleSystem::CommonJs | ModuleSystem::Hybrid
        ) {
          CjsPatch::replace_cjs_require(
            module_analyzer.mark,
            &mut ast,
            module_id,
            module_graph,
            &self.module_global_uniq_name,
            bundle_variable,
            &context.config,
          )
        }

        ast.body = mem::take(&mut ast.body)
          .into_iter()
          .filter_map(|item| match item {
            ModuleItem::Stmt(Stmt::Empty(_)) => None,
            _ => Some(item),
          })
          .collect::<Vec<_>>();

        ast.visit_mut_with(&mut replace_dynamic_import(
          self,
          module_id,
          bundle_variable,
        ));

        ast.visit_mut_with(&mut RenameIdent::new(rename_map));

        self.module_analyzer_mut(module_id).unwrap().ast = ast;
      }
    })
    .unwrap();

    Ok(())
  }

  pub fn link(
    &mut self,
    bundle_variable: &mut BundleVariable,
    order_index_map: &HashMap<ModuleId, usize>,
    context: &Arc<CompilationContext>,
  ) {
    farm_profile_scope!("link module analyzer");
    let root = &context.config.root;
    let mut ordered_module_ids = order_index_map.keys().collect::<Vec<_>>();

    ordered_module_ids.sort_by(|a, b| order_index_map[b].cmp(&order_index_map[a]));

    for module_id in ordered_module_ids {
      if self.module_map.contains_key(module_id) {
        self.build_export_names(module_id, bundle_variable);
      }

      let Some(module_analyzer) = self.module_map.get(module_id) else {
        continue;
      };

      farm_profile_scope!(format!(
        "link module analyzer: {}",
        module_analyzer.module_id.to_string()
      ));

      bundle_variable.set_namespace(module_analyzer.resource_pot_id.clone());

      // in this time, it import by cjs require
      if self.namespace_modules.contains(&module_analyzer.module_id)
        || self.is_external(&module_analyzer.module_id)
        || self.module_graph.is_dynamic(module_id)
      {
        self
          .module_global_uniq_name
          .add_namespace(&module_analyzer.module_id, |v| {
            bundle_variable.register_used_name_by_module_id(&module_analyzer.module_id, v, root)
          });

        if self.is_external(&module_analyzer.module_id) {
          continue;
        }
      };

      if module_analyzer.is_commonjs() {
        self
          .module_global_uniq_name
          .add_commonjs(&module_analyzer.module_id, |v| {
            bundle_variable.register_used_name_by_module_id(&module_analyzer.module_id, v, root)
          });
      }

      if matches!(module_analyzer.module_system, ModuleSystem::CommonJs) {
        continue;
      }

      let is_hybrid = matches!(module_analyzer.module_system, ModuleSystem::Hybrid);

      // hybrid | esm
      for statement in &module_analyzer.statements {
        if let Some(s) = &statement.import {
          if self.is_external(&s.source)
            || s
              .specifiers
              .iter()
              .any(|specify| matches!(specify, ImportSpecifierInfo::Namespace(_)))
          {
            self.module_global_uniq_name.add_namespace(&s.source, |v| {
              bundle_variable.register_used_name_by_module_id(&s.source, v, root)
            });
          }
        }

        if let Some(s) = &statement.export {
          if is_hybrid {
            if let Some(source) = s.source.as_ref() {
              if !self.is_commonjs(source) {
                self.module_global_uniq_name.add_namespace(source, |s| {
                  bundle_variable.register_used_name_by_module_id(source, s, root)
                });
              }
            }
          }

          if let Some(module_id) = s.source.as_ref() {
            if self.is_external(module_id) {
              self.module_global_uniq_name.add_namespace(module_id, |s| {
                bundle_variable.register_used_name_by_module_id(module_id, s, root)
              })
            }
          }

          for specify in &s.specifiers {
            match specify {
              ExportSpecifierInfo::Default(n) => {
                if bundle_variable.name(*n) == "default" {
                  self
                    .module_global_uniq_name
                    .add_default(&module_analyzer.module_id, |s| {
                      bundle_variable.register_used_name_by_module_id(&module_analyzer.module_id, s, root)
                    });
                }
              }

              ExportSpecifierInfo::Namespace(_) |
              // maybe used in namespace
              ExportSpecifierInfo::All(_) => {
                if let Some(source) = &s.source {
                  self
                    .module_global_uniq_name
                    .add_namespace(source, |s| bundle_variable.register_used_name_by_module_id(source, s, root));
                }
              }
              _ => {}
            }
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
  };

  use farmfe_core::{
    context::CompilationContext,
    module::{
      module_graph::ModuleGraph, Module, ModuleId, ModuleMetaData, ModuleSystem, ModuleType,
      ScriptModuleMetaData,
    },
    swc_common::{Globals, Mark, SourceMap, DUMMY_SP},
    swc_ecma_ast::{Ident, Module as EcmaAstModule},
  };
  use farmfe_toolkit::script::swc_try_with::try_with;

  use crate::resource_pot_to_bundle::{
    modules_analyzer::module_analyzer::{
      ExportInfo, ExportSpecifierInfo, ModuleAnalyzer, Statement, Variable,
    },
    uniq_name::BundleVariable,
  };

  use super::ModuleAnalyzerManager;

  #[test]
  fn test() {
    let mut map = HashMap::new();

    let module_index_id: ModuleId = "index".into();
    let module_a_id: ModuleId = "moduleA".into();
    let module_b_id: ModuleId = "moduleB".into();
    let module_c_id: ModuleId = "moduleC".into();
    let module_d_id: ModuleId = "moduleD".into();

    let context = Arc::new(CompilationContext::default());

    let resource_pot_id = "index";

    let globals = Globals::default();
    let cm = Arc::new(SourceMap::default());
    let create_script_module = |module_id: &ModuleId| -> Module {
      let mut module = Module::new(module_id.clone());
      try_with(cm.clone(), &globals, || {
        module.module_type = ModuleType::Js;
        module.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
          ast: EcmaAstModule {
            span: DUMMY_SP,
            body: vec![],
            shebang: None,
          },
          top_level_mark: Default::default(),
          unresolved_mark: Mark::default().as_u32(),
          module_system: ModuleSystem::EsModule,
          hmr_self_accepted: false,
          hmr_accepted_deps: HashSet::default(),
          comments: Default::default(),
          custom: Default::default(),
        }));
      })
      .unwrap();

      module
    };

    let module_index = create_script_module(&module_index_id);
    let module_a = create_script_module(&module_a_id);
    let module_b = create_script_module(&module_b_id);
    let module_c = create_script_module(&module_c_id);
    let module_d = create_script_module(&module_d_id);

    let create_module_analyzer = |module: &Module| {
      ModuleAnalyzer::new(
        &module,
        &context,
        resource_pot_id.to_string(),
        false,
        false,
        false,
      )
      .unwrap()
    };

    // ModuleAnalyzer::new()
    let mut module_analyzer_index = create_module_analyzer(&module_index);
    let mut module_analyzer_a = create_module_analyzer(&module_a);
    let mut module_analyzer_b = create_module_analyzer(&module_b);
    let mut module_analyzer_c = create_module_analyzer(&module_c);
    let mut module_analyzer_d = create_module_analyzer(&module_d);
    // index.js   export { foo as bar } from './moduleA.js';
    // moduleA.js export { foo } from './moduleB.js';
    // moduleB.js export { a1 as foo } from './moduleC.js';
    // moduleC.js export { a2 as a1 } from './moduleD.js';
    // moduleD.js export { a2 };
    let mut bundle_variables = BundleVariable::new();
    let module_graph = ModuleGraph::new();

    // index.js bar -> a2
    // moduleA.js foo -> a2
    // moduleB.js foo -> a2
    // moduleC.js a1 -> a2
    // moduleD.js a2
    let module_index_foo_export_origin =
      bundle_variables.register_var(&module_index_id, &Ident::from("foo"), false);
    let module_index_bar_export_as =
      bundle_variables.register_var(&module_index_id, &Ident::from("bar"), false);

    module_analyzer_index.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(module_a_id.clone()),
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          module_index_foo_export_origin,
          Some(module_index_bar_export_as),
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    let module_a_foo_export_origin =
      bundle_variables.register_var(&module_a_id, &Ident::from("foo"), false);
    module_analyzer_a.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(module_b_id.clone()),
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          module_a_foo_export_origin,
          None,
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    let module_b_a1_export_origin =
      bundle_variables.register_var(&module_b_id, &Ident::from("a1"), false);
    let module_b_foo_export_as =
      bundle_variables.register_var(&module_b_id, &Ident::from("foo"), false);
    module_analyzer_b.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(module_c_id.clone()),
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          module_b_a1_export_origin,
          Some(module_b_foo_export_as),
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    let module_c_a2_export_origin =
      bundle_variables.register_var(&module_c_id, &Ident::from("a2"), false);
    let module_c_a1_export_as =
      bundle_variables.register_var(&module_c_id, &Ident::from("a1"), false);
    let module_c_d3_export_namespace =
      bundle_variables.register_var(&module_c_id, &Ident::from("d3"), false);

    module_analyzer_c.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(module_d_id.clone()),
        specifiers: vec![
          ExportSpecifierInfo::Named(Variable(
            module_c_a2_export_origin,
            Some(module_c_a1_export_as),
          )),
          ExportSpecifierInfo::Namespace(module_c_d3_export_namespace),
        ],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    let module_d_a2_export_origin =
      bundle_variables.register_var(&module_d_id, &Ident::from("a2"), false);
    module_analyzer_d.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: None,
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          module_d_a2_export_origin,
          None,
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    map.insert(module_index_id.clone(), module_analyzer_index);
    map.insert(module_a_id.clone(), module_analyzer_a);
    map.insert(module_b_id.clone(), module_analyzer_b);
    map.insert(module_c_id.clone(), module_analyzer_c);
    map.insert(module_d_id.clone(), module_analyzer_d);

    let mut module_analyzer_manager = ModuleAnalyzerManager::new(map, &module_graph);

    {
      let exports = module_analyzer_manager.build_export_names(&module_d_id, &bundle_variables);

      assert_eq!(exports.export.named.len(), 1);

      let v = exports.export.named.iter().collect::<Vec<_>>();

      assert_eq!(
        v,
        vec![(&module_d_a2_export_origin, &module_d_a2_export_origin)]
      );
    }

    {
      // c
      let exports = module_analyzer_manager.build_export_names(&module_c_id, &bundle_variables);

      assert_eq!(exports.export.named.len(), 1);
      assert_eq!(exports.export.namespace, Some(module_c_d3_export_namespace));

      let v = exports.export.named.iter().collect::<Vec<_>>();

      assert_eq!(
        v,
        vec![(&module_c_a1_export_as, &module_d_a2_export_origin)]
      );
    }

    {
      // b
      let exports = module_analyzer_manager.build_export_names(&module_b_id, &bundle_variables);

      assert_eq!(exports.export.named.len(), 1);

      let v = exports.export.named.iter().collect::<Vec<_>>();

      assert_eq!(
        v,
        vec![(&module_b_foo_export_as, &module_d_a2_export_origin)]
      );
    }

    {
      // a
      let exports = module_analyzer_manager.build_export_names(&module_a_id, &bundle_variables);

      assert_eq!(exports.export.named.len(), 1);

      let v = exports.export.named.iter().collect::<Vec<_>>();

      assert_eq!(
        v,
        vec![(&module_a_foo_export_origin, &module_d_a2_export_origin)]
      );
    }

    {
      // index
      let exports = module_analyzer_manager.build_export_names(&module_index_id, &bundle_variables);

      assert_eq!(exports.export.named.len(), 1);

      let v = exports.export.named.iter().collect::<Vec<_>>();

      assert_eq!(
        v,
        vec![(&module_index_bar_export_as, &module_d_a2_export_origin)]
      );
    }
  }
}
