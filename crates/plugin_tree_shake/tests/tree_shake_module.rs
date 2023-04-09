use farmfe_core::swc_common::{Globals, GLOBALS};
use farmfe_plugin_tree_shake::module::UsedIdent;

use farmfe_plugin_tree_shake::module::{TreeShakeModule, UsedExports};

use common::{create_module, create_module_with_globals};

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
  let module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&module);
  tree_shake_module.used_exports = UsedExports::Partial(vec![
    "a".to_string(),
    "d".to_string(),
    "e".to_string(),
    "f".to_string(),
    "any".to_string(),
    "h".to_string(),
    "i".to_string(),
    "j".to_string(),
    "default".to_string(),
  ]);

  let result = tree_shake_module.used_exports_idents();

  assert_eq!(result.len(), 9);
  assert_eq!(
    result
      .iter()
      .map(|item| item.0.to_string())
      .collect::<Vec<_>>(),
    vec![
      "a".to_string(),
      "c".to_string(),
      "e".to_string(),
      "default".to_string(),
      "any".to_string(),
      "h".to_string(),
      "i".to_string(),
      "j".to_string(),
      "default".to_string(),
    ]
  );
  assert_eq!(
    result.iter().map(|item| item.1).collect::<Vec<_>>(),
    vec![3, 3, 4, 4, 5, 6, 7, 8, 9]
  );

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
  let module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&module);
  tree_shake_module.used_exports = UsedExports::Partial(vec!["a".to_string()]);
  let result = tree_shake_module.used_exports_idents();
  assert_eq!(result.len(), 1);
  assert!(matches!(result[0].0, UsedIdent::SwcIdent(_)));
  assert_eq!(result[0].0.to_string(), "a".to_string());

  let code = r#"
export * from './foo';
export const b = 2;"#;
  let module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&module);
  tree_shake_module.used_exports =
    UsedExports::Partial(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
  let result = tree_shake_module.used_exports_idents();
  assert_eq!(result.len(), 3);
  assert!(matches!(result[0].0, UsedIdent::InExportAll(_)));
  assert_eq!(result[0].0.to_string(), "a".to_string());
  assert!(matches!(result[1].0, UsedIdent::SwcIdent(_)));
  assert_eq!(result[1].0.to_string(), "b".to_string());
  assert!(matches!(result[2].0, UsedIdent::InExportAll(_)));
  assert_eq!(result[2].0.to_string(), "c".to_string());
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
  let module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&module);
  tree_shake_module.used_exports = UsedExports::All;
  let result = tree_shake_module.used_exports_idents();

  assert_eq!(result.len(), 11);
  assert_eq!(
    result
      .iter()
      .map(|item| item.0.to_string())
      .collect::<Vec<_>>(),
    vec![
      "a".to_string(),
      "b".to_string(),
      "c".to_string(),
      "e".to_string(),
      "default".to_string(),
      "g".to_string(),
      "any".to_string(),
      "h".to_string(),
      "i".to_string(),
      "j".to_string(),
      "default".to_string(),
    ]
  );
}

#[test]
fn used_exports_idents_export_all_multiple() {
  let code = r#"
export * from './foo';
export * from './bar';
export const b = 2;"#;
  let module = create_module_with_globals(code);

  let mut tree_shake_module = TreeShakeModule::new(&module);
  tree_shake_module.used_exports =
    UsedExports::Partial(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
  let result = tree_shake_module.used_exports_idents();
  println!("{:?}", result);

  assert_eq!(result.len(), 5);

  assert!(matches!(result[0].0, UsedIdent::InExportAll(_)));
  assert_eq!(result[0].0.to_string(), "a".to_string());
  assert!(matches!(result[0].1, 0));

  assert!(matches!(result[1].0, UsedIdent::InExportAll(_)));
  assert_eq!(result[1].0.to_string(), "a".to_string());
  assert!(matches!(result[1].1, 1));

  assert!(matches!(result[2].0, UsedIdent::SwcIdent(_)));
  assert_eq!(result[2].0.to_string(), "b".to_string());
  assert!(matches!(result[2].1, 2));

  assert!(matches!(result[3].0, UsedIdent::InExportAll(_)));
  assert_eq!(result[3].0.to_string(), "c".to_string());
  assert!(matches!(result[3].1, 0));

  assert!(matches!(result[4].0, UsedIdent::InExportAll(_)));
  assert_eq!(result[4].0.to_string(), "c".to_string());
  assert!(matches!(result[4].1, 1));
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

  GLOBALS.set(&Globals::new(), || {
    let (module, _) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&module);
    tree_shake_module.used_exports = UsedExports::Partial(vec![
      "default".to_string(),
      "j".to_string(),
      "d".to_string(),
      "f".to_string(),
    ]);

    let result = tree_shake_module.used_statements();

    println!("{:?}", result);

    assert_eq!(result.len(), 7);
    // result should be the same as [(3, ["c#1"]), (2, ["c#1"]), (4, ["default#2"]), (8, ["j#1"]), (7, ["i#0"]), (6, ["h#1"]), (9, [])]
    assert_eq!(result[0].0, 3);
    assert_eq!(result[0].1.len(), 1);
    assert_eq!(result[0].1[0].to_string(), "c#1".to_string());
    assert_eq!(result[1].0, 2);
    assert_eq!(result[1].1.len(), 1);
    assert_eq!(result[1].1[0].to_string(), "c#1".to_string());
    assert_eq!(result[2].0, 4);
    assert_eq!(result[2].1.len(), 1);
    assert_eq!(result[2].1[0].to_string(), "default#2".to_string());
    assert_eq!(result[3].0, 8);
    assert_eq!(result[3].1.len(), 1);
    assert_eq!(result[3].1[0].to_string(), "j#1".to_string());
    assert_eq!(result[4].0, 7);
    assert_eq!(result[4].1.len(), 1);
    assert_eq!(result[4].1[0].to_string(), "i#0".to_string());
    assert_eq!(result[5].0, 6);
    assert_eq!(result[5].1.len(), 1);
    assert_eq!(result[5].1[0].to_string(), "h#1".to_string());
    assert_eq!(result[6].0, 9);
    assert_eq!(result[6].1.len(), 0);
  });
}

#[test]
fn used_statements_with_import() {
  let code = r#"
import { foo, zoo } from './foo';
const a = foo;
export { a };
  "#;

  GLOBALS.set(&Globals::new(), || {
    let (module, _) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&module);
    tree_shake_module.used_exports = UsedExports::Partial(vec!["a".to_string()]);

    let result = tree_shake_module.used_statements();

    println!("{:?}", result);

    assert_eq!(result.len(), 3);
    // result should be the same as [(2, ["a#2"]), (1, ["a#2"]), (0, ["foo#1"])]
    assert_eq!(result[0].0, 2);
    assert_eq!(result[0].1.len(), 1);
    assert_eq!(result[0].1[0].to_string(), "a#2".to_string());
    assert_eq!(result[1].0, 1);
    assert_eq!(result[1].1.len(), 1);
    assert_eq!(result[1].1[0].to_string(), "a#2".to_string());
    assert_eq!(result[2].0, 0);
    assert_eq!(result[2].1.len(), 1);
    assert_eq!(result[2].1[0].to_string(), "foo#1".to_string());
  });
}

#[test]
fn used_statements_with_export_all() {
  let code = r#"
export * from './foo';
export const b = 2;
  "#;

  GLOBALS.set(&Globals::new(), || {
    let (module, _) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&module);
    tree_shake_module.used_exports =
      UsedExports::Partial(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

    let result = tree_shake_module.used_statements();

    println!("{:?}", result);

    assert_eq!(result.len(), 2);
    // result should be the same as [(0, ["a", "c"]), (1, ["b#1"])]
    assert_eq!(result[0].0, 0);
    assert_eq!(result[0].1.len(), 2);
    assert_eq!(result[0].1[0].to_string(), "a".to_string());
    assert_eq!(result[0].1[1].to_string(), "c".to_string());
    assert_eq!(result[1].0, 1);
    assert_eq!(result[1].1.len(), 1);
    assert_eq!(result[1].1[0].to_string(), "b#1".to_string());
  });
}

#[test]
fn used_exports_idents_export_all_mutiple() {
  let code = r#"
export * from './foo';
export const a = 1;
export * from './bar';
  "#;

  GLOBALS.set(&Globals::new(), || {
    let (module, _) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&module);
    tree_shake_module.used_exports =
      UsedExports::Partial(vec!["a".to_string(), "b".to_string(), "c".to_string()]);

    let result = tree_shake_module.used_statements();

    println!("{:?}", result);

    assert_eq!(result.len(), 3);
    // result should be the same as [(0, ["b", "c"]), (1, ["b#1"]), (2, ["b", "c"])]
    assert_eq!(result[0].0, 0);
    assert_eq!(result[0].1.len(), 2);
    assert_eq!(result[0].1[0].to_string(), "b".to_string());
    assert_eq!(result[0].1[1].to_string(), "c".to_string());
    assert_eq!(result[1].0, 1);
    assert_eq!(result[1].1.len(), 1);
    assert_eq!(result[1].1[0].to_string(), "a#1".to_string());
    assert_eq!(result[2].0, 2);
    assert_eq!(result[2].1.len(), 2);
    assert_eq!(result[2].1[0].to_string(), "b".to_string());
    assert_eq!(result[2].1[1].to_string(), "c".to_string());
  });
}
