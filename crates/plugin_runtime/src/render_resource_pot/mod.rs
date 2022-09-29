use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{module_graph::ModuleGraph, ModuleSystem},
  resource::resource_pot::ResourcePot,
  swc_common::{comments::SingleThreadedComments, Mark, DUMMY_SP, GLOBALS},
  swc_ecma_ast::{
    BlockStmt, Expr, FnExpr, Function, Ident, KeyValueProp, Module as SwcModule, ModuleItem,
    ObjectLit, Param, Pat, Prop, PropName, PropOrSpread, Str,
  },
};
use farmfe_toolkit::{
  swc_ecma_transforms::{
    feature::enable_available_feature_from_es_version,
    fixer,
    helpers::{inject_helpers, Helpers, HELPERS},
    modules::{common_js, import_analysis::import_analyzer, util::ImportInterop},
  },
  swc_ecma_visit::VisitMutWith,
};

use self::source_replacer::SourceReplacer;

mod source_replacer;
mod tla_transformer;

/// Merge all modules' ast in a [ResourcePot] to Farm's runtime [ObjectLit]. The [ObjectLit] looks like:
/// ```js
/// {
///   // commonjs or hybrid module system
///   "a.js": async function(module, exports, require) {
///       const b = await require('./b');
///       console.log(b);
///    },
///    // esm module system
///    "b.js": async function(module, exports, require) {
///       Promise.all([
///         require('./c'),
///         require('./d')
///       ]).then(([c, d]) => {
///       exports.c = c;
///       exports.d = d;
///     });
///    }
/// }
/// ```
pub fn render_resource_pot(
  resource_pot: &mut ResourcePot,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> ObjectLit {
  let mut rendered_resource_ast = ObjectLit {
    span: DUMMY_SP,
    props: vec![],
  };

  // TODO parallelize here
  for m_id in resource_pot.modules() {
    let module = module_graph.module(m_id).unwrap();
    let mut cloned_module = SwcModule {
      shebang: None,
      span: DUMMY_SP,
      body: module.meta.as_script().ast.body.to_vec(),
    };

    GLOBALS.set(&context.meta.script.globals, || {
      HELPERS.set(&Helpers::new(true), || {
        // transform esm to commonjs
        let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);

        // ESM to commonjs, then commonjs to farm's runtime module systems
        if matches!(
          module.meta.as_script().module_system,
          ModuleSystem::EsModule
        ) {
          cloned_module.visit_mut_with(&mut import_analyzer(ImportInterop::Swc, true));
          cloned_module.visit_mut_with(&mut inject_helpers());
          cloned_module.visit_mut_with(&mut common_js::<SingleThreadedComments>(
            unresolved_mark,
            Default::default(),
            enable_available_feature_from_es_version(context.config.script.target.clone()),
            None,
          ));
        }

        // replace import source with module id
        let mut source_replacer = SourceReplacer::new(
          unresolved_mark,
          module_graph,
          m_id.clone(),
          module.meta.as_script().module_system.clone(),
          context.config.mode.clone(),
        );
        cloned_module.visit_mut_with(&mut source_replacer);
        // TODO support comments
        cloned_module.visit_mut_with(&mut fixer(None));
      });
    });

    // wrap module function
    let wrapped_module = wrap_module_ast(cloned_module);

    rendered_resource_ast
      .props
      .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: PropName::Str(Str {
          span: DUMMY_SP,
          value: module.id.id(context.config.mode.clone()).into(),
          raw: None,
        }),
        value: Box::new(Expr::Fn(FnExpr {
          ident: None,
          function: wrapped_module,
        })),
      }))))
  }

  // TODO transform async function if target is lower than es2017

  rendered_resource_ast
}

/// Wrap the module ast to follow Farm's commonjs-style module system spec.
/// Note: this function won't render the esm to commonjs, if you want to render esm to commonjs, see [esm_to_commonjs].
///
/// For example:
/// ```js
/// const b = require('./b');
/// console.log(b);
/// exports.b = b;
/// ```
/// will be rendered to
/// ```js
/// async function(module, exports, require) {
///   const b = require('./b');
///   console.log(b);
///   exports.b = b;
/// }
/// ```
fn wrap_module_ast(ast: SwcModule) -> Function {
  let params = vec!["module", "exports", "require"]
    .into_iter()
    .map(|ident| Param {
      span: DUMMY_SP,
      decorators: vec![],
      pat: Pat::Ident(
        Ident {
          span: DUMMY_SP,
          sym: ident.into(),
          optional: false,
        }
        .into(),
      ),
    })
    .collect();

  let body = Some(BlockStmt {
    span: DUMMY_SP,
    stmts: ast
      .body
      .to_vec()
      .into_iter()
      .map(|item| match item {
        ModuleItem::ModuleDecl(decl) => {
          panic!(
            "should transform all esm module item to commonjs first! {:?}",
            decl
          )
        }
        ModuleItem::Stmt(stmt) => stmt,
      })
      .collect(),
  });

  Function {
    params,
    decorators: vec![],
    span: DUMMY_SP,
    body,
    is_generator: false,
    is_async: true,
    type_params: None,
    return_type: None,
  }
}
