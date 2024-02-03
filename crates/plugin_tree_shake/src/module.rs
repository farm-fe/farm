use std::{
  borrow::Borrow,
  collections::{HashMap, HashSet, VecDeque},
  mem,
};

use farmfe_core::{
  module::{Module, ModuleId, ModuleSystem},
  petgraph::{
    self,
    stable_graph::NodeIndex,
    visit::{EdgeRef, IntoEdgeReferences, IntoEdgesDirected},
    Direction::{Incoming, Outgoing},
  },
  swc_common::{Globals, Mark},
};
use farmfe_toolkit::{
  script::swc_try_with::try_with,
  swc_ecma_visit::{Visit, VisitWith},
};

use crate::{
  remove_useless_stmts::is_global_ident,
  statement_graph::{ExportInfo, ExportSpecifierInfo, ImportInfo, StatementGraph, StatementId},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsedIdent {
  /// Local ident
  SwcIdent(String),
  /// Default ident
  Default,
  /// This ident is used and may be exported from other module
  InExportAll(String),
  /// All idents is used and may be exported from other module
  ExportAll,
}

impl ToString for UsedIdent {
  fn to_string(&self) -> String {
    match self {
      UsedIdent::SwcIdent(ident) => ident.to_string(),
      UsedIdent::Default => "default".to_string(),
      UsedIdent::InExportAll(ident) => ident.to_string(),
      UsedIdent::ExportAll => "*".to_string(),
    }
  }
}

#[derive(Debug, Clone)]
pub enum UsedExports {
  All(HashMap<ModuleId, Vec<String>>),
  Partial(HashMap<ModuleId, Vec<String>>),
}

impl UsedExports {
  pub fn add_used_export(&mut self, module_id: &ModuleId, used_export: &dyn ToString) {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        self_used_exports
          .entry(module_id.clone())
          .or_insert_with(Vec::new)
          .push(used_export.to_string());

        // if let UsedExports::Partial(self_used_exports) = self {
        //   if ns == used_export.to_string() {
        //     *self = UsedExports::All(self_used_exports.drain(..).collect());
        //   }
        // }
      }
    }
  }

  pub fn remove_used_export(&mut self, module_id: &ModuleId, used_export: &dyn ToString) {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        let s = used_export.to_string();
        if !self_used_exports.contains_key(module_id) {
          return;
        }
        if let Some(pos) = self_used_exports[module_id]
          .iter()
          .position(|item| item == &s)
        {
          self_used_exports.get_mut(module_id).unwrap().remove(pos);

          // let ns = UsedIdent::Namespace.to_string();
          // if let UsedExports::All(self_used_exports) = self {
          //   if self_used_exports.iter().any(|item| item != &ns) {
          //     *self = UsedExports::Partial(self_used_exports.drain(..).collect());
          //   }
          // }
        }
      }
    };
  }

  pub fn remove_all_used_export(&mut self, module_id: &ModuleId) {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        self_used_exports.remove(module_id);
      }
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      UsedExports::All(_) => false,
      UsedExports::Partial(self_used_exports) => {
        !self_used_exports.values().any(|item| !item.is_empty())
      }
    }
  }

  pub fn change_all(&mut self) {
    match self {
      UsedExports::Partial(self_used_exports) => {
        *self = UsedExports::All(mem::take(self_used_exports));
      }
      // UsedExports::All(_) => todo!(),
      _ => {}
    }
  }

  pub fn to_string_vec(&self) -> Vec<String> {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        // let ns = UsedIdent::Namespace.to_string();
        self_used_exports
          .values()
          .flatten()
          .cloned()
          // .map(|ident| if ident == ns { "*".to_string() } else { ident })
          .collect()
      }
    }
  }

  pub fn idents(&mut self) -> HashSet<&String> {
    match self {
      UsedExports::Partial(self_used_exports) => {
        self_used_exports.values().flatten().collect::<HashSet<_>>()
      }
      _ => Default::default(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
  Write,
  Read,
}

#[derive(Debug)]
struct AnalyzeStatementIdent {
  pub is_reference_global_ident: bool,
  pub unresolved_mark: Mark,
  pub reads: HashSet<String>,
  pub writes: HashSet<String>,
  mode: Mode,
  nested: bool,
}
impl AnalyzeStatementIdent {
  fn with_mode<F>(&mut self, mode: Mode, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let prev = self.mode;
    self.mode = mode;
    f(self);
    self.mode = prev;
  }

  fn with_nested<F>(&mut self, nested: bool, f: F)
  where
    F: FnOnce(&mut Self),
  {
    let prev = self.nested;
    self.nested = nested;
    f(self);
    self.nested = prev;
  }
}

impl Visit for AnalyzeStatementIdent {
  fn visit_constructor(&mut self, n: &farmfe_core::swc_ecma_ast::Constructor) {
    self.with_nested(true, |this| n.visit_children_with(this));
  }
  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    if !self.nested && matches!(self.mode, Mode::Write) {
      self.writes.insert(n.to_string());
    } else {
      self.reads.insert(n.to_string());
    }

    if self.is_reference_global_ident || self.nested {
      return;
    }

    if is_global_ident(self.unresolved_mark, n) {
      self.is_reference_global_ident = true;
    }
  }

  fn visit_pat_or_expr(&mut self, n: &farmfe_core::swc_ecma_ast::PatOrExpr) {
    self.with_mode(Mode::Write, |this| n.visit_children_with(this))
  }

  fn visit_pat(&mut self, n: &farmfe_core::swc_ecma_ast::Pat) {
    self.with_mode(Mode::Write, |this| n.visit_children_with(this))
  }

  fn visit_fn_decl(&mut self, n: &farmfe_core::swc_ecma_ast::FnDecl) {
    self.with_nested(true, |this| n.visit_children_with(this));
  }

  fn visit_assign_expr(&mut self, n: &farmfe_core::swc_ecma_ast::AssignExpr) {
    self.with_mode(Mode::Write, |this| n.left.visit_with(this));

    self.with_mode(Mode::Read, |this| n.right.visit_with(this));
  }
}

// type ModuleDefineGraphEdge = (Mode, String);
struct ModuleDefineGraphEdge {
  mode: Mode,
}

struct ModuleDefineGraph {
  g: petgraph::graph::DiGraph<ModuleDefineId, ModuleDefineGraphEdge>,
  id_index_map: HashMap<ModuleDefineId, NodeIndex>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct ModuleDefineId {
  stmt_id: usize,
  ident: String,
}

impl From<(usize, &String)> for ModuleDefineId {
  fn from(id: (usize, &String)) -> Self {
    Self {
      stmt_id: id.0,
      ident: id.1.clone(),
    }
  }
}

impl ModuleDefineGraph {
  fn new() -> Self {
    Self {
      g: Default::default(),
      id_index_map: Default::default(),
    }
  }

  fn add_node(&mut self, id: ModuleDefineId) {
    if self.id_index_map.contains_key(&id) {
      return;
    }

    let index = self.g.add_node(id.clone());
    self.id_index_map.insert(id, index);
  }

  fn add_edge(&mut self, from: ModuleDefineId, to: ModuleDefineId, mode: Mode) {
    println!("___add_edge from: {:?}, to: {:?} {:?}", from, to, mode);
    let from_index = self.id_index_map.get(&from).unwrap();
    let to_index = self.id_index_map.get(&to).unwrap();

    self
      .g
      .add_edge(*from_index, *to_index, ModuleDefineGraphEdge { mode });
  }

  fn remove_node(&mut self, id: ModuleDefineId) {
    let index = self.id_index_map.remove(&id).unwrap();
    self.g.remove_node(index);
  }

  fn has_node(&self, id: &ModuleDefineId) -> bool {
    self.id_index_map.contains_key(id)
  }

  fn stmt_read_nodes(&self, id: usize) -> Vec<&ModuleDefineId> {
    self
      .id_index_map
      .iter()
      .filter(|item| item.0.stmt_id == id && self.g.edges_directed(*item.1, Outgoing).count() > 0)
      .map(|item| item.0)
      .collect()
  }

  fn edges(&self, id: &ModuleDefineId) -> Vec<(&ModuleDefineId, Mode)> {
    let index = self.id_index_map.get(&id).unwrap();
    self
      .g
      .edges_directed(*index, Outgoing)
      .map(|edge| {
        let mode = edge.weight().mode;

        let node = &self.g[edge.target()];
        (node, mode)
      })
      .collect()
  }

  fn write_node_edge(&self) -> Vec<(usize, usize)> {
    self
      .g
      .edge_references()
      .filter(|item| item.weight().mode == Mode::Write)
      .map(|item| (self.g[item.target()].stmt_id, self.g[item.source()].stmt_id))
      .collect()
  }
}

pub struct TreeShakeModule {
  pub module_id: ModuleId,
  pub side_effects: bool,
  pub stmt_graph: StatementGraph,
  pub contains_self_executed_stmt: bool,
  pub is_self_executed_import: bool,
  // used exports will be analyzed when tree shaking
  pub used_exports: UsedExports,
  pub module_system: ModuleSystem,
}

impl TreeShakeModule {
  pub fn new(module: &Module) -> Self {
    farmfe_core::farm_profile_function!(format!(
      "TreeShakeModule::new {:?}",
      module.id.to_string()
    ));
    let module_system = module.meta.as_script().module_system.clone();

    // 1. generate statement graph
    let ast = &module.meta.as_script().ast;
    let stmt_graph = if module_system == ModuleSystem::EsModule {
      StatementGraph::new(ast)
    } else {
      StatementGraph::empty()
    };

    // 2. set default used exports
    let used_exports = if module.side_effects {
      UsedExports::All(Default::default())
    } else {
      UsedExports::Partial(Default::default())
    };

    Self {
      module_id: module.id.clone(),
      contains_self_executed_stmt: stmt_graph.contains_self_executed_stmt(),
      stmt_graph,
      used_exports,
      side_effects: module.side_effects,
      module_system,
      is_self_executed_import: false,
    }
  }

  pub fn imports(&self) -> Vec<ImportInfo> {
    let mut imports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(import) = &stmt.import_info {
        imports.push(import.clone());
      }
    }

    imports
  }

  pub fn exports(&self) -> Vec<ExportInfo> {
    let mut exports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(export) = &stmt.export_info {
        exports.push(export.clone());
      }
    }

    exports
  }

  pub fn used_statements(
    &self,
    module: &mut Module,
    globals: &Globals,
  ) -> HashMap<StatementId, HashSet<String>> {
    println!("used_statements: {}", self.module_id.to_string());
    farmfe_core::farm_profile_function!(format!(
      "used_statements {:?}",
      self.module_id.to_string()
    ));
    // 通过 used_exports_idents 获取到所有的 ident
    // 1. get used exports
    let used_exports_idents = self.used_exports_idents();

    if used_exports_idents.is_empty() && !self.is_self_executed_import {
      return HashMap::new();
    }

    println!("used_exports_idents: {:#?}", used_exports_idents);
    let mut stmt_used_idents_map = HashMap::new();

    for (used_ident, stmt_id) in used_exports_idents {
      let used_idents = stmt_used_idents_map
        .entry(stmt_id)
        .or_insert(HashSet::new());
      used_idents.insert(used_ident);
    }

    // println!("___self.stmt_graph.stmts(): {:#?}", self.stmt_graph.stmts());

    let defined_vars = self
      .stmt_graph
      .stmts()
      .iter()
      .fold(HashMap::new(), |mut res, stmt| {
        // (stmt.id, stmt.defined_idents.clone())
        stmt.defined_idents.iter().for_each(|ident| {
          res.insert(ident.clone(), stmt.id);
        });

        res
      });
    // println!("___defined_vars: {:#?}", defined_vars);

    {
      farmfe_core::farm_profile_scope!(format!(
        "analyze self executed stmts {:?}",
        self.module_id.to_string()
      ));

      let mut module_define_graph = ModuleDefineGraph::new();
      let mut entries = HashSet::new();
      let mut used_self_execute_stmts = HashSet::new();

      println!("___stmt_graph: {:#?}", self.stmt_graph.stmts());

      for stmt in self.stmt_graph.stmts() {
        // if stmt_used_idents_map.contains_key(&stmt.id) {
        //   continue;
        // }

        let mut analyze_statement_ident = AnalyzeStatementIdent {
          is_reference_global_ident: false,
          unresolved_mark: Mark::from_u32(module.meta.as_script().unresolved_mark),
          reads: HashSet::new(),
          writes: HashSet::new(),
          mode: Mode::Read,
          nested: false,
        };

        let swc_module = &mut module.meta.as_script_mut().ast;
        try_with(Default::default(), globals, || {
          swc_module.body[stmt.id].visit_with(&mut analyze_statement_ident);
        })
        .unwrap();

        println!(
          "___check_ident_is_global_dent: index:{} {:#?}",
          stmt.id, analyze_statement_ident
        );

        for read in &analyze_statement_ident.reads {
          println!("___set read");

          if let Some(define_stmt_id) = defined_vars.get(read) {
            let reference_stmt = self.stmt_graph.stmt(&stmt.id);
            let reference_defines = if reference_stmt.defined_idents.contains(read) {
              vec![read]
            } else {
              reference_stmt
                .defined_idents_map
                .iter()
                .filter_map(|(define, references)| {
                  if references.contains(read) {
                    Some(define)
                  } else {
                    None
                  }
                })
                .collect::<Vec<_>>()
            };

            println!("read {} reference: {:#?}", read, reference_defines);

            let define_id: ModuleDefineId = (define_stmt_id.clone(), read).into();
            module_define_graph.add_node(define_id.clone());

            reference_defines.iter().for_each(|reference_ident| {
              let reference_id: ModuleDefineId = (stmt.id, *reference_ident).into();
              if reference_id == define_id {
                return;
              }
              module_define_graph.add_node(reference_id.clone());
              module_define_graph.add_edge(reference_id.clone(), define_id.clone(), Mode::Read);
            });
          };
        }

        for write in &analyze_statement_ident.writes {
          println!("___set write");
          let define_id: ModuleDefineId = (stmt.id, write).into();
          module_define_graph.add_node(define_id.clone());
          defined_vars.get(write).map(|&id| {
            let reference_id: ModuleDefineId = (id, write).into();
            if define_id == reference_id {
              return;
            }

            module_define_graph.add_node(reference_id.clone());
            module_define_graph.add_edge(define_id.clone(), reference_id, Mode::Write);
          });
        }

        println!(
          "___analyze_statement_ident.is_reference_global_ident {}\n    stmt.is_self_executed {}\n    stmt_used_idents_map.contains_key(&stmt.id) {}",
          analyze_statement_ident.is_reference_global_ident,
          stmt.is_self_executed,
          stmt_used_idents_map.contains_key(&stmt.id)
        );

        if analyze_statement_ident.is_reference_global_ident
          || stmt.is_self_executed
          || stmt_used_idents_map.contains_key(&stmt.id)
        {
          entries.insert(stmt.id);
        }
      }

      println!("___entries: {:?}", entries,);

      // println!("{}", module_define_graph.g.edge_indices());

      fn dfs<'a>(
        entry: &'a ModuleDefineId,
        stack: &mut Vec<ModuleDefineId>,
        result: &mut Vec<Vec<ModuleDefineId>>,
        visited: &mut HashSet<&'a ModuleDefineId>,
        module_define_graph: &'a ModuleDefineGraph,
        stmt_graph: &StatementGraph,
      ) {
        stack.push(entry.clone());

        // println!("___stack: entry: {:?} {:#?}", entry, stack,);

        let edges = module_define_graph.edges(entry);

        // println!("___edges: {:#?}", edges);

        if !module_define_graph.has_node(entry) || visited.contains(&entry) || edges.is_empty() {
          result.push(stack.clone());
        } else {
          visited.insert(entry);
          for (node, mode) in edges {
            match mode {
              Mode::Read => {
                dfs(
                  &node,
                  stack,
                  result,
                  visited,
                  module_define_graph,
                  stmt_graph,
                );
              }
              _ => {}
            }
          }
        }

        stack.pop();
      }

      let mut reference_chain = Vec::new();
      // let mut node_visited = HashSet::new();
      let write_stmt = module_define_graph.write_node_edge();
      for stmt in &entries {
        let idents = module_define_graph.stmt_read_nodes(stmt.clone());
        for ident in idents {
          dfs(
            ident,
            &mut vec![],
            &mut reference_chain,
            &mut HashSet::new(),
            &module_define_graph,
            &self.stmt_graph,
          );
        }
      }

      println!("___reference_chain: {:#?}", reference_chain);

      used_self_execute_stmts.extend(&entries);
      for chain in reference_chain {
        used_self_execute_stmts.extend(&chain.iter().map(|item| item.stmt_id).collect::<Vec<_>>());
      }

      for (target, source) in write_stmt {
        if used_self_execute_stmts.contains(&target) {
          used_self_execute_stmts.insert(source);
        }
      }

      // println!(
      //   "___module_define_graph: {:?}",
      //   module_define_graph
      //     .g
      //     .node_indices()
      //     .map(|item| { &module_define_graph.g[item] })
      //     .collect::<Vec<_>>()
      // );

      println!("___used_self_execute_stmts: {:?}", used_self_execute_stmts);

      for stmt_id in used_self_execute_stmts {
        stmt_used_idents_map
          .entry(stmt_id)
          .or_insert(HashSet::new());

        let dep_stmts = self.stmt_graph.dependencies(&stmt_id);
        // let mut is_contain_stmts = vec![];
        for (dep_stmt, referred_idents) in dep_stmts {
          let used_idents = stmt_used_idents_map
            .entry(dep_stmt.id)
            .or_insert(HashSet::new());
          used_idents.extend(referred_idents.into_iter().map(UsedIdent::SwcIdent));
        }
      }
    }

    println!("___stmt_used_idents_map: {:?}", stmt_used_idents_map);

    // 2. analyze used statements starting from used exports

    self
      .stmt_graph
      .analyze_used_statements_and_idents(stmt_used_idents_map)
  }

  pub fn used_exports_idents(&self) -> Vec<(UsedIdent, StatementId)> {
    farmfe_core::farm_profile_function!(format!(
      "used_exports_idents {:?}",
      self.module_id.to_string()
    ));
    println!(
      "used_exports: {} {:#?}",
      self.module_id.to_string(),
      self.used_exports
    );

    match &self.used_exports {
      UsedExports::All(_) => {
        // all exported identifiers are used
        let mut used_idents = vec![];

        for export_info in self.exports() {
          for sp in export_info.specifiers {
            match sp {
              ExportSpecifierInfo::Default => {
                used_idents.push((UsedIdent::Default, export_info.stmt_id));
              }
              ExportSpecifierInfo::Named { local, .. } => {
                used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
              }
              ExportSpecifierInfo::Namespace(ns) => {
                used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
              }
              ExportSpecifierInfo::All(_) => {
                used_idents.push((UsedIdent::ExportAll, export_info.stmt_id));
              }
            }
          }
        }

        used_idents
      }
      UsedExports::Partial(idents) => {
        let mut used_idents = vec![];

        let idents = idents.values().flatten().collect::<Vec<_>>();
        // if idents.contains(&UsedIdent::ExportAll.to_string()) {
        //   // all exported identifiers are used
        //   for export_info in self.exports() {

        //   }

        //   return used_idents;
        // };

        for ident in &idents {
          // find the export info that contains the ident
          let export_info = self.exports().into_iter().find(|export_info| {
            export_info.specifiers.iter().any(|sp| match sp {
              ExportSpecifierInfo::Default => *ident == "default",
              ExportSpecifierInfo::Named { local, exported } => {
                let exported_ident = if let Some(exported) = exported {
                  exported
                } else {
                  local
                };

                is_ident_equal(ident, exported_ident)
              }
              ExportSpecifierInfo::Namespace(ns) => is_ident_equal(ident, ns),
              ExportSpecifierInfo::All(_) => {
                /* Deal with All later */
                false
              }
            })
          });

          if let Some(export_info) = export_info {
            for sp in export_info.specifiers {
              match sp {
                ExportSpecifierInfo::Default => {
                  if *ident == "default" {
                    used_idents.push((UsedIdent::Default, export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::Named { local, exported } => {
                  if let Some(exported) = exported {
                    if is_ident_equal(ident, &exported) {
                      used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                    }
                  } else if is_ident_equal(ident, &local) {
                    used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::Namespace(ns) => {
                  if is_ident_equal(ident, &ns) {
                    used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::All(_) => {
                  unreachable!()
                }
              }
            }
          } else {
            let contain_all = idents.contains(&&UsedIdent::ExportAll.to_string());
            // if export info is not found, and there are ExportSpecifierInfo::All, then the ident may be exported by `export * from 'xxx'`
            for export_info in self.exports() {
              if export_info
                .specifiers
                .iter()
                .any(|sp| matches!(sp, ExportSpecifierInfo::All(_)))
              {
                let stmt_id = export_info.stmt_id;
                used_idents.push((UsedIdent::InExportAll(ident.to_string()), stmt_id));
              } else if contain_all {
                for sp in export_info.specifiers {
                  match sp {
                    ExportSpecifierInfo::Default => {
                      used_idents.push((UsedIdent::Default, export_info.stmt_id));
                    }
                    ExportSpecifierInfo::Named { local, .. } => {
                      used_idents.push((UsedIdent::SwcIdent(local.clone()), export_info.stmt_id));
                    }
                    ExportSpecifierInfo::Namespace(ns) => {
                      used_idents.push((UsedIdent::SwcIdent(ns.clone()), export_info.stmt_id));
                    }
                    ExportSpecifierInfo::All(_) => {
                      used_idents.push((UsedIdent::ExportAll, export_info.stmt_id));
                    }
                  }
                }
              }
            }
          }
        }

        used_idents
      }
    }
  }
}

fn is_ident_equal(ident1: &String, ident2: &String) -> bool {
  let split1 = ident1.split('#').collect::<Vec<_>>();
  let split2 = ident2.split('#').collect::<Vec<_>>();

  if split1.len() == 2 && split2.len() == 2 {
    split1[0] == split2[0] && split1[1] == split2[1]
  } else {
    split1[0] == split2[0]
  }
}
