use farmfe_core::{
  config::{FARM_DYNAMIC_REQUIRE, FARM_MODULE, FARM_MODULE_EXPORT, FARM_REQUIRE},
  swc_common::{util::take::Take, Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ArrowExpr, BindingIdent, BlockStmt, BlockStmtOrExpr, Expr, FnExpr, Function, Ident,
    Module as SwcModule, ModuleItem, Param,
  },
};

/// Wrap the module ast to follow Farm's commonjs-style module system.
/// Note: this function won't render the esm to commonjs, if you want to render esm to commonjs, see [common_js].
///
/// For example:
/// ```js
/// const b = farmRequire('./b');
/// console.log(b);
/// exports.b = b;
/// ```
/// will be rendered to
/// ```js
/// function(module, exports, farmRequire) {
///   const b = farmRequire('./b');
///   console.log(b);
///   exports.b = b;
/// }
/// ```
pub fn wrap_function(
  mut module: SwcModule,
  is_async_module: bool,
  is_target_legacy: bool,
  is_add_require: bool,
  unresolved_mark: Mark,
) -> Expr {
  let body = module.body.take();

  let mut params = vec![
    Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
        id: Ident::new(
          FARM_MODULE.into(),
          DUMMY_SP,
          SyntaxContext::empty().apply_mark(unresolved_mark),
        ),
        type_ann: None,
      }),
    },
    Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
        id: Ident::new(
          FARM_MODULE_EXPORT.into(),
          DUMMY_SP,
          SyntaxContext::empty().apply_mark(unresolved_mark),
        ),
        type_ann: None,
      }),
    },
  ];

  if is_add_require {
    params.push(Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
        id: Ident::new(
          FARM_REQUIRE.into(),
          DUMMY_SP,
          SyntaxContext::empty().apply_mark(unresolved_mark),
        ),
        type_ann: None,
      }),
    });
    params.push(Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
        id: Ident::new(
          FARM_DYNAMIC_REQUIRE.into(),
          DUMMY_SP,
          SyntaxContext::empty().apply_mark(unresolved_mark),
        ),
        type_ann: None,
      }),
    });
  }

  let stmts = body
    .into_iter()
    .map(|body| match body {
      ModuleItem::ModuleDecl(decl) => unreachable!("{:?}", decl),
      ModuleItem::Stmt(stmt) => stmt,
    })
    .collect();

  if !is_target_legacy {
    Expr::Arrow(ArrowExpr {
      span: DUMMY_SP,
      params: params.into_iter().map(|p| p.pat).collect(),
      body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
        span: DUMMY_SP,
        stmts,
        ctxt: SyntaxContext::empty(),
      })),
      is_async: is_async_module,
      is_generator: false,
      type_params: None,
      return_type: None,
      ctxt: SyntaxContext::empty(),
    })
  } else {
    Expr::Fn(FnExpr {
      ident: None,
      function: Box::new(Function {
        params,
        decorators: vec![],
        span: DUMMY_SP,
        body: Some(BlockStmt {
          span: DUMMY_SP,
          stmts,
          ctxt: SyntaxContext::empty(),
        }),
        is_generator: false,
        is_async: is_async_module,
        type_params: None,
        return_type: None,
        ctxt: SyntaxContext::empty(),
      }),
    })
  }
}
