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
  module::{module_graph::ModuleGraph, ModuleId},
  plugin::ResolveKind,
  swc_common::{util::take::Take, Mark, SyntaxContext, DUMMY_SP},
  swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit, MemberExpr, MemberProp, Str},
  HashMap,
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
  hoisted_external_modules: HashMap<(String, farmfe_core::plugin::ResolveKind), ModuleId>,

  pub external_modules: Vec<ModuleId>,
}

pub struct SourceReplacerOptions<'a> {
  pub unresolved_mark: Mark,
  pub top_level_mark: Mark,
  pub module_graph: &'a ModuleGraph,
  pub module_id: ModuleId,
  pub mode: Mode,
  pub hoisted_external_modules: HashMap<(String, farmfe_core::plugin::ResolveKind), ModuleId>,
}

impl<'a> SourceReplacer<'a> {
  pub fn new(options: SourceReplacerOptions<'a>) -> Self {
    let SourceReplacerOptions {
      unresolved_mark,
      top_level_mark,
      module_graph,
      module_id,
      mode,
      hoisted_external_modules,
    } = options;

    Self {
      unresolved_mark,
      top_level_mark,
      module_graph,
      module_id,
      mode,
      hoisted_external_modules,
      external_modules: vec![],
    }
  }
}

impl VisitMut for SourceReplacer<'_> {
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
  fn find_real_module_meta_by_source(&self, source: &str) -> Option<ModuleId> {
    let mut id = None;
    // treat non dynamic import as the same
    for kind in [
      ResolveKind::Import,
      ResolveKind::ExportFrom,
      ResolveKind::Require,
    ] {
      if let Some(dep_id) = self
        .module_graph
        .get_dep_by_source_optional(&self.module_id, source, Some(kind.clone()))
        .or(
          self
            .hoisted_external_modules
            .get(&(source.to_string(), kind.clone()))
            .cloned(),
        )
      {
        id = Some(dep_id);
        break;
      }
    }
    id
  }

  fn replace_source_with_id(&mut self, call_expr: &mut CallExpr) -> SourceReplaceResult {
    if call_expr.args.len() != 1 {
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
          ctxt: SyntaxContext::empty(),
        })));

        let Some(id) = self.find_real_module_meta_by_source(&source) else {
          panic!(
            "Cannot find module id for source {:?} from {:?}.",
            source, self.module_id
          )
        };

        // only execute script module
        let dep_module = self.module_graph.module(&id).unwrap();

        if dep_module.external {
          self.external_modules.push(id.clone());

          return SourceReplaceResult::NotReplaced;
        }

        if dep_module.module_type.is_script() {
          // println!("replace {:?} to {:?}", value, id.id(self.mode.clone()));
          str.value = id.id(self.mode).into();
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

        if let Some(id) = self.module_graph.get_dep_by_source_optional(
          &self.module_id,
          &source,
          Some(ResolveKind::DynamicImport),
        ) {
          // only execute script module
          let dep_module = self.module_graph.module(&id).unwrap();

          if dep_module.external {
            self.external_modules.push(id);

            return SourceReplaceResult::NotReplaced;
          }

          str.value = id.id(self.mode).into();
          str.span = DUMMY_SP;
          str.raw = None;
        } else {
          panic!(
            "cannot found {} of DynamicImport from {}",
            source, self.module_id
          );
        }

        // in partial ShareBundle, `module source` already rewrite at DynamicImportReplacer stage
        // so, even not found module by source, still replace `require`
        call_expr.callee = Callee::Expr(Box::new(Expr::Ident(Ident {
          span: DUMMY_SP,
          sym: FARM_DYNAMIC_REQUIRE.into(),
          optional: false,
          ctxt: SyntaxContext::empty(),
        })));

        return SourceReplaceResult::Replaced;
      }
    }

    call_expr.visit_mut_children_with(self);
    SourceReplaceResult::NotReplaced
  }
}

/// replace require('./xxx') to farmRequire.i(require('./xxx'))
pub struct ExistingCommonJsRequireVisitor<'a> {
  unresolved_mark: Mark,
  top_level_mark: Mark,
  module_graph: &'a ModuleGraph,
  module_id: ModuleId,
}

impl<'a> ExistingCommonJsRequireVisitor<'a> {
  pub fn new(
    unresolved_mark: Mark,
    top_level_mark: Mark,
    module_graph: &'a ModuleGraph,
    module_id: ModuleId,
  ) -> Self {
    Self {
      unresolved_mark,
      top_level_mark,
      module_graph,
      module_id,
    }
  }
}

impl VisitMut for ExistingCommonJsRequireVisitor<'_> {
  fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
    if call_expr.args.len() != 1 {
      call_expr.visit_mut_children_with(self);
      return;
    }

    // replace require('./xxx') to farmRequire.i(require('./xxx'))
    if is_commonjs_require(self.unresolved_mark, self.top_level_mark, &*call_expr) {
      if let ExprOrSpread {
        spread: None,
        expr: box Expr::Lit(Lit::Str(str)),
      } = &mut call_expr.args[0]
      {
        let source = str.value.to_string();

        if let Some(id) =
          self
            .module_graph
            .get_dep_by_source_optional(&self.module_id, &source, None)
        {
          let source_module = self.module_graph.module(&id).unwrap();

          if source_module.module_type.is_script() && source_module.external {
            let expr_take = call_expr.take();

            *call_expr = CallExpr {
              span: DUMMY_SP,
              callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(Ident {
                  span: DUMMY_SP,
                  sym: FARM_REQUIRE.into(),
                  optional: false,
                  ctxt: SyntaxContext::empty(),
                })),
                prop: MemberProp::Ident("i".into()),
              }))),
              args: vec![ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Call(expr_take)),
              }],
              type_args: None,
              ctxt: SyntaxContext::empty(),
            };
          }
        }
      }
    }
  }
}
