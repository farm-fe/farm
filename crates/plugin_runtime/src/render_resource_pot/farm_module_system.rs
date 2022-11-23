//! Transform commonjs or esm to Farm's module system, and replace the source with module id.

use farmfe_core::{
  module::{ModuleId, ModuleSystem},
  swc_common::DUMMY_SP,
  swc_ecma_ast::{
    AwaitExpr, BindingIdent, CallExpr, Callee, ComputedPropName, Decl, Expr, ExprOrSpread, Ident,
    ImportDecl, ImportSpecifier, Lit, MemberExpr, MemberProp, Module as SwcModule, ModuleDecl,
    ModuleItem, Number, Pat, Stmt, Str, VarDecl, VarDeclKind, VarDeclarator,
  },
};
use farmfe_toolkit::{
  regex,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

pub struct FarmModuleSystemTransformer {
  module_system: ModuleSystem,
  module_id: ModuleId,
}

impl FarmModuleSystemTransformer {
  pub fn new(module_system: ModuleSystem, module_id: ModuleId) -> Self {
    Self {
      module_system,
      module_id,
    }
  }

  /// transform import decl to await require call
  pub fn import_to_require(&mut self, module_item: &mut ModuleItem) {
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = module_item {

      // *module = ModuleItem::Stmt(require_call);
    } else {
      unreachable!("Unexpected module item: {:?}", module_item);
    }
  }

  /// prepend await to await require call
  pub fn prepend_await_to_require(&mut self, require_expr: Expr) -> Expr {
    Expr::Await(AwaitExpr {
      span: DUMMY_SP,
      arg: Box::new(require_expr),
    })
  }

  /// construct require call
  /// return
  /// ```js
  /// require('module_id')
  /// ```
  pub fn construct_require_expr(&mut self, src: &str) -> Expr {
    let require_expr = Expr::Call(CallExpr {
      span: DUMMY_SP,
      callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
        "require".into(),
        Default::default(),
      )))),
      args: vec![ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Str(Str {
          span: DUMMY_SP,
          value: src.into(),
          raw: None,
        }))),
      }],
      type_args: None,
    });

    require_expr
  }

  pub fn str_to_ident_str(&mut self, src: &str) -> String {
    let regexp = regex::Regex::new(r"[^a-zA-Z0-9_]").unwrap();
    let ident_str = regexp.replace_all(src, "_").to_string();
    ident_str
  }

  /// to:
  /// ```js
  /// var ident = farm_p[idx].ident;
  /// ```
  pub fn construct_array_assignment_stmt(&mut self, idx: u32, ident: Ident) -> Stmt {
    let farm_p = Ident::new("farm_p".into(), Default::default());
    let farm_p_idx = Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(Expr::Ident(farm_p.clone())),
      prop: MemberProp::Computed(ComputedPropName {
        span: DUMMY_SP,
        expr: Box::new(Expr::Lit(Lit::Num(Number {
          span: DUMMY_SP,
          value: idx as f64,
          raw: None,
        }))),
      }),
    });
    let farm_p_idx_ident = Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(farm_p_idx),
      prop: MemberProp::Ident(ident),
    });
    let var_decl = VarDecl {
      span: DUMMY_SP,
      kind: VarDeclKind::Var,
      declare: false,
      decls: vec![VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(BindingIdent::from(ident)),
        init: Some(Box::new(farm_p_idx_ident)),
        definite: false,
      }],
    };
    Stmt::Decl(Decl::Var(var_decl))
  }
}

impl VisitMut for FarmModuleSystemTransformer {
  fn visit_mut_module(&mut self, module: &mut SwcModule) {
    match &self.module_system {
      ModuleSystem::EsModule => {
        let mut import_decls_to_remove = vec![];
        let mut require_calls_to_add = vec![];
        let mut require_promises_ident_to_add = vec![];
        let mut assignments_from_require_calls = vec![];

        // transform module using top level await style code
        // 1. import -> require
        // 2. await Promise.all([require('xxx'), require('yyy')])
        // 3. export decl to exports.xxx = xxx
        for (index, item) in module.body.iter_mut().enumerate() {
          match item {
            ModuleItem::ModuleDecl(decl) => match decl {
              ModuleDecl::Import(import_decl) => {
                import_decls_to_remove.push(index);

                let src = import_decl.src.value.to_string();
                let require_expr = self.construct_require_expr(&src);
                let await_require_expr = self.prepend_await_to_require(require_expr);

                let require_call_decl_ident =
                  Ident::new(self.str_to_ident_str(&src).into(), DUMMY_SP);
                // 1. to `var require_call_decl_ident = await require(require_call_decl_ident)`
                let require_call_decl = ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
                  span: DUMMY_SP,
                  kind: VarDeclKind::Var,
                  decls: vec![VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(BindingIdent::from(require_call_decl_ident.clone())),
                    init: Some(Box::new(await_require_expr.clone())),
                    definite: false,
                  }],
                  declare: false,
                })));
                require_calls_to_add.push(require_call_decl);
                require_promises_ident_to_add.push(require_call_decl_ident.clone());

                for sp in &import_decl.specifiers {
                  match sp {
                    ImportSpecifier::Named(named) => {
                      let ident = named.local.clone();
                      let assignment_stmt = self.construct_array_assignment_stmt(
                        require_promises_ident_to_add.len() as u32 - 1,
                        ident,
                      );
                      assignments_from_require_calls.push(assignment_stmt);
                    }
                    ImportSpecifier::Default(default) => {
                      // let specifier = default.local.sym.to_string();
                      // let require_expr = self.construct_require_expr(&src, &specifier);
                      // let require_call = ModuleItem::Stmt(require_expr.into());
                      // module.body.insert(index, require_call);
                    }
                    ImportSpecifier::Namespace(namespace) => {
                      // let specifier = namespace.local.sym.to_string();
                      // let require_expr = self.construct_require_expr(&src, &specifier);
                      // let require_call = ModuleItem::Stmt(require_expr.into());
                      // module.body.insert(index, require_call);
                    }
                  }
                }
              }
              ModuleDecl::ExportDecl(_) => todo!(),
              ModuleDecl::ExportNamed(_) => todo!(),
              ModuleDecl::ExportDefaultDecl(_) => todo!(),
              ModuleDecl::ExportDefaultExpr(_) => todo!(),
              ModuleDecl::ExportAll(_) => todo!(),
              ModuleDecl::TsImportEquals(_)
              | ModuleDecl::TsExportAssignment(_)
              | ModuleDecl::TsNamespaceExport(_) => {
                unreachable!()
              }
            },
            ModuleItem::Stmt(stmt) => { /* only handle import/export for module decl */ }
          }
        }
      }
      ModuleSystem::CommonJs => {
        // prepend every require call with `await`, require('xxx') -> await require('xxx')
      }
      ModuleSystem::Hybrid => {
        // transform import to require first. and then prepend every require call with `await`
      }
      ModuleSystem::Custom(ty) => {
        if ty == "unknown" {
          panic!(
            "unknown module system for module {}",
            self.module_id.relative_path()
          )
        } else {
          // do nothing, leave it to the custom plugins
        }
      }
    }

    module.visit_mut_children_with(self);
  }
}
