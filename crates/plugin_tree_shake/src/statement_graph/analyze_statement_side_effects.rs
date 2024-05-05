use std::collections::HashSet;

use farmfe_core::{
  swc_common::Mark,
  swc_ecma_ast::{Expr, ModuleItem},
};
use farmfe_toolkit::swc_ecma_visit::{Visit, VisitWith};

use super::StatementSideEffects;

/// Analyze the side effects of a statement. See [StatementSideEffects] for more details.
/// If there are more side effects detection rules, add them here.
pub fn analyze_statement_side_effects(
  item: &ModuleItem,
  unresolved_mark: Mark,
  top_level_mark: Mark,
) -> StatementSideEffects {
  match item {
    ModuleItem::ModuleDecl(module_decl) => match module_decl {
      farmfe_core::swc_ecma_ast::ModuleDecl::Import(_) => StatementSideEffects::NoSideEffects,
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
        farmfe_core::swc_ecma_ast::Decl::Var(var_decl) => {
          let mut analyzer = SideEffectsAnalyzer::new(unresolved_mark, top_level_mark);
          analyzer.set_in_top_level(true);
          var_decl.visit_children_with(&mut analyzer);

          analyzer.side_effects
        }
        _ => StatementSideEffects::NoSideEffects,
      },
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportNamed(_) => StatementSideEffects::NoSideEffects,
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportDefaultDecl(_) => {
        StatementSideEffects::NoSideEffects
      }
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportDefaultExpr(default_expr) => {
        let mut analyzer = SideEffectsAnalyzer::new(unresolved_mark, top_level_mark);
        analyzer.set_in_top_level(true);
        default_expr.expr.visit_with(&mut analyzer);
        analyzer.side_effects
      }
      farmfe_core::swc_ecma_ast::ModuleDecl::ExportAll(_) => StatementSideEffects::NoSideEffects,
      _ => StatementSideEffects::NoSideEffects,
    },
    ModuleItem::Stmt(stmt) => {
      let mut analyzer = SideEffectsAnalyzer::new(unresolved_mark, top_level_mark);
      analyzer.set_in_top_level(true);
      stmt.visit_with(&mut analyzer);

      analyzer.side_effects
    }
  }
}

struct SideEffectsAnalyzer {
  unresolved_mark: Mark,
  top_level_mark: Mark,
  side_effects: StatementSideEffects,

  in_assign_left: bool,
  in_top_level: bool,
}

impl SideEffectsAnalyzer {
  pub fn new(unresolved_mark: Mark, top_level_mark: Mark) -> Self {
    Self {
      unresolved_mark,
      top_level_mark,
      side_effects: StatementSideEffects::NoSideEffects,
      in_assign_left: false,
      in_top_level: false,
    }
  }

  pub fn set_in_top_level(&mut self, in_top_level: bool) {
    self.in_top_level = in_top_level;
  }

  pub fn is_in_top_level(&self) -> bool {
    self.in_top_level
  }
}

impl Visit for SideEffectsAnalyzer {
  fn visit_block_stmt_or_expr(&mut self, n: &farmfe_core::swc_ecma_ast::BlockStmtOrExpr) {
    let pre = self.is_in_top_level();
    self.set_in_top_level(false);

    n.visit_children_with(self);

    self.set_in_top_level(pre);
  }

  fn visit_block_stmt(&mut self, n: &farmfe_core::swc_ecma_ast::BlockStmt) {
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

    match &member_expr.prop {
      farmfe_core::swc_ecma_ast::MemberProp::Computed(computed) => {
        computed.expr.visit_with(self);
      }
      _ => {}
    }

    member_expr.obj.visit_with(self);
  }

  fn visit_stmt(&mut self, n: &farmfe_core::swc_ecma_ast::Stmt) {
    // Do not analyze the side effects of nested statements for now
    if !self.is_in_top_level() {
      return;
    }

    match n {
      farmfe_core::swc_ecma_ast::Stmt::Block(_) => self
        .side_effects
        .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted),
      farmfe_core::swc_ecma_ast::Stmt::Empty(_) => self
        .side_effects
        .merge_side_effects(StatementSideEffects::NoSideEffects),
      farmfe_core::swc_ecma_ast::Stmt::Debugger(_)
      | farmfe_core::swc_ecma_ast::Stmt::With(_)
      | farmfe_core::swc_ecma_ast::Stmt::Labeled(_) => self
        .side_effects
        .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted),
      farmfe_core::swc_ecma_ast::Stmt::Return(_)
      | farmfe_core::swc_ecma_ast::Stmt::Break(_)
      | farmfe_core::swc_ecma_ast::Stmt::Continue(_) => unreachable!(),
      farmfe_core::swc_ecma_ast::Stmt::If(_)
      | farmfe_core::swc_ecma_ast::Stmt::Switch(_)
      | farmfe_core::swc_ecma_ast::Stmt::Throw(_)
      | farmfe_core::swc_ecma_ast::Stmt::Try(_)
      | farmfe_core::swc_ecma_ast::Stmt::While(_)
      | farmfe_core::swc_ecma_ast::Stmt::DoWhile(_)
      | farmfe_core::swc_ecma_ast::Stmt::For(_)
      | farmfe_core::swc_ecma_ast::Stmt::ForIn(_)
      | farmfe_core::swc_ecma_ast::Stmt::ForOf(_) => self
        .side_effects
        .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted),
      farmfe_core::swc_ecma_ast::Stmt::Decl(decl) => match decl {
        farmfe_core::swc_ecma_ast::Decl::Var(var_decl) => {
          var_decl.visit_with(self);
        }
        _ => {}
      },
      farmfe_core::swc_ecma_ast::Stmt::Expr(expr) => {
        expr.visit_with(self);
      }
    };
  }

  fn visit_expr(&mut self, expr: &Expr) {
    if !self.is_in_top_level() {
      return;
    }

    match expr {
      Expr::Fn(_) | Expr::Class(_) | Expr::Lit(_) | Expr::Arrow(_) => self
        .side_effects
        .merge_side_effects(StatementSideEffects::NoSideEffects),
      // TODO detect call expressions that have side effects by #pure annotation
      Expr::Call(_) | Expr::New(_) | Expr::This(_) | Expr::SuperProp(_) => {
        self
          .side_effects
          .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted);
      }
      Expr::Ident(ident) => {
        self.side_effects.merge_side_effects(
          if ident.span.ctxt().outer() == self.unresolved_mark {
            StatementSideEffects::AccessGlobalVar
          } else if self.in_assign_left {
            if ident.span.ctxt().outer() == self.top_level_mark {
              StatementSideEffects::WriteTopLevelVar(HashSet::from([ident.to_id()]))
            } else {
              StatementSideEffects::UnclassifiedSelfExecuted
            }
          } else {
            StatementSideEffects::NoSideEffects
          },
        );
      }
      Expr::Assign(assign_expr) => {
        self.in_assign_left = true;

        match &assign_expr.left {
          farmfe_core::swc_ecma_ast::AssignTarget::Simple(st) => match st {
            farmfe_core::swc_ecma_ast::SimpleAssignTarget::Ident(i) => {
              if i.id.span.ctxt.outer() == self.top_level_mark {
                self
                  .side_effects
                  .merge_side_effects(StatementSideEffects::WriteTopLevelVar(HashSet::from([i
                    .id
                    .to_id()])));
              } else {
                // when the assign target is not a top level variable, treat it as unclassified side effects for now
                self
                  .side_effects
                  .merge_side_effects(StatementSideEffects::UnclassifiedSelfExecuted);
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
        self.in_assign_left = false;

        assign_expr.right.visit_with(self);
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
