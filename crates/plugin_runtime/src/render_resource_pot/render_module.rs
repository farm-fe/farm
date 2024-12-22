use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  error::CompilationError,
  module::{
    meta_data::script::CommentsMetaData, module_graph::ModuleGraph, Module, ModuleId, ModuleSystem,
  },
  swc_common::{
    comments::SingleThreadedComments, util::take::Take, Mark, SourceMap, SyntaxContext,
  },
  swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr, Expr, ExprStmt, FnExpr},
};
use farmfe_plugin_bundle::resource_pot_to_bundle::GeneratorAstResult;
use farmfe_toolkit::{
  minify::minify_js_module,
  script::{
    codegen_module,
    generator::RenderModuleResult,
    module2cjs::{transform_module_decls, OriginalRuntimeCallee, TransformModuleDeclsOptions},
    swc_try_with::{resolve_module_mark, try_with},
    CodeGenCommentsConfig,
  },
  source_map::{build_source_map, create_swc_source_map},
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
  pub module: &'a Module,
  pub hoisted_ast: Option<GeneratorAstResult>,
  pub module_graph: &'a ModuleGraph,
  pub context: &'a Arc<CompilationContext>,
}

pub fn render_module(
  options: RenderModuleOptions,
) -> farmfe_core::error::Result<RenderModuleResult> {
  let RenderModuleOptions {
    module,
    hoisted_ast,
    module_graph,
    context,
  } = options;
  let is_async_module = module.meta.as_script().is_async;
  let is_use_hoisted = hoisted_ast.is_some();

  let (mut cloned_module, comments) =
    if let Some(GeneratorAstResult { ast, comments, .. }) = hoisted_ast {
      (ast, SingleThreadedComments::from(comments))
    } else {
      let script = module.meta.as_script();
      (script.ast.clone(), script.comments.clone().into())
    };
  let (cm, _) = context
    .meta
    .script
    .create_swc_source_map(&module.id, module.content.clone());

  let mut external_modules = vec![];

  let mut func_expr = Expr::default();

  try_with(cm.clone(), &context.meta.script.globals, || {
    let (unresolved_mark, top_level_mark) = if module.meta.as_script().unresolved_mark == 0
      && module.meta.as_script().top_level_mark == 0
    {
      resolve_module_mark(
        &mut cloned_module,
        module.module_type.is_typescript(),
        context,
      )
    } else {
      let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);
      let top_level_mark = Mark::from_u32(module.meta.as_script().top_level_mark);
      (unresolved_mark, top_level_mark)
    };

    // replace commonjs require('./xxx') to require('./xxx', true)
    if matches!(
      module.meta.as_script().module_system,
      ModuleSystem::CommonJs | ModuleSystem::Hybrid
    ) {
      cloned_module.visit_mut_with(&mut ExistingCommonJsRequireVisitor::new(
        unresolved_mark,
        top_level_mark,
      ));
    }

    cloned_module.visit_mut_with(&mut paren_remover(Some(&comments)));

    // ESM to commonjs, then commonjs to farm's runtime module systems
    if matches!(
      module.meta.as_script().module_system,
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
      module_id: module.id.clone(),
      mode: context.config.mode.clone(),
      target_env: context.config.output.target_env.clone(),
      is_strict_find_source: !is_use_hoisted,
    });
    cloned_module.visit_mut_with(&mut source_replacer);
    cloned_module.visit_mut_with(&mut hygiene_with_config(HygieneConfig {
      top_level_mark,
      ..Default::default()
    }));

    if matches!(
      module.meta.as_script().module_system,
      ModuleSystem::EsModule
    ) && is_async_module
    {
      // transform async module to meet the requirements of farm runtime
      transform_async_module::transform_async_module(&mut cloned_module);
    }
    // swc code gen would emit a trailing `;` when is_target_legacy is false.
    // we can not deal with this situation for now, so we set is_target_legacy to true here, it will be fixed in the future.
    let mut expr = wrap_function(cloned_module, is_async_module, true);

    // if minify_enabled {
    //   minify_js_module(
    //     &mut cloned_module,
    //     cm.clone(),
    //     &comments,
    //     unresolved_mark,
    //     top_level_mark,
    //     minify_builder.minify_options.as_ref().unwrap(),
    //   );
    // }

    expr.visit_mut_with(&mut fixer(Some(&comments)));
    func_expr = expr;

    external_modules = source_replacer.external_modules;
  })?;

  // // remove shebang
  // cloned_module.shebang = None;

  // let sourcemap_enabled = context.config.sourcemap.enabled(module.immutable);
  // // wrap module function
  // // let wrapped_module = wrap_module_ast(cloned_module);
  // let mut mappings = vec![];
  // let code_bytes = codegen_module(
  //   &cloned_module,
  //   context.config.script.target.clone(),
  //   cm.clone(),
  //   if sourcemap_enabled {
  //     Some(&mut mappings)
  //   } else {
  //     None
  //   },
  //   context.config.minify.enabled(),
  //   Some(CodeGenCommentsConfig {
  //     comments: &comments,
  //     // preserve all comments when generate module code.
  //     config: &context.config.comments,
  //   }),
  // )
  // .map_err(|e| CompilationError::RenderScriptModuleError {
  //   id: module.id.to_string(),
  //   source: Some(Box::new(e)),
  // })?;

  // let code = Arc::new(String::from_utf8(code_bytes).unwrap());

  // let mut rendered_module = RenderedModule {
  //   id: module.id.clone(),
  //   rendered_content: code.clone(),
  //   rendered_map: None,
  //   rendered_length: code.len(),
  //   original_length: module.content.len(),
  // };
  // let mut source_map_chain = vec![];

  // if sourcemap_enabled {
  //   let sourcemap = build_source_map(cm, &mappings);
  //   let mut buf = vec![];
  //   sourcemap
  //     .to_writer(&mut buf)
  //     .map_err(|e| CompilationError::RenderScriptModuleError {
  //       id: module.id.to_string(),
  //       source: Some(Box::new(e)),
  //     })?;
  //   let map = Arc::new(String::from_utf8(buf).unwrap());
  //   rendered_module.rendered_map = Some(map.clone());

  //   source_map_chain = module.source_map_chain.clone();
  //   source_map_chain.push(map);
  // }

  Ok(RenderModuleResult {
    module_id: module.id.clone(),
    comments: comments.into(),
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
