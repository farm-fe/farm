use std::collections::VecDeque;

pub use farmfe_core::module::meta_data::script::statement::{
  ExportInfo, ExportSpecifierInfo, ImportInfo, ImportSpecifierInfo, StatementId,
  StatementSideEffects,
};
use farmfe_core::module::meta_data::script::statement::{Statement, SwcId, WriteTopLevelVar};
use farmfe_core::module::Module;
use farmfe_core::petgraph::Direction;
use farmfe_core::swc_common::comments::SingleThreadedComments;
use farmfe_core::swc_common::Mark;
use farmfe_core::swc_ecma_ast::ModuleDecl;
use farmfe_core::{
  petgraph::{self, stable_graph::NodeIndex},
  swc_ecma_ast::{Module as SwcModule, ModuleItem},
};
use farmfe_core::{HashMap, HashSet};

pub(crate) mod analyze_deps_by_used_idents;
pub(crate) mod analyze_statement_side_effects;
pub(crate) mod analyze_used_import_all_fields;
pub(crate) mod analyze_written_imported_idents;
pub(crate) mod traced_used_import;

use analyze_used_import_all_fields::{update_used_import_all_fields_of_edges, UsedImportAllFields};

use self::analyze_deps_by_used_idents::AnalyzeUsedIdentsParams;
use self::traced_used_import::TracedUsedImportStatement;

/// UsedStatementIdent is used to represent the used idents of a statement, including import/export and normal statement
/// For normal statement and import statement, it should always be SwcIdent
/// For export statement, it should be Default, SwcIdent, ExportAll, InExportAll
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsedStatementIdent {
  // Means the default export of the statement is used
  Default,
  // Means the ident defined in the statement is used
  SwcIdent(SwcId),
  /// All idents exported by `export * from 'xxx'` are used.
  /// Only used in `export * from 'xxx'` statement
  ExportAll,
  /// This ident is used and may be exported from other module
  /// Only used in `export * from 'xxx'` statement
  InExportAll(String),
}

impl UsedStatementIdent {
  pub fn is_ident_matched(&self, ident: &SwcId) -> bool {
    matches!(self, Self::SwcIdent(id) if id == ident)
  }
}

impl ToString for UsedStatementIdent {
  fn to_string(&self) -> String {
    match self {
      UsedStatementIdent::Default => "default".to_string(),
      UsedStatementIdent::SwcIdent(id) => format!("{}{:?}", id.sym, id.ctxt()),
      UsedStatementIdent::ExportAll => "*".to_string(),
      UsedStatementIdent::InExportAll(id) => format!("*({id})"),
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
  pub used_idents_map: HashMap<SwcId, HashSet<SwcId>>,
  /// The same as used_idents_map, it's not defined in the statement, but used in the statement
  /// For example:
  /// ```js
  /// for (let i = 0; i < len; i++) {
  ///  console.log(a + i);
  /// }
  /// ```
  /// The result should be:
  /// ```ignore
  /// [a, len]
  /// ```
  pub used_idents: HashSet<SwcId>,
  /// used fields of import star statement of the dependency statement, for example:
  /// ```js
  /// import * as a from 'a';
  /// a.foo();
  /// a['bar']();
  /// console.log(a);
  /// ```
  /// The result should be:
  /// ```ignore
  /// [(a, [foo, bar, All])]
  /// ```
  pub used_import_all_fields: HashMap<SwcId, HashSet<UsedImportAllFields>>,
}

pub struct StatementGraph {
  g: petgraph::graph::Graph<Statement, StatementGraphEdge>,
  id_index_map: HashMap<StatementId, NodeIndex>,
  used_stmts: HashSet<StatementId>,

  /// reverse_defined_idents_map is used to find the statement that defined the ident
  pub reverse_defined_idents_map: HashMap<SwcId, usize>,
  /// written_imported_idents is the idents that are defined in import statement, and are written at the top level of the module
  pub written_imported_idents: HashSet<WriteTopLevelVar>,
}

impl StatementGraph {
  pub fn new(module: &Module, ast: &SwcModule, comments: &SingleThreadedComments) -> Self {
    let mut g = petgraph::graph::Graph::new();
    let mut id_index_map = HashMap::default();

    let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);
    let top_level_mark = Mark::from_u32(module.meta.as_script().top_level_mark);

    let mut reverse_defined_idents_map = HashMap::default();
    // 1. analyze all defined idents of each statement
    for (index, item) in ast.body.iter().enumerate() {
      let mut stmt = module.meta.as_script().statements[index].clone();

      let side_effects = analyze_statement_side_effects::analyze_statement_side_effects(
        item,
        unresolved_mark,
        top_level_mark,
        comments,
      );
      stmt.side_effects = side_effects;

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
      used_stmts: HashSet::default(),
      reverse_defined_idents_map: HashMap::default(),
      written_imported_idents: HashSet::default(),
    };

    for (index, item) in ast.body.iter().enumerate() {
      // 2.1 find usage of defined idents and add edges
      let deps =
        analyze_deps_by_used_idents::analyze_deps_by_used_idents(AnalyzeUsedIdentsParams {
          // 2.2 add edges to graph
          id: &index,
          stmt: item,
          reverse_defined_idents_map: &reverse_defined_idents_map,
        });
      // 2.3 update used_import_all_fields of deps
      let deps = update_used_import_all_fields_of_edges(item, &graph, deps);

      for (dep_stmt_id, edge_weight) in deps {
        graph.add_edge(index, dep_stmt_id, edge_weight);
      }
    }

    graph.reverse_defined_idents_map = reverse_defined_idents_map;
    // 3. find written imported idents
    graph.written_imported_idents =
      analyze_written_imported_idents::analyze_written_imported_idents(&graph);

    graph
  }

  pub fn empty() -> Self {
    Self {
      g: petgraph::graph::Graph::new(),
      id_index_map: HashMap::default(),
      used_stmts: HashSet::default(),
      reverse_defined_idents_map: HashMap::default(),
      written_imported_idents: HashSet::default(),
    }
  }

  pub fn used_stmts(&self) -> &HashSet<StatementId> {
    &self.used_stmts
  }

  pub fn preserved_side_effects_stmts(&self) -> Vec<StatementId> {
    let mut preserved_statements = self
      .g
      .node_indices()
      .filter(|i| self.g[*i].side_effects.is_preserved())
      .map(|i| self.g[i].id)
      .collect::<Vec<_>>();

    // preset statement that is marked as read global ident and write by other statement
    let read_global_ident_stmts = self
      .stmt_ids()
      .into_iter()
      .filter(|stmt_id| {
        let stmt = self.stmt(stmt_id);

        let is_read_global_ident =
          if let StatementSideEffects::ReadTopLevelVar(read_top_level_var) = &stmt.side_effects {
            read_top_level_var.iter().any(|i| i.is_global_var)
          } else {
            false
          };

        if is_read_global_ident {
          // find the statement that write the ident defined in this statement
          return self.stmt_ids().iter().any(|stmt_id| {
            let parent_stmt = self.stmt(stmt_id);
            if let StatementSideEffects::WriteTopLevelVar(write_top_level_var) =
              &parent_stmt.side_effects
            {
              write_top_level_var
                .iter()
                .any(|i| stmt.defined_idents.contains(&i.ident))
            } else {
              false
            }
          });
        }

        false
      })
      .collect::<HashSet<_>>();

    preserved_statements.extend(read_global_ident_stmts);

    preserved_statements
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
    // update write top level var side effects
    if !edge_weight.used_import_all_fields.is_empty() {
      let from_stmt = self.stmt_mut(&from);

      if let StatementSideEffects::WriteTopLevelVar(top_level_vars) = &mut from_stmt.side_effects {
        let local_top_level_vars = std::mem::take(top_level_vars);
        *top_level_vars = local_top_level_vars
          .into_iter()
          .map(|mut v| {
            if let Some(used_import_all_fields) = edge_weight.used_import_all_fields.get(&v.ident) {
              if let Some(fields) = &mut v.fields {
                fields.extend(used_import_all_fields.iter().cloned());
              } else {
                v.fields = Some(used_import_all_fields.clone().into_iter().collect());
              }
            }

            v
          })
          .collect();
      }
    }

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

  pub fn find_all_used_defined_idents(
    &self,
    stmt_id: &StatementId,
    dep_stmt: &Statement,
    used_defined_idents: &HashSet<UsedStatementIdent>,
    edge: &StatementGraphEdge,
  ) -> HashSet<SwcId> {
    let mut all_used_dep_defined_idents = edge.used_idents.clone();

    for used_defined_ident in used_defined_idents {
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

    all_used_dep_defined_idents
  }

  /// When tracing the used statements from export statement, all_used_import_all_fields is None.
  /// When tracing the used statements from import statement, all_used_import_all_fields is Some. This is used to handle write top level var side effects.
  pub fn trace_and_mark_used_statements(
    &mut self,
    used_stmts_exports: HashMap<StatementId, HashSet<UsedStatementIdent>>,
    all_used_import_all_fields: Option<HashMap<SwcId, HashSet<UsedImportAllFields>>>,
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
    let mut used_statements: Vec<_> = used_statements_map
      .into_iter()
      .map(|(id, used_defined_idents)| {
        (
          id,
          used_defined_idents,
          all_used_import_all_fields.clone().unwrap_or_default(),
          // HashMap::<Id, HashSet<UsedImportAllFields>>::new(),
        )
      })
      .collect();
    used_statements.sort_by(|a, b| a.0.cmp(&b.0));

    let mut stmts = VecDeque::from(used_statements);
    let mut visited = HashSet::default();
    let mut result: Vec<TracedUsedImportStatement> = vec![];

    // 3. traverse the used statements in the statement graph
    while let Some((stmt_id, used_defined_idents, used_import_all_fields)) = stmts.pop_front() {
      if visited.contains(&stmt_id) {
        // if all used defined idents are visited, skip the statement
        let stmt = self.stmt(&stmt_id);
        let stmt_used_defined_idents =
          self.get_stmt_used_defined_idents(&stmt_id, &used_defined_idents);
        if stmt_used_defined_idents.is_subset(&stmt.used_defined_idents) {
          if let Some(import_info) = &stmt.import_info {
            // extends used import all fields of the statement
            if let Some(traced_used_import_statement) =
              result.iter_mut().find(|i| i.stmt_id == stmt_id)
            {
              let temp_traced_import = TracedUsedImportStatement::from_import_info_and_used_idents(
                stmt_id,
                import_info,
                &used_defined_idents,
                used_import_all_fields,
              );
              traced_used_import_statement
                .used_stmt_idents
                .extend(temp_traced_import.used_stmt_idents);
            }
          }
          continue;
        }
      } else {
        visited.insert(stmt_id);
      }

      // 3.1 mark the statement as used
      self.mark_used_statements(stmt_id);
      // 3.2 update used defined idents of the statement
      let stmt_used_defined_idents =
        self.get_stmt_used_defined_idents(&stmt_id, &used_defined_idents);
      let stmt = self.stmt_mut(&stmt_id);
      stmt.used_defined_idents.extend(stmt_used_defined_idents);

      // 3.3 visit dependencies of the used statement
      for (dep_stmt, edge) in self.dependencies(&stmt_id) {
        // find all used defined idents of the dependency statement
        let all_used_dep_defined_idents =
          self.find_all_used_defined_idents(&stmt_id, dep_stmt, &used_defined_idents, edge);

        let unhandled_used_dep_defined_idents = all_used_dep_defined_idents
          .into_iter()
          .filter(|i| {
            !dep_stmt.used_defined_idents.contains(i)
              // the import namespace ident should be handled in the next step to append more import all fields at line 539
              // so we mark it as unhandled here
              || edge.used_import_all_fields.contains_key(i)
          })
          .collect::<HashSet<_>>();

        if !unhandled_used_dep_defined_idents.is_empty() {
          stmts.push_back((
            dep_stmt.id,
            unhandled_used_dep_defined_idents
              .into_iter()
              .map(UsedStatementIdent::SwcIdent)
              .collect(),
            edge.used_import_all_fields.clone(),
          ));
        }
      }

      // 3.4 visit dependents of the used statement, handle write side effects here
      for (dept_id, dept_used_idents, used_import_all_fields) in
        self.trace_dependents_side_effects(stmt_id, &all_used_import_all_fields)
      {
        stmts.push_back((
          dept_id,
          dept_used_idents
            .into_iter()
            .map(UsedStatementIdent::SwcIdent)
            .collect(),
          used_import_all_fields,
        ));
      }

      // 3.5 collect all used `import/export from` statements and push them into result
      if let Some(import_info) = &self.stmt(&stmt_id).import_info {
        result.push(TracedUsedImportStatement::from_import_info_and_used_idents(
          stmt_id,
          import_info,
          &used_defined_idents,
          used_import_all_fields,
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

  fn traverse_dependents_dfs(
    &self,
    stmt_id: StatementId,
    stmt_used_import_all_fields: &Option<HashMap<SwcId, HashSet<UsedImportAllFields>>>,
    visited: &mut HashSet<StatementId>,
    stack: &mut Vec<(
      StatementId,
      HashSet<SwcId>,
      HashMap<SwcId, HashSet<UsedImportAllFields>>,
    )>,
    result: &mut Vec<
      Vec<(
        StatementId,
        HashSet<SwcId>,
        HashMap<SwcId, HashSet<UsedImportAllFields>>,
      )>,
    >,
  ) {
    if visited.contains(&stmt_id) {
      return;
    }

    visited.insert(stmt_id);

    for (dept_stmt, edge) in self.dependents(&stmt_id) {
      match &dept_stmt.side_effects {
        StatementSideEffects::WriteTopLevelVar(written_top_level_vars) => {
          let has_intersection =
            |used_defined_idents: &HashSet<SwcId>,
             used_import_all_fields: &HashMap<SwcId, HashSet<UsedImportAllFields>>| {
              written_top_level_vars.iter().any(|v| {
                let is_ident_used = used_defined_idents.contains(&v.ident);

                // if stmt_used_import_all_fields is specified, we need to check if the fields are used.
                // it's only specified when we need to trace the write side effects of import all fields.
                if let Some(stmt_used_import_all_fields) = stmt_used_import_all_fields {
                  if let Some(stmt_used_import_all_fields) =
                    stmt_used_import_all_fields.get(&v.ident)
                  {
                    if let Some(used_import_all_fields) = used_import_all_fields.get(&v.ident) {
                      !used_import_all_fields.is_disjoint(stmt_used_import_all_fields)
                    } else {
                      is_ident_used
                    }
                  } else {
                    is_ident_used
                  }
                } else {
                  is_ident_used
                }
              })
            };
          // if the used defined idents are written by the dependent statement, mark the dependent statement as used
          // example:
          // ```
          // const a = 1;
          // a.prototype.b = () => {};
          // ```
          let write_used_defined_idents = has_intersection(
            &self.stmt(&stmt_id).used_defined_idents,
            &edge.used_import_all_fields,
          );

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
            |(_, used_defined_idents, used_import_all_fields)| {
              has_intersection(used_defined_idents, used_import_all_fields)
            },
          );

          if write_used_defined_idents || write_last_stack_defined_idents {
            stack.push((
              dept_stmt.id,
              dept_stmt.defined_idents.clone(),
              edge.used_import_all_fields.clone(),
            ));
            result.push(stack.clone());
            stack.pop();
          }
        }
        StatementSideEffects::ReadTopLevelVar(read_top_level_vars) => {
          let mut used_dept_defined_idents = HashSet::default();

          let has_intersection = |used_defined_idents: &HashSet<SwcId>| {
            read_top_level_vars
              .iter()
              .any(|v| used_defined_idents.contains(&v.ident))
          };
          // only trace the statement that defined idents
          for dept_defined_ident in &dept_stmt.defined_idents {
            if let Some(dept_used_cur_idents) = edge.used_idents_map.get(&dept_defined_ident) {
              if has_intersection(dept_used_cur_idents) {
                used_dept_defined_idents.insert(dept_defined_ident.clone());
              }
            }
          }

          if !used_dept_defined_idents.is_empty() {
            stack.push((
              dept_stmt.id,
              used_dept_defined_idents,
              edge.used_import_all_fields.clone(),
            ));
            self.traverse_dependents_dfs(
              dept_stmt.id,
              stmt_used_import_all_fields,
              visited,
              stack,
              result,
            );
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
    used_import_all_fields: &Option<HashMap<SwcId, HashSet<UsedImportAllFields>>>,
  ) -> Vec<(
    StatementId,
    HashSet<SwcId>,
    HashMap<SwcId, HashSet<UsedImportAllFields>>,
  )> {
    // we only trace the dependents side effects of the statement that has defined idents
    if self.stmt(&stmt_id).defined_idents.is_empty() {
      return vec![];
    }

    let mut result = vec![];
    let mut visited = HashSet::default();
    let mut stack = vec![];

    self.traverse_dependents_dfs(
      stmt_id,
      used_import_all_fields,
      &mut visited,
      &mut stack,
      &mut result,
    );

    result.into_iter().flatten().collect()
  }

  pub fn get_stmt_used_defined_idents(
    &self,
    stmt_id: &StatementId,
    used_defined_idents: &HashSet<UsedStatementIdent>,
  ) -> HashSet<SwcId> {
    used_defined_idents
      .iter()
      .filter_map(|i| match i {
        UsedStatementIdent::SwcIdent(id) => Some(HashSet::from_iter([id.clone()])),
        UsedStatementIdent::Default => {
          let stmt = self.stmt(stmt_id);
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
  }
}
