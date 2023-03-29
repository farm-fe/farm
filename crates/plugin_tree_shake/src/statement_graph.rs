use farmfe_core::{
  hashbrown::HashMap,
  petgraph::{self, stable_graph::NodeIndex},
  swc_ecma_ast::{Ident, Module as SwcModule, ModuleItem},
};

mod analyze_imports_and_exports;
mod used_idents_collector;

use analyze_imports_and_exports::analyze_imports_and_exports;

pub type StatementId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportSpecifierInfo {
  Namespace(Ident),
  Named {
    local: Ident,
    imported: Option<Ident>,
  },
  Default(Ident),
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
  pub source: String,
  pub specifiers: Vec<ImportSpecifierInfo>,
  pub stmt_id: StatementId,
}

pub enum ExportSpecifierInfo {
  All,
  Named {
    local: Ident,
    exported: Option<Ident>,
  },
  Default,
}

pub struct ExportInfo {
  pub source: Option<String>,
  pub specifiers: Vec<ExportSpecifierInfo>,
}

pub struct Statement {
  pub id: StatementId,
  pub import_info: Option<ImportInfo>,
  pub export_info: Option<ExportInfo>,
  pub defined_idents: Vec<Ident>,
  pub used_idents: Vec<Ident>,
}

impl Statement {
  pub fn new(id: StatementId, stmt: &ModuleItem) -> Self {
    let (import_info, export_info, defined_idents, used_idents) =
      analyze_imports_and_exports(&id, stmt);

    Self {
      id,
      import_info,
      export_info,
      defined_idents,
      used_idents,
    }
  }
}

pub struct StatementGraph {
  g: petgraph::graph::Graph<Statement, ()>,
  id_index_map: HashMap<StatementId, NodeIndex>,
}

impl StatementGraph {
  pub fn new(module: &SwcModule) -> Self {
    let mut g = petgraph::graph::Graph::new();
    let mut id_index_map = HashMap::new();

    for (index, stmt) in module.body.iter().enumerate() {
      let node = g.add_node(Statement::new(index, stmt));
      id_index_map.insert(index, node);
    }

    Self { g, id_index_map }
  }

  pub fn add_edge(&mut self, from: StatementId, to: StatementId) {
    let from_node = self.id_index_map.get(&from).unwrap();
    let to_node = self.id_index_map.get(&to).unwrap();
    self.g.add_edge(*from_node, *to_node, ());
  }

  pub fn get_node(&self, id: StatementId) -> &Statement {
    let node = self.id_index_map.get(&id).unwrap();
    &self.g[*node]
  }

  pub fn stmts(&self) -> Vec<&Statement> {
    self.g.node_indices().map(|i| &self.g[i]).collect()
  }
}
