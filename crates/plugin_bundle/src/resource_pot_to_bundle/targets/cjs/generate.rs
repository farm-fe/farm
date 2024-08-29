use std::collections::HashMap;

use farmfe_core::{
  error::Result,
  module::ModuleSystem,
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, CallExpr, Callee, Decl, Expr, ExprOrSpread,
    ExprStmt, KeyValueProp, Lit, MemberExpr, MemberProp, ModuleItem, ObjectLit, Pat, Prop,
    PropName, PropOrSpread, SimpleAssignTarget, Stmt, VarDecl, VarDeclKind, VarDeclarator,
  },
};

use crate::resource_pot_to_bundle::{
  bundle::{
    bundle_reference::{ExternalReferenceExport, ExternalReferenceImport, ReferenceKind},
    ModuleAnalyzerManager,
  },
  common::{with_bundle_reference_slot_name, OptionToResult},
  polyfill::{
    cjs::{wrap_export_star, wrap_require_default, wrap_require_wildcard},
    SimplePolyfill,
  },
  uniq_name::BundleVariable,
};

// export * from "./moduleA";
// esm => export * from "./moduleA";
// cjs =>
//  var m = require("./moduleA");
//  _export_star(m, module.exports)

pub struct CjsGenerate {}

impl CjsGenerate {
  pub fn generate_export(
    source: Option<&ReferenceKind>,
    export: &ExternalReferenceExport,
    bundle_variable: &BundleVariable,
    module_analyzer_manager: &ModuleAnalyzerManager,
    polyfill: &mut SimplePolyfill,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut ordered_keys = export.named.keys().collect::<Vec<_>>();

    ordered_keys.sort_by_key(|a| bundle_variable.name(**a));

    let index_is_entry = |i: usize| {
      bundle_variable
        .module_id_by_var_index(i)
        .is_some_and(|m| !module_analyzer_manager.is_entry(m))
    };

    let module_export = |exported_name: &String, named_render_name: &String| {
      ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Assign(AssignExpr {
          span: DUMMY_SP,
          op: AssignOp::Assign,
          left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Member(MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::Ident("module".into())),
              prop: MemberProp::Ident("exports".into()),
            })),
            prop: MemberProp::Ident(exported_name.as_str().into()),
          })),
          right: Box::new(Expr::Ident(named_render_name.as_str().into())),
        })),
      }))
    };

    for exported in ordered_keys {
      let local = &export.named[exported];
      if bundle_variable.var_by_index(*local).removed {
        continue;
      }

      let should_reexport_uniq = index_is_entry(*local);

      let named_render_name = bundle_variable.render_name(*local);
      let exported_name = bundle_variable.name(*exported);

      let exported_name = if should_reexport_uniq || named_render_name == exported_name {
        None
      } else {
        Some(exported_name.as_str().into())
      };

      stmts.push(module_export(
        exported_name.as_ref().unwrap_or(&named_render_name),
        &named_render_name,
      ));
    }

    if let Some(namespace) = export.namespace.as_ref() {
      let named_render_name = bundle_variable.render_name(*namespace);
      let exported_name = bundle_variable.name(*namespace);
      stmts.push(module_export(&exported_name, &named_render_name));
    }

    if let Some(ReferenceKind::Module(source)) = source {
      if export.all.0 {
        let is_external = module_analyzer_manager.is_external(source);
        let is_commonjs = module_analyzer_manager.is_commonjs(source);
        if is_external || is_commonjs {
          let ns = module_analyzer_manager
            .module_global_uniq_name
            .namespace_name(source)
            .to_result("export to cjs cannot find variable")?;
          let render_name = bundle_variable.render_name(ns);

          stmts.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: wrap_export_star(
              vec![
                ExprOrSpread {
                  spread: None,
                  expr: Box::new(Expr::Ident(render_name.as_str().into())),
                },
                ExprOrSpread {
                  spread: None,
                  expr: Box::new(Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: Box::new(Expr::Ident("module".into())),
                    prop: MemberProp::Ident("exports".into()),
                  })),
                },
              ],
              polyfill,
            ),
          })));
        }
      }

      // TODO: add esModule by export type
    }

    if let Some(default) = export.default.as_ref() {
      stmts.push(module_export(
        &"default".to_string(),
        &bundle_variable.render_name(*default),
      ));
    };

    if matches!(
      export.module_system,
      ModuleSystem::EsModule | ModuleSystem::Hybrid
    ) {
      // Object.defineProperty(exports, '__esModule', {
      //   value: true,
      // });

      stmts.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Call(CallExpr {
          span: DUMMY_SP,
          callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident("Object".into())),
            prop: MemberProp::Ident("defineProperty".into()),
          }))),
          args: vec![
            ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Ident("exports".into())),
            },
            ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Lit("__esModule".into())),
            },
            ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                  key: PropName::Ident("value".into()),
                  value: Box::new(Expr::Lit(Lit::Bool(true.into()))),
                })))],
              })),
            },
          ],
          type_args: None,
          ctxt: SyntaxContext::empty(),
        })),
      })));
    }

    Ok(stmts)
  }

  ///
  ///
  /// ```ts
  /// import { name, age } from "foo";
  /// // =>
  /// const foo_ns = require("foo");
  /// var name = foo_ns.name;
  /// var age = foo_ns.age;
  ///
  /// import * as foo from "foo";
  /// // =>
  /// const foo_ns = _interop_require_wildcard(require("foo"));
  ///
  /// import foo from "foo"
  /// // =>
  /// const foo_default = _interop_require_default(require("foo"));
  /// ```
  ///
  pub fn generate_import(
    bundle_variable: &BundleVariable,
    import_map: &HashMap<ReferenceKind, ExternalReferenceImport>,
    module_analyzer_manager: &ModuleAnalyzerManager,
    polyfill: &mut SimplePolyfill,
    resource_pot_id: &str,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut ordered_import = import_map.keys().collect::<Vec<_>>();
    ordered_import.sort();

    let mut generate_import_specifies: HashMap<String, MergedImportGenerate> = HashMap::new();

    for source in ordered_import {
      let import = &import_map[source];

      let mut is_import_uniq_name = false;

      let (module_id, url) = match source {
        ReferenceKind::Bundle(_) => continue,
        ReferenceKind::Module(m) => (
          m,
          if module_analyzer_manager.is_external(m) {
            m.to_string()
          } else {
            if !module_analyzer_manager.is_entry(m) {
              is_import_uniq_name = true;
            }

            with_bundle_reference_slot_name(
              &module_analyzer_manager
                .module_analyzer(m)
                .map(|m| m.resource_pot_id.clone())
                .unwrap(),
            )
          },
        ),
      };

      if import.named.is_empty() && import.namespace.is_none() && import.default.is_none() {
        continue;
      }

      // import * as foo_ns from "foo";
      // import foo from "foo";
      // =>
      // var foo_ns = _interop_require_wildcard(require("foo"));

      let try_wrap_require_default = |expr: Box<Expr>, polyfill: &mut SimplePolyfill| {
        if import.default.is_some() {
          return wrap_require_default(expr, polyfill);
        }

        expr
      };

      let source_bundle_id = module_analyzer_manager
        .module_analyzer(module_id)
        .map(|m| m.resource_pot_id.clone())
        // maybe external
        .unwrap_or(resource_pot_id.to_string());
      let is_commonjs = module_analyzer_manager.is_commonjs(module_id);
      let is_same_bundle = source_bundle_id == resource_pot_id;

      let namespace_name = bundle_variable.name(if !is_same_bundle {
        module_analyzer_manager
          .module_global_uniq_name
          .namespace_name(source_bundle_id.to_string())
          .unwrap()
      } else {
        module_analyzer_manager
          .module_global_uniq_name
          .namespace_name(module_id)
          .unwrap()
      });

      let mut decls: Vec<VarDeclarator> = vec![];
      let namespace_expr = Expr::Ident(namespace_name.as_str().into());

      let mut add_decl = |name: &str, property: &str| {
        let is_default = property == "default";
        let init_expr = Box::new(namespace_expr.clone());

        let init_expr = Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: if is_default {
            try_wrap_require_default(init_expr, polyfill)
          } else {
            init_expr
          },
          prop: MemberProp::Ident(property.into()),
        });

        let t = Box::new(if !is_commonjs {
          init_expr
        } else {
          Expr::Call(CallExpr {
            ctxt: SyntaxContext::empty(),
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(init_expr)),
            args: vec![],
            type_args: None,
          })
        });

        decls.push(VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: name.into(),
            type_ann: None,
          }),
          init: Some(t),
          definite: false,
        });
      };

      let mut ordered_named_keys = import.named.keys().collect::<Vec<_>>();
      ordered_named_keys.sort();

      for imported in ordered_named_keys {
        let local = &import.named[imported];
        let local_named = bundle_variable.render_name(*local);

        add_decl(&local_named, imported);
      }

      if let Some(default) = import.default.as_ref() {
        let name = bundle_variable.render_name(*default);
        if is_import_uniq_name {
          add_decl(&name, &name);
        } else {
          add_decl(&name, "default");
        }
      }

      if let Some(v) = generate_import_specifies.get_mut(&url) {
        v.merge(MergedImportGenerate {
          specifies: decls,
          namespace_name,
          is_contain_namespace: import.namespace.is_some(),
        });
      } else {
        generate_import_specifies.insert(
          url,
          MergedImportGenerate {
            specifies: decls,
            namespace_name,
            is_contain_namespace: import.namespace.is_some(),
          },
        );
      }

      // if !decls.is_empty() {
      //   stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
      //     span: DUMMY_SP,
      //     kind: VarDeclKind::Var,
      //     declare: false,
      //     decls,
      //   })))));
      // }
    }

    for (url, merged_import_generate) in generate_import_specifies {
      // if both namespace and default are imported, we need to import the namespace first
      stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
        ctxt: SyntaxContext::empty(),
        span: DUMMY_SP,
        kind: farmfe_core::swc_ecma_ast::VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: merged_import_generate.namespace_name.as_str().into(),
            type_ann: None,
          }),
          init: Some(try_wrap_namespace(
            Box::new(Expr::Call(CallExpr {
              ctxt: SyntaxContext::empty(),
              span: DUMMY_SP,
              callee: Callee::Expr(Box::new(Expr::Ident("require".into()))),
              args: vec![ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Lit(Lit::Str(url.as_str().into()))),
              }],
              type_args: None,
            })),
            polyfill,
            merged_import_generate.is_contain_namespace,
          )),
          definite: false,
        }],
      })))));

      stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
        ctxt: SyntaxContext::empty(),
        span: DUMMY_SP,
        kind: VarDeclKind::Var,
        declare: false,
        decls: merged_import_generate.specifies,
      })))));
    }

    Ok(stmts)
  }
}

// var foo_default = foo_ns.default;
fn try_wrap_namespace(
  expr: Box<Expr>,
  polyfill: &mut SimplePolyfill,
  is_contain_namespace: bool,
) -> Box<Expr> {
  if is_contain_namespace {
    return wrap_require_wildcard(expr, polyfill);
  }

  expr
}

struct MergedImportGenerate {
  specifies: Vec<VarDeclarator>,
  namespace_name: String,
  is_contain_namespace: bool,
}

impl MergedImportGenerate {
  fn merge(&mut self, other: MergedImportGenerate) {
    self.specifies.extend(other.specifies);
    self.is_contain_namespace = self.is_contain_namespace || other.is_contain_namespace;
  }
}
