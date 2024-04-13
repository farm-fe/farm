//! replace all `require('./xxx')` and `import(./xxx)`(TODO)'s argument to the actual id. for example
//! ```js
//! const { b } = require('./b');
//! ```
//! will be replaced to
//! ```js
//! const { b } = require("xxx"); // xxx is b's id.
//! ```

use farmfe_core::{
  config::{Mode, FARM_DYNAMIC_REQUIRE, FARM_REQUIRE},
  module::{module_graph::ModuleGraph, ModuleId, ModuleType},
  plugin::ResolveKind,
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{Bool, CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, Str},
};
use farmfe_toolkit::{
  script::{is_commonjs_require, is_dynamic_import},
  swc_ecma_visit::{VisitMut, VisitMutWith},
};

/// replace all `require('./xxx')` to the actual id and transform require('./xxx'). for example:
/// ```js
/// // a.js is originally a commonjs module
/// const { b } = require('./b');
/// // after transform
/// const { b } = require("xxx"); // xxx is b's id.
/// ```
pub struct SourceReplacer<'a> {
  unresolved_mark: Mark,
  top_level_mark: Mark,
  module_graph: &'a ModuleGraph,
  module_id: ModuleId,
  mode: Mode,
  pub external_modules: Vec<String>,
}

impl<'a> SourceReplacer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    top_level_mark: Mark,
    module_graph: &'a ModuleGraph,
    module_id: ModuleId,
    mode: Mode,
  ) -> Self {
    Self {
      unresolved_mark,
      top_level_mark,
      module_graph,
      module_id,
      mode,
      external_modules: vec![],
    }
  }
}

impl<'a> VisitMut for SourceReplacer<'a> {
  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    if let Expr::Call(call_expr) = expr {
      if let SourceReplaceResult::NotScriptModule = self.replace_source_with_id(call_expr) {
        *expr = Expr::Lit(Lit::Str(Str {
          span: DUMMY_SP,
          value: "".into(),
          raw: None,
        }));
      }
    } else {
      expr.visit_mut_children_with(self);
    }
  }
}

enum SourceReplaceResult {
  NotReplaced,
  Replaced,
  /// the source is not a script module
  NotScriptModule,
}

impl SourceReplacer<'_> {
  fn replace_source_with_id(&mut self, call_expr: &mut CallExpr) -> SourceReplaceResult {
    // require('./xxx') or require('./xxx', true)
    if call_expr.args.len() < 1 && call_expr.args.len() > 2 {
      call_expr.visit_mut_children_with(self);
      return SourceReplaceResult::NotReplaced;
    }

    if is_commonjs_require(self.unresolved_mark, self.top_level_mark, &*call_expr) {
      let args_len = call_expr.args.len();
      if let ExprOrSpread {
        spread: None,
        expr: box Expr::Lit(Lit::Str(str)),
      } = &mut call_expr.args[0]
      {
        let source = str.value.to_string();
        let module_type = self
          .module_graph
          .module(&self.module_id)
          .as_ref()
          .unwrap()
          .module_type
          .clone();

        call_expr.callee = Callee::Expr(Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: FARM_REQUIRE.into(),
          optional: false,
        })));

        if !matches!(module_type, ModuleType::Runtime)
          && [
            "@swc/helpers/_/_interop_require_default",
            "@swc/helpers/_/_interop_require_wildcard",
            "@swc/helpers/_/_export_star",
          ]
          .iter()
          .any(|s| source == *s)
        {
          return SourceReplaceResult::NotReplaced;
        }

        let mut id = None;
        // treat non dynamic import as the same
        for kind in [
          ResolveKind::Import,
          ResolveKind::ExportFrom,
          ResolveKind::Require,
        ] {
          if let Some(dep_id) =
            self
              .module_graph
              .get_dep_by_source_optional(&self.module_id, &source, Some(kind))
          {
            id = Some(dep_id);
            break;
          }
        }
        let id = id.unwrap_or_else(|| {
          panic!(
            "Cannot find module id for source {:?} from {:?}",
            source, self.module_id
          )
        });
        // only execute script module
        let dep_module = self.module_graph.module(&id).unwrap();

        if dep_module.external {
          self.external_modules.push(id.to_string());

          return SourceReplaceResult::NotReplaced;
        }

        if dep_module.module_type.is_script() || dep_module.module_type == ModuleType::Runtime {
          // println!("replace {:?} to {:?}", value, id.id(self.mode.clone()));
          str.value = id.id(self.mode.clone()).into();
          str.span = DUMMY_SP;
          str.raw = None;
          return SourceReplaceResult::Replaced;
        } else {
          // not script module should not be executed and should be removed
          return SourceReplaceResult::NotScriptModule;
        }
      }
    } else if is_dynamic_import(&*call_expr) {
      if let ExprOrSpread {
        spread: None,
        expr: box Expr::Lit(Lit::Str(str)),
      } = &mut call_expr.args[0]
      {
        let source = str.value.to_string();

        let id = self.module_graph.get_dep_by_source(
          &self.module_id,
          &source,
          Some(ResolveKind::DynamicImport),
        );
        // only execute script module
        let dep_module = self.module_graph.module(&id).unwrap();

        if dep_module.external {
          self.external_modules.push(id.to_string());

          return SourceReplaceResult::NotReplaced;
        }

        call_expr.callee = Callee::Expr(Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: FARM_DYNAMIC_REQUIRE.into(),
          optional: false,
        })));

        str.value = id.id(self.mode.clone()).into();
        str.span = DUMMY_SP;
        str.raw = None;
        return SourceReplaceResult::Replaced;
      }
    }

    call_expr.visit_mut_children_with(self);
    SourceReplaceResult::NotReplaced
  }
}

/// replace require('./xxx') to require('./xxx', true)
pub struct ExistingCommonJsRequireVisitor {
  unresolved_mark: Mark,
  top_level_mark: Mark,
}

impl ExistingCommonJsRequireVisitor {
  pub fn new(unresolved_mark: Mark, top_level_mark: Mark) -> Self {
    Self {
      unresolved_mark,
      top_level_mark,
    }
  }
}

impl VisitMut for ExistingCommonJsRequireVisitor {
  fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
    if call_expr.args.len() != 1 {
      call_expr.visit_mut_children_with(self);
      return;
    }

    if is_commonjs_require(self.unresolved_mark, self.top_level_mark, &*call_expr) {
      call_expr.args.push(ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Bool(Bool {
          span: DUMMY_SP,
          value: true,
        }))),
      });
    }
  }
}
