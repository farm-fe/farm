use std::collections::HashMap;

use farmfe_core::{
  error::Result,
  module::{ModuleId, ModuleSystem},
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, CallExpr, Callee, Decl, Expr, ExprOrSpread,
    ExprStmt, KeyValueProp, Lit, MemberExpr, MemberProp, ModuleItem, ObjectLit, Pat, Prop,
    PropName, PropOrSpread, SimpleAssignTarget, Stmt, VarDecl, VarDeclKind, VarDeclarator,
  },
};

use crate::resource_pot_to_bundle::{
  bundle::{
    bundle_external::{ExternalReferenceExport, ExternalReferenceImport, ReferenceKind},
    ModuleAnalyzerManager,
  },
  common::OptionToResult,
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
    source: Option<&ModuleId>,
    export: &ExternalReferenceExport,
    bundle_variable: &BundleVariable,
    module_analyzer_manager: &ModuleAnalyzerManager,
    polyfill: &mut SimplePolyfill,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut ordered_keys = export.named.keys().collect::<Vec<_>>();

    ordered_keys.sort_by_key(|a| bundle_variable.name(**a));

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

      let named_render_name = bundle_variable.render_name(*local);
      let exported_name = bundle_variable.name(*exported);

      stmts.push(module_export(&exported_name, &named_render_name));
    }

    if let Some(namespace) = export.namespace.as_ref() {
      let named_render_name = bundle_variable.render_name(*namespace);
      let exported_name = bundle_variable.name(*namespace);
      stmts.push(module_export(&exported_name, &named_render_name));
    }

    if let Some(source) = source {
      if export.all.0 {
        let is_external = module_analyzer_manager.is_external(source);
        if is_external || module_analyzer_manager.is_commonjs(source) {
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
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut ordered_import = import_map.keys().collect::<Vec<_>>();
    ordered_import.sort();

    for source in ordered_import {
      let module_id = match source {
        ReferenceKind::Bundle(_) => continue,
        ReferenceKind::Module(m) => m,
      };

      let import = &import_map[source];

      if import.named.is_empty() && import.namespace.is_none() && import.default.is_none() {
        continue;
      }

      let namespace_name = bundle_variable.name(
        module_analyzer_manager
          .module_global_uniq_name
          .namespace_name(module_id)
          .unwrap(),
      );

      // import * as foo_ns from "foo";
      // import foo from "foo";
      // =>
      // var foo_ns = _interop_require_wildcard(require("foo"));
      // var foo_default = foo_ns.default;
      let try_wrap_namespace = |expr: Box<Expr>, polyfill: &mut SimplePolyfill| {
        if import.namespace.is_some() {
          return wrap_require_wildcard(expr, polyfill);
        }

        expr
      };
      let try_wrap_require_default = |expr: Box<Expr>, polyfill: &mut SimplePolyfill| {
        if import.default.is_some() {
          return wrap_require_default(expr, polyfill);
        }

        expr
      };

      // if both namespace and default are imported, we need to import the namespace first
      stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        kind: farmfe_core::swc_ecma_ast::VarDeclKind::Var,
        declare: false,
        decls: vec![VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: namespace_name.as_str().into(),
            type_ann: None,
          }),
          init: Some(try_wrap_namespace(
            Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: Callee::Expr(Box::new(Expr::Ident("require".into()))),
              args: vec![ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Lit(Lit::Str(module_id.to_string().as_str().into()))),
              }],
              type_args: None,
              ctxt: SyntaxContext::empty(),
            })),
            polyfill,
          )),
          definite: false,
        }],
        ctxt: SyntaxContext::empty(),
      })))));

      let mut decls: Vec<VarDeclarator> = vec![];

      let mut add_decl = |name: &str, property: &str| {
        let is_default = property == "default";
        let init_expr = Box::new(Expr::Ident(namespace_name.as_str().into()));

        decls.push(VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: name.into(),
            type_ann: None,
          }),
          init: Some(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: if is_default {
              try_wrap_require_default(init_expr, polyfill)
            } else {
              init_expr
            },
            prop: MemberProp::Ident(property.into()),
          }))),
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
        add_decl(&bundle_variable.render_name(*default), "default");
      }

      if !decls.is_empty() {
        stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
          span: DUMMY_SP,
          kind: VarDeclKind::Var,
          declare: false,
          decls,
          ctxt: SyntaxContext::empty(),
        })))));
      }
    }

    Ok(stmts)
  }
}
