use std::{collections::HashSet, sync::Arc};

use farmfe_core::{
  swc_ecma_ast::{Id, ModuleItem},
  swc_ecma_parser::Syntax,
  HashSet,
};
use farmfe_toolkit::script::{parse_module, ParseScriptModuleResult};

use crate::statement_graph::{
  analyze_statement_info::AnalyzedStatementInfo, ExportSpecifierInfo, ImportSpecifierInfo,
};

use super::analyze_statement_info;

fn parse_module_item(stmt: &str) -> ModuleItem {
  let ParseScriptModuleResult { ast: module, .. } = parse_module(
    &"any".into(),
    Arc::new(stmt.to_string()),
    Syntax::Es(Default::default()),
    farmfe_core::swc_ecma_ast::EsVersion::Es2015,
    None,
  )
  .unwrap();
  module.body[0].clone()
}

fn print_id(id: &Id) -> String {
  format!("{}{:?}", id.0, id.1)
}

#[test]
fn import_default() {
  let stmt = parse_module_item(r#"import a from 'a'"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Default(_)
  ));

  if let ImportSpecifierInfo::Default(ident) = &import_info.specifiers[0] {
    assert_eq!(print_id(ident), "a#0".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(
    defined_idents
      .into_iter()
      .map(|i| print_id(&i))
      .collect::<Vec<_>>(),
    vec!["a#0".to_string()]
  );
}

#[test]
fn import_named() {
  let stmt = parse_module_item(r#"import { a, b, c as nc } from 'a'"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[0] {
    assert_eq!(print_id(local), "a#0".to_string());
    assert!(imported.is_none());
  }

  assert!(matches!(
    import_info.specifiers[1],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[1] {
    assert_eq!(print_id(local), "b#0".to_string());
    assert!(imported.is_none());
  }

  assert!(matches!(
    import_info.specifiers[2],
    ImportSpecifierInfo::Named { .. }
  ));

  if let ImportSpecifierInfo::Named { local, imported } = &import_info.specifiers[2] {
    assert_eq!(print_id(local), "nc#0".to_string());
    assert!(imported.is_some());
    assert_eq!(print_id(imported.as_ref().unwrap()), "c#0".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 3);
  let defined_idents_str = defined_idents
    .into_iter()
    .map(|ident| print_id(&ident))
    .collect::<HashSet<_>>();
  assert!(defined_idents_str.contains(&"a#0".to_string()));
  assert!(defined_idents_str.contains(&"b#0".to_string()));
  assert!(defined_idents_str.contains(&"nc#0".to_string()));
}

#[test]
fn import_namespace() {
  let stmt = parse_module_item(r#"import * as a from 'a'"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "a".to_string());
  assert!(matches!(
    import_info.specifiers[0],
    ImportSpecifierInfo::Namespace(_)
  ));

  if let ImportSpecifierInfo::Namespace(ident) = &import_info.specifiers[0] {
    assert_eq!(print_id(ident), "a#0".to_string());
  }

  assert!(export_info.is_none());
  assert_eq!(defined_idents.len(), 1);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| print_id(ident))
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );
}

#[test]
fn export_default_expr() {
  let stmt = parse_module_item(r#"export default a"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Default
  ));

  assert_eq!(defined_idents.len(), 0);
}

#[test]
fn export_default_decl() {
  let stmt = parse_module_item(r#"export default function a() { return b; }"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

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
      .map(|ident| print_id(ident))
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );
}

#[test]
fn export_decl() {
  let stmt = parse_module_item(r#"export function a() { return b; }"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Named { .. }
  ));

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
    assert_eq!(print_id(&local), "a#0".to_string());
    assert!(exported.is_none());
  }

  assert_eq!(defined_idents.len(), 1);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| print_id(ident))
      .collect::<Vec<_>>()[0],
    "a#0".to_string()
  );
}

#[test]
fn export_all() {
  let stmt = parse_module_item(r#"export * from 'a'"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let export_info = export_info.unwrap();

  assert_eq!(export_info.specifiers.len(), 1);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::All
  ));
  assert_eq!(export_info.source, Some("a".to_string()));

  assert_eq!(defined_idents.len(), 0);
}

#[test]
fn export_named_from() {
  let stmt = parse_module_item(r#"export { a, b as c, default as d } from 'a';"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

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
    assert_eq!(print_id(local), "a#0".to_string());
    assert!(exported.is_none());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(print_id(local), "b#0".to_string());
    assert!(exported.is_some());
    assert_eq!(print_id(exported.as_ref().unwrap()), "c#0".to_string());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[2] {
    assert_eq!(print_id(local), "default#0".to_string());
    assert!(exported.is_some());

    assert_eq!(print_id(exported.as_ref().unwrap()), "d#0".to_string());
  }

  assert_eq!(defined_idents.len(), 3);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| print_id(ident))
      .collect::<HashSet<_>>(),
    vec!["a#0".to_string(), "c#0".to_string(), "d#0".to_string()]
      .into_iter()
      .collect::<HashSet<_>>()
  );
}

#[test]
fn export_named() {
  let stmt = parse_module_item(r#"export { a, b as c, any as d };"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

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
    // assert_eq!(print_id(local), "a#0".to_string());
    assert_eq!(print_id(local), "a#0".to_string());
    assert!(exported.is_none());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    // assert_eq!(print_id(local), "b#0".to_string());
    assert_eq!(print_id(local), "b#0".to_string());
    assert!(exported.is_some());
    // assert_eq!(exported.as_ref().unwrap().to_string(), "c#0".to_string());
    assert_eq!(print_id(exported.as_ref().unwrap()), "c#0".to_string());
  }

  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[2] {
    // assert_eq!(print_id(local), "any#0".to_string());
    assert_eq!(print_id(local), "any#0".to_string());
    assert!(exported.is_some());
    // assert_eq!(exported.as_ref().unwrap().to_string(), "d#0".to_string());
    assert_eq!(print_id(exported.as_ref().unwrap()), "d#0".to_string());
  }

  assert_eq!(defined_idents.len(), 3);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| print_id(ident))
      .collect::<HashSet<_>>(),
    vec!["a#0".to_string(), "c#0".to_string(), "d#0".to_string()]
      .into_iter()
      .collect::<HashSet<_>>()
  );
}

#[test]
fn export_namespace() {
  let stmt = parse_module_item(r#"export * as a from 'a';"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

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
    // assert_eq!(print_id(local), "a#0".to_string());
    assert_eq!(print_id(local), "a#0".to_string());
  }

  assert_eq!(defined_idents.len(), 1);
  assert_eq!(
    defined_idents
      .iter()
      .map(|ident| print_id(ident))
      .collect::<HashSet<_>>(),
    vec!["a#0".to_string()].into_iter().collect::<HashSet<_>>()
  );
}

#[test]
fn func_decl() {
  let stmt = parse_module_item(r#"function a() { b(); return c; }"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 1);
  let defined_idents_str = defined_idents
    .iter()
    .map(|ident| print_id(ident))
    .collect::<HashSet<String>>();
  assert!(defined_idents_str.contains(&"a#0".to_string()));

  assert_eq!(defined_idents_str.len(), 1);
  assert_eq!(
    defined_idents_str.into_iter().collect::<Vec<_>>()[0],
    "a#0".to_string()
  );
}

#[test]
fn bar_decl() {
  let stmt = parse_module_item(r#"var a = b;"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 1);
  let defined_idents_str = defined_idents
    .iter()
    .map(|ident| print_id(ident))
    .collect::<HashSet<String>>();
  assert!(defined_idents_str.contains(&"a#0".to_string()));
}

#[test]
fn for_stmt() {
  let stmt = parse_module_item(r#"for (var a = b; c; d) { e; }"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 0);
}

#[test]
fn empty_specifier_import() {
  let stmt = parse_module_item(r#"import 'index.css';"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_some());
  let import_info = import_info.unwrap();
  assert_eq!(import_info.source, "index.css".to_string());
  assert_eq!(import_info.specifiers.len(), 0);

  assert!(export_info.is_none());

  assert_eq!(defined_idents.len(), 0);
}

#[test]
fn var_decl_pat() {
  let stmt = parse_module_item(r#"var { a, b: c, e: [f], h = i, ...g } = d;"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_none());
  assert!(export_info.is_none());

  let mut defined_idents = defined_idents
    .into_iter()
    .map(|item| print_id(&item))
    .collect::<Vec<_>>();
  defined_idents.sort();

  assert_eq!(defined_idents.len(), 5);
  assert_eq!(defined_idents[0], "a#0".to_string());
  assert_eq!(defined_idents[1], "c#0".to_string());
  assert_eq!(defined_idents[2], "f#0".to_string());
  assert_eq!(defined_idents[3], "g#0".to_string());
  assert_eq!(defined_idents[4], "h#0".to_string());
}

#[test]
fn export_var_decl_pat() {
  let stmt = parse_module_item(r#"export const { a, b: c, e: [f], h = i, ...g } = d;"#);

  let AnalyzedStatementInfo {
    import_info,
    export_info,
    defined_idents,
  } = analyze_statement_info(&0, &stmt);

  assert!(import_info.is_none());
  assert!(export_info.is_some());
  let mut export_info = export_info.unwrap();
  export_info.specifiers.sort_by(|a, b| {
    if let ExportSpecifierInfo::Named { local: a, .. } = a {
      if let ExportSpecifierInfo::Named { local: b, .. } = b {
        return print_id(a).cmp(&print_id(b));
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
    assert_eq!(print_id(local), "a#0".to_string());
    assert!(exported.is_none());
  }
  assert!(matches!(
    export_info.specifiers[1],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(print_id(local), "c#0".to_string());
    assert!(exported.is_none());
  }
  assert!(matches!(
    export_info.specifiers[2],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[2] {
    assert_eq!(print_id(local), "f#0".to_string());
    assert!(exported.is_none());
  }
  assert!(matches!(
    export_info.specifiers[3],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[3] {
    assert_eq!(print_id(local), "g#0".to_string());
    assert!(exported.is_none());
  }
  assert!(matches!(
    export_info.specifiers[4],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[4] {
    assert_eq!(print_id(local), "h#0".to_string());
    assert!(exported.is_none());
  }

  let mut defined_idents = defined_idents.iter().collect::<Vec<_>>();
  defined_idents.sort_by_key(|a| print_id(a));

  assert_eq!(defined_idents.len(), 5);
  assert_eq!(print_id(defined_idents[0]), "a#0".to_string());
  assert_eq!(print_id(defined_idents[1]), "c#0".to_string());
  assert_eq!(print_id(defined_idents[2]), "f#0".to_string());
  assert_eq!(print_id(defined_idents[3]), "g#0".to_string());
  assert_eq!(print_id(defined_idents[4]), "h#0".to_string());
}
