use std::{
  cell::RefMut,
  collections::{HashMap, HashSet},
  mem::{self, replace},
  sync::Arc,
};

use farmfe_core::{
  context::CompilationContext,
  error::Result,
  farm_profile_function, farm_profile_scope,
  module::{self, module_graph::ModuleGraph, ModuleId, ModuleMetaData, ModuleSystem},
  swc_common::{comments::SingleThreadedComments, util::take::Take, DUMMY_SP},
  swc_ecma_ast::{
    self, ArrayLit, ArrowExpr, BindingIdent, BlockStmt, Bool, CallExpr, ClassDecl, Decl, EmptyStmt,
    Expr, ExprOrSpread, ExprStmt, FnDecl, Ident, KeyValueProp, MemberExpr, ModuleDecl, ModuleItem,
    ObjectLit, Pat, Prop, PropName, PropOrSpread, Stmt, Str, VarDecl, VarDeclKind, VarDeclarator,
  },
};
use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_transforms::{
    feature::enable_available_feature_from_es_version,
    modules::{common_js, import_analysis::import_analyzer, util::Config, util::ImportInterop},
  },
  swc_ecma_visit::VisitMutWith,
};

use crate::resource_pot_to_bundle::{
  bundle_external::ReferenceKind, common::parse_module_item, targets::cjs::CjsModuleAnalyzer,
};

use super::{
  bundle_external::BundleReference,
  defined_idents_collector::RenameIdent,
  modules_analyzer::module_analyzer::{
    ExportAllSet, ExportInfo, ExportSpecifierInfo, ExportType, ImportSpecifierInfo, ModuleAnalyzer,
    StmtAction, Variable,
  },
  uniq_name::{safe_name_from_module_id, BundleVariable},
};

// TODO: global polyfill
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Polyfill {
  WrapCommonJs,
  MergeNamespace,
  /// compatible require and esm
  Wildcard,
  /// esm `export * from 'module'`
  ExportStar,
  /// esm `import fs from "node:fs"`
  InteropRequireDefault,
}

impl Polyfill {
  fn to_ast(&self) -> Result<Vec<ModuleItem>> {
    Ok(match self {
      Polyfill::WrapCommonJs => vec![parse_module_item(
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
      )?],
      Polyfill::MergeNamespace => vec![parse_module_item(
        r#"
function _mergeNamespaces(n, m) {
    m.forEach(function (e) {
        e && typeof e !== 'string' && !Array.isArray(e) && Object.keys(e).forEach(function (k) {
            if (k !== 'default' && !(k in n)) {
                var d = Object.getOwnPropertyDescriptor(e, k);
                Object.defineProperty(n, k, d.get ? d : {
                    enumerable: true,
                    get: function () { return e[k]; }
                });
            }
        });
    });
    return Object.freeze(n);
}
"#,
      )?],
      Polyfill::Wildcard => vec![parse_module_item(
        r#"
function _getRequireWildcardCache(nodeInterop) {
    if (typeof WeakMap !== "function") return null;
    var cacheBabelInterop = new WeakMap();
    var cacheNodeInterop = new WeakMap();
    return (_getRequireWildcardCache = function(nodeInterop) {
        return nodeInterop ? cacheNodeInterop : cacheBabelInterop;
    })(nodeInterop);
}
function _interop_require_wildcard(obj, nodeInterop) {
    if (!nodeInterop && obj && obj.__esModule) return obj;
    if (obj === null || typeof obj !== "object" && typeof obj !== "function") return {
        default: obj
    };
    var cache = _getRequireWildcardCache(nodeInterop);
    if (cache && cache.has(obj)) return cache.get(obj);
    var newObj = {
        __proto__: null
    };
    var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor;
    for(var key in obj){
        if (key !== "default" && Object.prototype.hasOwnProperty.call(obj, key)) {
            var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null;
            if (desc && (desc.get || desc.set)) Object.defineProperty(newObj, key, desc);
            else newObj[key] = obj[key];
        }
    }
    newObj.default = obj;
    if (cache) cache.set(obj, newObj);
    return newObj;
}
        "#,
      )?],
      Polyfill::ExportStar => vec![parse_module_item(
        r#"
function _export_star(from, to) {
    Object.keys(from).forEach(function(k) {
        if (k !== "default" && !Object.prototype.hasOwnProperty.call(to, k)) {
            Object.defineProperty(to, k, {
                enumerable: true,
                get: function() {
                    return from[k];
                }
            });
        }
    });
    return from;
}
      "#,
      )?],
      Polyfill::InteropRequireDefault => vec![parse_module_item(
        r#"
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
"#,
      )?],
    })
  }

  fn dependents(&self) -> Vec<Polyfill> {
    vec![]
  }

  fn name(&self) -> Vec<String> {
    (match self {
      Polyfill::WrapCommonJs => vec!["__commonJs"],
      Polyfill::MergeNamespace => vec!["_mergeNamespaces"],
      Polyfill::Wildcard => vec!["_getRequireWildcardCache", "_interop_require_wildcard"],
      Polyfill::ExportStar => vec!["_export_star"],
      Polyfill::InteropRequireDefault => vec!["_interop_require_default"],
    })
    .into_iter()
    .map(|item| item.into())
    .collect()
  }
}

#[derive(Default, Debug)]
pub struct SimplePolyfill {
  polyfills: HashSet<Polyfill>,
}

impl SimplePolyfill {
  fn new(polyfill: Vec<Polyfill>) -> Self {
    let mut polyfills = HashSet::new();

    polyfills.extend(polyfill);

    // TODO: delete after test
    // preset
    polyfills.insert(Polyfill::ExportStar);
    polyfills.insert(Polyfill::InteropRequireDefault);
    polyfills.insert(Polyfill::Wildcard);

    Self { polyfills }
  }

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

    let mut polyfills = self.polyfills.iter().collect::<Vec<_>>();

    polyfills.sort();

    for polyfill in &polyfills {
      asts.extend(polyfill.to_ast()?)
    }

    Ok(asts)
  }

  pub fn reserved_word() -> Vec<String> {
    vec![
      Polyfill::WrapCommonJs,
      Polyfill::MergeNamespace,
      Polyfill::Wildcard,
      Polyfill::ExportStar,
      Polyfill::InteropRequireDefault,
    ]
    .into_iter()
    .flat_map(|polyfill| polyfill.name())
    .collect()
  }
}

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
  pub polyfill: SimplePolyfill,
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
      polyfill: SimplePolyfill::new(vec![]),
      module_graph,
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
      farm_profile_function!(format!(
        "extract module statement: {:?}",
        module_id.to_string()
      ));

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
  pub fn is_contain_namespace(&self, module_id: &ModuleId) -> bool {
    self
      .module_global_uniq_name
      .namespace_name(module_id)
      .is_some()
  }

  pub fn build_export_names(
    &mut self,
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
  ) -> ExportAllSet {
    // let export_names_map: HashMap<ModuleId, ExportAllSet> = HashMap::default();
    // let deps = self.module_analyzer(module_id).map(|item| {
    //   item
    //     .exports_stmts()
    //     .into_iter()
    //     .filter_map(|item| item.source.as_ref())
    //     .collect::<Vec<_>>()
    // });

    let mut exports = ExportAllSet::new();

    // let deps = self.module_graph.dependencies_ids(module_id);

    // for dep in deps {
    //   export_all_set.merge(self.build_export_names(&dep));
    // }

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
              let result = self.export_names(source, bundle_variable, true);
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
              match specify {
                ExportSpecifierInfo::Default(_) => {
                  unreachable!("default export should not be here")
                }

                ExportSpecifierInfo::Named(importer) => {
                  if self.is_commonjs(source) {
                    exports.add((
                      ExportInfo {
                        source: export.source.clone(),
                        specifiers: vec![specify.clone()],
                        stmt_id: export.stmt_id,
                      },
                      module_id.clone(),
                    ))
                  } else {
                    let result = self.export_names(source, bundle_variable, true);
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

    return exports;
  }

  // TODO: cache module export
  /// ---
  /// 1. all export continue to search
  /// 2. named export need filter
  ///   2-1. has source continue search with filter
  ///   2-2. no source collect
  /// 3. namespace export collect and skip
  /// 4. default export skip
  ///
  /// if export all, should skip default export
  pub fn export_names(
    &self,
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
    filter_default: bool,
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
              let result = self.export_names(source, bundle_variable, true);
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
              match specify {
                ExportSpecifierInfo::Default(_) => {
                  unreachable!("default export should not be here")
                }

                ExportSpecifierInfo::Named(importer) => {
                  if self.is_commonjs(source) {
                    exports.add((
                      ExportInfo {
                        source: export.source.clone(),
                        specifiers: vec![specify.clone()],
                        stmt_id: export.stmt_id,
                      },
                      module_id.clone(),
                    ))
                  } else {
                    let result = self.export_names(source, bundle_variable, true);
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

    if filter_default {
      exports.data.retain(|(export, _)| {
        export.specifiers.iter().all(|specify| {
          if let ExportSpecifierInfo::Default(_) = specify {
            false
          } else {
            true
          }
        })
      });
    }

    exports
  }

  pub fn patch_module_analyzer_ast(
    &mut self,
    module_id: &ModuleId,
    context: &Arc<CompilationContext>,
    module_graph: &ModuleGraph,
    bundle_variable: &mut BundleVariable,
    bundle_reference: &mut BundleReference,
    commonjs_import_executed: &mut HashSet<ModuleId>,
  ) -> Result<()> {
    farm_profile_function!(format!("patch module analyzer ast: {}", module_id.to_string()));

    let namespace = self.module_global_uniq_name.namespace_name(module_id);

    self.patch_module(
      module_id,
      context,
      bundle_variable,
      module_graph,
      namespace,
      bundle_reference,
      commonjs_import_executed,
    )?;

    Ok(())
  }

  fn patch_namespace(
    &mut self,
    module_id: &ModuleId,
    namespace: Option<usize>,
    bundle_variable: &BundleVariable,
    bundle_reference: &mut BundleReference,
  ) -> Result<Vec<ModuleItem>> {
    let mut patch_ast_items = vec![];

    // TODO: patch namespace other bundle | entry
    if let Some(local) = namespace {
      let namespace = bundle_variable.name(local);

      let exports = self.export_names(module_id, bundle_variable, false);
      self.build_export_names(module_id, bundle_variable);

      let mut props: Vec<PropOrSpread> = vec![];
      let mut commonjs_fns: Vec<Ident> = vec![];
      let mut reexport_namespace: Vec<Ident> = vec![];

      for (export, export_source) in exports.data {
        if export.specifiers.is_empty() && self.is_commonjs(&export_source) {
          commonjs_fns.push(
            bundle_variable
              .name(
                self
                  .module_global_uniq_name
                  .commonjs_name(&export_source)
                  .unwrap(),
              )
              .as_str()
              .into(),
          );

          continue;
        }

        for specify in &export.specifiers {
          match specify {
            ExportSpecifierInfo::All(_) => {
              if let Some(ref source) = export.source {
                if self.is_external(&source) {
                  let ns_index = self.module_global_uniq_name.namespace_name(source).unwrap();
                  bundle_reference.sync_import(
                    source.clone().into(),
                    &ImportSpecifierInfo::Namespace(ns_index),
                    bundle_variable,
                    false,
                  )?;

                  reexport_namespace.push(bundle_variable.name(ns_index).as_str().into());
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

            ExportSpecifierInfo::Default(default) => {
              let default_ident = bundle_variable.render_name(*default);
              props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Str(Str::from("default")),
                value: Box::new(Expr::Ident(Ident::from(default_ident.as_str()))),
              }))));
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

      let declare_init =
        if matches!(exports.ty, ExportType::Static) && reexport_namespace.is_empty() {
          props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Ident(Ident::from("__esModule")),
            value: Box::new(Expr::Lit(swc_ecma_ast::Lit::Bool(Bool {
              span: DUMMY_SP,
              value: true,
            }))),
          }))));

          Some(Box::new(Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props,
          })))
        } else {
          // dynamic
          Some(Box::new(Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(Ident::from(
              "_mergeNamespaces",
            )))),
            args: vec![
              // static
              ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Object(ObjectLit {
                  span: DUMMY_SP,
                  props,
                })),
              },
              ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Array(ArrayLit {
                  span: DUMMY_SP,
                  elems: commonjs_fns
                    .into_iter()
                    .map(|ident| {
                      Some(ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Call(CallExpr {
                          span: DUMMY_SP,
                          callee: swc_ecma_ast::Callee::Expr(Box::new(Expr::Ident(ident))),
                          args: vec![],
                          type_args: None,
                        })),
                      })
                    })
                    .chain(reexport_namespace.into_iter().map(|ns| {
                      Some(ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Ident(ns)),
                      })
                    }))
                    .collect(),
                })),
              },
            ],
            type_args: None,
          })))
        };

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
          init: declare_init,
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
    bundle_reference: &mut BundleReference,
    commonjs_import_executed: &mut HashSet<ModuleId>,
  ) -> Result<()> {
    farm_profile_function!("");

    let module_analyzer = self.module_analyzer_mut(module_id).unwrap();

    try_with(
      module_analyzer.cm.clone(),
      &context.meta.script.globals,
      || {
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
          let _ = module_analyzer;


          stmt_actions.iter().for_each(|action| {

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

          let module_analyzer = self.module_analyzer_mut(module_id).unwrap();
          module_analyzer.ast = ast;
        }

        let mut patch_asts = vec![];

        // generate namespace
        if !self.is_commonjs(module_id) {
          patch_asts.extend(self.patch_namespace(module_id, namespace, bundle_variable, bundle_reference).unwrap());
        }

        // process hybrid or commonjs
        if let Some(module_analyzer) = self.module_analyzer_mut(module_id) {
          // transform commonjs module to esm
          if let Some(module) = module_graph.module(module_id) {

            // if hybrid module, should transform to cjs
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
              ).unwrap();
            }

            // if commonjs module, should wrap function
            if matches!(
              module_analyzer.module_system,
              ModuleSystem::Hybrid | ModuleSystem::CommonJs
            ) {
              let ast = module_analyzer.ast.body.take();

              self.polyfill.add(Polyfill::WrapCommonJs);
              patch_asts.extend(self.patch_wrap_commonjs(module_id, bundle_variable, ast).unwrap());
            }
          };
        }

        if let Some(import) = bundle_reference.declare_commonjs_import.get(&module_id.clone().into()) {
          let ast = CjsModuleAnalyzer::build_commonjs_export(module_id, bundle_variable, &self.module_global_uniq_name, import);

          patch_asts.extend(ast);
        }


        let module_global_uniq_name = self.module_global_uniq_name.take();

        // 1. append ast
        // 2. replace commonjs require
        // 3. rename
        if let Some(module_analyzer) = self.module_analyzer_mut(module_id) {
          let mut ast = module_analyzer.ast.take();

          ast.body.extend(patch_asts);

          if matches!(
            module_analyzer.module_system,
            ModuleSystem::CommonJs | ModuleSystem::Hybrid
          ) {
            module_analyzer.cjs_module_analyzer.replace_cjs_require(
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
      },
    ).unwrap();

    Ok(())
  }

  pub fn link(
    &mut self,
    bundle_variable: &mut BundleVariable,
    module_graph: &ModuleGraph,
    context: &Arc<CompilationContext>,
  ) {
    farm_profile_scope!("link module analyzer");
    // polyfill name should make sure it doesn't conflict.
    // tip: but it cannot be rename unresolved mark
    for name in SimplePolyfill::reserved_word() {
      bundle_variable.add_used_name(name);
    }

    for module_analyzer in self.module_map.values() {
      farm_profile_scope!(format!(
        "link module analyzer: {}",
        module_analyzer.module_id.to_string()
      ));

      // module_analyzer.cjs_module_analyzer.require_modules;
      let register_new_name = |bundle_variable: &mut BundleVariable,
                               module_id: &ModuleId,
                               suffix: &str| {
        // farm_profile_function!("register name");
        farm_profile_scope!("register name");
        let module_safe_name = format!("{}{suffix}", safe_name_from_module_id(module_id, context));

        let uniq_name_safe_name = bundle_variable.uniq_name().uniq_name(&module_safe_name);

        bundle_variable.add_used_name(uniq_name_safe_name.clone());

        bundle_variable.register_var(&module_id, &uniq_name_safe_name.as_str().into(), false)
      };

      // in this time, it import by cjs require
      if self.namespace_modules.contains(&module_analyzer.module_id)
        || self.is_external(&module_analyzer.module_id)
      {
        self
          .module_global_uniq_name
          .add_namespace(&module_analyzer.module_id, |v| {
            register_new_name(bundle_variable, &module_analyzer.module_id, v)
          });

        if self.is_external(&module_analyzer.module_id) {
          continue;
        }
      };

      if module_analyzer.is_commonjs() {
        self
          .module_global_uniq_name
          .add_commonjs(&module_analyzer.module_id, |s| {
            register_new_name(bundle_variable, &module_analyzer.module_id, s)
          });
      }

      if matches!(module_analyzer.module_system, ModuleSystem::CommonJs) {
        continue;
      }

      let is_hybrid = matches!(module_analyzer.module_system, ModuleSystem::Hybrid);

      // hybrid | esm
      for statement in &module_analyzer.statements {
        if let Some(s) = &statement.import {
          if s
            .specifiers
            .iter()
            .any(|specify| matches!(specify, ImportSpecifierInfo::Namespace(_)))
          {
            self.module_global_uniq_name.add_namespace(&s.source, |v| {
              register_new_name(bundle_variable, &s.source, v)
            });
          }
        }

        if let Some(s) = &statement.export {
          if is_hybrid {
            if let Some(source) = s.source.as_ref() {
              if !self.is_commonjs(source) {
                self
                  .module_global_uniq_name
                  .add_namespace(source, |s| register_new_name(bundle_variable, source, s));
              }
            }
          }
          for specify in &s.specifiers {
            match specify {
              ExportSpecifierInfo::Default(n) => {
                if bundle_variable.name(*n) == "default" {
                  self
                    .module_global_uniq_name
                    .add_default(&module_analyzer.module_id, |s| {
                      register_new_name(bundle_variable, &module_analyzer.module_id, s)
                    });
                }
              }

              ExportSpecifierInfo::Namespace(_) |
              // maybe used in namespace
              ExportSpecifierInfo::All(_) => {
                if let Some(source) = &s.source {
                  self
                    .module_global_uniq_name
                    .add_namespace(source, |s| register_new_name(bundle_variable, source, s));
                }
              }
              _ => {}
            }
          }
        }
      }
    }

    println!("bundle_variable: {}", bundle_variable.variables.len());
    if self
      .module_global_uniq_name
      .module_map
      .values()
      .any(|v| v.namespace.is_some())
    {
      self.polyfill.add(Polyfill::MergeNamespace);
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
      module_graph::{self, ModuleGraph},
      Module, ModuleId, ModuleMetaData, ModuleSystem, ModuleType, ScriptModuleMetaData,
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

    let module_analyzer_manager = ModuleAnalyzerManager::new(map, &module_graph);

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
      let exports = module_analyzer_manager.export_names(&module_d_id, &bundle_variables, true);

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
      let exports = module_analyzer_manager.export_names(&module_c_id, &bundle_variables, true);

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
      let exports = module_analyzer_manager.export_names(&module_b_id, &bundle_variables, true);

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
      let exports = module_analyzer_manager.export_names(&module_a_id, &bundle_variables, true);

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
      let exports = module_analyzer_manager.export_names(&module_index_id, &bundle_variables, true);

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
