use std::{collections::HashSet, sync::Arc};

use farmfe_core::{
  config::FARM_MODULE_SYSTEM,
  context::{self, CompilationContext},
  module::{module_graph::ModuleGraph, ModuleId},
  rayon::{iter::IntoParallelRefMutIterator, prelude::*},
  resource::resource_pot::ResourcePotId,
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
  source_map::get_swc_sourcemap_filename,
  swc_ecma_utils::StmtOrModuleItem,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

use super::render_module::RenderModuleResult;

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

  // RenderResourcePotAstResult {
  //   rendered_resource_pot_ast,
  //   external_modules: render_module_results
  //     .into_iter()
  //     .map(|item| item.external_modules)
  //     .flatten()
  //     .collect(),
  //   merged_sourcemap: cm,
  //   merged_comments: comments,
  // }
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
     moduleSystem.register(moduleId, module);
   }
 })("farm_module_system", "farm_object_lit");"#,
    true,
  )
  .unwrap();

  let args = &mut stmt.as_mut_expr().unwrap().expr.as_mut_call().unwrap().args;

  // let global_this = get_farm_global_this(
  //   &context.config.runtime.namespace,
  //   &context.config.output.target_env,
  // );
  let global_this = if context.config.output.target_env.is_node() {
    "global"
  } else {
    "window"
  };

  // window['hash'].__farm_module_system__;
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

pub fn merge_sourcemap(
  resource_pot_id: &ResourcePotId,
  render_module_results: &mut Vec<RenderModuleResult>,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Arc<SourceMap> {
  let module_ids = render_module_results
    .iter()
    .map(|item| &item.module_id)
    .collect();
  let new_cm = context
    .meta
    .script
    .merge_swc_source_map(resource_pot_id, module_ids, module_graph);

  // update Span in parallel
  render_module_results.par_iter_mut().for_each(|res| {
    let filename = get_swc_sourcemap_filename(&res.module_id);
    let source_file = new_cm
      .get_source_file(&filename)
      .unwrap_or_else(|| panic!("no source file found for {:?}", res.module_id));
    let start_pos = source_file.start_pos;
    res
      .rendered_ast
      .visit_mut_with(&mut SpanUpdater { start_pos });
  });

  new_cm
}

struct SpanUpdater {
  start_pos: BytePos,
}

impl VisitMut for SpanUpdater {
  fn visit_mut_span(&mut self, node: &mut farmfe_core::swc_common::Span) {
    node.lo = self.start_pos + node.lo;
    node.hi = self.start_pos + node.hi;
  }
}

pub fn merge_comments(
  render_module_results: &mut Vec<RenderModuleResult>,
  cm: Arc<SourceMap>,
) -> SingleThreadedComments {
  let merged_comments = SingleThreadedComments::default();

  for RenderModuleResult {
    module_id,
    comments: module_comments,
    ..
  } in render_module_results
  {
    let filename = get_swc_sourcemap_filename(module_id);
    let source_file = cm
      .get_source_file(&filename)
      .unwrap_or_else(|| panic!("no source file found for {:?}", module_id));
    let start_pos = source_file.start_pos;
    let comments = std::mem::take(module_comments);

    for item in comments.leading {
      let byte_pos = start_pos + item.byte_pos;
      for comment in item.comment {
        merged_comments.add_leading(byte_pos, comment);
      }
    }

    for item in comments.trailing {
      let byte_pos = start_pos + item.byte_pos;
      for comment in item.comment {
        merged_comments.add_trailing(byte_pos, comment);
      }
    }
  }

  merged_comments
}
