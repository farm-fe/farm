use std::sync::Arc;

use farmfe_core::{
  module::{Module, ModuleMetaData, ScriptModuleMetaData},
  swc_common::{FilePathMapping, SourceMap},
  swc_ecma_ast::{EsVersion, Module as SwcModule},
  swc_ecma_parser::Syntax,
};

use crate::{module::UsedIdent, statement_graph::ExportSpecifierInfo};

use super::{TreeShakeModule, UsedExports};

fn parse_module(code: &str) -> SwcModule {
  let swc_module = farmfe_toolkit::script::parse_module(
    "any",
    code,
    Syntax::Es(Default::default()),
    EsVersion::EsNext,
    Arc::new(SourceMap::new(FilePathMapping::empty())),
  )
  .unwrap();

  swc_module
}

fn create_module(code: &str) -> Module {
  let mut module = Module::new("used_exports_idents_test".into());
  module.meta = ModuleMetaData::Script(ScriptModuleMetaData {
    ast: parse_module(code),
    top_level_mark: 0,
    unresolved_mark: 0,
    module_system: farmfe_core::module::ModuleSystem::EsModule,
    hmr_accepted: false,
  });
  module
}

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
  let module = create_module(code);

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
  assert_eq!(export_info.specifiers.len(), 2);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Named { .. }
  ));
  assert!(matches!(
    export_info.specifiers[1],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
    assert!(exported.is_none());
    assert_eq!(local.sym.to_string(), "a".to_string())
  }
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(exported.as_ref().unwrap().sym.to_string(), "d".to_string());
    assert_eq!(local.sym.to_string(), "c".to_string());
  }

  let stmt = tree_shake_module.stmt_graph.stmt(&4);
  assert!(stmt.export_info.is_some());
  let export_info = stmt.export_info.as_ref().unwrap();
  assert_eq!(export_info.specifiers.len(), 2);
  assert!(matches!(
    export_info.specifiers[0],
    ExportSpecifierInfo::Named { .. }
  ));
  assert!(matches!(
    export_info.specifiers[1],
    ExportSpecifierInfo::Named { .. }
  ));
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[0] {
    assert!(exported.is_none());
    assert_eq!(local.sym.to_string(), "e".to_string())
  }
  if let ExportSpecifierInfo::Named { local, exported } = &export_info.specifiers[1] {
    assert_eq!(exported.as_ref().unwrap().sym.to_string(), "f".to_string());
    assert_eq!(local.sym.to_string(), "default".to_string());
  }
}

#[test]
fn used_exports_idents_export_all() {
  let code = "export const a = 1; export * from './foo'";
  let module = create_module(code);

  let mut tree_shake_module = TreeShakeModule::new(&module);
  tree_shake_module.used_exports = UsedExports::Partial(vec!["a".to_string()]);
  let result = tree_shake_module.used_exports_idents();
  assert_eq!(result.len(), 1);
  assert!(matches!(result[0].0, UsedIdent::SwcIdent(_)));
  assert_eq!(result[0].0.to_string(), "a".to_string());

  let code = r#"
export * from './foo';
export const b = 2;"#;
  let module = create_module(code);

  let mut tree_shake_module = TreeShakeModule::new(&module);
  tree_shake_module.used_exports = UsedExports::Partial(vec!["a".to_string()]);
  let result = tree_shake_module.used_exports_idents();
  assert_eq!(result.len(), 1);
  assert!(matches!(result[0].0, UsedIdent::InExportAll(_)));
  assert_eq!(result[0].0.to_string(), "a".to_string());
}
