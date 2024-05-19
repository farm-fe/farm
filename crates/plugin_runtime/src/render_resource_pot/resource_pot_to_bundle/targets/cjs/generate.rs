use std::collections::HashMap;

use farmfe_core::{
  error::{CompilationError, Result},
  module::ModuleId,
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, AssignTargetPat, BindingIdent, CallExpr, Callee, Decl,
    Expr, ExprOrSpread, ExprStmt, Lit, MemberExpr, MemberProp, ModuleItem, Pat, SimpleAssignTarget,
    Stmt, VarDecl, VarDeclKind, VarDeclarator,
  },
};

use crate::resource_pot_to_bundle::{
  bundle::ModuleAnalyzerManager,
  bundle_external::{ExternalReferenceExport, ExternalReferenceImport, ReferenceKind},
  common,
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
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut ordered_keys = export.named.keys().collect::<Vec<_>>();

    ordered_keys.sort_by(|a, b| bundle_variable.name(**a).cmp(&bundle_variable.name(**b)));

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
          let ns = common::otr!(
            module_analyzer_manager
              .module_global_uniq_name
              .namespace_name(source),
            CompilationError::GenericError("export to cjs cannot find variable".to_string())
          )?;
          let render_name = bundle_variable.render_name(ns);

          stmts.push(ModuleItem::Stmt(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(Expr::Call(CallExpr {
              span: DUMMY_SP,
              callee: Callee::Expr(Box::new(Expr::Ident("_export_star".into()))),
              args: vec![
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
              type_args: None,
            })),
          })));
        }
      }
    }

    if let Some(default) = export.default.as_ref() {
      stmts.push(module_export(
        &"default".to_string(),
        &bundle_variable.render_name(*default),
      ));
    };

    Ok(stmts)
  }

  ///
  ///
  /// ```ts
  /// import { name, age } from "shulan";
  /// // =>
  /// const shulan_ns = require("shulan");
  /// var name = shulan_ns.name;
  /// var age = shulan_ns.age;
  ///
  /// import * as shulan from "shulan";
  /// // =>
  /// const shulan_ns = _interop_require_wildcard(require("shulan"));
  ///
  /// import shulan from "shulan"
  /// // =>
  /// const shulan_ns = _interop_require_default(require("shulan"));
  /// ```
  ///
  pub fn generate_import(
    bundle_variable: &BundleVariable,
    import_map: &HashMap<ReferenceKind, ExternalReferenceImport>,
    module_analyzer_manager: &ModuleAnalyzerManager,
  ) -> Result<Vec<ModuleItem>> {
    let mut stmts = vec![];
    let mut ordered_import = import_map.keys().collect::<Vec<_>>();
    ordered_import.sort_by(|a, b| a.cmp(b));

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

      // import * as shulan_ns from "shulan";
      // import shulan from "shulan";
      // =>
      // var shulan_ns = _interop_require_wildcard(require("shulan"));
      // var shulan_default = shulan_ns.default;
      let try_wrap_namespace = |expr: Box<Expr>| {
        if import.namespace.is_some() || import.default.is_some() {
          return Box::new(Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Ident("_interop_require_wildcard".into()))),
            args: vec![ExprOrSpread { spread: None, expr }],
            type_args: None,
          }));
        }

        return expr;
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
          init: Some(try_wrap_namespace(Box::new(Expr::Call(CallExpr {
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(Expr::Ident("require".into()))),
            args: vec![ExprOrSpread {
              spread: None,
              expr: Box::new(Expr::Lit(Lit::Str(module_id.to_string().as_str().into()))),
            }],
            type_args: None,
          })))),
          definite: false,
        }],
      })))));

      let mut decls: Vec<VarDeclarator> = vec![];

      let mut add_decl = |name: &str, property: &str| {
        decls.push(VarDeclarator {
          span: DUMMY_SP,
          name: Pat::Ident(BindingIdent {
            id: name.into(),
            type_ann: None,
          }),
          init: Some(Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(namespace_name.as_str().into())),
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
        add_decl(&bundle_variable.name(*default), "default");
      }

      if !decls.is_empty() {
        stmts.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
          span: DUMMY_SP,
          kind: VarDeclKind::Var,
          declare: false,
          decls,
        })))));
      }
    }

    Ok(stmts)
  }

  // pub fn generate_export_star(
  //   bundle_variable: &BundleVariable,
  //   ns: usize,
  // ) -> Result<Vec<ModuleItem>> {
  //   let render_name = bundle_variable.render_name(ns);

  //   Ok(vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
  //     span: DUMMY_SP,
  //     expr: Box::new(Expr::Call(CallExpr {
  //       span: DUMMY_SP,
  //       callee: Callee::Expr(Box::new(Expr::Ident("_export_star".into()))),
  //       args: vec![ExprOrSpread {
  //         spread: None,
  //         expr: Box::new(Expr::Ident(render_name.as_str().into())),
  //       }],
  //       type_args: None,
  //     })),
  //   }))])
  // }
}
