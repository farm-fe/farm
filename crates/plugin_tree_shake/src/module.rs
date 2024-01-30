use std::collections::{HashMap, HashSet, VecDeque};

use farmfe_core::{
  module::{Module, ModuleId, ModuleSystem},
  petgraph::{
    self,
    stable_graph::NodeIndex,
    visit::{EdgeRef, IntoEdgesDirected},
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
  All(Vec<String>),
  Partial(Vec<String>),
}

impl UsedExports {
  pub fn add_used_export(&mut self, used_export: &dyn ToString) {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        self_used_exports.push(used_export.to_string());

        // if let UsedExports::Partial(self_used_exports) = self {
        //   if ns == used_export.to_string() {
        //     *self = UsedExports::All(self_used_exports.drain(..).collect());
        //   }
        // }
      }
    }
  }

  pub fn remove_used_export(&mut self, used_export: &dyn ToString) {
    match self {
      UsedExports::Partial(self_used_exports) | UsedExports::All(self_used_exports) => {
        let s = used_export.to_string();
        if let Some(pos) = self_used_exports.iter().position(|item| item == &s) {
          self_used_exports.remove(pos);

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

  pub fn is_empty(&self) -> bool {
    match self {
      UsedExports::All(_) => false,
      UsedExports::Partial(self_used_exports) => self_used_exports.is_empty(),
    }
  }

  pub fn change_all(&mut self) {
    match self {
      UsedExports::Partial(self_used_exports) => {
        *self = UsedExports::All(self_used_exports.drain(..).collect());
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
          .clone()
          .into_iter()
          // .map(|ident| if ident == ns { "*".to_string() } else { ident })
          .collect()
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
  Write,
  Read,
}

#[derive(Debug)]
struct CollectStatementUsedMeta {
  pub is_import_global_ident: bool,
  pub unresolved_mark: Mark,
  pub reads: HashSet<String>,
  pub writes: HashSet<String>,
  mode: Mode,
  nested: bool,
}
impl CollectStatementUsedMeta {
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

impl Visit for CollectStatementUsedMeta {
  fn visit_ident(&mut self, n: &farmfe_core::swc_ecma_ast::Ident) {
    if self.nested {
      return;
    }

    match self.mode {
      Mode::Write => {
        self.writes.insert(n.to_string());
      }
      Mode::Read => {
        self.reads.insert(n.to_string());
      }
    }

    if self.is_import_global_ident {
      return;
    }

    if is_global_ident(self.unresolved_mark, n) {
      self.is_import_global_ident = true;
    }
  }

  fn visit_expr(&mut self, n: &farmfe_core::swc_ecma_ast::Expr) {
    self.with_mode(Mode::Read, |this| n.visit_children_with(this))
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
}

// type ModuleDefineGraphEdge = (Mode, String);
struct ModuleDefineGraphEdge {
  mode: Mode,
}

struct ModuleDefineGraph {
  g: petgraph::graph::DiGraph<usize, ModuleDefineGraphEdge>,
  id_index_map: HashMap<usize, NodeIndex>,
}

impl ModuleDefineGraph {
  fn new() -> Self {
    Self {
      g: Default::default(),
      id_index_map: Default::default(),
    }
  }

  fn add_node(&mut self, id: usize) {
    if self.id_index_map.contains_key(&id) {
      return;
    }

    let index = self.g.add_node(id.clone());
    self.id_index_map.insert(id, index);
  }

  fn add_edge(&mut self, from: usize, to: usize, mode: Mode) {
    let from_index = self.id_index_map.get(&from).unwrap();
    let to_index = self.id_index_map.get(&to).unwrap();

    self
      .g
      .add_edge(*from_index, *to_index, ModuleDefineGraphEdge { mode });
  }

  fn remove_node(&mut self, id: usize) {
    let index = self.id_index_map.remove(&id).unwrap();
    self.g.remove_node(index);
  }

  fn has_node(&self, id: usize) -> bool {
    self.id_index_map.contains_key(&id)
  }

  fn edges(&self, id: usize) -> Vec<(usize, Mode)> {
    let index = self.id_index_map.get(&id).unwrap();
    self
      .g
      .edges_directed(*index, Incoming)
      .map(|edge| {
        let mode = edge.weight().mode;

        let node = self.g[edge.source()];
        (node, mode)
      })
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
      UsedExports::All(vec![])
    } else {
      UsedExports::Partial(vec![])
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

    if used_exports_idents.is_empty() {
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

    {
      farmfe_core::farm_profile_scope!(format!(
        "analyze self executed stmts {:?}",
        self.module_id.to_string()
      ));

      // let mut mark_vars_ident_used_meta = HashSet::new();

      // let mut used_self_execute_stmts = Vec::new();
      // let mut mark_used_var = Vec::new();
      // let mut write_var = HashMap::new();
      let mut module_define_graph = ModuleDefineGraph::new();
      let mut entry = vec![];
      // let mut declare_var = HashSet::new();

      println!("___stmt_graph: {:#?}", self.stmt_graph.stmts());

      for stmt in self.stmt_graph.stmts() {
        if stmt_used_idents_map.contains_key(&stmt.id) {
          continue;
        }

        let mut check_ident_is_global_dent = CollectStatementUsedMeta {
          is_import_global_ident: false,
          unresolved_mark: Mark::from_u32(module.meta.as_script().unresolved_mark),
          reads: HashSet::new(),
          writes: HashSet::new(),
          mode: Mode::Read,
          nested: false,
        };

        let swc_module = &mut module.meta.as_script_mut().ast;
        try_with(Default::default(), globals, || {
          swc_module.body[stmt.id].visit_with(&mut check_ident_is_global_dent);
        })
        .unwrap();

        for read in &check_ident_is_global_dent.reads {
          module_define_graph.add_node(stmt.id);

          defined_vars.get(read).map(|&id| {
            if id == stmt.id {
              return;
            }
            module_define_graph.add_node(id);
            module_define_graph.add_edge(stmt.id, id, Mode::Read);
          });
        }

        for write in &check_ident_is_global_dent.writes {
          module_define_graph.add_node(stmt.id);

          defined_vars.get(write).map(|&id| {
            if id == stmt.id {
              return;
            }
            module_define_graph.add_node(id);
            module_define_graph.add_edge(stmt.id, id, Mode::Write);
          });
        }

        if stmt.is_self_executed || check_ident_is_global_dent.is_import_global_ident {
          entry.push(stmt.id);
        }
      }

      println!("___entry: {:?}", entry,);
      // let mut nodes = module_define_graph
      //   .g
      //   .node_indices()
      //   .map(|index| module_define_graph.g[index])
      //   .collect::<VecDeque<_>>();

      let nodes = entry.into_iter().collect::<HashSet<_>>();

      let mut used_self_execute_stmts = HashSet::new();

      fn dfs(
        entry: usize,
        stack: &mut Vec<usize>,
        result: &mut Vec<Vec<usize>>,
        visited: &mut HashSet<usize>,
        module_define_graph: &ModuleDefineGraph,
        writes: &mut HashSet<(usize, usize)>,
      ) {
        if visited.contains(&entry) {
          return;
        }

        visited.insert(entry);
        stack.push(entry);

        if !module_define_graph.has_node(entry) {
          return;
        }

        let edges = module_define_graph.edges(entry);

        if stack.len() > 1 && edges.is_empty() {
          result.push(stack.clone());
          return;
        }

        for (node, mode) in edges {
          match mode {
            Mode::Read => {
              dfs(node, stack, result, visited, module_define_graph, writes);
            }
            Mode::Write => {
              writes.insert((entry, node));
            }
          }
        }
        stack.pop();
      }

      let mut reference_chain = Vec::new();
      let mut node_visited = HashSet::new();
      let mut writes_edge = HashSet::new();
      for stmt in self.stmt_graph.stmts() {
        dfs(
          stmt.id,
          &mut vec![],
          &mut reference_chain,
          &mut node_visited,
          &module_define_graph,
          &mut writes_edge,
        );
      }

      println!("___dfs_result: {:#?}", reference_chain);

      for item in reference_chain {
        let mut max_index = 0;
        for stmt in &nodes {
          if let Some(pos) = item.iter().position(|i| stmt == i) {
            max_index = max_index.max(pos);
          }
        }
        if max_index != 0 {
          used_self_execute_stmts.extend(&item[..max_index + 1]);
        }
      }

      for (target, source) in writes_edge {
        if used_self_execute_stmts.contains(&target) {
          used_self_execute_stmts.insert(source);
        }
      }

      println!(
        "___module_define_graph: {:?}",
        module_define_graph
          .g
          .node_indices()
          .map(|item| { module_define_graph.g[item] })
          .collect::<Vec<_>>()
      );

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

        // if idents.contains(&UsedIdent::ExportAll.to_string()) {
        //   // all exported identifiers are used
        //   for export_info in self.exports() {

        //   }

        //   return used_idents;
        // };

        for ident in idents {
          // find the export info that contains the ident
          let export_info = self.exports().into_iter().find(|export_info| {
            export_info.specifiers.iter().any(|sp| match sp {
              ExportSpecifierInfo::Default => ident == "default",
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
                  if ident == "default" {
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
            let contain_all = idents.contains(&UsedIdent::ExportAll.to_string());
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
