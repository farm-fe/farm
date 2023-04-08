use std::sync::Arc;

use farmfe_core::{
  swc_common::{FilePathMapping, SourceMap},
  swc_ecma_ast::ModuleItem,
  swc_ecma_parser::Syntax,
};
use farmfe_toolkit::script::parse_module;

use crate::statement_graph::{ExportSpecifierInfo, ImportSpecifierInfo};

use super::analyze_imports_and_exports;

fn parse_module_item(stmt: &str) -> ModuleItem {
  let module = parse_module(
    "any",
    stmt,
    Syntax::Es(Default::default()),
    farmfe_core::swc_ecma_ast::EsVersion::Es2015,
    Arc::new(SourceMap::new(FilePathMapping::empty())),
  )
  .unwrap();
  module.body[0].clone()
}

#[test]
fn import_default() {
  let stmt = parse_module_item(r#"import a from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Default(_)
  ));

  if let ImportSpecifierInfo::Default(ident) = &import_info.specifiers[0] {
    assert_eq!(ident.sym.to_string(), "a".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 1);
  assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents.len(), 0);
  assert_eq!(defined_idents_map.len(), 0);
}

#[test]
fn import_named() {
  let stmt = parse_module_item(r#"import { a, b, c as nc } from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[0] {
    assert_eq!(local.sym.to_string(), "a".to_string());
    assert!(imported.is_none());
  }

  assert!(matches!(
    import_info.specifiers[1],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[1] {
    assert_eq!(local.sym.to_string(), "b".to_string());
    assert!(imported.is_none());
  }

  assert!(matches!(
    import_info.specifiers[2],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[2] {
    assert_eq!(local.sym.to_string(), "nc".to_string());
    assert!(imported.is_some());
    assert_eq!(imported.as_ref().unwrap().sym.to_string(), "c".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 3);
  assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(defined_idents[1].sym.to_string(), "b".to_string());
  assert_eq!(defined_idents[2].sym.to_string(), "nc".to_string());
  assert_eq!(used_idents.len(), 0);
  assert_eq!(defined_idents_map.len(), 0);
}

#[test]
fn import_named_with_used_defined_idents() {
  let stmt = parse_module_item(r#"import { a, b, c as nc } from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, Some(vec!["a#0".to_string()]));

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[0] {
    assert_eq!(local.sym.to_string(), "a".to_string());
    assert!(imported.is_none());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 1);
  assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents.len(), 0);
  assert_eq!(defined_idents_map.len(), 0);
}

#[test]
fn import_namespace() {
  let stmt = parse_module_item(r#"import * as a from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Namespace(_)
  ));

  if let ImportSpecifierInfo::Namespace(ident) = &import_info.specifiers[0] {
    assert_eq!(ident.sym.to_string(), "a".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 1);
  assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents.len(), 0);
  assert_eq!(defined_idents_map.len(), 0);
}

#[test]
fn export_default_expr() {
  let stmt = parse_module_item(r#"export default a"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Default
  ));

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 1);
  assert_eq!(used_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(defined_idents_map.len(), 0);
}

#[test]
fn export_default_decl() {
  let stmt = parse_module_item(r#"export default function a() { return b; }"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Default
  ));

  assert_eq!(defined_idents.len(), 1);
  assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents.len(), 1);
  assert_eq!(used_idents[0].sym.to_string(), "b".to_string());
  assert_eq!(defined_idents_map.len(), 1);
  let keys = defined_idents_map.keys().collect::<Vec<_>>();
  assert_eq!(keys[0].sym.to_string(), "a".to_string());
  let values = defined_idents_map.values().collect::<Vec<_>>();
  assert_eq!(values.len(), 1);
  assert_eq!(values[0].len(), 1);
  assert_eq!(values[0][0].sym.to_string(), "b".to_string());
}

#[test]
fn export_decl() {
  let stmt = parse_module_item(r#"export function a() { return b; }"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Named { .. }
  ));

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
    assert_eq!(local.sym.to_string(), "a".to_string());
    assert!(exported.is_none());
  }

  assert_eq!(defined_idents.len(), 1);
  assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents.len(), 1);
  assert_eq!(used_idents[0].sym.to_string(), "b".to_string());
  assert_eq!(defined_idents_map.len(), 1);
  let keys = defined_idents_map.keys().collect::<Vec<_>>();
  assert_eq!(keys[0].sym.to_string(), "a".to_string());
  let values = defined_idents_map.values().collect::<Vec<_>>();
  assert_eq!(values.len(), 1);
  assert_eq!(values[0].len(), 1);
  assert_eq!(values[0][0].sym.to_string(), "b".to_string());
}

#[test]
fn export_all() {
  let stmt = parse_module_item(r#"export * from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::All(None)
  ));
  assert_eq!(export_info.source, Some("a".to_string()));

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 0);

  assert_eq!(defined_idents_map.len(), 0);
}

#[test]
fn export_named_from() {
  let stmt = parse_module_item(r#"export { a, b as c, default as d } from 'a';"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();
  assert_eq!(export_info.source, Some("a".to_string()));

  assert_eq!(export_info.specifiers.len(), 3);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Named { .. }
  ));
  assert!(matches!(
    export_info.specifiers[1],
    ExportSpecifierInfo::Named { .. }
  ));
  assert!(matches!(
    export_info.specifiers[2],
    ExportSpecifierInfo::Named { .. }
  ));

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
    assert_eq!(local.sym.to_string(), "a".to_string());
    assert!(exported.is_none());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(local.sym.to_string(), "b".to_string());
    assert!(exported.is_some());
    assert_eq!(exported.as_ref().unwrap().sym.to_string(), "c".to_string());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[2] {
    assert_eq!(local.sym.to_string(), "default".to_string());
    assert!(exported.is_some());
    assert_eq!(exported.as_ref().unwrap().sym.to_string(), "d".to_string());
  }

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 0);

  assert_eq!(defined_idents_map.len(), 0);
}

#[test]
fn export_named() {
  let stmt = parse_module_item(r#"export { a, b as c, any as d };"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();
  assert_eq!(export_info.source, None);

  assert_eq!(export_info.specifiers.len(), 3);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Named { .. }
  ));
  assert!(matches!(
    export_info.specifiers[1],
    ExportSpecifierInfo::Named { .. }
  ));
  assert!(matches!(
    export_info.specifiers[2],
    ExportSpecifierInfo::Named { .. }
  ));

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
    assert_eq!(local.sym.to_string(), "a".to_string());
    assert!(exported.is_none());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(local.sym.to_string(), "b".to_string());
    assert!(exported.is_some());
    assert_eq!(exported.as_ref().unwrap().sym.to_string(), "c".to_string());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[2] {
    assert_eq!(local.sym.to_string(), "any".to_string());
    assert!(exported.is_some());
    assert_eq!(exported.as_ref().unwrap().sym.to_string(), "d".to_string());
  }

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 3);
  assert_eq!(used_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents[1].sym.to_string(), "b".to_string());
  assert_eq!(used_idents[2].sym.to_string(), "any".to_string());

  assert_eq!(defined_idents_map.len(), 3);
}

#[test]
fn export_named_with_used_defined_idents() {
  let stmt = parse_module_item(r#"export { a, b as c, any as d };"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, Some(vec!["a#0".to_string()]));

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();
  assert_eq!(export_info.source, None);

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Named { .. }
  ));

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
    assert_eq!(local.sym.to_string(), "a".to_string());
    assert!(exported.is_none());
  }

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 1);
  assert_eq!(used_idents[0].sym.to_string(), "a".to_string());

  assert_eq!(defined_idents_map.len(), 1);
}

#[test]
fn export_namespace() {
  let stmt = parse_module_item(r#"export * as a from 'a';"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();
  assert_eq!(export_info.source, Some("a".to_string()));

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Namespace { .. }
  ));

  if let ExportSpecifierInfo::Namespace(local) = &export_info.specifiers[0] {
    assert_eq!(local.sym.to_string(), "a".to_string());
  }

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 0);

  assert_eq!(defined_idents_map.len(), 0);
}

#[test]
fn func_decl() {
  let stmt = parse_module_item(r#"function a() { b(); return c; }"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 1);
  assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents.len(), 2);
  assert_eq!(used_idents[0].sym.to_string(), "b".to_string());
  assert_eq!(used_idents[1].sym.to_string(), "c".to_string());

  assert_eq!(defined_idents_map.len(), 1);
  let keys = defined_idents_map.keys().collect::<Vec<_>>();
  assert_eq!(keys.len(), 1);
  assert_eq!(keys[0].sym.to_string(), "a".to_string());
  let values = defined_idents_map.values().collect::<Vec<_>>();
  assert_eq!(values.len(), 1);
  assert_eq!(values[0].len(), 2);
  assert_eq!(values[0][0].sym.to_string(), "b".to_string());
  assert_eq!(values[0][1].sym.to_string(), "c".to_string());
}

#[test]
fn bar_decl() {
  let stmt = parse_module_item(r#"var a = b;"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 1);
  assert_eq!(defined_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents.len(), 1);
  assert_eq!(used_idents[0].sym.to_string(), "b".to_string());

  assert_eq!(defined_idents_map.len(), 1);
  let keys = defined_idents_map.keys().collect::<Vec<_>>();
  assert_eq!(keys.len(), 1);
  assert_eq!(keys[0].sym.to_string(), "a".to_string());
  let values = defined_idents_map.values().collect::<Vec<_>>();
  assert_eq!(values.len(), 1);
  assert_eq!(values[0].len(), 1);
  assert_eq!(values[0][0].sym.to_string(), "b".to_string());
}

#[test]
fn for_stmt() {
  let stmt = parse_module_item(r#"for (var a = b; c; d) { e; }"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 5);
  // treat a as used for now as it does not affect the result.
  assert_eq!(used_idents[0].sym.to_string(), "a".to_string());
  assert_eq!(used_idents[1].sym.to_string(), "b".to_string());
  assert_eq!(used_idents[2].sym.to_string(), "c".to_string());
  assert_eq!(used_idents[3].sym.to_string(), "d".to_string());
  assert_eq!(used_idents[4].sym.to_string(), "e".to_string());

  assert_eq!(defined_idents_map.len(), 0);
}
