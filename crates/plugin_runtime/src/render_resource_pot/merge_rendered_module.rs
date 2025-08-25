use std::sync::Arc;

use farmfe_core::{
  config::FARM_MODULE_SYSTEM,
  context::CompilationContext,
  module::ModuleId,
  resource::resource_pot::ResourcePotId,
  swc_common::{SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{
    ComputedPropName, Expr, ExprOrSpread, Ident, IdentName, KeyValueProp, Lit, MemberExpr,
    MemberProp, Module as SwcModule, ModuleItem, ObjectLit, Prop, PropName, PropOrSpread,
  },
};
use farmfe_toolkit::script::parse_stmt;

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
  module_asts: &mut Vec<(ModuleId, SwcModule)>,
  context: &Arc<CompilationContext>,
) -> ObjectLit {
  let mut rendered_resource_pot_ast = ObjectLit {
    span: DUMMY_SP,
    props: vec![],
  };

  // insert props to the object lit
  for (module_id, rendered_ast) in module_asts {
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
  let url = {
    if context.config.output.target_env.is_node() {
      if context.config.output.format.contains_cjs() {
        r#"require("url").pathToFileURL(__filename).href"#.to_string()
      } else {
        "import.meta.url".to_string()
      }
    } else {
      format!(
        r#"typeof document === "undefined"
          ? location.href
          : (document.currentScript &&
              document.currentScript.tagName.toUpperCase() === "SCRIPT" &&
              document.currentScript.src) ||
            location.protocol + "//" + location.host + '/' + {:?}"#,
        resource_pot_id
      )
    }
  };
  let mut stmt = parse_stmt(
    resource_pot_id,
    format!(
      r#"(function (moduleSystem, modules) {{
   for (var moduleId in modules) {{
     var module = modules[moduleId];
     module.url = {};
     moduleSystem.g(moduleId, module);
    }}
    }})("farm_module_system", "farm_object_lit");"#,
      url
    )
    .as_str(),
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
