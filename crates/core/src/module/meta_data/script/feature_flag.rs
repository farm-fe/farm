use farmfe_macro_cache_item::cache_item;

/// Features that used in a script module
#[cache_item]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[archive_attr(derive(Hash, Eq, PartialEq))]
pub enum FeatureFlag {
  /// import('xxx')
  DynamicImport,
  /// use `await xxx` in top level scope
  TopLevelAwait,
  /// import xxx from 'xxx'
  DefaultImport,
  /// import * as xxx from 'xxx'
  ImportNamespace,
  /// export * from 'xxx'
  ExportAll,
  /// export * as xxx from './xxx'
  ExportNamespace,
  /// any import/export statement
  ModuleDecl,
}
