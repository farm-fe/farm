use std::sync::Arc;

use farmfe_core::{
  context::CompilationContext,
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, ImportSpecifierInfo, SwcId},
      ModuleExportIdent, ModuleExportIdentType, EXPORT_DEFAULT, EXPORT_EXTERNAL_NAMESPACE,
    },
    module_graph::ModuleGraph,
    ModuleId,
  },
  plugin::ResolveKind,
  swc_common::Mark,
  HashMap, HashSet,
};

use farmfe_toolkit::script::{
  concatenate_modules::EXPORT_NAMESPACE, create_export_default_ident,
  create_export_external_namespace_ident, create_export_namespace_ident, swc_try_with::try_with,
};

/// expand the export_ident_map of each module of the module graph
pub fn expand_exports_of_module_graph(
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) {
  let mut expand_context = ExpandModuleExportsContext::new();

  let mut modules = module_graph.modules();
  modules.sort_by_key(|module| module.execution_order);

  for module in modules {
    if module.module_type.is_script() {
      let cm = context.meta.get_module_source_map(&module.id);
      let globals = context.meta.get_globals(&module.id);

      try_with(cm, globals.value(), || {
        expand_module_exports_dfs(&module.id, module_graph, &mut expand_context);
      })
      .unwrap();
    }
  }

  // update the exports of module
  for module in module_graph.modules_mut() {
    if let Some(export_ident_map) = expand_context.remove_export_ident_map(&module.id) {
      if !module.module_type.is_script() {
        continue;
      }

      let script_meta = module.meta.as_script_mut();
      script_meta.export_ident_map = export_ident_map;
    }
  }
}

fn expand_module_exports_dfs(
  module_id: &ModuleId,
  module_graph: &ModuleGraph,
  expand_context: &mut ExpandModuleExportsContext,
) {
  let module = module_graph.module(module_id).unwrap();
  // skip
  if module.external || !module.module_type.is_script() || expand_context.is_visited(module_id) {
    return;
  }

  expand_context.mark_visited(module_id);

  let module_script_meta = module.meta.as_script();

  // find export by esm import/export in current module
  for statement in &module_script_meta.statements {
    if let Some(export_info) = &statement.export_info {
      for specifier in &export_info.specifiers {
        match specifier {
          // export default
          ExportSpecifierInfo::Default => {
            // if defined idents is empty, it's export default expression, a new export default ident will be added
            if statement.defined_idents.is_empty() {
              let top_level_mark = Mark::from_u32(module_script_meta.top_level_mark);
              expand_context.insert_export_ident(
                module_id,
                EXPORT_DEFAULT.to_string(),
                module_id.clone(),
                create_export_default_ident(module_id, top_level_mark)
                  .to_id()
                  .into(),
                ModuleExportIdentType::Declaration,
              );
            } else {
              // there will only be one defined ident in export default statement
              for defined_ident in &statement.defined_idents {
                expand_context.insert_export_ident(
                  module_id,
                  EXPORT_DEFAULT.to_string(),
                  module_id.clone(),
                  defined_ident.clone(),
                  ModuleExportIdentType::Declaration,
                );
              }
            }
          }
          // export { foo, bar as baz } or export { foo as bar } from './bar'
          ExportSpecifierInfo::Named { local, exported } => {
            let export_str = if let Some(exported) = exported {
              exported.sym.to_string()
            } else {
              local.sym.to_string()
            };

            if export_info.source.is_none() {
              expand_context.insert_export_ident(
                module_id,
                export_str,
                module_id.clone(),
                local.clone(),
                ModuleExportIdentType::Declaration,
              );
            }
          }
          _ => {}
        }
      }
    }
  }

  // find export from / import recursively
  for statement in &module_script_meta.statements {
    if let Some(export_info) = &statement.export_info {
      for specifier in &export_info.specifiers {
        match specifier {
          // export * from
          ExportSpecifierInfo::All => {
            let source = export_info.source.as_ref().unwrap();
            let source_module_id =
              module_graph.get_dep_by_source(module_id, source, Some(ResolveKind::ExportFrom));

            expand_module_exports_dfs(&source_module_id, module_graph, expand_context);

            // extend the export ident map with the source module export ident map
            if let Some(source_module_export_ident_map) =
              expand_context.get_export_ident_map(&source_module_id)
            {
              for (export_str, module_export_ident) in source_module_export_ident_map {
                // skip default for export *
                if export_str == EXPORT_DEFAULT || export_str == EXPORT_NAMESPACE {
                  continue;
                }

                expand_context.insert_export_ident(
                  module_id,
                  export_str,
                  module_export_ident.module_id,
                  module_export_ident.ident,
                  module_export_ident.export_type,
                );
              }
            }

            let source_module = module_graph.module(&source_module_id).unwrap();

            if source_module.external {
              expand_context.insert_export_ident(
                module_id,
                EXPORT_EXTERNAL_NAMESPACE.to_string(),
                source_module_id.clone(),
                create_export_external_namespace_ident(&source_module_id)
                  .to_id()
                  .into(),
                ModuleExportIdentType::ExternalNamespace,
              );
            }
          }
          ExportSpecifierInfo::Namespace(swc_id) => {
            // add the exported ident first
            expand_context.insert_export_ident(
              module_id,
              swc_id.sym.to_string(),
              module_id.clone(),
              swc_id.clone(),
              ModuleExportIdentType::Declaration,
            );

            let source = export_info.source.as_ref().unwrap();
            let source_module_id =
              module_graph.get_dep_by_source(module_id, source, Some(ResolveKind::ExportFrom));

            expand_module_exports_dfs(&source_module_id, module_graph, expand_context);

            let source_module = module_graph.module(&source_module_id).unwrap();

            if source_module.external {
              expand_context.insert_export_ident(
                module_id,
                EXPORT_EXTERNAL_NAMESPACE.to_string(),
                source_module_id.clone(),
                create_export_external_namespace_ident(&source_module_id)
                  .to_id()
                  .into(),
                ModuleExportIdentType::ExternalNamespace,
              );
            } else {
              // add a special export ident for namespace export * as ns in the source module
              expand_context.insert_export_namespace_ident(&source_module_id, module_graph);
            }
          }
          ExportSpecifierInfo::Named { local, exported } => {
            // export { foo, bar as baz } from './bar'
            if let Some(source) = &export_info.source {
              let export_str = if let Some(exported) = exported {
                exported.sym.to_string()
              } else {
                local.sym.to_string()
              };

              let source_module_id =
                module_graph.get_dep_by_source(module_id, source, Some(ResolveKind::ExportFrom));
              let source_module = module_graph.module(&source_module_id).unwrap();

              expand_module_exports_dfs(&source_module_id, module_graph, expand_context);

              if let Some(module_export_ident) =
                expand_context.get_export_ident(&source_module_id, &local.sym)
              {
                expand_context.insert_export_ident(
                  module_id,
                  export_str,
                  module_export_ident.module_id,
                  module_export_ident.ident,
                  module_export_ident.export_type,
                );
              } else if source_module.external {
                expand_context.insert_export_ident(
                  module_id,
                  export_str,
                  source_module_id,
                  local.clone(),
                  ModuleExportIdentType::External,
                );
              } else {
                // TODO: warning
                println!(
                  "[Farm Warning] export {} of module {} not found",
                  local.sym,
                  source_module_id.to_string()
                );
                expand_context.insert_export_ident(
                  module_id,
                  export_str,
                  source_module_id,
                  local.clone(),
                  ModuleExportIdentType::Unresolved,
                );
              }
            }
          }
          _ => {}
        }
      }
    } else if let Some(import_info) = &statement.import_info {
      let source_module_id =
        module_graph.get_dep_by_source(module_id, &import_info.source, Some(ResolveKind::Import));

      expand_module_exports_dfs(&source_module_id, module_graph, expand_context);

      if import_info
        .specifiers
        .iter()
        .any(|specifier| matches!(specifier, ImportSpecifierInfo::Namespace(_)))
      {
        let source_module = module_graph.module(&source_module_id).unwrap();

        if source_module.external {
          expand_context.insert_export_ident(
            module_id,
            EXPORT_EXTERNAL_NAMESPACE.to_string(),
            source_module_id.clone(),
            create_export_external_namespace_ident(&source_module_id)
              .to_id()
              .into(),
            ModuleExportIdentType::ExternalNamespace,
          );
        } else {
          expand_context.insert_export_namespace_ident(&source_module_id, module_graph);
        }
      }
    }
  }

  // find dynamic imported dependencies and insert a namespace export ident for them
  for (source_module_id, edge) in module_graph.dependencies(module_id) {
    if edge.contains_dynamic_import() {
      // expand exports first
      expand_module_exports_dfs(&source_module_id, module_graph, expand_context);

      expand_context.insert_export_namespace_ident(&source_module_id, module_graph);
    }
  }
}

struct ExpandModuleExportsContext {
  export_ident_map: HashMap<ModuleId, HashMap<String, ModuleExportIdent>>,
  visited: HashSet<ModuleId>,
}

impl ExpandModuleExportsContext {
  pub fn new() -> Self {
    Self {
      export_ident_map: HashMap::default(),
      visited: HashSet::default(),
    }
  }

  pub fn is_visited(&self, module_id: &ModuleId) -> bool {
    self.visited.contains(module_id)
  }

  pub fn mark_visited(&mut self, module_id: &ModuleId) {
    self.visited.insert(module_id.clone());
  }

  pub fn insert_export_ident(
    &mut self,
    module_id: &ModuleId,
    export_str: String,
    export_module_id: ModuleId,
    export_ident: SwcId,
    export_type: ModuleExportIdentType,
  ) {
    self
      .export_ident_map
      .entry(module_id.clone())
      .or_default()
      .insert(
        export_str,
        ModuleExportIdent {
          module_id: export_module_id,
          ident: export_ident,
          export_type,
        },
      );
  }

  fn insert_export_namespace_ident(
    &mut self,
    source_module_id: &ModuleId,
    module_graph: &ModuleGraph,
  ) {
    let source_module = module_graph.module(&source_module_id).unwrap();

    if source_module.external {
      return;
    }

    if !source_module.module_type.is_script() {
      return;
    }

    let source_module_script_meta = source_module.meta.as_script();
    let top_level_mark = Mark::from_u32(source_module_script_meta.top_level_mark);
    self.insert_export_ident(
      &source_module_id,
      EXPORT_NAMESPACE.to_string(),
      source_module_id.clone(),
      create_export_namespace_ident(&source_module_id, top_level_mark)
        .to_id()
        .into(),
      ModuleExportIdentType::VirtualNamespace,
    );
  }

  pub fn get_export_ident_map(
    &self,
    module_id: &ModuleId,
  ) -> Option<HashMap<String, ModuleExportIdent>> {
    self.export_ident_map.get(module_id).cloned()
  }

  pub fn remove_export_ident_map(
    &mut self,
    module_id: &ModuleId,
  ) -> Option<HashMap<String, ModuleExportIdent>> {
    self.export_ident_map.remove(module_id)
  }

  pub fn get_export_ident(
    &self,
    module_id: &ModuleId,
    export_str: &str,
  ) -> Option<ModuleExportIdent> {
    self
      .export_ident_map
      .get(module_id)
      .and_then(|export_ident_map| export_ident_map.get(export_str).cloned())
  }
}
