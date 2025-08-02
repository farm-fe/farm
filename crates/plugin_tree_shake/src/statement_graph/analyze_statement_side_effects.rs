use std::mem;

use farmfe_core::{
  module::meta_data::script::statement::{ReadTopLevelVar, SwcId, WriteTopLevelVar},
  swc_common::{
    comments::{Comments, SingleThreadedComments},
    Mark, Spanned,
  },
  swc_ecma_ast::{Expr, ModuleItem},
  HashMap, HashSet,
};
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

use super::StatementSideEffects;

/// Analyze the side effects of a statement. See [StatementSideEffects] for more details.
/// If there are more side effects detection rules, add them here.
pub fn analyze_statement_side_effects(
  item: &ModuleItem,
  unresolved_mark: Mark,
  top_level_mark: Mark,
  comments: &SingleThreadedComments,
) -> StatementSideEffects {
  match item {
    ModuleItem::ModuleDecl(module_decl) => match module_decl {
      farmfe_core::swc_ecma_ast::ModuleDecl::Import(_) => StatementSideEffects::NoSideEffects,
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
        farmfe_core::swc_ecma_ast::Decl::Var(var_decl) => {
          let mut analyzer = SideEffectsAnalyzer::new(unresolved_mark, top_level_mark, comments);
          analyzer.set_in_top_level(true);
          analyzer.analyze(|this| var_decl.visit_children_with(this));

          analyzer.side_effects
        }
        _ => StatementSideEffects::NoSideEffects,
      },
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportNamed(_) => StatementSideEffects::NoSideEffects,
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportDefaultDecl(_) => {
        StatementSideEffects::NoSideEffects
      }
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportDefaultExpr(default_expr) => {
        let mut analyzer = SideEffectsAnalyzer::new(unresolved_mark, top_level_mark, comments);
        analyzer.set_in_top_level(true);
        analyzer.analyze(|this| default_expr.expr.visit_with(this));
        analyzer.side_effects
      }
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportAll(_) => StatementSideEffects::NoSideEffects,
      _ => StatementSideEffects::NoSideEffects,
    },
    ModuleItem::Stmt(stmt) => {
      let mut analyzer = SideEffectsAnalyzer::new(unresolved_mark, top_level_mark, comments);
      analyzer.set_in_top_level(true);
      analyzer.analyze(|this| stmt.visit_with(this));

      analyzer.side_effects
    }
  }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct LeftIdentReference {
  ///
  /// ```unknown
  /// [a, b, c] = [target1, target2, target3]
  ///              ^^^^^^^  ^^^^^^^  ^^^^^^^
  /// ```
  ///
  idents: HashSet<SwcId>,
  ///
  /// mark write operation
  ///
  /// ```unknown
  /// a = target, a.namespace = [1, 2, 3]
  ///             ^           ^
  /// ```
  mark_write: bool,
}

impl LeftIdentReference {
  fn insert(&mut self, id: SwcId) {
    self.idents.insert(id);
  }
}

impl From<LeftIdentReference> for HashSet<SwcId> {
  fn from(value: LeftIdentReference) -> Self {
    value.idents
  }
}

impl From<LeftIdentReference> for HashSet<WriteTopLevelVar> {
  fn from(value: LeftIdentReference) -> Self {
    value
      .idents
      .into_iter()
      .map(|id| WriteTopLevelVar {
        ident: id,
        fields: None,
      })
      .collect()
  }
}

struct SideEffectsAnalyzer<'a> {
  unresolved_mark: Mark,
  top_level_mark: Mark,
  side_effects: StatementSideEffects,
  comments: &'a SingleThreadedComments,

  in_assign_left: Option<HashSet<SwcId>>,
  in_assign_right: Option<Option<HashSet<SwcId>>>,
  in_top_level: bool,
  in_call: bool,

  ///
  /// ```unknown
  /// a = target, a.namespace = [1, 2, 3]
  /// ^   ^^^^^^
  /// ```
  ///
  /// ```js
  /// {
  ///  a: [target]
  /// }
  /// ```
  ///
  assign_left_reference: HashMap<SwcId, LeftIdentReference>,

  is_read_global_var: bool,
}

impl<'a> SideEffectsAnalyzer<'a> {
  pub fn new(
    unresolved_mark: Mark,
    top_level_mark: Mark,
    comments: &'a SingleThreadedComments,
  ) -> Self {
    Self {
      unresolved_mark,
      top_level_mark,
      side_effects: StatementSideEffects::NoSideEffects,
      comments,
      in_assign_left: None,
      in_top_level: false,
      in_call: false,
      in_assign_right: None,
      assign_left_reference: HashMap::default(),
      is_read_global_var: false,
    }
  }

  pub fn set_in_top_level(&mut self, in_top_level: bool) {
    self.in_top_level = in_top_level;
  }

  pub fn is_in_top_level(&self) -> bool {
    self.in_top_level
  }

  pub fn with_assign_right<F: FnOnce(&mut Self)>(&mut self, id: Option<HashSet<SwcId>>, f: F) {
    let prev = self.in_assign_right.take();
    self.in_assign_right = Some(id);
    f(self);
    self.in_assign_right = prev;
  }

  pub fn analyze<F: FnOnce(&mut Self)>(&mut self, visitor: F) {
    visitor(self);

    for (_, rights) in mem::take(&mut self.assign_left_reference) {
      if !rights.mark_write {
        continue;
      }
      self
        .side_effects
        .merge_side_effects(StatementSideEffects::WriteTopLevelVar(rights.into()));
    }
  }
}

impl<'a> Visit for SideEffectsAnalyzer<'a> {
  fn visit_arrow_expr(&mut self, n: &farmfe_core::swc_ecma_ast::ArrowExpr) {
    let pre = self.is_in_top_level();
    self.set_in_top_level(false);

    n.visit_children_with(self);

    self.set_in_top_level(pre);
  }

  fn visit_function(&mut self, n: &farmfe_core::swc_ecma_ast::Function) {
    let pre = self.is_in_top_level();
    self.set_in_top_level(false);

    n.visit_children_with(self);

    self.set_in_top_level(pre);
  }

  fn visit_var_decl(&mut self, n: &farmfe_core::swc_ecma_ast::VarDecl) {
    if !self.is_in_top_level() {
      return;
    }

    for decl in &n.decls {
      if let Some(init) = &decl.init {
        init.visit_with(self);
      }
    }
  }

  fn visit_member_expr(&mut self, member_expr: &farmfe_core::swc_ecma_ast::MemberExpr) {
    if !self.is_in_top_level() {
      return;
    }

    if let farmfe_core::swc_ecma_ast::MemberProp::Computed(computed) = &member_expr.prop {
      computed.expr.visit_with(self);
    }

    member_expr.obj.visit_with(self);
  }

  fn visit_stmt(&mut self, n: &farmfe_core::swc_ecma_ast::Stmt) {
    // Do not analyze the side effects of functions for now
    if !self.is_in_top_level() {
      return;
    }

    match n {
      farmfe_core::swc_ecma_ast::Stmt::Empty(_) => self
        .side_effects
        .merge_side_effects(StatementSideEffects::NoSideEffects),
      farmfe_core::swc_ecma_ast::Stmt::Debugger(_)
      | farmfe_core::swc_ecma_ast::Stmt::Block(_)
      | farmfe_core::swc_ecma_ast::Stmt::With(_)
      | farmfe_core::swc_ecma_ast::Stmt::Labeled(_)
      | farmfe_core::swc_ecma_ast::Stmt::Return(_)
      | farmfe_core::swc_ecma_ast::Stmt::Break(_)
      | farmfe_core::swc_ecma_ast::Stmt::Continue(_)
      | farmfe_core::swc_ecma_ast::Stmt::If(_)
      | farmfe_core::swc_ecma_ast::Stmt::Switch(_)
      | farmfe_core::swc_ecma_ast::Stmt::Throw(_)
      | farmfe_core::swc_ecma_ast::Stmt::Try(_)
      | farmfe_core::swc_ecma_ast::Stmt::While(_)
      | farmfe_core::swc_ecma_ast::Stmt::DoWhile(_)
      | farmfe_core::swc_ecma_ast::Stmt::For(_)
      | farmfe_core::swc_ecma_ast::Stmt::ForIn(_)
      | farmfe_core::swc_ecma_ast::Stmt::ForOf(_) => {
        n.visit_children_with(self);

        // For case which should not be tree shaken:
        // ```js
        // {
        //    const a = window || {};
        //    a.require = function require() {}
        // }
        // ```
        if let StatementSideEffects::WriteTopLevelVar(_) = &self.side_effects {
          if self.is_read_global_var {
            self
              .side_effects
              .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted)
          }
        }
      }
      farmfe_core::swc_ecma_ast::Stmt::Decl(decl) => {
        if let farmfe_core::swc_ecma_ast::Decl::Var(var_decl) = decl {
          var_decl.visit_with(self);
        }
      }
      farmfe_core::swc_ecma_ast::Stmt::Expr(expr) => {
        expr.visit_children_with(self);
      }
    };
  }

  fn visit_prop_or_spread(&mut self, n: &farmfe_core::swc_ecma_ast::PropOrSpread) {
    match n {
      farmfe_core::swc_ecma_ast::PropOrSpread::Spread(s) => s.visit_with(self),
      farmfe_core::swc_ecma_ast::PropOrSpread::Prop(prop) => match &**prop {
        farmfe_core::swc_ecma_ast::Prop::Shorthand(ident) => {
          if self.in_top_level
            || ident.ctxt.outer() == self.top_level_mark
            || ident.ctxt.outer() == self.unresolved_mark
          {
            let is_read_global_var = ident.ctxt.outer() == self.unresolved_mark;
            self.is_read_global_var = self.is_read_global_var || is_read_global_var;
            self
              .side_effects
              .merge_side_effects(StatementSideEffects::ReadTopLevelVar(HashSet::from_iter([
                ReadTopLevelVar {
                  ident: ident.to_id().into(),
                  is_global_var: is_read_global_var,
                },
              ])));
          }
        }
        farmfe_core::swc_ecma_ast::Prop::KeyValue(kv) => {
          if let farmfe_core::swc_ecma_ast::PropName::Computed(c) = &kv.key {
            c.visit_with(self);
          }
          kv.value.visit_with(self);
        }
        farmfe_core::swc_ecma_ast::Prop::Assign(_)
        | farmfe_core::swc_ecma_ast::Prop::Getter(_)
        | farmfe_core::swc_ecma_ast::Prop::Setter(_)
        | farmfe_core::swc_ecma_ast::Prop::Method(_) => {
          // these prop do not has side effects
          self
            .side_effects
            .merge_side_effects(StatementSideEffects::NoSideEffects);
        }
      },
    }
  }

  fn visit_ident(&mut self, ident: &farmfe_core::swc_ecma_ast::Ident) {
    if let Some(ref mut left) = self.in_assign_left {
      left.insert(ident.to_id().into());
    }
  }

  fn visit_expr(&mut self, expr: &Expr) {
    if !self.is_in_top_level() {
      return;
    }
    // for expr that is annotated by /*#__PURE__*/ or /*@__PURE__*/, we treat it as no side effects
    if self.comments.has_flag(expr.span_lo(), "PURE") {
      self
        .side_effects
        .merge_side_effects(StatementSideEffects::NoSideEffects);
      return;
    }

    match expr {
      Expr::Fn(_) | Expr::Class(_) | Expr::Lit(_) | Expr::Arrow(_) => self
        .side_effects
        .merge_side_effects(StatementSideEffects::NoSideEffects),
      Expr::Call(_) | Expr::New(_) | Expr::This(_) | Expr::SuperProp(_) => {
        self.in_call = true;
        expr.visit_children_with(self);
        self
          .side_effects
          .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted);
        self.in_call = false;
      }
      Expr::Ident(ident) => {
        if let Some(ref mut left) = self.in_assign_left {
          left.insert(ident.to_id().into());
          if let Some(v) = self.assign_left_reference.get_mut(&ident.to_id().into()) {
            v.mark_write = true;
          }
        }

        self
          .side_effects
          .merge_side_effects(if self.in_assign_left.is_some() {
            if ident.ctxt.outer() == self.unresolved_mark {
              StatementSideEffects::WriteOrCallGlobalVar
            } else if self.in_top_level || ident.ctxt.outer() == self.top_level_mark {
              StatementSideEffects::WriteTopLevelVar(HashSet::from_iter([ident.to_id().into()]))
            } else {
              StatementSideEffects::NoSideEffects
            }
          } else if self.in_call && ident.ctxt.outer() == self.unresolved_mark {
            StatementSideEffects::WriteOrCallGlobalVar
          } else if self.in_top_level
            || ident.ctxt.outer() == self.top_level_mark
            || ident.ctxt.outer() == self.unresolved_mark
          {
            let is_read_global_var = ident.ctxt.outer() == self.unresolved_mark;
            self.is_read_global_var = self.is_read_global_var || is_read_global_var;

            StatementSideEffects::ReadTopLevelVar(HashSet::from_iter([ReadTopLevelVar {
              ident: ident.to_id().into(),
              is_global_var: is_read_global_var,
            }]))
          } else {
            StatementSideEffects::NoSideEffects
          });

        if let Some(Some(ref lefts)) = self.in_assign_right
          && (self.in_top_level || ident.ctxt.outer() == self.top_level_mark)
        {
          for left in lefts {
            self
              .assign_left_reference
              .entry(left.clone())
              .or_default()
              .insert(ident.to_id().into());
          }
        }
      }
      Expr::Assign(assign_expr) => {
        self.in_assign_left = Some(HashSet::default());

        match &assign_expr.left {
          farmfe_core::swc_ecma_ast::AssignTarget::Simple(st) => match st {
            farmfe_core::swc_ecma_ast::SimpleAssignTarget::Ident(i) => {
              self
                .in_assign_left
                .get_or_insert_with(HashSet::default)
                .insert(i.id.to_id().into());
              // for idents that are added by ast transform, the mark may not be top_level_mark
              // in this case, we treat it as top level as long as current assign expr is in top level
              if self.in_top_level || i.id.ctxt.outer() == self.top_level_mark {
                self
                  .side_effects
                  .merge_side_effects(StatementSideEffects::WriteTopLevelVar(HashSet::from_iter(
                    [i.id.to_id().into()],
                  )));
              } else if i.id.ctxt.outer() == self.unresolved_mark {
                self
                  .side_effects
                  .merge_side_effects(StatementSideEffects::WriteOrCallGlobalVar);
              } else {
                // it's local variable, treat it as no side effects
                self
                  .side_effects
                  .merge_side_effects(StatementSideEffects::NoSideEffects);
              }
            }
            farmfe_core::swc_ecma_ast::SimpleAssignTarget::Member(member_expr) => {
              member_expr.visit_with(self);
            }
            farmfe_core::swc_ecma_ast::SimpleAssignTarget::Paren(param_expr) => {
              param_expr.expr.visit_with(self)
            }
            farmfe_core::swc_ecma_ast::SimpleAssignTarget::SuperProp(_)
            | farmfe_core::swc_ecma_ast::SimpleAssignTarget::OptChain(_)
            | farmfe_core::swc_ecma_ast::SimpleAssignTarget::TsAs(_)
            | farmfe_core::swc_ecma_ast::SimpleAssignTarget::TsSatisfies(_)
            | farmfe_core::swc_ecma_ast::SimpleAssignTarget::TsNonNull(_)
            | farmfe_core::swc_ecma_ast::SimpleAssignTarget::TsTypeAssertion(_)
            | farmfe_core::swc_ecma_ast::SimpleAssignTarget::TsInstantiation(_)
            | farmfe_core::swc_ecma_ast::SimpleAssignTarget::Invalid(_) => {
              self
                .side_effects
                .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted);
            }
          },
          farmfe_core::swc_ecma_ast::AssignTarget::Pat(pat) => {
            pat.visit_with(self);
          }
        }

        let left_ident = self.in_assign_left.take();

        self.with_assign_right(left_ident, |this| {
          assign_expr.right.visit_with(this);
        });
      }
      Expr::Array(_)
      | Expr::Object(_)
      | Expr::Unary(_)
      | Expr::Update(_)
      | Expr::Seq(_)
      | Expr::Cond(_)
      | Expr::Bin(_)
      | Expr::Await(_)
      | Expr::Paren(_)
      | Expr::Tpl(_)
      | Expr::Yield(_) => {
        expr.visit_children_with(self);
      }
      Expr::Member(member_expr) => {
        member_expr.visit_with(self);
      }
      Expr::TaggedTpl(_)
      | Expr::MetaProp(_)
      | Expr::JSXMember(_)
      | Expr::JSXNamespacedName(_)
      | Expr::JSXEmpty(_)
      | Expr::JSXElement(_)
      | Expr::JSXFragment(_)
      | Expr::TsTypeAssertion(_)
      | Expr::TsConstAssertion(_)
      | Expr::TsNonNull(_)
      | Expr::TsAs(_)
      | Expr::TsInstantiation(_)
      | Expr::TsSatisfies(_)
      | Expr::PrivateName(_)
      | Expr::OptChain(_)
      | Expr::Invalid(_) => self
        .side_effects
        .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted),
    };
  }
}

#[cfg(test)]
mod test;
