use common::create_module;
use farmfe_core::{
  swc_common::{Globals, GLOBALS},
  swc_ecma_ast::EsVersion,
};
use farmfe_plugin_tree_shake::{
  module::{TreeShakeModule, UsedExports},
  remove_useless_stmts::remove_useless_stmts,
  statement_graph::{ExportSpecifierInfo, ImportSpecifierInfo},
};
use farmfe_toolkit::script::codegen_module;

mod common;

#[test]
fn remove_useless_stmts_basic() {
  let code = r#"
import { aValue, bar } from './foo';
const a = aValue;
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
    let (mut module, cm) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&module);
    tree_shake_module.used_exports = UsedExports::Partial(vec![
      "default".to_string(),
      "j".to_string(),
      "d".to_string(),
      "f".to_string(),
      "a".to_string(),
    ]);

    let swc_module = &mut module.meta.as_script_mut().ast;

    let (import_info, export_info) = remove_useless_stmts(&mut tree_shake_module, swc_module);

    // println!("import_info: {:#?}", import_info);
    // println!("export_info: {:#?}", export_info);

    let bytes = codegen_module(swc_module, EsVersion::EsNext, cm, None).unwrap();
    let result = String::from_utf8(bytes).unwrap();
    let expect = r#"import { aValue } from './foo';
const a = aValue;
const c = 3;
export { a, c as d };
export { default as f } from './src/foo';
export const h = 1;
export function i() {
    return h;
}
export class j {
    constructor(){
        this.i = i();
    }
}
export default 'default';
    "#
    .trim();
    // asset result and expect line by line
    let result_lines = result.trim().lines();
    let expect_lines = expect.lines();
    for (result_line, expect_line) in result_lines.zip(expect_lines) {
      assert_eq!(result_line, expect_line);
    }

    assert_eq!(import_info.len(), 1);
    assert_eq!(import_info[0].specifiers.len(), 1);
    assert!(matches!(
      import_info[0].specifiers[0],
      ImportSpecifierInfo::Named { .. }
    ));
    if let ImportSpecifierInfo::Named { local, imported } = &import_info[0].specifiers[0] {
      assert_eq!(local.to_string(), "aValue#1".to_string());
      assert!(imported.is_none());
    }

    assert_eq!(export_info.len(), 1);
    assert_eq!(export_info[0].specifiers.len(), 1);
    assert!(matches!(
      export_info[0].specifiers[0],
      ExportSpecifierInfo::Named { .. }
    ));
    if let ExportSpecifierInfo::Named { local, exported } = &export_info[0].specifiers[0] {
      assert_eq!(local.to_string(), "default#1".to_string());
      assert!(exported.is_some());

      if let Some(exported) = exported {
        assert_eq!(exported.to_string(), "f#1".to_string());
      }
    }
  });
}

#[test]
fn remove_useless_stmts_export_all() {
  let code = r#"
import { aValue, bar } from './foo';
export const a = aValue;
const b = 2;
export * from './src/foo';
"#;

  GLOBALS.set(&Globals::new(), || {
    let (mut module, cm) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&module);
    tree_shake_module.used_exports =
      UsedExports::Partial(vec!["a".to_string(), "c".to_string(), "d".to_string()]);

    let swc_module = &mut module.meta.as_script_mut().ast;

    let (import_info, export_info) = remove_useless_stmts(&mut tree_shake_module, swc_module);

    // println!("import_info: {:#?}", import_info);
    // println!("export_info: {:#?}", export_info);

    let bytes = codegen_module(swc_module, EsVersion::EsNext, cm, None).unwrap();
    let result = String::from_utf8(bytes).unwrap();
    assert_eq!(
      result,
      r#"import { aValue } from './foo';
export const a = aValue;
export * from './src/foo';
"#
    );

    assert_eq!(import_info.len(), 1);
    assert_eq!(import_info[0].specifiers.len(), 1);
    assert!(matches!(
      import_info[0].specifiers[0],
      ImportSpecifierInfo::Named { .. }
    ));
    if let ImportSpecifierInfo::Named { local, imported } = &import_info[0].specifiers[0] {
      assert_eq!(local.to_string(), "aValue#1".to_string());
      assert!(imported.is_none());
    }

    // Only contains the export * from './src/foo';
    assert_eq!(export_info.len(), 1);
    assert_eq!(export_info[0].specifiers.len(), 1);
    assert!(matches!(
      export_info[0].specifiers[0],
      ExportSpecifierInfo::All(_)
    ));
    if let ExportSpecifierInfo::All(used_idents) = &export_info[0].specifiers[0] {
      assert!(used_idents.is_some());
      let used_idents = used_idents.as_ref().unwrap();
      assert_eq!(used_idents.len(), 2);
      assert!(used_idents.contains(&"c".to_string()));
      assert!(used_idents.contains(&"d".to_string()));
    }
  });
}

#[test]
fn remove_useless_stmts_export_all_multiple() {
  let code = r#"
import { aValue, bar } from './foo';
export const a = aValue;
export * from './src/foo';
export * from './src/bar';
"#;

  GLOBALS.set(&Globals::new(), || {
    let (mut module, cm) = create_module(code);
    let mut tree_shake_module = TreeShakeModule::new(&module);
    tree_shake_module.used_exports = UsedExports::Partial(vec!["c".to_string(), "d".to_string()]);

    let swc_module = &mut module.meta.as_script_mut().ast;

    let (import_info, export_info) = remove_useless_stmts(&mut tree_shake_module, swc_module);

    // println!("import_info: {:#?}", import_info);
    // println!("export_info: {:#?}", export_info);

    let bytes = codegen_module(swc_module, EsVersion::EsNext, cm, None).unwrap();
    let result = String::from_utf8(bytes).unwrap();
    assert_eq!(
      result,
      r#"export * from './src/foo';
export * from './src/bar';
"#
    );

    assert_eq!(import_info.len(), 0);

    // contains the export * from './src/foo'; and export * from './src/bar';
    assert_eq!(export_info.len(), 2);
    assert_eq!(export_info[0].specifiers.len(), 1);
    assert!(matches!(
      export_info[0].specifiers[0],
      ExportSpecifierInfo::All(_)
    ));
    if let ExportSpecifierInfo::All(used_idents) = &export_info[0].specifiers[0] {
      assert!(used_idents.is_some());
      let used_idents = used_idents.as_ref().unwrap();
      assert_eq!(used_idents.len(), 2);
      assert!(used_idents.contains(&"c".to_string()));
      assert!(used_idents.contains(&"d".to_string()));
    }

    assert_eq!(export_info[1].specifiers.len(), 1);
    assert!(matches!(
      export_info[1].specifiers[0],
      ExportSpecifierInfo::All(_)
    ));
    if let ExportSpecifierInfo::All(used_idents) = &export_info[1].specifiers[0] {
      assert!(used_idents.is_some());
      let used_idents = used_idents.as_ref().unwrap();
      assert_eq!(used_idents.len(), 2);
      assert!(used_idents.contains(&"c".to_string()));
      assert!(used_idents.contains(&"d".to_string()));
    }
  });
}
