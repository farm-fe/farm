use std::{
  borrow::Cow,
  cell::RefMut,
  collections::{HashMap, HashSet, VecDeque},
  mem::{self, replace},
  sync::Arc,
};

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  module::{module_graph::ModuleGraph, ModuleId, ModuleMetaData, ModuleSystem},
  swc_common::{comments::SingleThreadedComments, util::take::Take, DUMMY_SP},
  swc_ecma_ast::{
    self, ArrowExpr, BindingIdent, BlockStmt, CallExpr, ClassDecl, ComputedPropName, Decl,
    EmptyStmt, Expr, ExprOrSpread, ExprStmt, FnDecl, Ident, KeyValueProp, MemberExpr, ModuleDecl,
    ModuleItem, ObjectLit, Pat, Prop, PropName, PropOrSpread, SpreadElement, Stmt, Str, VarDecl,
    VarDeclKind, VarDeclarator,
  },
};
use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_transforms::{
    feature::enable_available_feature_from_es_version,
    helpers::inject_helpers,
    modules::{common_js, import_analysis::import_analyzer, util::Config, util::ImportInterop},
  },
  swc_ecma_visit::VisitMutWith,
};

use crate::resource_pot_to_bundle::{bundle_external::ReferenceKind, common::parse_module_item};

use super::{
  bundle_external::BundleReference,
  defined_idents_collector::RenameIdent,
  modules_analyzer::module_analyzer::{
    ExportAllSet, ExportInfo, ExportSpecifierInfo, ExportType, ImportSpecifierInfo, ModuleAnalyzer,
    StmtAction, Variable,
  },
  uniq_name::{safe_name_form_module_id, BundleVariable},
};

// TODO: global polyfill
#[derive(Debug, Hash, PartialEq, Eq)]
enum Polyfill {
  WrapCommonJs,
}

impl Polyfill {
  fn to_ast(&self) -> Result<Vec<ModuleItem>> {
    Ok(vec![parse_module_item(
      r#"
function __commonJs(mod) {
	var module;
	return () => {
		if (module) {
			return module.exports;
		}
		module = {
			exports: {},
		};
		mod[Object.keys(mod)[0]](module, module.exports);
		return module.exports;
	};
}
      "#,
    )?])
  }

  fn dependents(&self) -> Vec<Polyfill> {
    vec![]
  }
}

#[derive(Default, Debug)]
pub struct SimplePolyfill {
  polyfills: HashSet<Polyfill>,
}

impl SimplePolyfill {
  fn add(&mut self, polyfill: Polyfill) {
    if self.polyfills.contains(&polyfill) {
      return;
    }

    let dependents = polyfill.dependents();

    self.polyfills.insert(polyfill);

    dependents.into_iter().for_each(|dep| self.add(dep));
  }

  pub fn to_ast(&self) -> Result<Vec<ModuleItem>> {
    let mut asts = vec![];

    for polyfill in &self.polyfills {
      asts.extend(polyfill.to_ast()?)
    }

    Ok(asts)
  }
}

pub struct ModuleAnalyzerManager {
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
  pub polyfill: SimplePolyfill,
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
      .map(|item| item.namespace)
      .flatten()
  }

  pub fn default_name(&self, module_id: &ModuleId) -> Option<usize> {
    self
      .module_map
      .get(module_id)
      .map(|item| item.default)
      .flatten()
  }

  pub fn commonjs_name(&self, module_id: &ModuleId) -> Option<usize> {
    self
      .module_map
      .get(module_id)
      .map(|item| item.commonjs)
      .flatten()
  }

  fn entry_module(&mut self, module_id: &ModuleId) -> &mut ModuleGlobalName {
    if !self.module_map.contains_key(module_id) {
      self
        .module_map
        .insert(module_id.clone(), ModuleGlobalName::new());
    }

    self.module_map.get_mut(module_id).unwrap()
  }
}

impl Take for ModuleGlobalUniqName {
  fn dummy() -> Self {
    Default::default()
  }
}

impl ModuleAnalyzerManager {
  pub fn new(module_map: HashMap<ModuleId, ModuleAnalyzer>) -> Self {
    Self {
      module_map,
      namespace_modules: HashSet::new(),
      module_global_uniq_name: ModuleGlobalUniqName::new(),
      polyfill: SimplePolyfill::default(),
    }
  }

  pub fn extract_modules_statements(
    &mut self,
    modules: &Vec<&ModuleId>,
    context: &Arc<CompilationContext>,
    module_graph: &ModuleGraph,
    mut bundle_variable: RefMut<BundleVariable>,
  ) -> Result<()> {
    for module_id in modules {
      if let Some(module_analyzer) = self.module_map.get_mut(module_id) {
        module_analyzer.extract_statement(module_graph, context, &mut bundle_variable)?;
        self
          .namespace_modules
          .extend(module_analyzer.cjs_module_analyzer.require_modules.clone());
      }
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

  #[inline]
  pub fn module_analyzer(&self, module_id: &ModuleId) -> Option<&ModuleAnalyzer> {
    self.module_map.get(module_id)
  }

  #[inline]
  pub fn module_analyzer_mut(&mut self, module_id: &ModuleId) -> Option<&mut ModuleAnalyzer> {
    self.module_map.get_mut(module_id)
  }

  #[inline]
  pub fn is_contain_namespace(&self, module_id: &ModuleId) -> bool {
    self
      .module_global_uniq_name
      .namespace_name(module_id)
      .is_some()
  }

  // TODO: cache module export
  /// ---
  /// 1. all export continue to search
  /// 2. named export need filter
  ///   2-1. has source continue search with filter
  ///   2-2. no source collect
  /// 3. namespace export collect and skip
  /// 4. default export skip
  pub fn export_names(
    &self,
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
  ) -> ExportAllSet {
    let mut exports = ExportAllSet::new();

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

    if let Some(module_analyzer) = self.module_analyzer(module_id) {
      if module_analyzer.is_commonjs() {
        // TODO: support commonjs module export
        exports.merge(ExportAllSet {
          data: vec![(
            ExportInfo {
              source: None,
              specifiers: vec![],
              stmt_id: 0,
            },
            module_id.clone(),
          )],
          ty: ExportType::HybridDynamic,
        });
      }

      if matches!(module_analyzer.module_system, ModuleSystem::CommonJs) {
        return exports;
      }
    }

    for export in exports_stmts {
      if let Some(source) = export.source.as_ref() {
        let module_analyzer_option = self.module_analyzer(source);

        if module_analyzer_option.is_none() || module_analyzer_option.is_some_and(|m| m.external) {
          exports.add((export, module_id.clone()));
          continue;
        }
      }

      for specify in export.specifiers.iter() {
        match specify {
          ExportSpecifierInfo::All(_) => {
            if let Some(source) = &export.source {
              let result = self.export_names(source, bundle_variable);
              exports.merge(result);
            } else {
              unreachable!("export All source should not be None")
            }
          }

          ExportSpecifierInfo::Namespace(_) => {
            exports.add((
              ExportInfo {
                source: export.source.clone(),
                specifiers: vec![specify.clone()],
                stmt_id: export.stmt_id,
              },
              module_id.clone(),
            ));
          }

          _ => {
            if let Some(source) = &export.source {
              let result = self.export_names(source, bundle_variable);

              match specify {
                ExportSpecifierInfo::Default(_) => {
                  unreachable!("default export should not be here")
                }

                ExportSpecifierInfo::Named(importer) => {
                  for (source_export, source_module_id) in result.data {
                    for source_specify in source_export.specifiers {
                      match source_specify {
                        ExportSpecifierInfo::Named(export) => {
                          if bundle_variable.name(importer.local())
                            == bundle_variable.name(export.export_as())
                          {
                            exports.add((
                              ExportInfo {
                                source: source_export.source.clone(),
                                specifiers: vec![ExportSpecifierInfo::Named(Variable(
                                  export.local(),
                                  Some(importer.export_as()),
                                ))],
                                stmt_id: source_export.stmt_id,
                              },
                              source_module_id.clone(),
                            ));
                          }
                        }

                        _ => {}
                      }
                    }
                  }
                }

                _ => {}
              }
            } else {
              exports.add((
                ExportInfo {
                  source: export.source.clone(),
                  specifiers: vec![specify.clone()],
                  stmt_id: export.stmt_id,
                },
                module_id.clone(),
              ));
            }
          }
        }
      }
    }

    exports
  }

  pub fn patch_module_analyzer_ast(
    &mut self,
    module_id: &ModuleId,
    context: &Arc<CompilationContext>,
    module_graph: &ModuleGraph,
    bundle_variable: &mut BundleVariable,
    bundle_reference: &BundleReference,
  ) -> Result<()> {
    let namespace = self.module_global_uniq_name.namespace_name(module_id);

    self.patch_module(
      module_id,
      context,
      bundle_variable,
      module_graph,
      namespace,
      bundle_reference,
    )?;

    Ok(())
  }

  fn patch_namespace(
    &mut self,
    module_id: &ModuleId,
    namespace: Option<usize>,
    bundle_variable: &BundleVariable,
  ) -> Result<Vec<ModuleItem>> {
    let mut patch_ast_items = vec![];

    // TODO: patch namespace other bundle | entry
    if let Some(local) = namespace {
      let namespace = bundle_variable.name(local);

      let mut statements = self
        .module_analyzer(module_id)
        .map(|item| {
          item
            .exports_stmts()
            .into_iter()
            .map(|item| Cow::Borrowed(item))
            .collect::<VecDeque<_>>()
        })
        .unwrap_or(VecDeque::default());

      let mut props: Vec<PropOrSpread> = vec![];

      while let Some(export) = statements.pop_front() {
        for specify in &export.specifiers {
          match specify {
            ExportSpecifierInfo::All(_) => {
              if let Some(source) = &export.source {
                let export_names = self.export_names(source, &bundle_variable);

                for (export, _) in export_names.data {
                  if export.specifiers.is_empty()
                    && matches!(export_names.ty, ExportType::HybridDynamic)
                  {
                    let commonjs_name_index =
                      self.module_global_uniq_name.commonjs_name(&source).unwrap();
                    // TODO: polyfill
                    props.push(PropOrSpread::Spread(SpreadElement {
                      dot3_token: DUMMY_SP,
                      expr: Box::new(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(
                          bundle_variable.name(commonjs_name_index).as_str().into(),
                        ))),
                        args: vec![],
                        type_args: None,
                      })),
                    }));
                  } else {
                    statements.push_back(Cow::Owned(export));
                  }
                }
              }
            }

            ExportSpecifierInfo::Named(named) => {
              if let Some(exported) = &named.1 {
                let exported = bundle_variable.name(*exported);
                let local_ident = bundle_variable.render_name(named.local());

                props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                  key: PropName::Str(exported.as_str().into()),
                  value: Box::new(Expr::Ident(Ident::from(local_ident.as_str()))),
                }))));
              } else {
                let local = bundle_variable.var_by_index(named.local());
                let local_key = local.origin_name();
                let local_ident = local.render_name();

                props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                  key: PropName::Str(local_key.as_str().into()),
                  value: Box::new(Expr::Ident(Ident::from(local_ident.as_str()))),
                }))));
              };
            }

            ExportSpecifierInfo::Default(_) => {
              // let default_ident = bundle_variable.render_name(*default);
              // props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
              //   key: PropName::Str(Str::from("default")),
              //   value: Box::new(Expr::Ident(Ident::from(default_ident.as_str()))),
              // }))));
            }

            ExportSpecifierInfo::Namespace(ns) => {
              let namespace = bundle_variable.var_by_index(*ns);

              let ns_key = namespace.origin_name();
              let ns_value = namespace.render_name();

              props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Str(ns_key.as_str().into()),
                value: Box::new(Expr::Ident(ns_value.as_str().into())),
              }))));
            }
          }
        }
      }

      patch_ast_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: Ident::new(namespace.as_str().into(), DUMMY_SP),
            type_ann: None,
          }),
          init: Some(Box::new(Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props,
          }))),
          definite: false,
        }],
      })))));
    }

    Ok(patch_ast_items)
  }

  fn patch_wrap_commonjs(
    &self,
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
    ast: Vec<ModuleItem>,
  ) -> Result<Vec<ModuleItem>> {
    let mut patch_ast_items = vec![];

    let result = self
      .module_global_uniq_name
      .commonjs_name(module_id)
      .unwrap();

    patch_ast_items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
      span: DUMMY_SP,
      kind: VarDeclKind::Var,
      declare: false,
      decls: vec![VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(BindingIdent {
          id: bundle_variable.render_name(result).as_str().into(),
          type_ann: None,
        }),
        init: Some(Box::new(Expr::Call(CallExpr {
          span: DUMMY_SP,
          callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(Ident::from("__commonJs")))),
          args: vec![ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Object(ObjectLit {
              span: DUMMY_SP,
              props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Str(module_id.to_string().into()),
                value: Box::new(Expr::Arrow(ArrowExpr {
                  span: DUMMY_SP,
                  params: vec![
                    Pat::Ident(BindingIdent {
                      id: Ident::from("module"),
                      type_ann: None,
                    }),
                    Pat::Ident(BindingIdent {
                      id: Ident::from("exports"),
                      type_ann: None,
                    }),
                  ],
                  body: Box::new(swc_ecma_ast::BlockStmtOrExpr::BlockStmt(BlockStmt {
                    span: DUMMY_SP,
                    stmts: ast
                      .into_iter()
                      .map(|module_item| match module_item {
                        // if esm module, should transform to commonjs before
                        ModuleItem::ModuleDecl(_) => unreachable!("module_decl should not be here"),
                        ModuleItem::Stmt(stmt) => stmt,
                      })
                      .collect(),
                  })),
                  is_async: false,
                  is_generator: false,
                  type_params: None,
                  return_type: None,
                })),
              })))],
            })),
          }],
          type_args: None,
        }))),
        definite: false,
      }],
    })))));

    Ok(patch_ast_items)
  }

  fn patch_module(
    &mut self,
    module_id: &ModuleId,
    context: &Arc<CompilationContext>,
    bundle_variable: &mut BundleVariable,
    module_graph: &ModuleGraph,
    namespace: Option<usize>,
    bundle_reference: &BundleReference,
  ) -> Result<()> {
    // strip/remove export
    {
      let module_analyzer = self.module_analyzer_mut(module_id).unwrap();
      let mut stmt_actions = module_analyzer
        .statement_actions
        .clone()
        .into_iter()
        .collect::<Vec<_>>();
      stmt_actions.sort_by(|a, b| b.index().cmp(&a.index()));
      let mut ast = module_analyzer.ast.take();
      let cm = module_analyzer.cm.clone();
      let _ = module_analyzer;

      try_with(cm, &context.meta.script.globals, || {
        let mut commonjs_import_process = HashSet::new();

        println!("stmt_actions: {:#?}", stmt_actions);
        stmt_actions.iter().for_each(|action| {
            // let stmt = replace(
            // &mut ast.body[index],
            // ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP })),
            // );
            let mut replace_ast_item = |index: usize| {
              replace(&mut ast.body[index], ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP })))
            };

            match action {
              StmtAction::StripExport(index) => match replace_ast_item(*index) {
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) => {
                  ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(export_decl.decl))
                }
                _ => {}
              },

              StmtAction::StripDefaultExport(index, rename) => match replace_ast_item(*index) {
                  ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl)) => {
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
                  _ => {
                  }
                }

              StmtAction::DeclDefaultExpr(index, var) => {
                if let ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(export_default_decl)) = replace_ast_item(*index)
                {
                  // TODO: 看看 case
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
                }
              }

              StmtAction::ReplaceCjsImport(index, source) => {
                if let ModuleItem::ModuleDecl(ModuleDecl::Import(_)) = replace_ast_item(*index) {
                  if !commonjs_import_process.contains(source) {
                    if let Some(import_external) = bundle_reference.commonjs_import_map.get(&ReferenceKind::Module(source.clone().into())) {
                      let cjs_name = bundle_variable.render_name(self.module_global_uniq_name.commonjs_name(&source).unwrap());

                      let mut decls = vec![];

                      let cjs_caller = CallExpr {
                        span: DUMMY_SP,
                        callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(cjs_name.as_str().into()))),
                        args: vec![],
                        type_args: None,
                    };

                      if let Some(default) = import_external.default {
                        decls.push(
                          VarDeclarator { span: DUMMY_SP, name: Pat::Ident(BindingIdent { id: Ident::from(
                            bundle_variable.render_name(default).as_str()
                          ), type_ann: None }), init: Some(Box::new(Expr::Member(
                            MemberExpr {
                                span: DUMMY_SP,
                                obj: Box::new(Expr::Call(cjs_caller.clone())),
                                prop: swc_ecma_ast::MemberProp::Ident(
                                  "default".into()
                                ),
                            }
                          ))), definite: false }
                        );
                      }

                      if let Some(ns) = import_external.namespace {
                        decls.push(
                          VarDeclarator { span: DUMMY_SP, name: Pat::Ident(BindingIdent { id: Ident::from(
                            bundle_variable.render_name(ns).as_str()
                          ), type_ann: None }), init: Some(Box::new(Expr::Call(cjs_caller.clone()))), definite: false }
                        );
                      }

                      let ordered_keys = import_external.named.keys().collect::<Vec<_>>();
                      for imported in ordered_keys {
                        let named = &import_external.named[imported];

                        decls.push(
                          VarDeclarator { span: DUMMY_SP, name: Pat::Ident(BindingIdent { id: Ident::from(
                            bundle_variable.render_name(*named).as_str()
                          ), type_ann: None }), init: Some(Box::new(Expr::Member(
                            MemberExpr {
                                span: DUMMY_SP,
                                obj: Box::new(Expr::Call(cjs_caller.clone())),
                                prop: swc_ecma_ast::MemberProp::Ident(
                                  imported.as_str().into()
                                ),
                            }
                          ))), definite: false }
                        );
                      }


                      ast.body[*index] = ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                        span: DUMMY_SP,
                        kind: VarDeclKind::Var,
                        declare: false,
                        decls
                      }))));

                    }else {
                      // self executed cjs module
                      ast.body[*index] = ModuleItem::Stmt(Stmt::Expr(ExprStmt {
                        span: DUMMY_SP,
                        expr: Box::new(Expr::Call(CallExpr { span: DUMMY_SP, callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(bundle_variable.name(self.module_global_uniq_name.commonjs_name(source).unwrap()).as_str().into()))), args: vec![], type_args: None })),
                      }));
                    };

                    commonjs_import_process.insert(source);
                  }else {

                  }
                }
              }

              StmtAction::ReplaceCjsExport(source) => {
                if let Some(module_analyzer) = self.module_analyzer(module_id) {
                  println!("\n\nmodule_id: {}\ncommonjs_export\n{:#?}", module_id.to_string(), module_analyzer.cjs_module_analyzer.commonjs_export);
                  let res = module_analyzer.cjs_module_analyzer.build_commonjs_export(source, bundle_variable, module_analyzer, &self.module_global_uniq_name);

                  for item in res.into_iter().rev() {
                    ast.body.insert(0, item);
                  };
                }
              }

              StmtAction::RemoveImport(index) => {
                replace_ast_item(*index);
              }
              _ => {}
            }

        });
      })?;

      let module_analyzer = self.module_analyzer_mut(module_id).unwrap();
      module_analyzer.ast = ast;
    }

    let mut patch_asts = vec![];

    // generate namespace
    if !self.is_commonjs(module_id) {
      patch_asts.extend(self.patch_namespace(module_id, namespace, bundle_variable)?);
    }

    if let Some(module_analyzer) = self.module_analyzer_mut(module_id) {
      if let Some(module) = module_graph.module(module_id) {
        // transform commonjs module to esm
        if matches!(module_analyzer.module_system, ModuleSystem::Hybrid) {
          try_with(
            module_analyzer.cm.clone(),
            &context.meta.script.globals,
            || {
              let comments: SingleThreadedComments =
                module.meta.as_script().comments.clone().into();

              module_analyzer
                .ast
                .visit_mut_with(&mut import_analyzer(ImportInterop::Swc, true));

              module_analyzer
                .ast
                .visit_mut_with(&mut inject_helpers(module_analyzer.mark.0));

              module_analyzer
                .ast
                .visit_mut_with(&mut common_js::<&SingleThreadedComments>(
                  module_analyzer.mark.0,
                  Config {
                    ignore_dynamic: true,
                    preserve_import_meta: true,
                    ..Default::default()
                  },
                  enable_available_feature_from_es_version(context.config.script.target),
                  Some(&comments),
                ));
            },
          )?;
        }

        if matches!(
          module_analyzer.module_system,
          ModuleSystem::Hybrid | ModuleSystem::CommonJs
        ) {
          let ast = module_analyzer.ast.body.take();

          self.polyfill.add(Polyfill::WrapCommonJs);
          patch_asts.extend(self.patch_wrap_commonjs(module_id, bundle_variable, ast)?);
        }
      };
    }

    let module_global_uniq_name = self.module_global_uniq_name.take();

    // append ast
    if let Some(module_analyzer) = self.module_analyzer_mut(module_id) {
      let mut ast = module_analyzer.ast.take();
      ast.body.extend(patch_asts);

      if matches!(
        module_analyzer.module_system,
        ModuleSystem::CommonJs | ModuleSystem::Hybrid
      ) {
        module_analyzer.cjs_module_analyzer.replace_require_require(
          module_analyzer.mark.clone(),
          &mut ast,
          &module_analyzer.module_id.clone(),
          module_graph,
          &module_global_uniq_name,
          &bundle_variable,
        )
      }

      let rename_map = module_analyzer.build_rename_map(bundle_variable);

      ast.body = mem::take(&mut ast.body)
        .into_iter()
        .filter_map(|item| match item {
          ModuleItem::Stmt(Stmt::Empty(_)) => None,
          _ => Some(item),
        })
        .collect::<Vec<_>>();

      ast.visit_mut_with(&mut RenameIdent::new(rename_map));

      module_analyzer.ast = ast;
    }

    self.module_global_uniq_name = module_global_uniq_name;

    Ok(())
  }

  pub fn link(
    &mut self,
    bundle_variable: &mut BundleVariable,
    module_graph: &ModuleGraph,
    context: &Arc<CompilationContext>,
  ) {
    for module_analyzer in self.module_map.values_mut() {
      // module_analyzer.cjs_module_analyzer.require_modules;
      let register_new_name = |bundle_variable: &mut BundleVariable,
                               module_id: &ModuleId,
                               suffix: &str| {
        let module_safe_name = format!("{}{suffix}", safe_name_form_module_id(module_id, context));

        let uniq_name_safe_name = bundle_variable.uniq_name().uniq_name(&module_safe_name);

        bundle_variable.add_used_name(uniq_name_safe_name.clone());

        let var =
          bundle_variable.register_var(&module_id, &uniq_name_safe_name.as_str().into(), false);

        return var;
      };

      // it import by cjs require
      if self.namespace_modules.contains(&module_analyzer.module_id) {
        let module_global_name = self
          .module_global_uniq_name
          .entry_module(&module_analyzer.module_id);

        if module_global_name.namespace.is_none() {
          module_global_name.namespace = Some(register_new_name(
            bundle_variable,
            &module_analyzer.module_id,
            "_ns",
          ));
        }
      };

      if matches!(
        module_analyzer.module_system,
        ModuleSystem::CommonJs | ModuleSystem::Hybrid
      ) {
        // TODO: support Hybrid
        let module_global_name = self
          .module_global_uniq_name
          .entry_module(&module_analyzer.module_id);

        if module_global_name.commonjs.is_none() {
          module_global_name.commonjs = Some(register_new_name(
            bundle_variable,
            &module_analyzer.module_id,
            "_cjs",
          ));
        }
      }

      for statement in &module_analyzer.statements {
        if let Some(s) = &statement.import {
          if s
            .specifiers
            .iter()
            .any(|specify| matches!(specify, ImportSpecifierInfo::Namespace(_)))
          {
            let module_global_name = self.module_global_uniq_name.entry_module(&s.source);

            if module_global_name.namespace.is_none() {
              module_global_name.namespace =
                Some(register_new_name(bundle_variable, &s.source, "_ns"));
            }
          }
        }

        if let Some(s) = &statement.export {
          for specify in &s.specifiers {
            match specify {
              ExportSpecifierInfo::Default(n) => {
                if bundle_variable.name(*n) == "default" {
                  let module_global_name = self
                    .module_global_uniq_name
                    .entry_module(&module_analyzer.module_id);

                  if module_global_name.default.is_none() {
                    module_global_name.default = Some(register_new_name(
                      bundle_variable,
                      &module_analyzer.module_id,
                      "_default",
                    ));
                  }
                }
              }

              ExportSpecifierInfo::Namespace(_) => {
                if let Some(source) = &s.source {
                  let module_global_name = self.module_global_uniq_name.entry_module(&source);
                  if module_global_name.namespace.is_none() {
                    module_global_name.namespace =
                      Some(register_new_name(bundle_variable, source, "_ns"));
                  }
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
    module::{Module, ModuleId, ModuleMetaData, ModuleSystem, ModuleType, ScriptModuleMetaData},
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

    // index.js bar -> a2
    // moduleA.js foo -> a2
    // moduleB.js foo -> a2
    // moduleC.js a1 -> a2
    // moduleD.js a2
    module_analyzer_index.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(module_a_id.clone()),
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          bundle_variables.register_var(&module_index_id, &Ident::from("foo"), false),
          Some(bundle_variables.register_var(&module_index_id, &Ident::from("bar"), false)),
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    module_analyzer_a.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(module_b_id.clone()),
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          bundle_variables.register_var(&module_a_id, &Ident::from("foo"), false),
          None,
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    module_analyzer_b.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(module_c_id.clone()),
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          bundle_variables.register_var(&module_b_id, &Ident::from("a1"), false),
          Some(bundle_variables.register_var(&module_b_id, &Ident::from("foo"), false)),
        ))],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    module_analyzer_c.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: Some(module_d_id.clone()),
        specifiers: vec![
          ExportSpecifierInfo::Named(Variable(
            bundle_variables.register_var(&module_c_id, &Ident::from("a2"), false),
            Some(bundle_variables.register_var(&module_c_id, &Ident::from("a1"), false)),
          )),
          ExportSpecifierInfo::Namespace(bundle_variables.register_var(
            &module_c_id,
            &Ident::from("d3"),
            false,
          )),
        ],
        stmt_id: 0,
      }),
      defined: vec![],
    });

    module_analyzer_d.statements.push(Statement {
      id: 0,
      import: None,
      export: Some(ExportInfo {
        source: None,
        specifiers: vec![ExportSpecifierInfo::Named(Variable(
          bundle_variables.register_var(&module_d_id, &Ident::from("a2"), false),
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

    let module_analyzer_manager = ModuleAnalyzerManager::new(map);

    let print = |exports: Vec<(ExportInfo, ModuleId)>| {
      for (export, module_id) in exports {
        println!(
          "export {:?} from {:?}",
          module_id.to_string(),
          export.source.map(|item| item.to_string())
        );
        for specify in export.specifiers {
          match specify {
            ExportSpecifierInfo::All(_) => {
              println!("All")
            }
            ExportSpecifierInfo::Named(named) => {
              println!(
                "Named: {} as {}",
                bundle_variables.name(named.local()),
                bundle_variables.name(named.export_as())
              );
            }
            ExportSpecifierInfo::Default(default) => {
              println!("Default: {}", bundle_variables.name(default));
            }
            ExportSpecifierInfo::Namespace(ns) => {
              println!("namespace: {}", bundle_variables.name(ns));
            }
          }
        }
      }
    };

    let is_debug = false;
    {
      let exports = module_analyzer_manager.export_names(&module_d_id, &bundle_variables);

      assert_eq!(exports.data.len(), 1);
      assert!(matches!(
        exports.data[0].0.specifiers[0],
        ExportSpecifierInfo::Named(_)
      ));

      if let ExportSpecifierInfo::Named(named) = &exports.data[0].0.specifiers[0] {
        assert_eq!(bundle_variables.name(named.local()), "a2");
      }

      if is_debug {
        println!("\n\nd");
        print(exports.data);
      }
    }

    {
      // c
      let exports = module_analyzer_manager.export_names(&module_c_id, &bundle_variables);

      assert_eq!(exports.data.len(), 2);
      assert!(matches!(
        exports.data[0].0.specifiers[0],
        ExportSpecifierInfo::Named(_)
      ));

      if let ExportSpecifierInfo::Named(named) = &exports.data[0].0.specifiers[0] {
        assert_eq!(bundle_variables.name(named.local()), "a2");
        assert_eq!(bundle_variables.name(named.export_as()), "a1");
      }

      if is_debug {
        println!("\n\nc");
        print(exports.data);
      }
    }

    {
      // b
      let exports = module_analyzer_manager.export_names(&module_b_id, &bundle_variables);

      assert_eq!(exports.data.len(), 1);
      assert!(matches!(
        exports.data[0].0.specifiers[0],
        ExportSpecifierInfo::Named(_)
      ));

      if let ExportSpecifierInfo::Named(named) = &exports.data[0].0.specifiers[0] {
        assert_eq!(bundle_variables.name(named.local()), "a2");
        assert_eq!(bundle_variables.name(named.export_as()), "foo");
      }

      if is_debug {
        println!("\n\nb");
        print(exports.data);
      }
    }

    {
      // a
      let exports = module_analyzer_manager.export_names(&module_a_id, &bundle_variables);

      assert_eq!(exports.data.len(), 1);
      assert!(matches!(
        exports.data[0].0.specifiers[0],
        ExportSpecifierInfo::Named(_)
      ));

      if let ExportSpecifierInfo::Named(named) = &exports.data[0].0.specifiers[0] {
        assert_eq!(bundle_variables.name(named.local()), "a2");
        assert_eq!(bundle_variables.name(named.export_as()), "foo");
      }

      if is_debug {
        println!("\n\na");
        print(exports.data);
      }
    }

    {
      // index
      let exports = module_analyzer_manager.export_names(&module_index_id, &bundle_variables);

      assert_eq!(exports.data.len(), 1);
      assert!(matches!(
        exports.data[0].0.specifiers[0],
        ExportSpecifierInfo::Named(_)
      ));

      if let ExportSpecifierInfo::Named(named) = &exports.data[0].0.specifiers[0] {
        assert_eq!(bundle_variables.name(named.local()), "a2");
        assert_eq!(bundle_variables.name(named.export_as()), "bar");
      }

      println!("\n\nindex");
      print(exports.data);
    }
  }
}
