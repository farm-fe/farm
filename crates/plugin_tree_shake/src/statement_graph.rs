use farmfe_core::petgraph;

pub struct Statement {
  pub index: usize,
}

pub struct StatementGraph {
  g: petgraph::graph::Graph<Statement, ()>,
}
