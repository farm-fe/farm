use std::{collections::HashMap, sync::Arc};

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
  script::module2cjs::{self, TransformModuleDeclsOptions},
  swc_ecma_transforms::{
    feature::enable_available_feature_from_es_version,
    modules::{
      common_js,
      import_analysis::import_analyzer,
      path::Resolver,
      util::{Config as SwcConfig, ImportInterop},
    },
  },
  swc_ecma_visit::VisitMutWith,
};

use crate::resource_pot_to_bundle::{
  bundle::{bundle_reference::BundleReference, ModuleAnalyzerManager, ModuleGlobalUniqName},
  modules_analyzer::module_analyzer::ModuleAnalyzer,
  polyfill::{Polyfill, SimplePolyfill},
  targets::util::wrap_commonjs,
  uniq_name::BundleVariable,
  ShareBundleContext, ShareBundleOptions,
};

use super::util::CJSReplace;

pub struct CjsPatch {}

impl CjsPatch {
  fn wrap_commonjs(
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

  fn to_cjs(
    module_id: &ModuleId,
    ast: &mut EcmaAstModule,
    module_graph: &ModuleGraph,
    unresolved_mark: Mark,
    context: &Arc<CompilationContext>,
    options: &ShareBundleContext,
  ) {
    if options.options.concatenation_module {
      module2cjs::transform_module_decls(
        ast,
        unresolved_mark,
        &module2cjs::OriginalRuntimeCallee { unresolved_mark },
        TransformModuleDeclsOptions {
          is_target_legacy: context.config.script.is_target_legacy(),
        },
      );
    } else {
      ast.visit_mut_with(&mut import_analyzer(ImportInterop::Swc, true));

      ast.visit_mut_with(&mut common_js(
        Resolver::Default,
        unresolved_mark,
        SwcConfig {
          ignore_dynamic: true,
          preserve_import_meta: true,
          ..Default::default()
        },
        enable_available_feature_from_es_version(context.config.script.target),
      ));
    }
  }

  /// transform hybrid and commonjs module to esm
  pub fn transform_hybrid_or_commonjs_to_esm(
    module_analyzer_manager: &mut ModuleAnalyzerManager,
    module_id: &ModuleId,
    context: &Arc<CompilationContext>,
    bundle_variable: &BundleVariable,
    bundle_reference: &mut BundleReference,
    polyfill: &mut SimplePolyfill,
    options: &ShareBundleContext,
  ) -> Result<()> {
    let module_analyzer = module_analyzer_manager
      .module_map
      .get_mut(module_id)
      .unwrap();

    let unresolved_mark = module_analyzer.mark.0;
    // if hybrid module, should transform to cjs
    if matches!(module_analyzer.module_system, ModuleSystem::Hybrid) {
      CjsPatch::to_cjs(
        module_id,
        &mut module_analyzer.ast,
        module_analyzer_manager.module_graph,
        unresolved_mark,
        context,
        options,
      );
    }

    let mut ast = module_analyzer.ast.body.take();
    // if commonjs module, should wrap function
    // see [Polyfill::WrapCommonJs]
    if module_analyzer.is_commonjs() {
      ast = CjsPatch::wrap_commonjs(
        module_id,
        bundle_variable,
        &module_analyzer_manager.module_global_uniq_name,
        ast,
        context.config.mode.clone(),
        polyfill,
      )?;
    }

    module_analyzer_manager.set_ast_body(module_id, ast);

    Ok(())
  }

  pub fn replace_cjs_require<'a>(
    mark: (Mark, Mark),
    ast: &mut EcmaAstModule,
    module_id: &'a ModuleId,
    bundle_variable: &'a BundleVariable,
    config: &'a Config,
    polyfill: &'a mut SimplePolyfill,
    external_config: &'a ExternalConfig,
    bundle_reference: &'a mut BundleReference,
    module_graph: &ModuleGraph,
    module_global_uniq_name: &ModuleGlobalUniqName,
    module_map: &HashMap<ModuleId, ModuleAnalyzer>,
    options: &'a ShareBundleContext,
  ) {
    let mut replacer: CJSReplace = CJSReplace {
      unresolved_mark: mark.0,
      top_level_mark: mark.1,
      module_id: module_id.clone(),
      bundle_variable,
      config,
      polyfill,
      external_config,
      module_global_uniq_name,
      module_graph,
      bundle_reference,
      module_map,
      context: options,
    };

    ast.visit_mut_with(&mut replacer);
  }
}
