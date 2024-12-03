use farmfe_core::swc_common::{Globals, GLOBALS};
use farmfe_core::HashSet;
use farmfe_plugin_tree_shake::module::UsedExportsIdent;

use farmfe_plugin_tree_shake::module::{TreeShakeModule, UsedExports};

use common::{create_module, create_module_with_globals};

use crate::common::print_id;

mod common;

#[test]
fn used_exports_idents_basic() {
  let code = r#"
const a = 1;
const b = 2;
const c = 3;
export { a, b, c as d };
export { e, default as f, g } from './src/foo';
export * as any from './src/bar';
export const h = 1;
export function i() {}
export class j {}
export default 'default';
  "#;
  let mut module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&mut module);
  tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from_iter([
    UsedExportsIdent::SwcIdent("a".to_string()),
    UsedExportsIdent::SwcIdent("d".to_string()),
    UsedExportsIdent::SwcIdent("e".to_string()),
    UsedExportsIdent::SwcIdent("f".to_string()),
    UsedExportsIdent::SwcIdent("any".to_string()),
    UsedExportsIdent::SwcIdent("h".to_string()),
    UsedExportsIdent::SwcIdent("i".to_string()),
    UsedExportsIdent::SwcIdent("j".to_string()),
    UsedExportsIdent::Default,
  ]));

  let result = tree_shake_module.used_exports_to_statement_idents();
  let mut idents = result
    .iter()
    .map(|item| item.0.to_string())
    .collect::<Vec<_>>();
  idents.sort();

  assert_eq!(result.len(), 9);
  assert_eq!(
    idents,
    vec![
      "a#2".to_string(),
      "any#0".to_string(),
      "c#2".to_string(),
      "default".to_string(),
      "default#0".to_string(),
      "e#0".to_string(),
      "h#2".to_string(),
      "i#2".to_string(),
      "j#2".to_string(),
    ]
  );

  let mut stmts = result.iter().map(|item| item.1).collect::<Vec<_>>();
  stmts.sort();
  assert_eq!(stmts, vec![3, 3, 4, 4, 5, 6, 7, 8, 9]);

  let stmt = tree_shake_module.stmt_graph.stmt(&3);
  assert!(stmt.export_info.is_some());
  let export_info = stmt.export_info.as_ref().unwrap();
  assert_eq!(export_info.specifiers.len(), 3);

  let stmt = tree_shake_module.stmt_graph.stmt(&4);
  assert!(stmt.export_info.is_some());
  let export_info = stmt.export_info.as_ref().unwrap();
  assert_eq!(export_info.specifiers.len(), 3);
}

#[test]
fn used_exports_idents_export_all() {
  let code = "export const a = 1; export * from './foo'";
  let mut module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&mut module);
  // tree_shake_module.used_exports =
  //   UsedExports::Partial(HashMap::from([("index".into(), vec!["a".to_string()])]));
  tree_shake_module.pending_used_exports =
    UsedExports::Partial(HashSet::from_iter([UsedExportsIdent::SwcIdent(
      "a".to_string(),
    )]));
  // let result = tree_shake_module.used_exports_idents();
  let result = tree_shake_module.used_exports_to_statement_idents();
  assert_eq!(result.len(), 1);
  let mut idents = result
    .iter()
    .map(|item| item.0.to_string())
    .collect::<Vec<_>>();
  idents.sort();
  assert_eq!(idents, vec!["a#2".to_string()]);

  let code = r#"
export * from './foo';
export const b = 2;"#;
  let mut module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&mut module);
  tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from_iter([
    UsedExportsIdent::SwcIdent("a".to_string()),
    UsedExportsIdent::SwcIdent("b".to_string()),
    UsedExportsIdent::SwcIdent("c".to_string()),
  ]));
  let result = tree_shake_module.used_exports_to_statement_idents();
  assert_eq!(result.len(), 3);

  let mut idents = result
    .iter()
    .map(|item| item.0.to_string())
    .collect::<Vec<_>>();
  idents.sort();
  assert_eq!(
    idents,
    vec!["*(a)".to_string(), "*(c)".to_string(), "b#2".to_string(),]
  );
}

#[test]
fn used_exports_idents_used_all() {
  let code = r#"
const a = 1;
const b = 2;
const c = 3;
export { a, b, c as d };
export { e, default as f, g } from './src/foo';
export * as any from './src/bar';
export const h = 1;
export function i() {}
export class j {}
export default 'default';
  "#;
  let mut module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&mut module);
  // tree_shake_module.used_exports = UsedExports::All(HashMap::new());
  // let result = tree_shake_module.used_exports_idents();
  tree_shake_module.pending_used_exports = UsedExports::All;
  let result = tree_shake_module.used_exports_to_statement_idents();

  assert_eq!(result.len(), 11);

  let mut idents = result
    .iter()
    .map(|item| item.0.to_string())
    .collect::<Vec<_>>();
  idents.sort();
  assert_eq!(
    idents,
    vec![
      "a#2".to_string(),
      "any#0".to_string(),
      "b#2".to_string(),
      "c#2".to_string(),
      "default".to_string(),
      "default#0".to_string(),
      "e#0".to_string(),
      "g#0".to_string(),
      "h#2".to_string(),
      "i#2".to_string(),
      "j#2".to_string(),
    ]
  );
}

#[test]
fn used_exports_idents_export_all_multiple() {
  let code = r#"
export * from './foo';
export * from './bar';
export const b = 2;"#;
  let mut module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&mut module);
  tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from_iter([
    UsedExportsIdent::SwcIdent("a".to_string()),
    UsedExportsIdent::SwcIdent("b".to_string()),
    UsedExportsIdent::SwcIdent("c".to_string()),
  ]));
  let result = tree_shake_module.used_exports_to_statement_idents();

  assert_eq!(result.len(), 5);

  let mut idents = result
    .iter()
    .map(|item| item.0.to_string())
    .collect::<Vec<_>>();
  idents.sort();
  assert_eq!(
    idents,
    vec![
      "*(a)".to_string(),
      "*(a)".to_string(),
      "*(c)".to_string(),
      "*(c)".to_string(),
      "b#2".to_string(),
    ]
  );
  let mut stmts = result.iter().map(|item| item.1).collect::<Vec<_>>();
  stmts.sort();
  assert_eq!(stmts, vec![0, 0, 1, 1, 2]);
}

#[test]
fn used_statements_basic() {
  let code = r#"
const a = 1;
const b = 2;
const c = 3;
export { a, b, c as d };
export { e, default as f, g } from './src/foo';
export * as any from './src/bar';
export const h = 1;
export function i() {
  return h;
}
export class j {
  constructor() {
    this.i = i();
  }
}
export default 'default';
  "#;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let (mut module, _) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from_iter([
      UsedExportsIdent::Default,
      UsedExportsIdent::SwcIdent("j".to_string()),
      UsedExportsIdent::SwcIdent("d".to_string()),
      UsedExportsIdent::SwcIdent("f".to_string()),
    ]));

    let result = tree_shake_module.trace_and_mark_used_statements();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].stmt_id, 4);
    assert_eq!(result[0].source, "./src/foo");
    assert_eq!(result[0].used_stmt_idents.as_partial().len(), 1);
    assert!(result[0]
      .used_stmt_idents
      .as_partial()
      .contains(&UsedExportsIdent::Default));

    let used_stmts = tree_shake_module.stmt_graph.used_stmts();
    let mut stmts = used_stmts.iter().map(|stmt| *stmt).collect::<Vec<_>>();
    stmts.sort();
    assert_eq!(stmts, vec![2, 3, 4, 6, 7, 8, 9]);

    macro_rules! assert_stmt {
      ($stmt_id: expr, $expected: expr) => {
        let stmt = tree_shake_module.stmt_graph.stmt(&$stmt_id);
        let mut stmt_used_idents = stmt
          .used_defined_idents
          .iter()
          .map(|i| print_id(i))
          .collect::<Vec<_>>();
        stmt_used_idents.sort();
        assert_eq!(stmt_used_idents, $expected);
      };
    }

    assert_stmt!(2, vec!["c#2"]);
    assert_stmt!(3, vec!["c#2"]);
    assert_stmt!(4, vec!["default#0"]);
    assert_stmt!(6, vec!["h#2"]);
    assert_stmt!(7, vec!["i#2"]);
    assert_stmt!(8, vec!["j#2"]);
    assert_stmt!(9, Vec::<String>::new());
  });
}

#[test]
fn used_statements_with_import() {
  let code = r#"
import { foo, zoo } from './foo';
const a = foo;
export { a };
  "#;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let (mut module, _) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    // tree_shake_module.used_exports =
    //   UsedExports::Partial(HashMap::from([("index.ts".into(), vec!["a".to_string()])]));
    tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from_iter([
      UsedExportsIdent::SwcIdent("a".to_string()),
      UsedExportsIdent::SwcIdent("foo".to_string()),
    ]));

    let result = tree_shake_module.trace_and_mark_used_statements();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].stmt_id, 0);
    assert_eq!(result[0].source, "./foo");
    assert!(result[0]
      .used_stmt_idents
      .contains(&UsedExportsIdent::SwcIdent("foo".to_string())));

    let used_stmts = tree_shake_module.stmt_graph.used_stmts();
    let mut stmts = used_stmts.iter().map(|stmt| *stmt).collect::<Vec<_>>();
    stmts.sort();
    assert_eq!(stmts, vec![0, 1, 2]);

    macro_rules! assert_stmt {
      ($stmt_id: expr, $expected: expr) => {
        let stmt = tree_shake_module.stmt_graph.stmt(&$stmt_id);
        let mut stmt_used_idents = stmt
          .used_defined_idents
          .iter()
          .map(|i| print_id(i))
          .collect::<Vec<_>>();
        stmt_used_idents.sort();
        assert_eq!(stmt_used_idents, $expected);
      };
    }

    assert_stmt!(0, vec!["foo#2"]);
    assert_stmt!(1, vec!["a#2"]);
    assert_stmt!(2, vec!["a#2"]);
  });
}

#[test]
fn used_statements_with_export_all() {
  let code = r#"
export * from './foo';
export const b = 2;
  "#;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let (mut module, _) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);
    tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from_iter([
      UsedExportsIdent::SwcIdent("a".to_string()),
      UsedExportsIdent::SwcIdent("b".to_string()),
      UsedExportsIdent::SwcIdent("c".to_string()),
    ]));

    let result = tree_shake_module.trace_and_mark_used_statements();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].stmt_id, 0);
    assert_eq!(result[0].source, "./foo");
    assert_eq!(result[0].used_stmt_idents.as_partial().len(), 2);
    assert!(result[0]
      .used_stmt_idents
      .as_partial()
      .contains(&UsedExportsIdent::SwcIdent("a".to_string())));
    assert!(result[0]
      .used_stmt_idents
      .as_partial()
      .contains(&UsedExportsIdent::SwcIdent("c".to_string())));

    let used_stmts = tree_shake_module.stmt_graph.used_stmts();
    let mut stmts = used_stmts.iter().map(|stmt| *stmt).collect::<Vec<_>>();
    stmts.sort();
    assert_eq!(stmts, vec![0, 1]);

    macro_rules! assert_stmt {
      ($stmt_id: expr, $expected: expr) => {
        let stmt = tree_shake_module.stmt_graph.stmt(&$stmt_id);
        let mut stmt_used_idents = stmt
          .used_defined_idents
          .iter()
          .map(|i| print_id(i))
          .collect::<Vec<_>>();
        stmt_used_idents.sort();
        assert_eq!(stmt_used_idents, $expected);
      };
    }

    assert_stmt!(0, Vec::<&str>::new());
    assert_stmt!(1, vec!["b#2"]);
  });
}

#[test]
fn used_exports_idents_export_all_mutiple() {
  let code = r#"
export * from './foo';
export const a = 1;
export * from './bar';
  "#;

  let globals = Globals::new();
  GLOBALS.set(&globals, || {
    let (mut module, _) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&mut module);

    tree_shake_module.pending_used_exports = UsedExports::Partial(HashSet::from_iter([
      UsedExportsIdent::SwcIdent("a".to_string()),
      UsedExportsIdent::SwcIdent("b".to_string()),
      UsedExportsIdent::SwcIdent("c".to_string()),
    ]));
    let result = tree_shake_module.trace_and_mark_used_statements();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].stmt_id, 0);
    assert_eq!(result[0].source, "./foo");
    assert_eq!(result[0].used_stmt_idents.as_partial().len(), 2);
    println!("{result:?}");
    assert!(result[0]
      .used_stmt_idents
      .contains(&UsedExportsIdent::SwcIdent("b".to_string())));
    assert!(result[0]
      .used_stmt_idents
      .contains(&UsedExportsIdent::SwcIdent("c".to_string())));

    assert_eq!(result[1].stmt_id, 2);
    assert_eq!(result[1].source, "./bar");
    assert_eq!(result[1].used_stmt_idents.as_partial().len(), 2);
    assert!(result[1]
      .used_stmt_idents
      .contains(&UsedExportsIdent::SwcIdent("b".to_string())));
    assert!(result[1]
      .used_stmt_idents
      .contains(&UsedExportsIdent::SwcIdent("c".to_string())));

    let used_stmts = tree_shake_module.stmt_graph.used_stmts();
    let mut stmts = used_stmts.iter().map(|stmt| *stmt).collect::<Vec<_>>();
    stmts.sort();

    assert_eq!(stmts, vec![0, 1, 2]);

    macro_rules! assert_stmt {
      ($stmt_id: expr, $expected: expr) => {
        let stmt = tree_shake_module.stmt_graph.stmt(&$stmt_id);
        let mut stmt_used_idents = stmt
          .used_defined_idents
          .iter()
          .map(|i| print_id(i))
          .collect::<Vec<_>>();
        stmt_used_idents.sort();
        assert_eq!(stmt_used_idents, $expected);
      };
    }

    assert_stmt!(0, Vec::<&str>::new());
    assert_stmt!(1, vec!["a#2"]);
    assert_stmt!(2, Vec::<&str>::new());
  });
}
