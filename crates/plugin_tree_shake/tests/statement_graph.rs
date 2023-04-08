use farmfe_core::swc_common::{Globals, GLOBALS};

mod common;

use common::{create_module, parse_module};
use farmfe_plugin_tree_shake::{
  module::{TreeShakeModule, UsedExports},
  statement_graph::StatementGraph,
};

#[test]
fn construct_statement_graph_basic() {
  let code = r#"
import { aValue } from './foo';
const a = aValue;
const b = 2;
const c = a + b;

export function e() {
  console.log(c);
}
class f {
  constructor() {
    const a = 'a';
    this.a = a;
    this.b = b;
  }
}

export { a, b, c as d };"#;

  GLOBALS.set(&Globals::new(), || {
    let (ast, _) = parse_module(code);

    let stmt_graph = StatementGraph::new(&ast);
    assert_eq!(stmt_graph.stmts().len(), 7);
    let edges = stmt_graph.edges();

    assert_eq!(edges.len(), 8);
    // statement 1 -> statement 0
    assert_eq!(edges[0].0.id, 1);
    assert_eq!(edges[0].1.id, 0);
    assert_eq!(edges[0].2.idents.len(), 1);
    assert_eq!(edges[0].2.idents[0].sym.to_string(), "aValue".to_string());

    // statement 3 -> statement 1
    assert_eq!(edges[1].0.id, 3);
    assert_eq!(edges[1].1.id, 1);
    assert_eq!(edges[1].2.idents.len(), 1);
    assert_eq!(edges[1].2.idents[0].sym.to_string(), "a".to_string());

    // statement 3 -> statement 2
    assert_eq!(edges[2].0.id, 3);
    assert_eq!(edges[2].1.id, 2);
    assert_eq!(edges[2].2.idents.len(), 1);
    assert_eq!(edges[2].2.idents[0].sym.to_string(), "b".to_string());

    // statement 4 -> statement 3
    assert_eq!(edges[3].0.id, 4);
    assert_eq!(edges[3].1.id, 3);
    assert_eq!(edges[3].2.idents.len(), 1);
    assert_eq!(edges[3].2.idents[0].sym.to_string(), "c".to_string());

    // statement 5 -> statement 2
    assert_eq!(edges[4].0.id, 5);
    assert_eq!(edges[4].1.id, 2);
    assert_eq!(edges[4].2.idents.len(), 1);
    assert_eq!(edges[4].2.idents[0].sym.to_string(), "b".to_string());

    // statement 6 -> statement 1
    assert_eq!(edges[5].0.id, 6);
    assert_eq!(edges[5].1.id, 1);
    assert_eq!(edges[5].2.idents.len(), 1);
    assert_eq!(edges[5].2.idents[0].sym.to_string(), "a".to_string());

    // statement 6 -> statement 2
    assert_eq!(edges[6].0.id, 6);
    assert_eq!(edges[6].1.id, 2);
    assert_eq!(edges[6].2.idents.len(), 1);
    assert_eq!(edges[6].2.idents[0].sym.to_string(), "b".to_string());

    // statement 6 -> statement 3
    assert_eq!(edges[7].0.id, 6);
    assert_eq!(edges[7].1.id, 3);
    assert_eq!(edges[7].2.idents.len(), 1);
    assert_eq!(edges[7].2.idents[0].sym.to_string(), "c".to_string());
  });
}
