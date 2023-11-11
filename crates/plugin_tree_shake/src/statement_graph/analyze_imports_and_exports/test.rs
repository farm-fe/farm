use farmfe_core::{hashbrown::HashSet, swc_ecma_ast::ModuleItem, swc_ecma_parser::Syntax};
use farmfe_toolkit::script::parse_module;

use crate::statement_graph::{ExportSpecifierInfo, ImportSpecifierInfo};

use super::analyze_imports_and_exports;

fn parse_module_item(stmt: &str) -> ModuleItem {
  let module = parse_module(
    "any",
    stmt,
    Syntax::Es(Default::default()),
    farmfe_core::swc_ecma_ast::EsVersion::Es2015,
  )
  .unwrap();
  module.body[0].clone()
}

#[test]
fn import_default() {
  let stmt = parse_module_item(r#"import a from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Default(_)
  ));

  if let ImportSpecifierInfo::Default(ident) = &import_info.specifiers[0] {
    assert_eq!(ident, &"a#0".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 1);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| ident.to_string())
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );
  assert_eq!(used_idents.len(), 0);
  assert_eq!(defined_idents_map.len(), 0);
  assert!(!is_self_executed);
}

#[test]
fn import_named() {
  let stmt = parse_module_item(r#"import { a, b, c as nc } from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[0] {
    assert_eq!(local.to_string(), "a#0".to_string());
    assert!(imported.is_none());
  }

  assert!(matches!(
    import_info.specifiers[1],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[1] {
    assert_eq!(local.to_string(), "b#0".to_string());
    assert!(imported.is_none());
  }

  assert!(matches!(
    import_info.specifiers[2],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[2] {
    assert_eq!(local.to_string(), "nc#0".to_string());
    assert!(imported.is_some());
    assert_eq!(imported.as_ref().unwrap().to_string(), "c#0".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 3);
  let defined_idents_str = defined_idents
    .into_iter()
    .map(|ident| ident)
    .collect::<HashSet<_>>();
  assert!(defined_idents_str.contains(&"a#0".to_string()));
  assert!(defined_idents_str.contains(&"b#0".to_string()));
  assert!(defined_idents_str.contains(&"nc#0".to_string()));

  assert_eq!(used_idents.len(), 0);
  assert_eq!(defined_idents_map.len(), 0);
  assert!(!is_self_executed);
}

#[test]
fn import_named_with_used_defined_idents() {
  let stmt = parse_module_item(r#"import { a, b, c as nc } from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, Some(["a#0".to_string()].into()));

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[0] {
    assert_eq!(local.to_string(), "a#0".to_string());
    assert!(imported.is_none());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 1);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| ident.to_string())
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );

  assert_eq!(used_idents.len(), 0);
  assert_eq!(defined_idents_map.len(), 0);
  assert!(!is_self_executed);
}

#[test]
fn import_namespace() {
  let stmt = parse_module_item(r#"import * as a from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Namespace(_)
  ));

  if let ImportSpecifierInfo::Namespace(ident) = &import_info.specifiers[0] {
    assert_eq!(ident.to_string(), "a#0".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 1);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| ident.to_string())
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );

  assert_eq!(used_idents.len(), 0);
  assert_eq!(defined_idents_map.len(), 0);
  assert!(!is_self_executed);
}

#[test]
fn export_default_expr() {
  let stmt = parse_module_item(r#"export default a"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
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
  assert_eq!(
    used_idents
      .iter()
      .map(|ident| ident.to_string())
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );

  assert_eq!(defined_idents_map.len(), 0);
  assert!(!is_self_executed);
}

#[test]
fn export_default_decl() {
  let stmt = parse_module_item(r#"export default function a() { return b; }"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
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
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| ident.to_string())
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );
  assert_eq!(used_idents.len(), 1);
  assert_eq!(
    used_idents
      .iter()
      .map(|ident| ident.to_string())
      .collect::<Vec<_>>()[0],
    "b#0".to_string()
  );
  assert_eq!(defined_idents_map.len(), 1);
  let keys = defined_idents_map.keys().collect::<Vec<_>>();
  assert_eq!(keys[0].to_string(), "a#0".to_string());
  let values = defined_idents_map.values().collect::<Vec<_>>();
  assert_eq!(values.len(), 1);
  assert_eq!(values[0].len(), 1);
  let values_0 = values[0].iter().collect::<Vec<_>>();
  assert_eq!(values_0[0].to_string(), "b#0".to_string());
  assert!(!is_self_executed);
}

#[test]
fn export_decl() {
  let stmt = parse_module_item(r#"export function a() { return b; }"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
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
    assert_eq!(local.to_string(), "a#0".to_string());
    assert!(exported.is_none());
  }

  assert_eq!(defined_idents.len(), 1);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| ident.to_string())
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );
  assert_eq!(used_idents.len(), 1);
  assert_eq!(
    used_idents
      .iter()
      .map(|ident| ident.to_string())
      .collect::<Vec<_>>()[0],
    "b#0".to_string()
  );
  assert_eq!(defined_idents_map.len(), 1);
  let keys = defined_idents_map.keys().collect::<Vec<_>>();
  assert_eq!(keys[0].to_string(), "a#0".to_string());
  let values = defined_idents_map.values().collect::<Vec<_>>();
  assert_eq!(values.len(), 1);
  assert_eq!(values[0].len(), 1);
  let values_0 = values[0].iter().collect::<Vec<_>>();
  assert_eq!(values_0[0].to_string(), "b#0".to_string());
  assert!(!is_self_executed);
}

#[test]
fn export_all() {
  let stmt = parse_module_item(r#"export * from 'a'"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
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
  assert!(!is_self_executed);
}

#[test]
fn export_named_from() {
  let stmt = parse_module_item(r#"export { a, b as c, default as d } from 'a';"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
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
    assert_eq!(local.to_string(), "a#0".to_string());
    assert!(exported.is_none());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(local.to_string(), "b#0".to_string());
    assert!(exported.is_some());
    assert_eq!(exported.as_ref().unwrap().to_string(), "c#0".to_string());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[2] {
    assert_eq!(local.to_string(), "default#0".to_string());
    assert!(exported.is_some());
    assert_eq!(exported.as_ref().unwrap().to_string(), "d#0".to_string());
  }

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 0);

  assert_eq!(defined_idents_map.len(), 0);
  assert!(!is_self_executed);
}

#[test]
fn export_named() {
  let stmt = parse_module_item(r#"export { a, b as c, any as d };"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
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
    assert_eq!(local.to_string(), "a#0".to_string());
    assert!(exported.is_none());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(local.to_string(), "b#0".to_string());
    assert!(exported.is_some());
    assert_eq!(exported.as_ref().unwrap().to_string(), "c#0".to_string());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[2] {
    assert_eq!(local.to_string(), "any#0".to_string());
    assert!(exported.is_some());
    assert_eq!(exported.as_ref().unwrap().to_string(), "d#0".to_string());
  }

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 3);
  let used_idents_str = used_idents
    .iter()
    .map(|ident| ident.to_string())
    .collect::<HashSet<String>>();
  assert!(used_idents_str.contains(&"a#0".to_string()));
  assert!(used_idents_str.contains(&"b#0".to_string()));
  assert!(used_idents_str.contains(&"any#0".to_string()));

  assert_eq!(defined_idents_map.len(), 3);
  assert!(!is_self_executed);
}

#[test]
fn export_named_with_used_defined_idents() {
  let stmt = parse_module_item(r#"export { a, b as c, any as d };"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, Some(["a#0".to_string()].into()));

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
    assert_eq!(local.to_string(), "a#0".to_string());
    assert!(exported.is_none());
  }

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 1);
  let used_idents_str = used_idents
    .iter()
    .map(|ident| ident.to_string())
    .collect::<HashSet<String>>();
  assert!(used_idents_str.contains(&"a#0".to_string()));

  assert_eq!(defined_idents_map.len(), 1);
  assert!(!is_self_executed);
}

#[test]
fn export_namespace() {
  let stmt = parse_module_item(r#"export * as a from 'a';"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
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
    assert_eq!(local.to_string(), "a#0".to_string());
  }

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 0);

  assert_eq!(defined_idents_map.len(), 0);
  assert!(!is_self_executed);
}

#[test]
fn func_decl() {
  let stmt = parse_module_item(r#"function a() { b(); return c; }"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 1);
  let defined_idents_str = defined_idents
    .iter()
    .map(|ident| ident.to_string())
    .collect::<HashSet<String>>();
  assert!(defined_idents_str.contains(&"a#0".to_string()));

  assert_eq!(used_idents.len(), 2);
  let used_idents_str = used_idents
    .iter()
    .map(|ident| ident.to_string())
    .collect::<HashSet<String>>();
  assert!(used_idents_str.contains(&"b#0".to_string()));
  assert!(used_idents_str.contains(&"c#0".to_string()));

  assert_eq!(defined_idents_map.len(), 1);
  let keys = defined_idents_map.keys().collect::<Vec<_>>();
  assert_eq!(keys.len(), 1);
  assert_eq!(keys[0].to_string(), "a#0".to_string());
  let values = defined_idents_map.values().collect::<Vec<_>>();
  assert_eq!(values.len(), 1);

  let values_0 = values[0].iter().map(|v| v.to_string()).collect::<Vec<_>>();
  assert_eq!(values_0.len(), 2);
  assert!(values_0.contains(&"b#0".to_string()));
  assert!(values_0.contains(&"c#0".to_string()));

  assert!(!is_self_executed);
}

#[test]
fn bar_decl() {
  let stmt = parse_module_item(r#"var a = b;"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 1);
  let defined_idents_str = defined_idents
    .iter()
    .map(|ident| ident.to_string())
    .collect::<HashSet<String>>();
  assert!(defined_idents_str.contains(&"a#0".to_string()));

  assert_eq!(used_idents.len(), 1);
  let used_idents_str = used_idents
    .iter()
    .map(|ident| ident.to_string())
    .collect::<HashSet<String>>();
  assert!(used_idents_str.contains(&"b#0".to_string()));

  assert_eq!(defined_idents_map.len(), 1);
  let keys = defined_idents_map.keys().collect::<Vec<_>>();
  assert_eq!(keys.len(), 1);
  assert_eq!(keys[0].to_string(), "a#0".to_string());
  let values = defined_idents_map.values().collect::<Vec<_>>();
  assert_eq!(values.len(), 1);
  let values_0 = values[0].iter().collect::<Vec<_>>();
  assert_eq!(values_0.len(), 1);
  assert_eq!(values_0[0].to_string(), "b#0".to_string());

  assert!(!is_self_executed);
}

#[test]
fn for_stmt() {
  let stmt = parse_module_item(r#"for (var a = b; c; d) { e; }"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 5);
  let mut used_idents = used_idents.into_iter().map(|item| item).collect::<Vec<_>>();
  used_idents.sort();
  // treat a as used for now as it does not affect the result.
  assert_eq!(used_idents[0], "a#0".to_string());
  assert_eq!(used_idents[1], "b#0".to_string());
  assert_eq!(used_idents[2], "c#0".to_string());
  assert_eq!(used_idents[3], "d#0".to_string());
  assert_eq!(used_idents[4], "e#0".to_string());

  assert_eq!(defined_idents_map.len(), 0);

  assert!(is_self_executed);
}

#[test]
fn empty_specifier_import() {
  let stmt = parse_module_item(r#"import 'index.css';"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "index.css".to_string());
  assert_eq!(import_info.specifiers.len(), 0);

  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 0);
  assert_eq!(used_idents.len(), 0);

  assert_eq!(defined_idents_map.len(), 0);

  assert!(is_self_executed);
}

#[test]
fn var_decl_pat() {
  let stmt = parse_module_item(r#"var { a, b: c, e: [f], h = i, ...g } = d;"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  let mut defined_idents = defined_idents
    .into_iter()
    .map(|item| item)
    .collect::<Vec<_>>();
  defined_idents.sort();

  assert_eq!(defined_idents.len(), 5);
  assert_eq!(defined_idents[0], "a#0".to_string());
  assert_eq!(defined_idents[1], "c#0".to_string());
  assert_eq!(defined_idents[2], "f#0".to_string());
  assert_eq!(defined_idents[3], "g#0".to_string());
  assert_eq!(defined_idents[4], "h#0".to_string());

  let mut used_idents = used_idents.into_iter().map(|item| item).collect::<Vec<_>>();
  used_idents.sort();
  assert_eq!(used_idents.len(), 2);
  assert_eq!(used_idents[0], "d#0".to_string());
  assert_eq!(used_idents[1], "i#0".to_string());

  assert_eq!(defined_idents_map.len(), 5);
  let mut keys = defined_idents_map.keys().collect::<Vec<_>>();
  keys.sort_by_key(|a| a.to_string());
  assert_eq!(keys.len(), 5);
  assert_eq!(keys[0].to_string(), "a#0".to_string());
  assert_eq!(keys[1].to_string(), "c#0".to_string());
  assert_eq!(keys[2].to_string(), "f#0".to_string());
  assert_eq!(keys[3].to_string(), "g#0".to_string());
  assert_eq!(keys[4].to_string(), "h#0".to_string());

  let values = defined_idents_map
    .values()
    .map(|idents| {
      let mut idents = idents.iter().collect::<Vec<_>>();
      idents.sort_by_key(|a| a.to_string());
      idents
    })
    .collect::<Vec<_>>();
  assert_eq!(values.len(), 5);
  assert_eq!(values[0].len(), 2);
  assert_eq!(values[0][0].to_string(), "d#0".to_string());
  assert_eq!(values[0][1].to_string(), "i#0".to_string());
  assert_eq!(values[1].len(), 2);
  assert_eq!(values[1][0].to_string(), "d#0".to_string());
  assert_eq!(values[1][1].to_string(), "i#0".to_string());
  assert_eq!(values[2].len(), 2);
  assert_eq!(values[2][0].to_string(), "d#0".to_string());
  assert_eq!(values[2][1].to_string(), "i#0".to_string());
  assert_eq!(values[3].len(), 2);
  assert_eq!(values[3][0].to_string(), "d#0".to_string());
  assert_eq!(values[3][1].to_string(), "i#0".to_string());
  assert_eq!(values[4].len(), 2);
  assert_eq!(values[4][0].to_string(), "d#0".to_string());
  assert_eq!(values[4][1].to_string(), "i#0".to_string());

  assert!(!is_self_executed);
}

#[test]
fn export_var_decl_pat() {
  let stmt = parse_module_item(r#"export const { a, b: c, e: [f], h = i, ...g } = d;"#);

  let (import_info, export_info, defined_idents, used_idents, defined_idents_map, is_self_executed) =
    analyze_imports_and_exports(&0, &stmt, None);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let mut export_info = export_info.unwrap();
  export_info.specifiers.sort_by(|a, b| {
    if let ExportSpecifierInfo::Named { local: a, .. } = a {
      if let ExportSpecifierInfo::Named { local: b, .. } = b {
        return a.to_string().cmp(&b.to_string());
      }
    }
    unreachable!();
  });
  assert_eq!(export_info.specifiers.len(), 5);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
    assert_eq!(local.to_string(), "a#0".to_string());
    assert!(exported.is_none());
  }
  assert!(matches!(
    export_info.specifiers[1],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(local.to_string(), "c#0".to_string());
    assert!(exported.is_none());
  }
  assert!(matches!(
    export_info.specifiers[2],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[2] {
    assert_eq!(local.to_string(), "f#0".to_string());
    assert!(exported.is_none());
  }
  assert!(matches!(
    export_info.specifiers[3],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[3] {
    assert_eq!(local.to_string(), "g#0".to_string());
    assert!(exported.is_none());
  }
  assert!(matches!(
    export_info.specifiers[4],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[4] {
    assert_eq!(local.to_string(), "h#0".to_string());
    assert!(exported.is_none());
  }

  let mut defined_idents = defined_idents.iter().collect::<Vec<_>>();
  defined_idents.sort_by_key(|a| a.to_string());

  assert_eq!(defined_idents.len(), 5);
  assert_eq!(defined_idents[0].to_string(), "a#0".to_string());
  assert_eq!(defined_idents[1].to_string(), "c#0".to_string());
  assert_eq!(defined_idents[2].to_string(), "f#0".to_string());
  assert_eq!(defined_idents[3].to_string(), "g#0".to_string());
  assert_eq!(defined_idents[4].to_string(), "h#0".to_string());

  let mut used_idents = used_idents.iter().collect::<Vec<_>>();
  used_idents.sort_by_key(|a| a.to_string());
  assert_eq!(used_idents.len(), 2);
  assert_eq!(used_idents[0].to_string(), "d#0".to_string());
  assert_eq!(used_idents[1].to_string(), "i#0".to_string());

  assert_eq!(defined_idents_map.len(), 5);
  let mut keys = defined_idents_map.keys().collect::<Vec<_>>();
  keys.sort_by_key(|a| a.to_string());
  assert_eq!(keys.len(), 5);
  assert_eq!(keys[0].to_string(), "a#0".to_string());
  assert_eq!(keys[1].to_string(), "c#0".to_string());
  assert_eq!(keys[2].to_string(), "f#0".to_string());
  assert_eq!(keys[3].to_string(), "g#0".to_string());
  assert_eq!(keys[4].to_string(), "h#0".to_string());

  let values = defined_idents_map
    .values()
    .map(|idents| {
      let mut idents = idents.iter().collect::<Vec<_>>();
      idents.sort_by_key(|a| a.to_string());
      idents
    })
    .collect::<Vec<_>>();
  assert_eq!(values.len(), 5);
  assert_eq!(values[0].len(), 2);
  assert_eq!(values[0][0].to_string(), "d#0".to_string());
  assert_eq!(values[0][1].to_string(), "i#0".to_string());
  assert_eq!(values[1].len(), 2);
  assert_eq!(values[1][0].to_string(), "d#0".to_string());
  assert_eq!(values[1][1].to_string(), "i#0".to_string());
  assert_eq!(values[2].len(), 2);
  assert_eq!(values[2][0].to_string(), "d#0".to_string());
  assert_eq!(values[2][1].to_string(), "i#0".to_string());
  assert_eq!(values[3].len(), 2);
  assert_eq!(values[3][0].to_string(), "d#0".to_string());
  assert_eq!(values[3][1].to_string(), "i#0".to_string());
  assert_eq!(values[4].len(), 2);
  assert_eq!(values[4][0].to_string(), "d#0".to_string());
  assert_eq!(values[4][1].to_string(), "i#0".to_string());

  assert!(!is_self_executed);
}
