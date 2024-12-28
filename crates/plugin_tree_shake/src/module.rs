use std::mem;

use farmfe_core::{
  module::{Module, ModuleId, ModuleSystem},
  swc_common::{comments::SingleThreadedComments, Mark},
  HashMap, HashSet,
};

use crate::statement_graph::{
  traced_used_import::TracedUsedImportStatement, ExportInfo, ExportSpecifierInfo, ImportInfo,
  StatementGraph, StatementId, UsedStatementIdent,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsedExportsIdent {
  /// Local ident
  SwcIdent(String),
  /// Default ident
  Default,
  /// All idents is used and may be exported from other module
  /// Marked as [UsedExportsIdent::ExportAll] when all potential exports of `export * from 'xxx'` are used.
  /// Usually works for side effect module(e.g. entry module) that re-export other modules
  ExportAll,
  /// All idents are used and may be imported from other module
  /// Marked as [UsedExportsIdent::ImportAll] when `import * as xx from 'xxx'` are used.
  ImportAll,
}

impl UsedExportsIdent {
  pub fn is_default(&self) -> bool {
    matches!(self, Self::Default)
  }

  pub fn is_ident_matched(&self, ident_name: &str) -> bool {
    matches!(self, Self::SwcIdent(i) if i == ident_name)
      || (ident_name == "default" && self.is_default()) // handle `export { a as default }`
  }

  pub fn expect_ident(&self) -> &str {
    if let Self::SwcIdent(ident) = self {
      ident.as_str()
    } else {
      unreachable!("called expect_ident of UsedExportsIdent on no SwcIdent value")
    }
  }
}

impl ToString for UsedExportsIdent {
  fn to_string(&self) -> String {
    match self {
      UsedExportsIdent::SwcIdent(ident) => ident.to_string(),
      UsedExportsIdent::Default => "default".to_string(),
      UsedExportsIdent::ExportAll => "*".to_string(),
      UsedExportsIdent::ImportAll => "import_*_as".to_string(),
    }
  }
}

#[derive(Debug, Clone)]
pub enum UsedExports {
  All,
  Partial(HashSet<UsedExportsIdent>),
}

impl Default for UsedExports {
  fn default() -> Self {
    UsedExports::Partial(Default::default())
  }
}

impl UsedExports {
  pub fn extend(&mut self, other: UsedExports) {
    match (self, other) {
      (UsedExports::All, _) => {}
      (self_value, UsedExports::All) => {
        *self_value = UsedExports::All;
      }
      (UsedExports::Partial(self_used_exports), UsedExports::Partial(other_used_exports)) => {
        self_used_exports.extend(other_used_exports);
      }
    }
  }

  pub fn as_partial(&self) -> &HashSet<UsedExportsIdent> {
    match self {
      UsedExports::All => panic!("UsedExports is not Partial"),
      UsedExports::Partial(res) => res,
    }
  }

  pub fn add_used_export(&mut self, used_export: UsedExportsIdent) {
    // All means all exports are used, only handle Partial here
    if let UsedExports::Partial(used_exports) = self {
      used_exports.insert(used_export);
    }
  }

  pub fn is_empty(&self) -> bool {
    match self {
      UsedExports::All => false,
      UsedExports::Partial(used_exports) => used_exports.is_empty(),
    }
  }

  pub fn set_export_all(&mut self) {
    match self {
      UsedExports::Partial(_) => {
        *self = UsedExports::All;
      }
      _ => {}
    }
  }

  pub fn to_string_vec(&self) -> Vec<String> {
    match self {
      Self::Partial(self_used_exports) => self_used_exports.iter().map(|i| i.to_string()).collect(),
      Self::All => {
        vec!["All".to_string()]
      }
    }
  }

  pub fn contains(&self, used_export: &UsedExportsIdent) -> bool {
    match self {
      UsedExports::All => true,
      UsedExports::Partial(used_exports) => used_exports.contains(used_export),
    }
  }
}

// TODO cache tree shake module
pub struct TreeShakeModule {
  pub module_id: ModuleId,
  pub side_effects: bool,
  pub stmt_graph: StatementGraph,
  /// true if the module or it's dependency modules contains self executed statement or has side effects
  pub contains_self_executed_stmt: bool,
  /// used exports will be analyzed when tree shaking
  /// side effects statement will be added to used exports too.
  pub handled_used_exports: UsedExports,
  /// pending used exports will be used to analyze the used exports of the module
  pub pending_used_exports: UsedExports,
  pub module_system: ModuleSystem,
}

impl TreeShakeModule {
  pub fn new(module: &mut Module) -> Self {
    farmfe_core::farm_profile_function!(format!(
      "TreeShakeModule::new {:?}",
      module.id.to_string()
    ));
    let module_system = module.meta.as_script().module_system.clone();

    // 1. generate statement graph
    let comments_meta = module.meta.as_script_mut().take_comments();
    let ast = &module.meta.as_script().ast;
    let unresolved_mark = Mark::from_u32(module.meta.as_script().unresolved_mark);
    let top_level_mark = Mark::from_u32(module.meta.as_script().top_level_mark);
    let comments = SingleThreadedComments::from(comments_meta);
    let stmt_graph = if module_system == ModuleSystem::EsModule {
      StatementGraph::new(ast, unresolved_mark, top_level_mark, &comments)
    } else {
      StatementGraph::empty()
    };

    module.meta.as_script_mut().set_comments(comments.into());

    // 2. set default used exports
    let handled_used_exports = UsedExports::Partial(Default::default());

    Self {
      module_id: module.id.clone(),
      contains_self_executed_stmt: !matches!(module_system, ModuleSystem::EsModule)
        || stmt_graph.contains_bare_import_stmt()
        || !stmt_graph.preserved_side_effects_stmts().is_empty(),
      stmt_graph,
      pending_used_exports: handled_used_exports.clone(),
      handled_used_exports,
      side_effects: module.side_effects,
      module_system,
    }
  }

  pub fn is_all_pending_used_exports_handled(&self) -> bool {
    self.pending_used_exports.is_empty()
  }

  pub fn is_used_exports_ident_handled(&self, used_export: &UsedExportsIdent) -> bool {
    match &self.handled_used_exports {
      UsedExports::All => true,
      UsedExports::Partial(used_exports) => used_exports.contains(used_export),
    }
  }

  pub fn clear_pending_used_exports(&mut self) {
    let current_pending_used_exports = mem::take(&mut self.pending_used_exports);

    match current_pending_used_exports {
      UsedExports::Partial(used_exports) => {
        for used_export in used_exports {
          self.handled_used_exports.add_used_export(used_export);
        }
      }
      UsedExports::All => {
        self.handled_used_exports = UsedExports::All;
      }
    }
  }

  pub fn imports(&self) -> Vec<&ImportInfo> {
    let mut imports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(import) = &stmt.import_info {
        imports.push(import);
      }
    }

    imports
  }

  pub fn exports(&self) -> Vec<&ExportInfo> {
    let mut exports = vec![];

    for stmt in self.stmt_graph.stmts() {
      if let Some(export) = &stmt.export_info {
        exports.push(export);
      }
    }

    exports
  }

  /// Trace the used statement starting from pending_used_exports and mark them as used.
  /// Then merge pending_used_exports into used_exports
  pub fn trace_and_mark_used_statements(&mut self) -> Vec<TracedUsedImportStatement> {
    // 1. get used exports
    let used_exports_idents = self.used_exports_to_statement_idents();
    let mut stmt_used_idents_map = HashMap::default();

    for (used_ident, stmt_id) in used_exports_idents {
      let used_idents = stmt_used_idents_map
        .entry(stmt_id)
        .or_insert(HashSet::default());
      used_idents.insert(used_ident);
    }

    // 2. trace used statements starting from used exports
    self
      .stmt_graph
      .trace_and_mark_used_statements(stmt_used_idents_map)
  }

  /// For param include_default_export: If it's false, the default export will not be included,
  /// for example, export * from 'xxx' should not include default export
  fn all_exports_to_statement_idents(
    &self,
    include_default_export: bool,
  ) -> Vec<(UsedStatementIdent, StatementId)> {
    let mut used_idents = vec![];

    for export_info in self.exports() {
      for sp in &export_info.specifiers {
        match sp {
          ExportSpecifierInfo::Default => {
            if include_default_export {
              used_idents.push((UsedStatementIdent::Default, export_info.stmt_id));
            }
          }
          ExportSpecifierInfo::Named { local, .. } => {
            used_idents.push((
              UsedStatementIdent::SwcIdent(local.clone()),
              export_info.stmt_id,
            ));
          }
          ExportSpecifierInfo::Namespace(ns) => {
            used_idents.push((
              UsedStatementIdent::SwcIdent(ns.clone()),
              export_info.stmt_id,
            ));
          }
          ExportSpecifierInfo::All => {
            used_idents.push((UsedStatementIdent::ExportAll, export_info.stmt_id));
          }
        }
      }
    }

    used_idents
  }

  pub fn used_exports_to_statement_idents(&self) -> Vec<(UsedStatementIdent, StatementId)> {
    farmfe_core::farm_profile_function!(format!(
      "used_exports_idents {:?}",
      self.module_id.to_string()
    ));

    match &self.pending_used_exports {
      UsedExports::All => {
        // all exported identifiers are used
        self.all_exports_to_statement_idents(true)
      }
      UsedExports::Partial(idents) => {
        let mut used_idents = vec![];
        // statement `import * as xxx from './xxx'` is marked as used, and the usage can not be statically determined, e.g. xxx[expr].
        // so we need to mark all exported idents as used the same as UsedExports::All
        if idents.contains(&UsedExportsIdent::ImportAll) {
          // all export information needs to be collected
          return self.all_exports_to_statement_idents(true);
        }
        // statement `export * from './xxx'` is marked as used, we need to mark all exported idents as used the same as UsedExports::All
        // except the default export when idents does not contain default export
        if idents.contains(&UsedExportsIdent::ExportAll) {
          return self.all_exports_to_statement_idents(idents.contains(&UsedExportsIdent::Default));
        }
        // find exported ident for every used idents.
        for ident in idents {
          let mut export_all_stmt_ids: Option<Vec<usize>> = None;
          // find the export info that contains the ident
          let export_info = self.exports().into_iter().find(|export_info| {
            export_info.specifiers.iter().any(|sp| match sp {
              ExportSpecifierInfo::Default => ident.is_default(),
              ExportSpecifierInfo::Named { local, exported } => {
                let exported_ident = if let Some(exported) = exported {
                  exported
                } else {
                  local
                };

                ident.is_ident_matched(exported_ident.0.as_str())
              }
              ExportSpecifierInfo::Namespace(ns) => ident.is_ident_matched(ns.0.as_str()),
              ExportSpecifierInfo::All => {
                /* Deal with All later */
                if let Some(export_all_stmt_id) = &mut export_all_stmt_ids {
                  export_all_stmt_id.push(export_info.stmt_id);
                } else {
                  export_all_stmt_ids = Some(vec![export_info.stmt_id]);
                }
                false
              }
            })
          });

          if let Some(export_info) = export_info {
            for sp in &export_info.specifiers {
              match sp {
                ExportSpecifierInfo::Default => {
                  if ident.is_default() {
                    used_idents.push((UsedStatementIdent::Default, export_info.stmt_id));
                  }
                }
                ExportSpecifierInfo::Named { local, exported } => {
                  if let Some(exported) = exported {
                    if ident.is_ident_matched(exported.0.as_str()) {
                      used_idents.push((
                        UsedStatementIdent::SwcIdent(local.clone()),
                        export_info.stmt_id,
                      ));
                    }
                  } else if ident.is_ident_matched(local.0.as_str()) {
                    used_idents.push((
                      UsedStatementIdent::SwcIdent(local.clone()),
                      export_info.stmt_id,
                    ));
                  }
                }
                ExportSpecifierInfo::Namespace(ns) => {
                  if ident.is_ident_matched(ns.0.as_str()) {
                    used_idents.push((
                      UsedStatementIdent::SwcIdent(ns.clone()),
                      export_info.stmt_id,
                    ));
                  }
                }
                ExportSpecifierInfo::All => {
                  unreachable!()
                }
              }
            }
          } else {
            // if export info is not found, and there are ExportSpecifierInfo::All, then the ident may be exported by `export * from 'xxx'`
            if let Some(export_all_stmt_id) = export_all_stmt_ids {
              let ident = ident.expect_ident().to_string();
              // skip default for export * from 'xxx'
              if ident != *"default" {
                for export_all_stmt_id in export_all_stmt_id {
                  used_idents.push((
                    UsedStatementIdent::InExportAll(ident.clone()),
                    export_all_stmt_id,
                  ));
                }
              }
            }
          }
        }

        used_idents
      }
    }
  }
}
