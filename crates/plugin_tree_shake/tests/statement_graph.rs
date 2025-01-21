use std::cmp::Ordering;

use farmfe_core::module::meta_data::script::statement::Statement;
use farmfe_core::module::meta_data::script::ScriptModuleMetaData;
use farmfe_core::module::{Module, ModuleMetaData};
use farmfe_core::swc_common::{Globals, SyntaxContext, GLOBALS};
use farmfe_core::{HashMap, HashSet};

mod common;

use common::parse_module_with_comments;
use farmfe_plugin_tree_shake::{
  module::UsedExportsIdent,
  statement_graph::{StatementGraph, UsedStatementIdent},
};
use farmfe_toolkit::script::analyze_statement::{analyze_statement_info, AnalyzedStatementInfo};

use crate::common::print_id;

fn create_test_statement_graph(code: &str) -> StatementGraph {
  let (ast, comment, unresolved_mark, top_level_mark) = parse_module_with_comments(code);
  let mut module = Module::new("test".into());
  let statements = ast
    .body
    .iter()
    .enumerate()
    .map(|(i, item)| {
      let AnalyzedStatementInfo {
        export_info,
        import_info,
        defined_idents,
      } = analyze_statement_info(&i, item);
      Statement::new(i, export_info, import_info, defined_idents)
    })
    .collect::<Vec<_>>();

  module.meta = Box::new(ModuleMetaData::Script(ScriptModuleMetaData {
    statements,
    unresolved_mark: unresolved_mark.as_u32(),
    top_level_mark: top_level_mark.as_u32(),
    ..Default::default()
  }));
  StatementGraph::new(&module, &ast, &comment)
}

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
export default class f {
  constructor() {
    const a = 'a';
    this.a = a;
    this.b = b;
  }
}

export { a, b, c as d };"#;

  GLOBALS.set(&Globals::new(), || {
    let stmt_graph = create_test_statement_graph(code);
    assert_eq!(stmt_graph.stmts().len(), 7);
    let mut edges = stmt_graph.edges();
    edges.sort_by(|a, b| {
      let result = a.0.id.cmp(&b.0.id);

      if result == Ordering::Equal {
        a.1.id.cmp(&b.1.id)
      } else {
        result
      }
    });
    println!(
      "{:?}",
      edges
        .iter()
        .map(|e| (e.0.id.clone(), e.1.id.clone(), e.2))
        .collect::<Vec<_>>()
    );

    assert_eq!(edges.len(), 8);
    // statement 1 -> statement 0
    assert_eq!(edges[0].0.id, 1);
    assert_eq!(edges[0].1.id, 0);
    assert_eq!(edges[0].2.used_idents.len(), 0);
    assert_eq!(edges[0].2.used_idents_map.len(), 1);
    let used_idents = edges[0]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        (
          print_id(i),
          deps.iter().map(|i| print_id(i)).collect::<Vec<_>>(),
        )
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![("a#2".to_string(), vec!["aValue#2".to_string()])]
    );

    // statement 3 -> statement 1
    assert_eq!(edges[1].0.id, 3);
    assert_eq!(edges[1].1.id, 1);
    assert_eq!(edges[1].2.used_idents.len(), 0);
    assert_eq!(edges[1].2.used_idents_map.len(), 1);
    let used_idents = edges[1]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        let mut deps = deps.iter().map(|i| print_id(i)).collect::<Vec<_>>();
        deps.sort();
        (print_id(i), deps)
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![("c#2".to_string(), vec!["a#2".to_string()])]
    );

    // statement 3 -> statement 2
    assert_eq!(edges[2].0.id, 3);
    assert_eq!(edges[2].1.id, 2);
    assert_eq!(edges[2].2.used_idents.len(), 0);
    let used_idents = edges[2]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        let mut deps = deps.iter().map(|i| print_id(i)).collect::<Vec<_>>();
        deps.sort();
        (print_id(i), deps)
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![("c#2".to_string(), vec!["b#2".to_string()])]
    );

    // statement 4 -> statement 3
    assert_eq!(edges[3].0.id, 4);
    assert_eq!(edges[3].1.id, 3);
    // assert_eq!(edges[3].2.idents.len(), 1);
    // assert!(edges[3].2.idents.contains(&"c#2".to_string()));
    assert_eq!(edges[3].2.used_idents.len(), 0);

    let mut used_idents = edges[3]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        (
          print_id(i),
          deps.iter().map(|i| print_id(i)).collect::<Vec<_>>(),
        )
      })
      .collect::<Vec<_>>();
    used_idents.sort();
    assert_eq!(
      used_idents,
      vec![("e#2".to_string(), vec!["c#2".to_string()])]
    );

    // statement 5 -> statement 2
    assert_eq!(edges[4].0.id, 5);
    assert_eq!(edges[4].1.id, 2);
    assert_eq!(edges[4].2.used_idents.len(), 0);
    assert_eq!(edges[4].2.used_idents_map.len(), 1);
    let used_idents = edges[4]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        let mut deps = deps.iter().map(|i| print_id(i)).collect::<Vec<_>>();
        deps.sort();
        (print_id(i), deps)
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![("f#2".to_string(), vec!["b#2".to_string()])]
    );

    // statement 6 -> statement 1
    assert_eq!(edges[5].0.id, 6);
    assert_eq!(edges[5].1.id, 1);
    assert_eq!(edges[5].2.used_idents.len(), 0);
    assert_eq!(edges[5].2.used_idents_map.len(), 1);
    let used_idents = edges[5]
      .2
      .used_idents_map
      .iter()
      .map(|i| {
        (
          print_id(i.0),
          i.1.iter().map(|i| print_id(i)).collect::<Vec<_>>(),
        )
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![("a#2".to_string(), vec!["a#2".to_string()])]
    );

    // statement 6 -> statement 2
    assert_eq!(edges[6].0.id, 6);
    assert_eq!(edges[6].1.id, 2);
    assert_eq!(edges[6].2.used_idents.len(), 0);
    assert_eq!(edges[6].2.used_idents_map.len(), 1);
    let used_idents = edges[6]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        let mut deps = deps.iter().map(|i| print_id(i)).collect::<Vec<_>>();
        deps.sort();
        (print_id(i), deps)
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![("b#2".to_string(), vec!["b#2".to_string()])]
    );

    // statement 6 -> statement 3
    assert_eq!(edges[7].0.id, 6);
    assert_eq!(edges[7].1.id, 3);
    assert_eq!(edges[7].2.used_idents.len(), 0);
    assert_eq!(edges[7].2.used_idents_map.len(), 1);
    let used_idents = edges[7]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        let mut deps = deps.iter().map(|i| print_id(i)).collect::<Vec<_>>();
        deps.sort();
        (print_id(i), deps)
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![("c#2".to_string(), vec!["c#2".to_string()])]
    );
  });
}

#[test]
fn construct_statement_graph_complex_1() {
  let code = r#"
  var LOADABLE_REQUIRED_CHUNKS_KEY = '__LOADABLE_REQUIRED_CHUNKS__';
  function getRequiredChunkKey(namespace) {
    return "" + namespace + LOADABLE_REQUIRED_CHUNKS_KEY;
  }
  
  var sharedInternals = /*#__PURE__*/Object.freeze({
    __proto__: null,
    getRequiredChunkKey: getRequiredChunkKey,
    invariant: invariant,
    Context: Context
  });"#;

  GLOBALS.set(&Globals::new(), || {
    let stmt_graph = create_test_statement_graph(code);
    assert_eq!(stmt_graph.stmts().len(), 3);

    let mut edges = stmt_graph.edges();
    edges.sort_by(|a, b| {
      let result = a.0.id.cmp(&b.0.id);

      if result == Ordering::Equal {
        a.1.id.cmp(&b.1.id)
      } else {
        result
      }
    });

    // statement 1 -> statement 0
    assert_eq!(edges[0].0.id, 1);
    assert_eq!(edges[0].1.id, 0);
    assert_eq!(edges[0].2.used_idents.len(), 0);
    assert_eq!(edges[0].2.used_idents_map.len(), 1);
    let used_idents = edges[0]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        (
          print_id(i),
          deps.iter().map(|i| print_id(i)).collect::<Vec<_>>(),
        )
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![(
        "getRequiredChunkKey#2".to_string(),
        vec!["LOADABLE_REQUIRED_CHUNKS_KEY#2".to_string()]
      )]
    );

    // statement 2 -> statement 1
    assert_eq!(edges[1].0.id, 2);
    assert_eq!(edges[1].1.id, 1);
    assert_eq!(edges[1].2.used_idents.len(), 0);
    let used_idents = edges[1]
      .2
      .used_idents_map
      .iter()
      .map(|(i, deps)| {
        let mut deps = deps.iter().map(|i| print_id(i)).collect::<Vec<_>>();
        deps.sort();
        (print_id(i), deps)
      })
      .collect::<Vec<_>>();
    assert_eq!(
      used_idents,
      vec![(
        "sharedInternals#2".to_string(),
        vec!["getRequiredChunkKey#2".to_string()]
      )]
    );
  });
}

#[test]
fn trace_and_mark_used_statements() {
  let code = r#"
import { aValue } from './foo';
const a = aValue;
const b = 2;
const c = a + b;

export function e() {
  console.log(c);
}
export default class f {
  constructor() {
    const a = 'a';
    this.a = a;
    this.b = b;
  }
}

export { a, b, c as d };"#;

  GLOBALS.set(&Globals::new(), || {
    let mut stmt_graph = create_test_statement_graph(code);

    let traced_import_stmts = stmt_graph.trace_and_mark_used_statements(HashMap::from_iter([
      (5, HashSet::from_iter([UsedStatementIdent::Default])),
      (
        4,
        HashSet::from_iter([UsedStatementIdent::SwcIdent(
          ("e".into(), SyntaxContext::from_u32(2)).into(),
        )]),
      ),
    ]));

    assert_eq!(traced_import_stmts.len(), 1);
    assert_eq!(traced_import_stmts[0].stmt_id, 0);
    assert_eq!(traced_import_stmts[0].source, "./foo");
    assert_eq!(
      traced_import_stmts[0].used_stmt_idents.as_partial().len(),
      1
    );
    assert!(traced_import_stmts[0]
      .used_stmt_idents
      .as_partial()
      .contains(&UsedExportsIdent::SwcIdent("aValue".to_string())));

    let mut used_stmts = stmt_graph
      .used_stmts()
      .iter()
      .map(|i| *i)
      .collect::<Vec<_>>();
    used_stmts.sort();
    assert_eq!(used_stmts, [0, 1, 2, 3, 4, 5]);

    macro_rules! assert_used_defined_idents {
      ($stmt_id:expr, $expr_value:expr) => {
        let stmt = stmt_graph.stmt(&$stmt_id);
        let mut stmt_used_defined_idents = stmt
          .used_defined_idents
          .iter()
          .map(|i| print_id(i))
          .collect::<Vec<_>>();
        stmt_used_defined_idents.sort();
        println!("{} {:?}", $stmt_id, stmt_used_defined_idents);
        assert_eq!(stmt_used_defined_idents, $expr_value);
      };
    }

    assert_used_defined_idents!(0, vec!["aValue#2"]);
    assert_used_defined_idents!(1, vec!["a#2"]);
    assert_used_defined_idents!(2, vec!["b#2"]);
    assert_used_defined_idents!(3, vec!["c#2"]);
    assert_used_defined_idents!(4, vec!["e#2"]);
    assert_used_defined_idents!(5, vec!["f#2"]);
  });
}

#[test]
fn trace_and_mark_used_statements_from_export_named() {
  let code = r#"
import { aValue } from './foo';
const a = aValue;
const b = 2;
const c = a + b;

export function e() {
  console.log(c);
}
export default class f {
  constructor() {
    const a = 'a';
    this.a = a;
    this.b = b;
  }
}

export { a, b, c as d };"#;

  GLOBALS.set(&Globals::new(), || {
    let mut stmt_graph = create_test_statement_graph(code);

    let traced_import_stmts = stmt_graph.trace_and_mark_used_statements(HashMap::from_iter([(
      6,
      HashSet::from_iter([UsedStatementIdent::SwcIdent(
        ("c".into(), SyntaxContext::from_u32(2)).into(),
      )]),
    )]));

    assert_eq!(traced_import_stmts.len(), 1);
    assert_eq!(traced_import_stmts[0].stmt_id, 0);
    assert_eq!(traced_import_stmts[0].source, "./foo");
    assert_eq!(
      traced_import_stmts[0].used_stmt_idents.as_partial().len(),
      1
    );
    assert!(traced_import_stmts[0]
      .used_stmt_idents
      .as_partial()
      .contains(&UsedExportsIdent::SwcIdent("aValue".to_string())));

    let mut used_stmts = stmt_graph
      .used_stmts()
      .iter()
      .map(|i| *i)
      .collect::<Vec<_>>();
    used_stmts.sort();
    assert_eq!(used_stmts, [0, 1, 2, 3, 6]);

    macro_rules! assert_used_defined_idents {
      ($stmt_id:expr, $expr_value:expr) => {
        let stmt = stmt_graph.stmt(&$stmt_id);
        let mut stmt_used_defined_idents = stmt
          .used_defined_idents
          .iter()
          .map(|i| print_id(i))
          .collect::<Vec<_>>();
        stmt_used_defined_idents.sort();
        println!("{} {:?}", $stmt_id, stmt_used_defined_idents);
        assert_eq!(stmt_used_defined_idents, $expr_value);
      };
    }

    assert_used_defined_idents!(0, vec!["aValue#2"]);
    assert_used_defined_idents!(1, vec!["a#2"]);
    assert_used_defined_idents!(2, vec!["b#2"]);
    assert_used_defined_idents!(3, vec!["c#2"]);
    assert_used_defined_idents!(6, vec!["c#2"]);
  });
}

#[test]
fn trace_and_mark_used_statements_commonjs_exports() {
  let code = r#"
  module.exports = {
    program: function() {}
  }"#;

  GLOBALS.set(&Globals::new(), || {
    let mut stmt_graph = create_test_statement_graph(code);

    let traced_import_stmts = stmt_graph.trace_and_mark_used_statements(Default::default());

    assert_eq!(traced_import_stmts.len(), 0);

    let mut used_stmts = stmt_graph
      .used_stmts()
      .iter()
      .map(|i| *i)
      .collect::<Vec<_>>();
    used_stmts.sort();
    assert_eq!(used_stmts, [0]);
  });
}

#[test]
fn trace_and_mark_used_statements_default() {
  let code = r#"
  import commander from './command.js';

// wrapper to provide named exports for ESM.
export const {
  program,
  createCommand,
  createArgument,
  createOption,
  CommanderError,
  InvalidArgumentError,
  InvalidOptionArgumentError, // deprecated old name
  Command,
  Argument,
  Option,
  Help
} = commander;
"#;

  GLOBALS.set(&Globals::new(), || {
    let mut stmt_graph = create_test_statement_graph(code);

    let traced_import_stmts = stmt_graph.trace_and_mark_used_statements(HashMap::from_iter([(
      1,
      HashSet::from_iter([UsedStatementIdent::SwcIdent(
        ("program".into(), SyntaxContext::from_u32(2)).into(),
      )]),
    )]));

    assert_eq!(traced_import_stmts.len(), 1);
    assert_eq!(traced_import_stmts[0].stmt_id, 0);
    assert_eq!(traced_import_stmts[0].source, "./command.js");
    assert_eq!(
      traced_import_stmts[0].used_stmt_idents.as_partial().len(),
      1
    );
    assert!(traced_import_stmts[0]
      .used_stmt_idents
      .as_partial()
      .contains(&UsedExportsIdent::Default));

    let mut used_stmts = stmt_graph
      .used_stmts()
      .iter()
      .map(|i| *i)
      .collect::<Vec<_>>();
    used_stmts.sort();
    assert_eq!(used_stmts, [0, 1]);
  });
}
