use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  error::{CompilationError, Result},
  hashbrown::HashMap,
  module::{module_graph::ModuleGraph, ModuleSystem},
  parking_lot::Mutex,
  rayon::prelude::*,
  resource::resource_pot::ResourcePot,
  swc_common::{comments::SingleThreadedComments, Mark, DUMMY_SP},
  swc_ecma_ast::{
    BlockStmt, Expr, FnExpr, Function, Ident, KeyValueProp, Module as SwcModule, ModuleItem,
    ObjectLit, Param, Pat, Prop, PropName, PropOrSpread, Str,
  },
};
use farmfe_toolkit::{
  script::{codegen_module, swc_try_with::try_with},
  swc_ecma_transforms::{
    feature::enable_available_feature_from_es_version,
    fixer,
    helpers::inject_helpers,
    hygiene::{hygiene_with_config, Config as HygieneConfig},
    modules::{
      common_js,
      import_analysis::import_analyzer,
      util::{Config, ImportInterop},
    },
  },
  swc_ecma_visit::VisitMutWith,
};

use self::source_replacer::{SourceReplacer, DYNAMIC_REQUIRE, FARM_REQUIRE};

// mod farm_module_system; // TODO: replace with farm_module_system later, as soon as it's ready
mod source_replacer;

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
pub fn resource_pot_to_runtime_object_lit(
  resource_pot: Vec<&ResourcePot>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Result<ObjectLit> {
  let mut rendered_resource_ast = ObjectLit {
    span: DUMMY_SP,
    props: vec![],
  };

  let props = Mutex::new(HashMap::new());

  resource_pot
    .into_iter()
    .flat_map(|resource_pot| resource_pot.modules())
    .try_for_each(|m_id| {
      let module = module_graph
        .module(m_id)
        .unwrap_or_else(|| panic!("Module not found: {:?}", m_id));
      let mut cloned_module = SwcModule {
        shebang: None,
        span: DUMMY_SP,
        body: module.meta.as_script().ast.body.to_vec(),
      };

      try_with(
        context.meta.script.cm.clone(),
        &context.meta.script.globals,
        || {
          // transform esm to commonjs
          let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);
          let top_level_mark = Mark::from_u32(module.meta.as_script().top_level_mark);

          // ESM to commonjs, then commonjs to farm's runtime module systems
          if matches!(
            module.meta.as_script().module_system,
            ModuleSystem::EsModule | ModuleSystem::Hybrid
          ) {
            cloned_module.visit_mut_with(&mut import_analyzer(ImportInterop::Swc, true));
            cloned_module.visit_mut_with(&mut inject_helpers(unresolved_mark));
            cloned_module.visit_mut_with(&mut common_js::<SingleThreadedComments>(
              unresolved_mark,
              Config {
                // TODO process dynamic import by ourselves later
                ignore_dynamic: true,
                preserve_import_meta: true,
                ..Default::default()
              },
              enable_available_feature_from_es_version(context.config.script.target),
              None,
            ));
          }

          // replace import source with module id
          let mut source_replacer = SourceReplacer::new(
            unresolved_mark,
            top_level_mark,
            module_graph,
            m_id.clone(),
            module.meta.as_script().module_system.clone(),
            context.config.mode.clone(),
          );
          cloned_module.visit_mut_with(&mut source_replacer);
          cloned_module.visit_mut_with(&mut hygiene_with_config(HygieneConfig {
            top_level_mark,
            ..Default::default()
          }));
          // TODO support comments
          cloned_module.visit_mut_with(&mut fixer(None));
        },
      )?;

      // wrap module function
      let wrapped_module = wrap_module_ast(cloned_module);

      props.lock().insert(
        module.id.id(context.config.mode.clone()),
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
          key: PropName::Str(Str {
            span: DUMMY_SP,
            value: module.id.id(context.config.mode.clone()).into(),
            raw: None,
          }),
          value: Box::new(Expr::Fn(FnExpr {
            ident: None,
            function: Box::new(wrapped_module),
          })),
        }))),
      );
      Ok::<(), CompilationError>(())
    })?;

  // sort props by module id to make sure the order is stable
  let mut props = props.into_inner();
  let mut props: Vec<_> = props.drain().collect();
  props.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
  // insert props to the object lit
  rendered_resource_ast.props = props.into_iter().map(|(_, v)| v).collect();

  Ok(rendered_resource_ast)
}

/// Wrap the module ast to follow Farm's commonjs-style module system.
/// Note: this function won't render the esm to commonjs, if you want to render esm to commonjs, see [common_js].
///
/// For example:
/// ```js
/// const b = require('./b');
/// console.log(b);
/// exports.b = b;
/// ```
/// will be rendered to
/// ```js
/// async function(module, exports, farmRequire) {
///   const b = farmRequire('./b');
///   console.log(b);
///   exports.b = b;
/// }
/// ```
fn wrap_module_ast(ast: SwcModule) -> Function {
  let params = vec!["module", "exports", FARM_REQUIRE, DYNAMIC_REQUIRE]
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
      .iter()
      .cloned()
      .map(|item| match item {
        ModuleItem::ModuleDecl(decl) => {
          let code = codegen_module(
            &SwcModule {
              span: DUMMY_SP,
              shebang: None,
              body: vec![ModuleItem::ModuleDecl(decl)],
            },
            Default::default(),
            Arc::new(Default::default()),
            None,
            false,
          )
          .unwrap();

          panic!(
            "should transform all esm module item to commonjs first! code: {}",
            String::from_utf8(code).unwrap()
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
    is_async: false,
    type_params: None,
    return_type: None,
  }
}
