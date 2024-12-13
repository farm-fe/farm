use std::{
  fmt::Debug,
  mem::{self, replace},
  sync::{Arc, Mutex, RwLock},
};

use bundle_reference::{CombineBundleReference, CommonJsImportMap, ReferenceKind};
use farmfe_core::{
  config::external::ExternalConfig,
  context::CompilationContext,
  error::{CompilationError, MapCompletionError, Result},
  farm_profile_function, farm_profile_scope,
  module::{module_graph::ModuleGraph, ModuleId, ModuleMetaData, ModuleSystem},
  rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator},
  resource::resource_pot::ResourcePotId,
  swc_common::{util::take::Take, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    self, BindingIdent, CallExpr, ClassDecl, Decl, EmptyStmt, Expr, ExprStmt, FnDecl, Ident,
    Module as ModuleAst, ModuleDecl, ModuleItem, Stmt, VarDecl, VarDeclarator,
  },
  HashMap, HashSet,
};
use farmfe_toolkit::{
  itertools::Itertools, script::swc_try_with::try_with, swc_ecma_visit::VisitMutWith,
};

pub mod bundle_analyzer;
pub mod bundle_reference;
pub mod bundle_variable;
pub mod reference;
use crate::resource_pot_to_bundle::targets::{
  cjs::patch::CjsPatch, dynamic_import::replace_dynamic_import,
};

use self::reference::ReferenceMap;

use super::{
  common::OptionToResult,
  defined_idents_collector::RenameIdent,
  modules_analyzer::module_analyzer::{
    ExportSpecifierInfo, ImportSpecifierInfo, ModuleAnalyzer, StmtAction,
  },
  polyfill::SimplePolyfill,
  targets::generate::generate_namespace_by_reference_map,
  uniq_name::BundleVariable,
  ShareBundleContext, ShareBundleOptions, FARM_BUNDLE_POLYFILL_SLOT,
};

pub type ModuleMap = HashMap<ModuleId, ModuleAnalyzer>;

pub struct ModuleAnalyzerManager<'a> {
  pub module_map: ModuleMap,
  pub namespace_modules: HashSet<ModuleId>,

  ///
  ///
  /// TODO: dynamic generate
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
  pub module_graph: &'a ModuleGraph,
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
  module_map: HashMap<ReferenceKind, ModuleGlobalName>,
}

impl ModuleGlobalUniqName {
  fn new() -> Self {
    Self {
      module_map: HashMap::default(),
    }
  }

  pub fn namespace_name<R: Into<ReferenceKind>>(&self, module_id: R) -> Option<usize> {
    self
      .module_map
      .get(&module_id.into())
      .and_then(|item| item.namespace)
  }

  pub fn default_name<R: Into<ReferenceKind>>(&self, module_id: R) -> Option<usize> {
    self
      .module_map
      .get(&module_id.into())
      .and_then(|item| item.default)
  }

  pub fn commonjs_name<R: Into<ReferenceKind>>(&self, module_id: R) -> Option<usize> {
    self
      .module_map
      .get(&module_id.into())
      .and_then(|item| item.commonjs)
  }

  pub fn namespace_name_result<R: Into<ReferenceKind> + Debug + Clone>(
    &self,
    module_id: R,
  ) -> Result<usize> {
    Ok(self.namespace_name(module_id.clone()).unwrap())
  }

  pub fn commonjs_name_result<R: Into<ReferenceKind> + Debug + Clone>(
    &self,
    module_id: R,
  ) -> Result<usize> {
    self
      .commonjs_name(module_id.clone())
      .to_result(format!("not found module commonjs name by {:?}", module_id))
  }

  pub fn default_name_result<R: Into<ReferenceKind> + Debug + Clone>(
    &self,
    module_id: R,
  ) -> Result<usize> {
    Ok(
      self
        .default_name(module_id.clone())
        .expect(format!("not found module default name by {:?}", module_id).as_str()),
    )
    // .to_result(format!("not found module default name by {:?}", module_id))
  }

  fn entry_module<R: Into<ReferenceKind>>(&mut self, module_id: R) -> &mut ModuleGlobalName {
    let reference_kind = module_id.into();
    if !self.module_map.contains_key(&reference_kind) {
      self
        .module_map
        .insert(reference_kind.clone(), ModuleGlobalName::new());
    }

    self.module_map.get_mut(&reference_kind).unwrap()
  }

  fn add_namespace<F: FnOnce(&str) -> usize, R: Into<ReferenceKind>>(
    &mut self,
    module_id: R,
    v: F,
  ) {
    let m = self.entry_module(module_id);

    if m.namespace.is_none() {
      m.namespace = Some(v("_ns"));
    }
  }

  fn add_default<F: FnOnce(&str) -> usize, R: Into<ReferenceKind>>(&mut self, module_id: R, v: F) {
    let m = self.entry_module(module_id);

    if m.default.is_none() {
      m.default = Some(v("_default"));
    }
  }

  fn add_commonjs<F: FnOnce(&str) -> usize, R: Into<ReferenceKind>>(&mut self, module_id: R, v: F) {
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
      namespace_modules: HashSet::default(),
      module_global_uniq_name: ModuleGlobalUniqName::new(),
      module_graph,
    }
  }

  pub fn polyfill_resource_pot(&self) -> Option<ResourcePotId> {
    self
      .module_analyzer(&ModuleId::from(FARM_BUNDLE_POLYFILL_SLOT))
      .and_then(|m| Some(m.bundle_group_id.clone()))
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
    // TODO: performance optimization
    let module_map = {
      mem::take(&mut self.module_map)
        .into_iter()
        .map(|(key, val)| (key, Arc::new(RwLock::new(val))))
        .collect::<HashMap<ModuleId, Arc<RwLock<ModuleAnalyzer>>>>()
    };

    let atomic_index = bundle_variable.index.clone();
    let module_order_map = bundle_variable.module_order_map.clone();
    let module_order_index_set = bundle_variable.module_order_index_set.clone();

    modules
      .into_par_iter()
      .enumerate()
      .try_for_each(|(index, module_id)| {
        let mut module_analyzer = module_map
          .get(module_id)
          .map(|item| item.write().unwrap())
          .unwrap();
        let mut new_bundle_variable =
          BundleVariable::branch(&atomic_index, &module_order_map, &module_order_index_set);

        new_bundle_variable.set_namespace(module_analyzer.bundle_group_id.clone());

        farm_profile_scope!(format!(
          "extract module statement: {:?}",
          module_id.to_string()
        ));

        module_analyzer.extract_statement(module_graph, context, &mut new_bundle_variable)?;

        data.lock().map_c_error()?.push((
          index,
          (
            module_analyzer.cjs_module_analyzer.require_modules.clone(),
            new_bundle_variable,
          ),
        ));
        Ok::<(), CompilationError>(())
      })?;

    let mut map = HashMap::default();

    for (key, val) in module_map {
      map.insert(
        key,
        Arc::try_unwrap(val).unwrap().into_inner().map_c_error()?,
      );
    }

    let _ = mem::replace(&mut self.module_map, map);

    for (_, (require_modules, inner_bundle_variable)) in data
      .into_inner()
      .map_c_error()?
      .into_iter()
      .sorted_by_key(|(index, _)| *index)
    {
      self.namespace_modules.extend(require_modules);

      bundle_variable.merge(inner_bundle_variable);
    }

    Ok(())
  }

  #[inline]
  pub fn is_commonjs(&self, module_id: &ModuleId) -> bool {
    self
      .module_map
      .get(module_id)
      .is_some_and(|item| item.is_commonjs())
      || self.module_graph.module(module_id).is_some_and(|m| {
        if matches!(m.meta.as_ref(), ModuleMetaData::Script(_)) {
          let s = m.meta.as_script();
          matches!(
            s.module_system,
            ModuleSystem::CommonJs | ModuleSystem::Hybrid
          )
        } else {
          false
        }
      })
  }

  #[inline]
  pub fn is_entry(&self, module_id: &ModuleId) -> bool {
    self
      .module_map
      .get(module_id)
      .map(|item| item.entry)
      .is_some_and(|v| v)
      || self.module_graph.entries.contains_key(module_id)
  }

  pub fn module_system(&self, module_id: &ModuleId) -> ModuleSystem {
    self
      .module_map
      .get(module_id)
      .map(|item| Some(item.module_system.clone()))
      .unwrap_or_else(|| {
        self.module_graph.module(module_id).map(|m| {
          let script = m.meta.as_script();

          script.module_system.clone()
        })
      })
      .expect(
        format!(
          "expect ModuleSystem of module \"{}\", but not found.",
          module_id.to_string()
        )
        .as_str(),
      )
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
  pub fn contain(&self, module_id: &ModuleId) -> bool {
    self.module_map.contains_key(module_id)
  }

  #[inline]
  pub fn group_id(&self, module_id: &ModuleId) -> Option<&ResourcePotId> {
    self.module_map.get(module_id).map(|m| &m.bundle_group_id)
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

  #[inline]
  pub fn set_ast_body(&mut self, module_id: &ModuleId, ast_body: Vec<ModuleItem>) {
    self.module_analyzer_mut_unchecked(module_id).ast.body = ast_body;
  }

  #[inline]
  pub fn set_ast(&mut self, module_id: &ModuleId, ast_body: ModuleAst) {
    self.module_analyzer_mut_unchecked(module_id).ast = ast_body;
  }

  pub fn is_same_bundle(&self, a: &ModuleId, b: &ModuleId) -> bool {
    match (self.module_analyzer(a), self.module_analyzer(b)) {
      (Some(a), Some(b)) => a.bundle_group_id == b.bundle_group_id,
      _ => false,
    }
  }

  pub fn is_same_bundle_by_bundle(&self, source: &ModuleId, name: &str) -> bool {
    self
      .module_analyzer(source)
      .is_some_and(|m| m.bundle_group_id == name)
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

    let mut map = ReferenceMap::new(self.module_system(module_id));

    // preventing circular references
    if let Some(m) = self.module_analyzer_mut(module_id) {
      m.export_names = Some(Arc::new(map.clone()));
    }

    for export in exports_stmts {
      for specify in export.specifiers.iter() {
        if let Some(ref source) = export.source {
          if self.is_external(source) || !self.contain(source) {
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

            ExportSpecifierInfo::Default(default) => {
              let export_map = self.build_export_names(source, bundle_variable);

              if let Some(i) = export_map.query(*default, bundle_variable) {
                map.add_local(&ExportSpecifierInfo::Default(i));
              }
            }

            ExportSpecifierInfo::Namespace(_) => {
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
    bundle_variable: &mut BundleVariable,
    bundle_reference: &mut CombineBundleReference,
    commonjs_import_executed: &mut HashSet<ModuleId>,
    order_index_map: &HashMap<ModuleId, usize>,
    polyfill: &mut SimplePolyfill,
    external_config: &ExternalConfig,
    options: &ShareBundleContext,
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
      namespace,
      bundle_reference,
      commonjs_import_executed,
      order_index_map,
      polyfill,
      external_config,
      options,
    )
  }

  fn patch_namespace(
    &mut self,
    module_id: &ModuleId,
    namespace: Option<usize>,
    bundle_variable: &BundleVariable,
    bundle_reference: &mut CombineBundleReference,
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
    ctx: &ShareBundleContext,
  ) {
    let module_analyzer = self.module_analyzer_mut_unchecked(module_id);
    let mut stmt_actions = module_analyzer
      .statement_actions
      .clone()
      .into_iter()
      .collect::<Vec<_>>();
    stmt_actions.sort_by_key(|a| std::cmp::Reverse(a.index()));
    let mut ast = module_analyzer.ast.take();

    for action in &stmt_actions {
      let mut replace_ast_item = |index: usize| {
        replace(
          &mut ast.body[index],
          ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP })),
        )
      };

      match action {
        StmtAction::StripExport(index) => {
          if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) =
            replace_ast_item(*index)
          {
            ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(export_decl.decl))
          }
        }

        StmtAction::StripDefaultExport(index, rename) => {
          if let ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl)) =
            replace_ast_item(*index)
          {
            let rendered_name = bundle_variable.render_name(*rename);
            ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(match export_decl.decl {
              swc_ecma_ast::DefaultDecl::Class(class) => Decl::Class(ClassDecl {
                ident: Ident::from(rendered_name.as_str()),
                declare: false,
                class: class.class,
              }),
              swc_ecma_ast::DefaultDecl::Fn(f) => Decl::Fn(FnDecl {
                ident: Ident::from(rendered_name.as_str()),
                declare: false,
                function: f.function,
              }),
              _ => {
                unreachable!(
                  "export_default_decl.decl should not be anything clone() other than a class, function"
                )
              }
            }));
          }
        }

        StmtAction::DeclDefaultExpr(index, var) => match replace_ast_item(*index) {
          ModuleItem::ModuleDecl(decl) => match decl {
            ModuleDecl::ExportDefaultDecl(export) => {
              let default_name = bundle_variable.render_name(
                self
                  .module_global_uniq_name
                  .default_name(module_id)
                  .unwrap(),
              );
              match export.decl {
                swc_ecma_ast::DefaultDecl::Class(class) => {
                  ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
                    ident: Ident::from(default_name.as_str()),
                    declare: false,
                    class: class.class,
                  })));
                }
                swc_ecma_ast::DefaultDecl::Fn(f) => {
                  ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
                    ident: Ident::from(default_name.as_str()),
                    declare: false,
                    function: f.function,
                  })));
                }
                _ => {
                  unreachable!("ExportDefault should not be anything other than a class, function")
                }
              }
            }
            ModuleDecl::ExportDefaultExpr(export_default_decl) => {
              ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                ctxt: SyntaxContext::empty(),
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
            }
            _ => {}
          },
          _ => {
            unreachable!("ExportDefault should not be anything other than a class, function");
          }
        },

        StmtAction::StripCjsImport(index, import_execute_module) => {
          replace_ast_item(*index);
          if let Some(source) = import_execute_module {
            if !commonjs_import_executed.contains(source) {
              if self.contain(source) {
                ast.body[*index] = ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                  span: DUMMY_SP,
                  expr: Box::new(Expr::Call(CallExpr {
                    ctxt: SyntaxContext::empty(),
                    span: DUMMY_SP,
                    callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(
                      bundle_variable
                        .name(self.module_global_uniq_name.commonjs_name(source).unwrap())
                        .as_str()
                        .into(),
                    ))),
                    args: vec![],
                    type_args: None,
                  })),
                }));
                commonjs_import_executed.insert(source.clone());
              }
            }
          }
        }

        StmtAction::RemoveImport(index) => {
          replace_ast_item(*index);
        }
      }
    }

    let module_analyzer = self.module_analyzer_mut_unchecked(module_id);
    module_analyzer.ast = ast;
  }

  fn patch_module(
    &mut self,
    module_id: &ModuleId,
    context: &Arc<CompilationContext>,
    bundle_variable: &mut BundleVariable,
    namespace: Option<usize>,
    bundle_reference: &mut CombineBundleReference,
    commonjs_import_executed: &mut HashSet<ModuleId>,
    order_index_map: &HashMap<ModuleId, usize>,
    polyfill: &mut SimplePolyfill,
    external_config: &ExternalConfig,
    ctx: &ShareBundleContext,
  ) -> Result<()> {
    farm_profile_function!("");

    if self.module_analyzer(module_id).is_none() {
      return Ok(());
    }

    let module_analyzer = self.module_analyzer_mut(module_id).unwrap();

    let cm = module_analyzer.cm.clone();

    try_with(cm, &context.meta.script.globals, || {
      // 1. strip/remove export/import
      self.strip_module_decl(module_id, bundle_variable, commonjs_import_executed, ctx);

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
      CjsPatch::transform_hybrid_or_commonjs_to_esm(
        self,
        module_id,
        context,
        bundle_variable,
        // bundle_reference,
        polyfill,
        ctx,
      )
      .unwrap();

      // 1. append ast
      // 2. replace commonjs require
      // 3. rename
      {
        let module_analyzer = self.module_analyzer_mut_unchecked(module_id);
        let mark = module_analyzer.mark.clone();
        let mut ast = module_analyzer.ast.take();

        ast.body.extend(patch_asts);

        if matches!(
          module_analyzer.module_system,
          ModuleSystem::CommonJs | ModuleSystem::Hybrid
        ) {
          CjsPatch::replace_cjs_require(
            mark,
            &mut ast,
            module_id,
            bundle_variable,
            &context.config,
            polyfill,
            external_config,
            bundle_reference,
            &self.module_graph,
            &self.module_global_uniq_name,
            &self.module_map,
            ctx,
          );
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
          ctx,
        ));

        self.set_ast(module_id, ast);
      }
    })
    .unwrap();

    Ok(())
  }

  pub fn patch_rename(
    &mut self,
    module_id: &ModuleId,
    context: &Arc<CompilationContext>,
    bundle_variable: &mut BundleVariable,
    commonjs_import_map: &CommonJsImportMap,
  ) {
    let module_analyzer = self.module_analyzer_mut_unchecked(module_id);

    let cm = module_analyzer.cm.clone();
    let mut ast = module_analyzer.ast.take();

    try_with(cm, &context.meta.script.globals, || {
      let module_analyzer = self.module_analyzer_unchecked(module_id);

      let rename_map = module_analyzer.build_rename_map(bundle_variable, commonjs_import_map);
      ast.visit_mut_with(&mut RenameIdent::new(rename_map, &bundle_variable, self));

      self.set_ast(module_id, ast);
    })
    .unwrap();
  }

  // more accurate generation
  pub fn link(
    &mut self,
    bundle_variable: &mut BundleVariable,
    order_index_map: &HashMap<ModuleId, usize>,
    context: &Arc<CompilationContext>,
    ordered_groups_id: &Vec<ResourcePotId>,
  ) {
    farm_profile_scope!("link module analyzer");
    let root = &context.config.root;
    let ordered_module_ids = order_index_map
      .keys()
      .sorted_by_key(|a| order_index_map[a])
      .collect::<Vec<_>>();

    for group_id in ordered_groups_id {
      self.module_global_uniq_name.add_namespace(group_id, |v| {
        bundle_variable.register_common_used_name(v, &group_id)
      });
    }

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

      bundle_variable.set_namespace(module_analyzer.bundle_group_id.clone());

      // in this time, it import by cjs require
      if self.namespace_modules.contains(&module_analyzer.module_id)
        || self.is_external(&module_analyzer.module_id)
        || (module_analyzer.is_dynamic
          && self
            .module_graph
            .dependents_ids(&module_analyzer.module_id)
            .iter()
            .any(|importer| self.is_same_bundle(module_id, importer)))
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

        if module_analyzer.entry {
          self
            .module_global_uniq_name
            .add_namespace(&module_analyzer.module_id, |v| {
              bundle_variable.register_used_name_by_module_id(&module_analyzer.module_id, v, root)
            });
        }
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
              ExportSpecifierInfo::Default(_) => {
                // TODO: only add default when it is export default expression e.g: export default 1 + 1
                self
                  .module_global_uniq_name
                  .add_default(&module_analyzer.module_id, |s| {
                    bundle_variable.register_used_name_by_module_id(
                      &module_analyzer.module_id,
                      s,
                      root,
                    )
                  });
              }

              ExportSpecifierInfo::Namespace(_) => {
                if let Some(source) = &s.source {
                  if self.module_map.contains_key(source) {
                    self.module_global_uniq_name.add_namespace(source, |s| {
                      bundle_variable.register_used_name_by_module_id(source, s, root)
                    });
                  }
                }
              }

              // maybe used in namespace
              ExportSpecifierInfo::All(_) => {
                if let Some(source) = &s.source {
                  if self.is_commonjs(source) {
                    self.module_global_uniq_name.add_namespace(source, |s| {
                      bundle_variable.register_used_name_by_module_id(source, s, root)
                    });
                  }
                }
                if !module_analyzer.entry {
                  self.module_global_uniq_name.add_namespace(module_id, |s| {
                    bundle_variable.register_used_name_by_module_id(module_id, s, root)
                  });
                }
              }
              ExportSpecifierInfo::Named(var) => {
                if bundle_variable.name(var.local()) == "default" {
                  self
                    .module_global_uniq_name
                    .add_default(&module_analyzer.module_id, |s| {
                      bundle_variable.register_used_name_by_module_id(
                        &module_analyzer.module_id,
                        s,
                        root,
                      )
                    });
                }
              }
            }
          }
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use farmfe_core::{
    context::CompilationContext,
    module::{
      module_graph::ModuleGraph, Module, ModuleId, ModuleMetaData, ModuleSystem, ModuleType,
      ScriptModuleMetaData,
    },
    swc_common::{Globals, Mark, SourceMap, DUMMY_SP},
    swc_ecma_ast::{Ident, Module as EcmaAstModule},
    HashMap, HashSet,
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
    let mut map = HashMap::default();

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
