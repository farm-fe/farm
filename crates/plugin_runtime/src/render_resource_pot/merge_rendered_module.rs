use std::{collections::HashSet, sync::Arc};

use farmfe_core::{
  config::FARM_MODULE_SYSTEM,
  context::{self, CompilationContext},
  module::{module_graph::ModuleGraph, ModuleId},
  rayon::{iter::IntoParallelRefMutIterator, prelude::*},
  resource::{meta_data::js::RenderModuleResult, resource_pot::ResourcePotId},
  swc_common::{
    comments::{Comments, SingleThreadedComments},
    BytePos, FilePathMapping, SourceMap, SyntaxContext, DUMMY_SP,
  },
  swc_ecma_ast::{
    ComputedPropName, Expr, ExprOrSpread, Ident, IdentName, KeyValueProp, Lit, MemberExpr,
    MemberProp, Module as SwcModule, ModuleItem, ObjectLit, Prop, PropName, PropOrSpread,
  },
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  script::parse_stmt,
  sourcemap::get_swc_sourcemap_filename,
  swc_ecma_utils::StmtOrModuleItem,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

/// Merge all modules' ast in a [ResourcePot] to Farm's runtime [ObjectLit]. The [ObjectLit] looks like:
/// ```js
/// {
///   // commonjs or hybrid module system
///   "a.js": function(module, exports, require) {
///       const b = require('./b');
///       console.log(b);
///    },
///    // esm module system
///    "b.js": async function(module, exports, require) {
///       const [c, d] = await Promise.all([
///         require('./c'),
///         require('./d')
///       ]);
///
///       exports.c = c;
///       exports.d = d;
///    }
/// }
/// ```
pub(crate) fn merge_rendered_module(
  render_module_results: &mut Vec<RenderModuleResult>,
  context: &Arc<CompilationContext>,
) -> ObjectLit {
  // let cm = merge_sourcemap(&mut render_module_results, module_graph, context);
  // let comments = merge_comments(&mut render_module_results, cm.clone());

  let mut rendered_resource_pot_ast = ObjectLit {
    span: DUMMY_SP,
    props: vec![],
  };

  // insert props to the object lit
  for RenderModuleResult {
    module_id,
    rendered_ast,
    ..
  } in render_module_results
  {
    let mut ast = std::mem::take(rendered_ast);
    // panic if the first item is not a function expression
    let expr = ast.body.remove(0).stmt().unwrap().expr().unwrap().expr;
    rendered_resource_pot_ast
      .props
      .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: PropName::Str(module_id.id(context.config.mode.clone()).into()),
        value: expr,
      }))));
  }

  rendered_resource_pot_ast
}

pub(crate) fn wrap_resource_pot_ast(
  rendered_resource_pot_ast: ObjectLit,
  resource_pot_id: &ResourcePotId,
  context: &Arc<CompilationContext>,
) -> SwcModule {
  let mut stmt = parse_stmt(
    resource_pot_id,
    r#"(function (moduleSystem, modules) {
   for (var moduleId in modules) {
     var module = modules[moduleId];
     moduleSystem.g(moduleId, module);
   }
 })("farm_module_system", "farm_object_lit");"#,
    true,
  )
  .unwrap();

  let args = &mut stmt.as_mut_expr().unwrap().expr.as_mut_call().unwrap().args;

  let global_this = if context.config.output.target_env.is_node() {
    "global"
  } else {
    "window"
  };

  // window['hash'].m;
  args[0] = ExprOrSpread {
    spread: None,
    expr: Box::new(Expr::Member(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(Expr::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(Expr::Ident(Ident::new(
          global_this.into(),
          DUMMY_SP,
          SyntaxContext::empty(),
        ))),
        prop: MemberProp::Computed(ComputedPropName {
          span: DUMMY_SP,
          expr: Box::new(Expr::Lit(Lit::Str(
            context.config.runtime.namespace.as_str().into(),
          ))),
        }),
      })),
      prop: MemberProp::Ident(IdentName::new(FARM_MODULE_SYSTEM.into(), DUMMY_SP)),
    })),
  };
  args[1] = ExprOrSpread {
    spread: None,
    expr: Box::new(Expr::Object(rendered_resource_pot_ast)),
  };

  SwcModule {
    span: DUMMY_SP,
    shebang: None,
    body: vec![ModuleItem::Stmt(stmt)],
  }
}
