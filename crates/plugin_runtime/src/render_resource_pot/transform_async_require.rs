//! Await original CommonJS requires of async graph dependencies in-place.
//! Unlike generated ESM imports, conditional requires must never be hoisted.
use farmfe_core::{
  module::{ModuleId, module_graph::ModuleGraph},
  plugin::ResolveKind,
  swc_common::{DUMMY_SP, Mark, errors::HANDLER},
  swc_ecma_ast::{
    ArrowExpr, AwaitExpr, Callee, Class, Expr, Function, GetterProp, IdentName, KeyValueProp, Lit,
    MemberExpr, MemberProp, Module, Null, Number, ObjectLit, ParenExpr, Prop, PropName,
    PropOrSpread, ReturnStmt, SeqExpr, SetterProp, Stmt, UnaryExpr, UnaryOp,
  },
};
use farmfe_toolkit::{
  script::is_commonjs_require,
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

pub fn transform_async_require(
  ast: &mut Module,
  graph: &ModuleGraph,
  module_id: &ModuleId,
  unresolved: Mark,
  top_level: Mark,
) {
  let is_async = |source: &str| {
    graph
      .get_dep_by_source_optional(module_id, source, Some(ResolveKind::Require))
      .and_then(|id| graph.module(&id))
      .is_some_and(|module| {
        !module.external && module.module_type.is_script() && module.meta.as_script().is_async
      })
  };
  let is_cycle = |source: &str| {
    let Some(dependency) =
      graph.get_dep_by_source_optional(module_id, source, Some(ResolveKind::Require))
    else {
      return false;
    };
    let mut pending = vec![dependency];
    let mut visited = farmfe_core::HashSet::default();
    while let Some(id) = pending.pop() {
      if &id == module_id {
        return true;
      }
      if visited.insert(id.clone()) {
        pending.extend(
          graph
            .dependencies(&id)
            .into_iter()
            .filter(|(_, edge)| !edge.is_dynamic_import() && !edge.is_dynamic_entry())
            .map(|(id, _)| id),
        );
      }
    }
    false
  };
  ast.visit_mut_with(&mut AsyncRequire {
    is_async: &is_async,
    is_cycle: &is_cycle,
    unresolved,
    top_level,
    allow_await: true,
    changes: 0,
  });
}

struct AsyncRequire<'a> {
  is_async: &'a dyn Fn(&str) -> bool,
  is_cycle: &'a dyn Fn(&str) -> bool,
  unresolved: Mark,
  top_level: Mark,
  allow_await: bool,
  changes: usize,
}
impl VisitMut for AsyncRequire<'_> {
  fn visit_mut_function(&mut self, function: &mut Function) {
    let previous = self.allow_await;
    let changes = self.changes;
    // Parameters cannot contain await, even for async functions.
    self.allow_await = false;
    function.params.visit_mut_with(self);
    self.allow_await = function.is_async;
    function.body.visit_mut_with(self);
    self.allow_await = previous;
    self.changes = changes;
  }
  fn visit_mut_arrow_expr(&mut self, arrow: &mut ArrowExpr) {
    let previous = self.allow_await;
    let changes = self.changes;
    self.allow_await = false;
    arrow.params.visit_mut_with(self);
    self.allow_await = arrow.is_async;
    arrow.body.visit_mut_with(self);
    self.allow_await = previous;
    self.changes = changes;
  }
  fn visit_mut_getter_prop(&mut self, getter: &mut GetterProp) {
    // Computed keys execute in the surrounding scope, not the accessor body.
    getter.key.visit_mut_with(self);
    let previous = self.allow_await;
    let changes = self.changes;
    self.allow_await = false;
    getter.body.visit_mut_with(self);
    self.allow_await = previous;
    self.changes = changes;
  }
  fn visit_mut_setter_prop(&mut self, setter: &mut SetterProp) {
    setter.key.visit_mut_with(self);
    let previous = self.allow_await;
    let changes = self.changes;
    self.allow_await = false;
    setter.param.visit_mut_with(self);
    setter.body.visit_mut_with(self);
    self.allow_await = previous;
    self.changes = changes;
  }
  fn visit_mut_class(&mut self, class: &mut Class) {
    let previous = self.allow_await;
    let changes = self.changes;
    self.allow_await = false;
    class.visit_mut_children_with(self);
    self.allow_await = previous;
    self.changes = changes;
  }
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    // A direct anonymous IIFE can suspend its caller without changing a
    // reusable function's contract. Do not lift named/recursive/generator calls.
    if self.allow_await
      && let Expr::Call(call) = expr
      && let Callee::Expr(callee) = &mut call.callee
      && let Expr::Fn(function) = &mut **callee
      && function.ident.is_none()
      && !function.function.is_async
      && !function.function.is_generator
    {
      self.allow_await = false;
      function.function.params.visit_mut_with(self);
      self.allow_await = true;
      let changes = self.changes;
      function.function.body.visit_mut_with(self);
      let lifted = self.changes != changes;
      function.function.is_async = lifted;
      if lifted && let Some(body) = &mut function.function.body {
        body.visit_mut_with(&mut PreserveIifeReturns);
        // Falling through has the same result as a bare return.
        body.stmts.push(Stmt::Return(ReturnStmt {
          span: DUMMY_SP,
          arg: Some(box_iife_result(None)),
        }));
      }
      call.args.visit_mut_with(self);
      if lifted {
        let value = Expr::Member(MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Paren(ParenExpr {
            span: DUMMY_SP,
            expr: Box::new(Expr::Await(AwaitExpr {
              span: DUMMY_SP,
              arg: Box::new(expr.take()),
            })),
          })),
          prop: MemberProp::Ident(IdentName::new("value".into(), DUMMY_SP)),
        });
        // The original call produces a value, not a property reference. Without
        // this sequence, immediately calling/tagging a returned function binds
        // the transport object as its receiver.
        *expr = Expr::Paren(ParenExpr {
          span: DUMMY_SP,
          expr: Box::new(Expr::Seq(SeqExpr {
            span: DUMMY_SP,
            exprs: vec![
              Box::new(Expr::Lit(Lit::Num(Number {
                span: DUMMY_SP,
                value: 0.0,
                raw: None,
              }))),
              Box::new(value),
            ],
          })),
        });
      }
      return;
    }
    expr.visit_mut_children_with(self);
    let Expr::Call(call) = expr else { return };
    // Match only unresolved require, never a local binding or user farmRequire.
    if matches!(&call.callee, Callee::Expr(callee) if matches!(&**callee, Expr::Ident(id) if id.ctxt.outer() == self.top_level))
    {
      return;
    }
    if !is_commonjs_require(self.unresolved, self.top_level, call)
      || call.args.len() != 1
      || call.args[0].spread.is_some()
    {
      return;
    }
    let Expr::Lit(Lit::Str(source)) = &*call.args[0].expr else {
      return;
    };
    if !(self.is_async)(&source.value.to_string_lossy()) {
      return;
    }
    if (self.is_cycle)(&source.value.to_string_lossy()) {
      HANDLER.with(|handler| handler.struct_span_err(call.span, "Cannot require an asynchronous module through a static dependency cycle; break the cycle with import() at an explicit async boundary").emit());
      return;
    }
    if !self.allow_await {
      HANDLER.with(|handler| handler.struct_span_err(call.span, "Cannot synchronously require an asynchronous module inside a function or class initializer; use import() in an async function instead").emit());
      return;
    }
    self.changes += 1;
    *expr = Expr::Await(AwaitExpr {
      span: DUMMY_SP,
      arg: Box::new(expr.take()),
    });
  }
}
// Async functions assimilate Promise/thenable return values. Transport the
// original result inside a null-prototype object so even inherited `then`
// properties cannot change its meaning.
fn box_iife_result(value: Option<Box<Expr>>) -> Box<Expr> {
  let value = value.unwrap_or_else(|| {
    Box::new(Expr::Unary(UnaryExpr {
      span: DUMMY_SP,
      op: UnaryOp::Void,
      arg: Box::new(Expr::Lit(Lit::Num(Number {
        span: DUMMY_SP,
        value: 0.0,
        raw: None,
      }))),
    }))
  });
  Box::new(Expr::Object(ObjectLit {
    span: DUMMY_SP,
    props: vec![
      PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: PropName::Ident(IdentName::new("__proto__".into(), DUMMY_SP)),
        value: Box::new(Expr::Lit(Lit::Null(Null { span: DUMMY_SP }))),
      }))),
      PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
        key: PropName::Ident(IdentName::new("value".into(), DUMMY_SP)),
        value,
      }))),
    ],
  }))
}

struct PreserveIifeReturns;
impl VisitMut for PreserveIifeReturns {
  fn visit_mut_return_stmt(&mut self, statement: &mut ReturnStmt) {
    statement.arg = Some(box_iife_result(statement.arg.take()));
  }
  // Returns owned by a nested callable must retain that callable's contract.
  fn visit_mut_function(&mut self, _: &mut Function) {}
  fn visit_mut_arrow_expr(&mut self, _: &mut ArrowExpr) {}
  fn visit_mut_getter_prop(&mut self, _: &mut GetterProp) {}
  fn visit_mut_setter_prop(&mut self, _: &mut SetterProp) {}
  fn visit_mut_class(&mut self, _: &mut Class) {}
}

use farmfe_core::swc_common::util::take::Take;

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::{
    swc_common::{FilePathMapping, Globals, SourceMap},
    swc_ecma_ast::EsVersion,
    swc_ecma_parser::Syntax,
  };
  use farmfe_toolkit::script::{
    codegen_module, parse_module,
    swc_try_with::{resolve_module_mark, try_with},
  };
  use std::sync::Arc;

  fn transform(source: &str) -> farmfe_core::error::Result<String> {
    let mut ast = parse_module(
      &"test.js".into(),
      Arc::new(source.into()),
      Syntax::Es(Default::default()),
      EsVersion::EsNext,
    )
    .unwrap()
    .ast;
    let globals = Globals::new();
    let (unresolved, top_level) = resolve_module_mark(&mut ast, false, &globals);
    let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    try_with(cm.clone(), &globals, || {
      ast.visit_mut_with(&mut farmfe_toolkit::swc_ecma_transforms_base::fixer::paren_remover(None));
      ast.visit_mut_with(&mut AsyncRequire {
        is_async: &|s| s == "async",
        is_cycle: &|_| false,
        unresolved,
        top_level,
        allow_await: true,
        changes: 0,
      });
    })?;
    Ok(
      String::from_utf8(codegen_module(&ast, cm, None, Default::default(), None).unwrap()).unwrap(),
    )
  }
  #[test]
  fn preserves_condition_order_and_assignments() {
    let output=transform("before(); if (enabled) module.exports = require('async'); after(); const sync = require('sync');").unwrap();
    assert!(
      output.contains("if (enabled) module.exports = await require('async')"),
      "{output}"
    );
    assert!(output.find("before()").unwrap() < output.find("await require").unwrap());
    assert!(output.find("await require").unwrap() < output.find("after()").unwrap());
    assert!(output.contains("const sync = require('sync')"));
  }
  #[test]
  fn multiple_requires_and_nested_async_functions() {
    let output=transform("const pair=[require('async'),require('async')]; async function load(){return require('async');}").unwrap();
    assert_eq!(output.matches("await require").count(), 3);
  }
  #[test]
  fn direct_anonymous_iife_can_wait_without_hoisting() {
    let output =
      transform("if (enabled) (function(){module.exports=require('async')})();").unwrap();
    assert!(output.contains("if (enabled) (0, (await"), "{output}");
    assert!(output.contains("async function"), "{output}");
    assert!(
      output.contains("module.exports = await require"),
      "{output}"
    );
  }
  #[test]
  fn local_requires_are_unchanged() {
    let output = transform(
      "function load(require){return require('async')} const require = custom; require('async');",
    )
    .unwrap();
    assert!(!output.contains("await"));
  }
  #[test]
  fn synchronous_function_and_class_fields_report_error() {
    for source in [
      "function load(){return require('async')}",
      "class C {field=require('async')}",
      "async function load(value=require('async')){}",
    ] {
      assert!(transform(source).is_err(), "{source}");
    }
  }
  #[test]
  fn synchronous_requires_in_functions_are_unchanged() {
    assert!(
      !transform("function load(){return require('sync')}")
        .unwrap()
        .contains("await")
    );
  }
  #[test]
  fn synchronous_accessors_reject_async_requires() {
    for source in [
      "const obj={get value(){return require('async')}};",
      "const obj={set value(x){require('async')}};",
      "const obj={set value(x=require('async')){}};",
    ] {
      assert!(transform(source).is_err(), "{source}");
    }
  }
  #[test]
  fn accessor_keys_keep_outer_await_context() {
    let output = transform("(function(){const obj={get [require('async')](){return 7},set [require('async')](x){}}; return obj;})();").unwrap();
    assert_eq!(output.matches("await require").count(), 2, "{output}");
    assert!(output.contains("async function"), "{output}");
    assert!(transform("function f(){return {get [require('async')](){}}}").is_err());
  }
  #[test]
  fn lifted_iife_boxes_returns_without_touching_nested_functions() {
    let output = transform("const p=(function(){require('async');function nested(){return Promise.resolve(9)};if(flag)return;return Promise.resolve(42)})();").unwrap();
    assert!(output.contains("__proto__: null"), "{output}");
    assert!(output.contains("value: Promise.resolve(42)"), "{output}");
    assert!(output.contains("return Promise.resolve(9)"), "{output}");
    assert!(output.contains("value: void 0"), "{output}");
    assert!(output.contains(").value"), "{output}");
  }
  #[test]
  fn lifted_iife_returns_values_to_calls_and_tags() {
    for source in [
      "const ok=(function(){require('async');return function(){'use strict';return this===undefined}})()();",
      "const ok=(function(){require('async');return function(){'use strict';return this===undefined}})()`tag`;",
    ] {
      let output = transform(source).unwrap();
      assert!(output.contains("(0, (await"), "{output}");
      assert!(output.contains(").value)"), "{output}");
      parse_module(
        &"generated.js".into(),
        Arc::new(output),
        Syntax::Es(Default::default()),
        EsVersion::EsNext,
      )
      .unwrap();
    }
  }
}
