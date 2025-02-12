use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{
    meta_data::script::CommentsMetaData, module_graph::ModuleGraph, ModuleId, ModuleSystem,
  },
  resource::meta_data::js::RenderModuleResult,
  swc_common::{
    comments::SingleThreadedComments, util::take::Take, Mark, SourceMap, SyntaxContext,
  },
  swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr, EsVersion, Expr, ExprStmt, FnExpr},
  HashMap,
};

use farmfe_toolkit::{
  script::{
    module2cjs::{transform_module_decls, OriginalRuntimeCallee, TransformModuleDeclsOptions},
    swc_try_with::try_with,
  },
  swc_ecma_transforms::{
    fixer,
    hygiene::{hygiene_with_config, Config as HygieneConfig},
  },
  swc_ecma_transforms_base::fixer::paren_remover,
  swc_ecma_visit::VisitMutWith,
};

use farmfe_core::{
  config::{FARM_DYNAMIC_REQUIRE, FARM_MODULE, FARM_MODULE_EXPORT, FARM_REQUIRE},

  swc_common::DUMMY_SP,
  swc_ecma_ast::{BindingIdent, BlockStmt, Function, Module as SwcModule, ModuleItem, Param, Stmt}, // swc_ecma_ast::Function
};

use super::{
  source_replacer::{ExistingCommonJsRequireVisitor, SourceReplacer, SourceReplacerOptions},
  transform_async_module,
};

pub struct RenderModuleOptions<'a> {
  pub module_id: ModuleId,
  pub ast: SwcModule,
  pub comments: CommentsMetaData,
  pub hoisted_sourcemap: Arc<SourceMap>,
  pub module_graph: &'a ModuleGraph,
  pub hoisted_external_modules: HashMap<(String, farmfe_core::plugin::ResolveKind), ModuleId>,
  pub context: &'a Arc<CompilationContext>,
}

pub fn render_module(
  options: RenderModuleOptions,
) -> farmfe_core::error::Result<RenderModuleResult> {
  let RenderModuleOptions {
    module_id,
    ast: mut cloned_module,
    hoisted_sourcemap: cm,
    comments,
    module_graph,
    hoisted_external_modules,
    context,
  } = options;
  let comments = SingleThreadedComments::from(comments);
  let module_script_meta = module_graph.module(&module_id).unwrap().meta.as_script();
  let is_async_module = module_script_meta.is_async;

  let mut external_modules = vec![];

  let mut func_expr = Expr::default();

  try_with(cm.clone(), &context.meta.script.globals, || {
    let (unresolved_mark, top_level_mark) = {
      let unresolved_mark = Mark::from_u32(module_script_meta.unresolved_mark);
      let top_level_mark = Mark::from_u32(module_script_meta.top_level_mark);

      (unresolved_mark, top_level_mark)
    };

    // replace commonjs require('./xxx') to require('./xxx', true)
    if matches!(
      module_script_meta.module_system,
      ModuleSystem::CommonJs | ModuleSystem::Hybrid
    ) {
      cloned_module.visit_mut_with(&mut ExistingCommonJsRequireVisitor::new(
        unresolved_mark,
        top_level_mark,
        module_graph,
        module_id.clone(),
      ));
    }

    cloned_module.visit_mut_with(&mut paren_remover(Some(&comments)));

    // ESM to commonjs, then commonjs to farm's runtime module systems
    if matches!(
      module_script_meta.module_system,
      ModuleSystem::EsModule | ModuleSystem::Hybrid
    ) {
      transform_module_decls(
        &mut cloned_module,
        unresolved_mark,
        &OriginalRuntimeCallee { unresolved_mark },
        TransformModuleDeclsOptions {
          is_target_legacy: context.config.script.is_target_legacy(),
        },
      );
    }

    // replace import source with module id
    let mut source_replacer = SourceReplacer::new(SourceReplacerOptions {
      unresolved_mark,
      top_level_mark,
      module_graph,
      module_id: module_id.clone(),
      hoisted_external_modules,
      mode: context.config.mode,
    });
    cloned_module.visit_mut_with(&mut source_replacer);
    cloned_module.visit_mut_with(&mut hygiene_with_config(HygieneConfig {
      top_level_mark,
      ..Default::default()
    }));

    if matches!(module_script_meta.module_system, ModuleSystem::EsModule) && is_async_module {
      // transform async module to meet the requirements of farm runtime
      transform_async_module::transform_async_module(&mut cloned_module);
    }
    // swc code gen would emit a trailing `;` when is_target_legacy is false.
    // we can not deal with this situation for now, so we set is_target_legacy to true here, it will be fixed in the future.
    let mut expr = wrap_function(
      cloned_module,
      is_async_module,
      context.config.script.target == EsVersion::Es5,
    );

    expr.visit_mut_with(&mut fixer(Some(&comments)));
    func_expr = expr;

    external_modules = source_replacer.external_modules;
  })?;

  Ok(RenderModuleResult {
    module_id: module_id.clone(),
    comments: comments.into(),
    hoisted_module_ids: vec![],
    rendered_ast: SwcModule {
      span: DUMMY_SP,
      shebang: None,
      body: vec![ModuleItem::Stmt(Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(func_expr),
      }))],
    },
    external_modules,
  })
}

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
fn wrap_function(mut module: SwcModule, is_async_module: bool, is_target_legacy: bool) -> Expr {
  let body = module.body.take();

  let params = vec![
    Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
        id: FARM_MODULE.into(),
        type_ann: None,
      }),
    },
    Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
        id: FARM_MODULE_EXPORT.into(),
        type_ann: None,
      }),
    },
    Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
        id: FARM_REQUIRE.into(),
        type_ann: None,
      }),
    },
    Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: farmfe_core::swc_ecma_ast::Pat::Ident(BindingIdent {
        id: FARM_DYNAMIC_REQUIRE.into(),
        type_ann: None,
      }),
    },
  ];

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
