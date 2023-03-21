use farmfe_core::{
  petgraph,
  swc_ecma_ast::{Ident, Module as SwcModule},
};

pub struct Statement {
  pub index: usize,
}

pub struct StatementGraph {
  g: petgraph::graph::Graph<Statement, ()>,
}

impl StatementGraph {
  pub fn new(module: &SwcModule) -> Self {
    let mut g = petgraph::graph::Graph::new();
    let mut index = 0;
    for stmt in &module.body {
      let node = g.add_node(Statement { index });
      index += 1;
    }
    Self { g }
  }
}
