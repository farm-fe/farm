use std::sync::Arc;

use farmfe_core::{
  HashMap,
  config::{FARM_DYNAMIC_REQUIRE, FARM_MODULE, FARM_MODULE_EXPORT, FARM_REQUIRE},
  context::CompilationContext,
  module::{
    ModuleId, ModuleSystem, meta_data::script::CommentsMetaData, module_graph::ModuleGraph,
  },
  resource::meta_data::js::RenderModuleResult,
  swc_common::{Globals, Mark, SourceMap, SyntaxContext, comments::SingleThreadedComments},
  swc_ecma_ast::{EsVersion, Expr, ExprStmt, Ident},
};

use farmfe_toolkit::{
  script::{
    module2cjs::{OriginalRuntimeCallee, TransformModuleDeclsOptions, transform_module_decls},
    swc_try_with::try_with,
    wrap_farm_runtime,
  },
  swc_ecma_transforms::{
    fixer,
    hygiene::{Config as HygieneConfig, hygiene_with_config},
  },
  swc_ecma_transforms_base::fixer::paren_remover,
  swc_ecma_visit::{Visit, VisitMutWith, VisitWith},
};

use farmfe_core::{
  swc_common::DUMMY_SP,
  swc_ecma_ast::{Module as SwcModule, ModuleItem, Stmt}, // swc_ecma_ast::Function
};

use super::{
  source_replacer::{ExistingCommonJsRequireVisitor, SourceReplacer, SourceReplacerOptions},
  transform_async_module,
};

pub struct RenderModuleOptions<'a> {
  pub module_id: ModuleId,
  pub is_concatenated: bool,
  pub ast: SwcModule,
  pub comments: CommentsMetaData,
  pub hoisted_sourcemap: Arc<SourceMap>,
  pub module_graph: &'a ModuleGraph,
  pub hoisted_external_modules: HashMap<(String, farmfe_core::plugin::ResolveKind), ModuleId>,
  pub globals: &'a Globals,
  pub unresolved_mark: Mark,
  pub top_level_mark: Mark,
  pub context: &'a Arc<CompilationContext>,
}

pub fn render_module(
  options: RenderModuleOptions,
) -> farmfe_core::error::Result<RenderModuleResult> {
  let RenderModuleOptions {
    module_id,
    is_concatenated,
    ast: mut cloned_module,
    hoisted_sourcemap: cm,
    comments,
    module_graph,
    hoisted_external_modules,
    globals,
    unresolved_mark,
    top_level_mark,
    context,
  } = options;

  if unresolved_mark.as_u32() == 0 || top_level_mark.as_u32() == 0 {
    panic!(
      "unresolved_mark of {:?} is 0. This is may be a bug of farm. Please report it to https://github.com/farm-fe/farm/issues",
      module_id
    );
  }

  let comments = SingleThreadedComments::from(comments);
  let module_script_meta = module_graph.module(&module_id).unwrap().meta.as_script();
  let is_async_module = module_script_meta.is_async;

  let mut external_modules = vec![];
  let mut func_expr = Expr::default();

  if !is_concatenated {
    farmfe_core::swc_common::GLOBALS.set(globals, || {
      farmfe_toolkit::script::entry_live_bindings::instrument(
        &module_id,
        &mut cloned_module,
        module_graph,
        context,
      )
    })?;
  }
  try_with(cm.clone(), globals, || {
    // replace commonjs require('./xxx') to require('./xxx', true)
    if matches!(
      module_script_meta.module_system,
      ModuleSystem::CommonJs | ModuleSystem::Hybrid | ModuleSystem::EsModule
    ) {
      let mut require_visitor = ExistingCommonJsRequireVisitor::new(
        unresolved_mark,
        top_level_mark,
        module_graph,
        module_id.clone(),
        context.config.output.target_env.is_node(),
        &hoisted_external_modules,
      );
      cloned_module.visit_mut_with(&mut require_visitor);
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
          ..Default::default()
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

    // `wrap_function` below declares these names as the wrapper's parameters.
    // A module that never references one of them (e.g. `module` in an ES
    // module) would keep a top level binding of the same name through
    // hygiene, and that binding would then collide with the parameter. So
    // reference only names bound at module scope while hygiene runs. Nested
    // parameters can safely shadow the wrapper and must not be renamed.
    struct TopLevelWrapperBindings {
      mark: Mark,
      names: farmfe_core::HashSet<String>,
    }
    impl Visit for TopLevelWrapperBindings {
      fn visit_ident(&mut self, ident: &Ident) {
        if ident.ctxt.outer() == self.mark {
          self.names.insert(ident.sym.to_string());
        }
      }
    }
    let mut bindings = TopLevelWrapperBindings {
      mark: top_level_mark,
      names: Default::default(),
    };
    cloned_module.visit_with(&mut bindings);
    let wrapper_params: Vec<_> = [
      FARM_MODULE,
      FARM_MODULE_EXPORT,
      FARM_REQUIRE,
      FARM_DYNAMIC_REQUIRE,
    ]
    .into_iter()
    .filter(|name| bindings.names.contains(*name))
    .collect();
    cloned_module.body.splice(
      0..0,
      wrapper_params.iter().map(|name| {
        ModuleItem::Stmt(Stmt::Expr(ExprStmt {
          span: DUMMY_SP,
          expr: Box::new(Expr::Ident(Ident::new(
            (*name).into(),
            DUMMY_SP,
            SyntaxContext::empty().apply_mark(unresolved_mark),
          ))),
        }))
      }),
    );
    cloned_module.visit_mut_with(&mut hygiene_with_config(HygieneConfig {
      top_level_mark,
      ..Default::default()
    }));
    cloned_module.body.drain(0..wrapper_params.len());

    if matches!(
      module_script_meta.module_system,
      ModuleSystem::EsModule | ModuleSystem::Hybrid
    ) && is_async_module
    {
      // transform async module to meet the requirements of farm runtime
      transform_async_module::transform_async_module(&mut cloned_module);
    }
    // swc code gen would emit a trailing `;` when is_target_legacy is false.
    // we can not deal with this situation for now, so we set is_target_legacy to true here, it will be fixed in the future.
    let mut expr = wrap_farm_runtime::wrap_function(
      cloned_module,
      is_async_module,
      context.config.script.target == EsVersion::Es5,
      true,
      unresolved_mark,
    );

    expr.visit_mut_with(&mut fixer(Some(&comments)));
    func_expr = expr;

    external_modules.extend(source_replacer.external_modules);
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

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use farmfe_core::{
    config::Config,
    context::CompilationContext,
    module::{
      Module, ModuleId, ModuleMetaData, ModuleSystem, ModuleType, module_graph::ModuleGraph,
    },
    swc_common::{GLOBALS, Globals, Mark},
    swc_ecma_ast::EsVersion,
    swc_ecma_parser::Syntax,
  };
  use farmfe_toolkit::{
    script::{codegen_module, parse_module},
    swc_ecma_transforms::resolver,
    swc_ecma_visit::VisitMutWith,
  };

  use super::{RenderModuleOptions, render_module};

  fn render(code: &str) -> String {
    let globals = Globals::new();
    let module_id: ModuleId = "a.js".into();
    let parsed = parse_module(
      &module_id,
      Arc::new(code.to_string()),
      Syntax::Es(Default::default()),
      EsVersion::Es2022,
    )
    .unwrap();
    let cm = parsed.source_map.clone();

    let (ast, unresolved_mark, top_level_mark) = GLOBALS.set(&globals, || {
      let unresolved_mark = Mark::new();
      let top_level_mark = Mark::new();
      let mut ast = parsed.ast;
      ast.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));
      (ast, unresolved_mark, top_level_mark)
    });

    let mut module_graph = ModuleGraph::new();
    let mut module = Module::new(module_id.clone());
    module.module_type = ModuleType::Js;
    *module.meta = ModuleMetaData::Script(Default::default());
    module.meta.as_script_mut().module_system = ModuleSystem::EsModule;
    module_graph.add_module(module);
    let context = Arc::new(CompilationContext::new(Config::default(), vec![]).unwrap());

    let result = render_module(RenderModuleOptions {
      is_concatenated: false,
      module_id,
      ast,
      comments: parsed.comments.into(),
      hoisted_sourcemap: cm.clone(),
      module_graph: &module_graph,
      hoisted_external_modules: Default::default(),
      globals: &globals,
      unresolved_mark,
      top_level_mark,
      context: &context,
    })
    .unwrap();

    let code = GLOBALS.set(&globals, || {
      codegen_module(&result.rendered_ast, cm, None, Default::default(), None).unwrap()
    });
    String::from_utf8(code).unwrap()
  }

  #[test]
  fn top_level_bindings_do_not_collide_with_wrapper_params() {
    let code = render(
      r#"
const module = "m";
let exports = 1;
var farmRequire = 2;
function farmDynamicRequire() {}
export const v = [module, exports, farmRequire, farmDynamicRequire];
"#,
    );

    for name in ["module", "exports", "farmRequire", "farmDynamicRequire"] {
      for decl in [
        format!("const {name} ="),
        format!("let {name} ="),
        format!("var {name} ="),
        format!("function {name}("),
      ] {
        assert!(
          !code.contains(&decl),
          "`{decl}` collides with a wrapper param:\n{code}"
        );
      }
    }
  }
  #[test]
  fn nested_parameters_can_shadow_wrapper_names_without_renaming() {
    let code = render(
      "export function resolve(module, props, Loadable) { return [module, props, Loadable]; }",
    );
    assert!(
      code.contains("function resolve(module, props, Loadable)"),
      "{code}"
    );
    assert!(!code.contains("module1"), "{code}");
  }
}
