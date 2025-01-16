use farmfe_macro_cache_item::cache_item;
use swc_ecma_ast::{Id, ImportSpecifier, ModuleExportName};

use crate::HashSet;

pub type StatementId = usize;

#[derive(Debug, Clone)]
#[cache_item]
pub struct Statement {
  pub id: StatementId,
  pub import_info: Option<ImportInfo>,
  pub export_info: Option<ExportInfo>,
  pub defined_idents: HashSet<SwcId>,
  /// whether the statement has side effects, the side effect statement will be preserved
  pub side_effects: StatementSideEffects,

  /// used idents of defined idents, updated when trace the statement graph
  pub used_defined_idents: HashSet<SwcId>,
}

impl Statement {
  pub fn new(
    id: StatementId,
    // stmt: &ModuleItem,
    // unresolved_mark: Mark,
    // top_level_mark: Mark,
    // comments: &SingleThreadedComments,
  ) -> Self {
    // // 1. analyze all import, export and defined idents of the ModuleItem
    // let AnalyzedStatementInfo {
    //   import_info,
    //   export_info,
    //   defined_idents,
    // } = analyze_statement_info(&id, stmt);

    // // 2. analyze side effects of the ModuleItem
    // let side_effects = analyze_statement_side_effects::analyze_statement_side_effects(
    //   stmt,
    //   unresolved_mark,
    //   top_level_mark,
    //   comments,
    // );

    Self {
      id,
      import_info: None,
      export_info: None,
      defined_idents: HashSet::default(),
      used_defined_idents: HashSet::default(), // updated when trace the statement graph while tree shaking
      side_effects: StatementSideEffects::NoSideEffects,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cache_item]
pub enum StatementSideEffects {
  /// If the statement is a write operation, it will be considered as a side effect, when the written value is used, the statement will be preserved, otherwise it will be removed
  /// Example:
  /// ```js
  /// a = 2, b = 3;
  /// a.prototype.b = 3;
  /// a.set('c', 4);
  /// ```
  WriteTopLevelVar(HashSet<SwcId>),

  /// Example:
  /// ```js
  /// const a = {};
  /// const p = a.prototype; // p is read top level value
  /// ```
  ReadTopLevelVar(HashSet<SwcId>),

  /// Maybe modify global variable, it's always preserved, for example:
  /// ```js
  /// console.log('123');
  /// window.b = 3;
  /// document.body.addEventListener('click', () =/*  */> {});
  /// ```
  WriteOrCallGlobalVar,

  /// Unclassified default self executed statements are always treated as side effects. For example:
  /// ```js
  /// for (let i = 0; i < 10; i++) {
  ///  a[i] = i;
  ///  b[i] = a[i] + i;
  /// }
  /// (function() {
  ///   a = 2;
  /// })()
  /// function foo() {
  ///  console.log('123');
  /// }
  /// foo();
  /// ```
  /// They may be classified in the future to improve the accuracy of tree shaking
  UnclassifiedSelfExecuted,
  /// The statement does not have side effects, for example:
  /// ```js
  /// const a = 2;
  /// function foo() {}
  /// ```
  NoSideEffects,
}

impl StatementSideEffects {
  pub fn is_preserved(&self) -> bool {
    matches!(
      self,
      Self::WriteOrCallGlobalVar | Self::UnclassifiedSelfExecuted
    )
  }

  pub fn merge_side_effects(&mut self, other: Self) {
    let mut original_self_value = std::mem::replace(self, Self::NoSideEffects);

    match (&mut original_self_value, &other) {
      (StatementSideEffects::WriteTopLevelVar(a), StatementSideEffects::WriteTopLevelVar(b)) => {
        a.extend(b.iter().cloned())
      }
      (StatementSideEffects::WriteTopLevelVar(_), StatementSideEffects::ReadTopLevelVar(_)) => {}
      (StatementSideEffects::WriteTopLevelVar(_), StatementSideEffects::WriteOrCallGlobalVar) => {
        original_self_value = other;
      }
      (
        StatementSideEffects::WriteTopLevelVar(_),
        StatementSideEffects::UnclassifiedSelfExecuted,
      ) => {
        original_self_value = other;
      }
      (StatementSideEffects::WriteTopLevelVar(_), StatementSideEffects::NoSideEffects) => {}
      (StatementSideEffects::ReadTopLevelVar(_), StatementSideEffects::WriteTopLevelVar(_)) => {
        original_self_value = other;
      }
      (StatementSideEffects::ReadTopLevelVar(a), StatementSideEffects::ReadTopLevelVar(b)) => {
        a.extend(b.iter().cloned());
      }
      (StatementSideEffects::ReadTopLevelVar(_), StatementSideEffects::WriteOrCallGlobalVar) => {
        original_self_value = other;
      }
      (
        StatementSideEffects::ReadTopLevelVar(_),
        StatementSideEffects::UnclassifiedSelfExecuted,
      ) => {
        original_self_value = other;
      }
      (StatementSideEffects::ReadTopLevelVar(_), StatementSideEffects::NoSideEffects) => {}
      (
        StatementSideEffects::WriteOrCallGlobalVar | StatementSideEffects::UnclassifiedSelfExecuted,
        _,
      ) => {}
      (StatementSideEffects::NoSideEffects, _) => original_self_value = other,
    }

    *self = original_self_value;
  }
}

#[cache_item]
#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[archive_attr(derive(Hash, Eq, PartialEq))]
pub struct SwcId {
  pub sym: swc_atoms::Atom,
  ctxt: u32,
}

impl From<Id> for SwcId {
  fn from(value: Id) -> Self {
    Self {
      sym: value.0,
      ctxt: value.1.as_u32(),
    }
  }
}

impl SwcId {
  pub fn ctxt(&self) -> swc_common::SyntaxContext {
    swc_common::SyntaxContext::from_u32(self.ctxt)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cache_item]
#[serde(rename_all = "camelCase")]
pub enum ImportSpecifierInfo {
  /// import * as foo from 'foo';
  Namespace(SwcId),
  /// import { foo, bar as zoo } from 'foo';
  Named {
    /// foo or zoo in `import { foo, bar as zoo } from 'foo';`
    local: SwcId,
    /// bar in `import { foo, bar as zoo } from 'foo';`
    imported: Option<SwcId>,
  },
  /// import foo from 'foo';
  Default(SwcId),
}

impl From<&ImportSpecifier> for ImportSpecifierInfo {
  fn from(value: &ImportSpecifier) -> Self {
    match value {
      ImportSpecifier::Named(named) => ImportSpecifierInfo::Named {
        local: named.local.to_id().into(),
        imported: named.imported.as_ref().map(|i| match i {
          ModuleExportName::Ident(i) => i.to_id().into(),
          _ => panic!("non-ident imported is not supported when tree shaking"),
        }),
      },
      ImportSpecifier::Default(default) => {
        ImportSpecifierInfo::Default(default.local.to_id().into())
      }
      ImportSpecifier::Namespace(ns) => ImportSpecifierInfo::Namespace(ns.local.to_id().into()),
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cache_item]
#[serde(rename_all = "camelCase")]
pub struct ImportInfo {
  pub source: String,
  pub specifiers: Vec<ImportSpecifierInfo>,
  /// index of the import statement in the module's body
  pub stmt_idx: usize,
}

/// collect all exports and gathering them into a simpler structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cache_item]
#[serde(rename_all = "camelCase")]
pub enum ExportSpecifierInfo {
  /// export * from 'foo';
  All,
  /// export { foo, bar, default as zoo } from 'foo';
  Named {
    local: SwcId,
    exported: Option<SwcId>,
  },
  /// export default xxx;
  Default,
  /// export * as foo from 'foo';
  Namespace(SwcId),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cache_item]
#[serde(rename_all = "camelCase")]
pub struct ExportInfo {
  pub source: Option<String>,
  pub specifiers: Vec<ExportSpecifierInfo>,
  /// index of the import statement in the module's body
  pub stmt_idx: usize,
}

impl ExportInfo {
  pub fn contains_default_export(&self) -> bool {
    self
      .specifiers
      .iter()
      .any(|s| matches!(s, ExportSpecifierInfo::Default))
  }
}
