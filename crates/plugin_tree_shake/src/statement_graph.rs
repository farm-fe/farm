use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

use farmfe_core::petgraph::Direction;
use farmfe_core::swc_common::comments::SingleThreadedComments;
use farmfe_core::swc_common::Mark;
use farmfe_core::swc_ecma_ast::{Id, ImportSpecifier, ModuleDecl, ModuleExportName};
use farmfe_core::{
  petgraph::{self, stable_graph::NodeIndex},
  swc_ecma_ast::{Module as SwcModule, ModuleItem},
};

pub(crate) mod analyze_deps_by_used_idents;
pub(crate) mod analyze_statement_info;
pub(crate) mod analyze_statement_side_effects;
pub(crate) mod defined_idents_collector;
pub(crate) mod traced_used_import;

use analyze_statement_info::analyze_statement_info;

use self::analyze_deps_by_used_idents::AnalyzeUsedIdentsParams;
use self::analyze_statement_info::AnalyzedStatementInfo;
use self::traced_used_import::TracedUsedImportStatement;

pub type StatementId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportSpecifierInfo {
  Namespace(Id),
  Named { local: Id, imported: Option<Id> },
  Default(Id),
}

impl From<&ImportSpecifier> for ImportSpecifierInfo {
  fn from(value: &ImportSpecifier) -> Self {
    match value {
      ImportSpecifier::Named(named) => ImportSpecifierInfo::Named {
        local: named.local.to_id(),
        imported: named.imported.as_ref().map(|i| match i {
          ModuleExportName::Ident(i) => i.to_id(),
          _ => panic!("non-ident imported is not supported when tree shaking"),
        }),
      },
      ImportSpecifier::Default(default) => ImportSpecifierInfo::Default(default.local.to_id()),
      ImportSpecifier::Namespace(ns) => ImportSpecifierInfo::Namespace(ns.local.to_id()),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
  pub source: String,
  pub specifiers: Vec<ImportSpecifierInfo>,
  pub stmt_id: StatementId,
}

/// collect all exports and gathering them into a simpler structure
#[derive(Debug, Clone)]
pub enum ExportSpecifierInfo {
  /// export * from 'foo';
  All,
  /// export { foo, bar, default as zoo } from 'foo';
  Named { local: Id, exported: Option<Id> },
  /// export default xxx;
  Default,
  /// export * as foo from 'foo';
  Namespace(Id),
}

#[derive(Debug, Clone)]
pub struct ExportInfo {
  pub source: Option<String>,
  pub specifiers: Vec<ExportSpecifierInfo>,
  pub stmt_id: StatementId,
}

impl ExportInfo {
  pub fn contains_default_export(&self) -> bool {
    self
      .specifiers
      .iter()
      .any(|s| matches!(s, ExportSpecifierInfo::Default))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementSideEffects {
  /// If the statement is a write operation, it will be considered as a side effect, when the written value is used, the statement will be preserved, otherwise it will be removed
  /// Example:
  /// ```js
  /// a = 2, b = 3;
  /// a.prototype.b = 3;
  /// a.set('c', 4);
  /// ```
  WriteTopLevelVar(HashSet<Id>),

  /// Example:
  /// ```js
  /// const a = {};
  /// const p = a.prototype; // p is read top level value
  /// ```
  ReadTopLevelVar(HashSet<Id>),

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

/// UsedStatementIdent is used to represent the used idents of a statement, including import/export and normal statement
/// For normal statement and import statement, it should always be SwcIdent
/// For export statement, it should be Default, SwcIdent, ExportAll, InExportAll
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsedStatementIdent {
  // Means the default export of the statement is used
  Default,
  // Means the ident defined in the statement is used
  SwcIdent(Id),
  /// All idents exported by `export * from 'xxx'` are used.
  /// Only used in `export * from 'xxx'` statement
  ExportAll,
  /// This ident is used and may be exported from other module
  /// Only used in `export * from 'xxx'` statement
  InExportAll(String),
}

impl UsedStatementIdent {
  pub fn is_ident_matched(&self, ident: &Id) -> bool {
    matches!(self, Self::SwcIdent(id) if id == ident)
  }
}

impl ToString for UsedStatementIdent {
  fn to_string(&self) -> String {
    match self {
      UsedStatementIdent::Default => "default".to_string(),
      UsedStatementIdent::SwcIdent(id) => format!("{}{:?}", id.0, id.1),
      UsedStatementIdent::ExportAll => "*".to_string(),
      UsedStatementIdent::InExportAll(id) => format!("*({id})"),
    }
  }
}

#[derive(Debug)]
pub struct Statement {
  pub id: StatementId,
  pub import_info: Option<ImportInfo>,
  pub export_info: Option<ExportInfo>,
  pub defined_idents: HashSet<Id>,
  /// used idents of defined idents, updated when trace the statement graph
  pub used_defined_idents: HashSet<Id>,
  /// whether the statement has side effects, the side effect statement will be preserved
  pub side_effects: StatementSideEffects,
}

impl Statement {
  pub fn new(
    id: StatementId,
    stmt: &ModuleItem,
    unresolved_mark: Mark,
    top_level_mark: Mark,
    comments: &SingleThreadedComments,
  ) -> Self {
    // 1. analyze all import, export and defined idents of the ModuleItem
    let AnalyzedStatementInfo {
      import_info,
      export_info,
      defined_idents,
    } = analyze_statement_info(&id, stmt);

    // 2. analyze side effects of the ModuleItem
    let side_effects = analyze_statement_side_effects::analyze_statement_side_effects(
      stmt,
      unresolved_mark,
      top_level_mark,
      comments,
    );

    Self {
      id,
      import_info,
      export_info,
      defined_idents,
      used_defined_idents: HashSet::new(),
      side_effects,
    }
  }
}

#[derive(Debug, Default)]
pub struct StatementGraphEdge {
  /// used idents of the dependency statement, for example:
  /// ```js
  /// const a = b, c = d;
  /// ```
  /// The map should be:
  /// ```ignore
  /// {
  ///  a: [b],
  ///  c: [d],
  /// }
  /// ```
  pub used_idents_map: HashMap<Id, HashSet<Id>>,
  /// The same as used_idents_map, it's not defined in the statement, but used in the statement
  /// For example:
  /// ```js
  /// for (let i = 0; i < len; i++) {
  ///  console.log(a + i);
  /// }
  /// ```ignore
  /// The result should be:
  /// ```
  /// [a, len]
  /// ```
  pub used_idents: HashSet<Id>,
}

pub struct StatementGraph {
  g: petgraph::graph::Graph<Statement, StatementGraphEdge>,
  id_index_map: HashMap<StatementId, NodeIndex>,
  used_stmts: HashSet<StatementId>,
}

impl StatementGraph {
  pub fn new(
    module: &SwcModule,
    unresolved_mark: Mark,
    top_level_mark: Mark,
    comments: &SingleThreadedComments,
  ) -> Self {
    let mut g = petgraph::graph::Graph::new();
    let mut id_index_map = HashMap::new();

    let mut reverse_defined_idents_map = HashMap::new();
    // 1. analyze all defined idents of each statement
    for (index, item) in module.body.iter().enumerate() {
      let stmt = Statement::new(index, item, unresolved_mark, top_level_mark, comments);

      // export named does not define any idents
      if !matches!(item, ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(_))) {
        stmt.defined_idents.iter().for_each(|i| {
          reverse_defined_idents_map.insert(i.clone(), index);
        });
      }

      let node = g.add_node(stmt);
      id_index_map.insert(index, node);
    }
    // 2. build statement graph based on defined idents and used idents
    let mut graph = Self {
      g,
      id_index_map,
      used_stmts: HashSet::new(),
    };

    for (index, item) in module.body.iter().enumerate() {
      // 2.1 find usage of defined idents and add edges
      let deps =
        analyze_deps_by_used_idents::analyze_deps_by_used_idents(AnalyzeUsedIdentsParams {
          // 2.2 add edges to graph
          id: &index,
          stmt: item,
          reverse_defined_idents_map: &reverse_defined_idents_map,
        });

      for (dep_stmt_id, edge_weight) in deps {
        graph.add_edge(index, dep_stmt_id, edge_weight);
      }
    }

    graph
  }

  pub fn empty() -> Self {
    Self {
      g: petgraph::graph::Graph::new(),
      id_index_map: HashMap::new(),
      used_stmts: HashSet::new(),
    }
  }

  pub fn used_stmts(&self) -> &HashSet<StatementId> {
    &self.used_stmts
  }

  pub fn preserved_side_effects_stmts(&self) -> Vec<StatementId> {
    self
      .g
      .node_indices()
      .filter(|i| self.g[*i].side_effects.is_preserved())
      .map(|i| self.g[i].id)
      .collect()
  }

  pub fn contains_bare_import_stmt(&self) -> bool {
    self
      .stmt_ids()
      .into_iter()
      .any(|stmt_id| self.is_bare_import_stmt(stmt_id))
  }

  /// true if stmt is import './xxx'. (without specifiers)
  pub fn is_bare_import_stmt(&self, stmt_id: StatementId) -> bool {
    let stmt = self.stmt(&stmt_id);

    if let Some(import_info) = &stmt.import_info {
      return import_info.specifiers.is_empty();
    }

    false
  }

  pub fn add_edge(&mut self, from: StatementId, to: StatementId, edge_weight: StatementGraphEdge) {
    let from_node = self.id_index_map.get(&from).unwrap();
    let to_node = self.id_index_map.get(&to).unwrap();

    if let Some(edge) = self.g.find_edge(*from_node, *to_node) {
      let edge = self.g.edge_weight_mut(edge).unwrap();
      edge
        .used_idents
        .extend(edge_weight.used_idents.iter().cloned());

      for (k, v) in edge_weight.used_idents_map {
        edge.used_idents_map.entry(k).or_default().extend(v);
      }
    } else {
      self.g.add_edge(*from_node, *to_node, edge_weight);
    }
  }

  pub fn stmt(&self, id: &StatementId) -> &Statement {
    let node = self.id_index_map.get(id).unwrap();
    &self.g[*node]
  }

  pub fn stmt_mut(&mut self, id: &StatementId) -> &mut Statement {
    let node = self.id_index_map.get(id).unwrap();
    &mut self.g[*node]
  }

  pub fn dependencies(&self, id: &StatementId) -> Vec<(&Statement, &StatementGraphEdge)> {
    let node = self.id_index_map.get(id).unwrap();
    self
      .g
      .neighbors_directed(*node, Direction::Outgoing)
      .map(|i| {
        let edge = self.g.find_edge(*node, i).unwrap();
        let edge = self.g.edge_weight(edge).unwrap();
        (&self.g[i], edge)
      })
      .collect()
  }

  pub fn dependents(&self, id: &StatementId) -> Vec<(&Statement, &StatementGraphEdge)> {
    let node = self.id_index_map.get(id).unwrap();
    self
      .g
      .neighbors_directed(*node, Direction::Incoming)
      .map(|i| {
        let edge = self.g.find_edge(i, *node).unwrap();
        let edge = self.g.edge_weight(edge).unwrap();
        (&self.g[i], edge)
      })
      .collect()
  }

  pub fn stmts(&self) -> Vec<&Statement> {
    self.g.node_indices().map(|i| &self.g[i]).collect()
  }

  pub fn stmt_ids(&self) -> Vec<StatementId> {
    self.g.node_indices().map(|i| self.g[i].id).collect()
  }

  pub fn edges(&self) -> Vec<(&Statement, &Statement, &StatementGraphEdge)> {
    self
      .g
      .edge_indices()
      .map(|i| {
        let (from, to) = self.g.edge_endpoints(i).unwrap();
        let edge = self.g.edge_weight(i).unwrap();
        (&self.g[from], &self.g[to], edge)
      })
      .collect()
  }

  pub fn mark_used_statements(&mut self, id: StatementId) {
    self.used_stmts.insert(id);
  }

  pub fn trace_and_mark_used_statements(
    &mut self,
    used_stmts_exports: HashMap<StatementId, HashSet<UsedStatementIdent>>,
  ) -> Vec<TracedUsedImportStatement> {
    farmfe_core::farm_profile_function!("trace_and_mark_used_statements".to_string());

    let mut used_statements_map = used_stmts_exports;

    // 1. preserve all statements that have side effects
    for stmt_id in self.preserved_side_effects_stmts() {
      let stmt = self.stmt(&stmt_id);
      used_statements_map.insert(
        stmt_id,
        stmt
          .defined_idents
          .clone()
          .into_iter()
          .map(UsedStatementIdent::SwcIdent)
          .collect(),
      );
    }

    // 2. sort by statement id
    let mut used_statements: Vec<_> = used_statements_map.into_iter().collect();
    used_statements.sort_by(|a, b| a.0.cmp(&b.0));

    let mut stmts = VecDeque::from(used_statements);
    let mut visited = HashSet::new();
    let mut result = vec![];

    // 3. traverse the used statements in the statement graph
    while let Some((stmt_id, used_defined_idents)) = stmts.pop_front() {
      let get_stmt_used_defined_idents = |stmt: &Statement| {
        used_defined_idents
          .iter()
          .filter_map(|i| match i {
            UsedStatementIdent::SwcIdent(id) => Some(HashSet::from([id.clone()])),
            UsedStatementIdent::Default => {
              // add all defined idents to used defined idents if it's a default export
              if let Some(export_info) = &stmt.export_info {
                if export_info.contains_default_export() {
                  // defined_idents should always be empty
                  return Some(stmt.defined_idents.clone());
                }
              }

              None
            }
            _ => None,
          })
          .flatten()
          .collect::<HashSet<_>>()
      };

      if visited.contains(&stmt_id) {
        // if all used defined idents are visited, skip the statement
        let stmt = self.stmt(&stmt_id);
        let stmt_used_defined_idents = get_stmt_used_defined_idents(stmt);
        if stmt_used_defined_idents.is_subset(&stmt.used_defined_idents) {
          continue;
        }
      } else {
        visited.insert(stmt_id);
      }

      // 3.1 mark the statement as used
      self.mark_used_statements(stmt_id);
      // 3.2 update used defined idents of the statement
      let stmt = self.stmt_mut(&stmt_id);
      let stmt_used_defined_idents = get_stmt_used_defined_idents(stmt);
      stmt.used_defined_idents.extend(stmt_used_defined_idents);

      // 3.3 visit dependencies of the used statement
      for (dep_stmt, edge) in self.dependencies(&stmt_id) {
        // find all used defined idents of the dependency statement
        let mut all_used_dep_defined_idents = edge.used_idents.clone();

        for used_defined_ident in &used_defined_idents {
          if let UsedStatementIdent::SwcIdent(used_defined_ident) = used_defined_ident {
            if let Some(used_dep_defined_idents) = edge.used_idents_map.get(used_defined_ident) {
              all_used_dep_defined_idents.extend(used_dep_defined_idents.clone());
            }
          } else if let UsedStatementIdent::Default = used_defined_ident {
            // if the used defined ident is default, add all defined idents to used defined idents
            if let Some(export_info) = &self.stmt(&stmt_id).export_info {
              if export_info.contains_default_export() {
                all_used_dep_defined_idents.extend(dep_stmt.defined_idents.clone());
              }
            }
          }
        }

        let unhandled_used_dep_defined_idents = all_used_dep_defined_idents
          .into_iter()
          .filter(|i| !dep_stmt.used_defined_idents.contains(i))
          .collect::<HashSet<_>>();

        if !unhandled_used_dep_defined_idents.is_empty() {
          stmts.push_back((
            dep_stmt.id,
            unhandled_used_dep_defined_idents
              .into_iter()
              .map(UsedStatementIdent::SwcIdent)
              .collect(),
          ));
        }
      }

      // 3.4 visit dependents of the used statement, handle write side effects here
      for (dept_id, dept_used_idents) in self.trace_dependents_side_effects(stmt_id) {
        stmts.push_back((
          dept_id,
          dept_used_idents
            .into_iter()
            .map(UsedStatementIdent::SwcIdent)
            .collect(),
        ));
      }

      // 3.5 collect all used `import/export from` statements and push them into result
      if let Some(import_info) = &self.stmt(&stmt_id).import_info {
        result.push(TracedUsedImportStatement::from_import_info_and_used_idents(
          stmt_id,
          import_info,
          &used_defined_idents,
        ));
      }

      if let Some(export_info) = &self.stmt(&stmt_id).export_info {
        if let Some(used_import_stmt) = TracedUsedImportStatement::from_export_info_and_used_idents(
          stmt_id,
          export_info,
          &used_defined_idents,
        ) {
          result.push(used_import_stmt);
        }
      }
    }

    result.sort_by_key(|i| i.stmt_id);
    result
  }

  fn traverse_dependents_bfs(
    &self,
    stmt_id: StatementId,
    visited: &mut HashSet<StatementId>,
    stack: &mut Vec<(StatementId, HashSet<Id>)>,
    result: &mut Vec<Vec<(StatementId, HashSet<Id>)>>,
  ) {
    if visited.contains(&stmt_id) {
      return;
    }

    visited.insert(stmt_id);

    for (dept_stmt, edge) in self.dependents(&stmt_id) {
      match &dept_stmt.side_effects {
        StatementSideEffects::WriteTopLevelVar(written_top_level_vars) => {
          // if the used defined idents are written by the dependent statement, mark the dependent statement as used
          // example:
          // ```
          // const a = 1;
          // a.prototype.b = () => {};
          // ```
          let write_used_defined_idents = !self
            .stmt(&stmt_id)
            .used_defined_idents
            .is_disjoint(written_top_level_vars);

          // if defined idents of last dependency in the statement are written by the dependent statement, mark the dependent statement as used
          // example:
          // ```
          // function a() {}
          // const prototype = a.prototype;
          // prototype.b = () => {}
          // ```
          let last_dependency = stack.last();
          let write_last_stack_defined_idents = last_dependency.map_or_else(
            || false,
            |(_, used_defined_idents)| !used_defined_idents.is_disjoint(written_top_level_vars),
          );

          if write_used_defined_idents || write_last_stack_defined_idents {
            stack.push((dept_stmt.id, dept_stmt.defined_idents.clone()));
            result.push(stack.clone());
            stack.pop();
          }
        }
        StatementSideEffects::ReadTopLevelVar(read_top_level_vars) => {
          let mut used_dept_defined_idents = HashSet::new();

          // only trace the statement that defined idents
          for dept_defined_ident in &dept_stmt.defined_idents {
            if let Some(dept_used_cur_idents) = edge.used_idents_map.get(&dept_defined_ident) {
              if !dept_used_cur_idents.is_disjoint(read_top_level_vars) {
                used_dept_defined_idents.insert(dept_defined_ident.clone());
              }
            }
          }

          if !used_dept_defined_idents.is_empty() {
            stack.push((dept_stmt.id, used_dept_defined_idents));
            self.traverse_dependents_bfs(dept_stmt.id, visited, stack, result);
            stack.pop();
          }
        }
        StatementSideEffects::WriteOrCallGlobalVar
        | StatementSideEffects::UnclassifiedSelfExecuted
        | StatementSideEffects::NoSideEffects => {
          /* These 3 types are handled already, do not need to trace their dependents */
        }
      }
    }
  }

  pub fn trace_dependents_side_effects(
    &self,
    stmt_id: StatementId,
  ) -> Vec<(StatementId, HashSet<Id>)> {
    // we only trace the dependents side effects of the statement that has defined idents
    if self.stmt(&stmt_id).defined_idents.is_empty() {
      return vec![];
    }

    let mut result = vec![];
    let mut visited = HashSet::new();
    let mut stack = vec![];

    self.traverse_dependents_bfs(stmt_id, &mut visited, &mut stack, &mut result);

    result.into_iter().flatten().collect()
  }
}
