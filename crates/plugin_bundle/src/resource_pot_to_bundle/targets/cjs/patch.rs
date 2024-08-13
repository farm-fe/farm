use std::sync::Arc;

use farmfe_core::{
  config::{external::ExternalConfig, Config, Mode},
  context::CompilationContext,
  error::Result,
  module::{module_graph::ModuleGraph, ModuleId, ModuleSystem},
  swc_common::{util::take::Take, Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ArrowExpr, BindingIdent, BlockStmt, BlockStmtOrExpr, Decl, EsVersion, Expr, ExprOrSpread,
    Ident, KeyValueProp, Module as EcmaAstModule, ModuleItem, ObjectLit, Pat, Program, Prop,
    PropName, PropOrSpread, Stmt, VarDecl, VarDeclKind, VarDeclarator,
  },
};
use farmfe_toolkit::{
  swc_ecma_transforms::{
    feature::enable_available_feature_from_es_version,
    modules::{
      common_js,
      import_analysis::import_analyzer,
      util::{Config as SwcConfig, ImportInterop},
    },
  },
  swc_ecma_visit::VisitMutWith,
};

use crate::resource_pot_to_bundle::{
  bundle::{bundle_reference::BundleReference, ModuleAnalyzerManager, ModuleGlobalUniqName},
  polyfill::{cjs::wrap_commonjs, Polyfill, SimplePolyfill},
  uniq_name::BundleVariable,
};

use super::{util::CJSReplace, CjsModuleAnalyzer};

pub struct CjsPatch {}

impl CjsPatch {
  pub fn wrap_commonjs(
    module_id: &ModuleId,
    bundle_variable: &BundleVariable,
    module_global_uniq_name: &ModuleGlobalUniqName,
    ast: Vec<ModuleItem>,
    mode: Mode,
    polyfill: &mut SimplePolyfill,
  ) -> Result<Vec<ModuleItem>> {
    polyfill.add(Polyfill::WrapCommonJs);

    let mut patch_ast_items = vec![];

    let result = module_global_uniq_name.commonjs_name(module_id).unwrap();
    let fn_expr = Box::new(Expr::Arrow(ArrowExpr {
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
      body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
        span: DUMMY_SP,
        stmts: ast
          .into_iter()
          .map(|module_item| match module_item {
            // if esm module, should transform to commonjs before
            ModuleItem::ModuleDecl(_) => unreachable!("module_decl should not be here"),
            ModuleItem::Stmt(stmt) => stmt,
          })
          .collect(),
        ctxt: SyntaxContext::empty(),
      })),
      is_async: false,
      is_generator: false,
      type_params: None,
      return_type: None,
      ctxt: SyntaxContext::empty(),
    }));

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
        init: Some(wrap_commonjs(
          vec![ExprOrSpread {
            spread: None,
            expr: match mode {
              Mode::Development => Box::new(Expr::Object(ObjectLit {
                span: DUMMY_SP,
                props: vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                  key: PropName::Str(module_id.to_string().into()),
                  value: fn_expr,
                })))],
              })),
              Mode::Production => fn_expr,
            },
          }],
          polyfill,
        )),
        definite: false,
      }],
      ctxt: SyntaxContext::empty(),
    })))));

    Ok(patch_ast_items)
  }

  pub fn to_cjs(
    module_id: &ModuleId,
    ast: &mut EcmaAstModule,
    module_graph: &ModuleGraph,
    unresolved_mark: Mark,
    es_version: EsVersion,
  ) {
    // let module = module_graph.module(module_id).unwrap();

    // let comments = module.meta.as_script().comments.clone().into();
    let take_ast = std::mem::take(ast);
    let mut program = Program::Module(take_ast);
    program.mutate(&mut import_analyzer(ImportInterop::Swc, true));

    program.mutate(&mut common_js(
      Default::default(),
      unresolved_mark,
      SwcConfig {
        ignore_dynamic: true,
        preserve_import_meta: true,
        ..Default::default()
      },
      enable_available_feature_from_es_version(es_version),
      // Some(&comments),
    ));
    let take_ast = program.expect_module();
    *ast = take_ast;
  }

  /// transform hybrid and commonjs module to esm
  pub fn patch_cjs_module(
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    module_id: &ModuleId,
    module_graph: &ModuleGraph,
    context: &Arc<CompilationContext>,
    patch_ast: &mut Vec<ModuleItem>,
    bundle_variable: &BundleVariable,
    bundle_reference: &BundleReference,
    polyfill: &mut SimplePolyfill,
  ) {
    let module_analyzer = module_analyzer_manager.module_analyzer_mut_unchecked(module_id);

    let unresolved_mark = module_analyzer.mark.0;
    // if hybrid module, should transform to cjs
    if matches!(module_analyzer.module_system, ModuleSystem::Hybrid) {
      CjsPatch::to_cjs(
        module_id,
        &mut module_analyzer.ast,
        module_graph,
        unresolved_mark,
        context.config.script.target,
      );
    }

    // if commonjs module, should wrap function
    // see [Polyfill::WrapCommonJs]
    if module_analyzer.is_commonjs() {
      let ast = module_analyzer.ast.body.take();

      module_analyzer_manager.set_ast_body(
        module_id,
        CjsPatch::wrap_commonjs(
          module_id,
          bundle_variable,
          &module_analyzer_manager.module_global_uniq_name,
          ast,
          context.config.mode.clone(),
          polyfill,
        )
        .unwrap(),
      );
    }
    println!("module_id {:#?}", module_id.to_string());

    if let Some(import) = bundle_reference
      .redeclare_commonjs_import
      .get(&module_id.clone().into())
    {
      println!("patch_cjs_module bundle_reference: {:#?}", import);
      patch_ast.extend(CjsModuleAnalyzer::redeclare_commonjs_export(
        module_id,
        bundle_variable,
        &module_analyzer_manager.module_global_uniq_name,
        import,
        polyfill,
      ));
    }
  }

  pub fn replace_cjs_require(
    mark: (Mark, Mark),
    ast: &mut EcmaAstModule,
    module_id: &ModuleId,
    module_graph: &ModuleGraph,
    module_global_uniq_name: &ModuleGlobalUniqName,
    bundle_variable: &BundleVariable,
    config: &Config,
    polyfill: &mut SimplePolyfill,
    external_config: &ExternalConfig,
  ) {
    let mut replacer: CJSReplace = CJSReplace {
      unresolved_mark: mark.0,
      top_level_mark: mark.1,
      module_graph,
      module_id: module_id.clone(),
      module_global_uniq_name,
      bundle_variable,
      config,
      polyfill,
      external_config,
    };

    ast.visit_mut_with(&mut replacer);
  }
}
