//! replace all `require('./xxx')` and `import(./xxx)`(TODO)'s argument to the actual id. for example
//! ```js
//! const { b } = require('./b');
//! ```
//! will be replaced to
//! ```js
//! const { b } = require("xxx"); // xxx is b's id.
//! ```

use farmfe_core::{
  config::{Mode, TargetEnv, FARM_DYNAMIC_REQUIRE, FARM_REQUIRE},
  module::{module_graph::ModuleGraph, ModuleId, ModuleType},
  plugin::ResolveKind,
  swc_common::{Mark, DUMMY_SP},
  swc_ecma_ast::{
    Bool, CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, MemberExpr, MemberProp, Str,
  },
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
  target_env: TargetEnv,
}

impl<'a> SourceReplacer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    top_level_mark: Mark,
    module_graph: &'a ModuleGraph,
    module_id: ModuleId,
    mode: Mode,
    target_env: TargetEnv,
  ) -> Self {
    Self {
      unresolved_mark,
      top_level_mark,
      module_graph,
      module_id,
      mode,
      external_modules: vec![],
      target_env,
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
  fn find_real_module_meta_by_source(&self, source: &str) -> Option<(ModuleId, ResolveKind)> {
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
          .get_dep_by_source_optional(&self.module_id, &source, Some(kind.clone()))
      {
        id = Some((dep_id, kind));
        break;
      }
    }
    id
  }

  fn replace_source_with_id(&mut self, call_expr: &mut CallExpr) -> SourceReplaceResult {
    // require('./xxx') or require('./xxx', true)
    if call_expr.args.len() < 1 && call_expr.args.len() > 2 {
      call_expr.visit_mut_children_with(self);
      return SourceReplaceResult::NotReplaced;
    }

    if is_commonjs_require(self.unresolved_mark, self.top_level_mark, &*call_expr) {
      if let ExprOrSpread {
        spread: None,
        expr: box Expr::Lit(Lit::Str(str)),
      } = &mut call_expr.args[0]
      {
        let source = str.value.to_string();

        call_expr.callee = Callee::Expr(Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: FARM_REQUIRE.into(),
          optional: false,
        })));

        let (id, resolve_kind) =
          (self.find_real_module_meta_by_source(&source)).unwrap_or_else(|| {
            panic!(
              "Cannot find module id for source {:?} from {:?}",
              source, self.module_id
            )
          });
        // only execute script module
        let dep_module = self.module_graph.module(&id).unwrap();

        if dep_module.external {
          if matches!(resolve_kind, ResolveKind::Require)
            && matches!(self.target_env, TargetEnv::Node)
          {
            // transform require("external") to globalThis.nodeRequire("external")
            call_expr.callee = Callee::Expr(Box::new(Expr::Member(MemberExpr {
              span: DUMMY_SP,
              obj: Box::new(Expr::Ident("globalThis".into())),
              prop: MemberProp::Ident("nodeRequire".into()),
            })));
            return SourceReplaceResult::NotReplaced;
          }

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
