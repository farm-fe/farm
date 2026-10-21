use std::{collections::VecDeque, sync::Arc};

use farmfe_core::{
  HashMap, HashSet,
  context::CompilationContext,
  module::{
    ModuleId,
    meta_data::script::{
      AMBIGUOUS_EXPORT_ALL, EXPORT_DEFAULT, ModuleExportIdent, ModuleExportIdentType,
      ModuleReExportIdentType, ScriptModuleMetaData,
      statement::{ExportSpecifierInfo, ImportSpecifierInfo, SwcId},
    },
    module_graph::ModuleGraph,
  },
  plugin::ResolveKind,
};

use farmfe_toolkit::script::{
  concatenate_modules::EXPORT_NAMESPACE, create_ambiguous_export_all_ident,
  create_export_default_ident, create_export_namespace_ident, swc_try_with::try_with,
};

/// expand the export_ident_map of each module of the module graph
pub fn expand_exports_of_module_graph(
  module_graph: &mut ModuleGraph,
  context: &Arc<CompilationContext>,
) {
  let mut expand_context = ExpandModuleExportsContext::new();

  let mut modules = module_graph.modules();
  modules.sort_by_key(|module| module.execution_order);
  // modules.reverse(); // traverse the module from top to bottom

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

  complete_cyclic_exports(module_graph, &mut expand_context, Some(context));

  // update the exports of module
  for module in module_graph.modules_mut() {
    if !module.module_type.is_script() {
      continue;
    }

    if let Some(export_ident_map) = expand_context.remove_export_ident_map(&module.id) {
      let script_meta = module.meta.as_script_mut();
      script_meta.export_ident_map = export_ident_map;
    }

    if let Some(ambiguous_export_ident_map) =
      expand_context.remove_ambiguous_export_ident_map(&module.id)
    {
      let script_meta = module.meta.as_script_mut();
      script_meta.ambiguous_export_ident_map = ambiguous_export_ident_map;
    }

    if let Some(reexport_ident_map) = expand_context.reexport_ident_map.remove(&module.id) {
      let script_meta = module.meta.as_script_mut();
      script_meta.reexport_ident_map = reexport_ident_map;
    }
  }
}

pub fn expand_dynamic_entry_exports(module_graph: &mut ModuleGraph) {
  let mut expand_context = ExpandModuleExportsContext::new();

  for dynamic_entry_id in module_graph.dynamic_entries.keys() {
    expand_module_exports_dfs(dynamic_entry_id, module_graph, &mut expand_context);
  }

  complete_cyclic_exports(module_graph, &mut expand_context, None);

  for (module_id, export_ident_map) in expand_context.export_ident_map {
    let module = module_graph.module_mut(&module_id).unwrap();
    let script_meta = module.meta.as_script_mut();
    script_meta.export_ident_map = export_ident_map;
  }

  for (module_id, ambiguous_export_ident_map) in expand_context.ambiguous_export_ident_map {
    let module = module_graph.module_mut(&module_id).unwrap();
    let script_meta = module.meta.as_script_mut();
    script_meta.ambiguous_export_ident_map = ambiguous_export_ident_map;
  }

  for (module_id, reexport_ident_map) in expand_context.reexport_ident_map {
    let module = module_graph.module_mut(&module_id).unwrap();
    let script_meta = module.meta.as_script_mut();
    script_meta.reexport_ident_map = reexport_ident_map;
  }
}

pub(crate) fn get_basic_module_export_ident(
  module_id: &ModuleId,
  module_script_meta: &ScriptModuleMetaData,
  should_contains_export_from: bool,
) -> Vec<(String, ModuleExportIdent)> {
  let mut module_export_idents = vec![];

  for statement in &module_script_meta.statements {
    if let Some(export_info) = &statement.export_info {
      for specifier in &export_info.specifiers {
        match specifier {
          // export default
          ExportSpecifierInfo::Default => {
            // if defined idents is empty, it's export default expression, a new export default ident will be added
            if statement.defined_idents.is_empty() {
              module_export_idents.push((
                EXPORT_DEFAULT.to_string(),
                ModuleExportIdent::new(
                  module_id.clone(),
                  create_export_default_ident(module_id).to_id().into(),
                  ModuleExportIdentType::Declaration,
                ),
              ));
            } else {
              // there will only be one defined ident in export default statement
              for defined_ident in &statement.defined_idents {
                module_export_idents.push((
                  EXPORT_DEFAULT.to_string(),
                  ModuleExportIdent::new(
                    module_id.clone(),
                    defined_ident.clone(),
                    ModuleExportIdentType::Declaration,
                  ),
                ));
              }
            }
          }
          // export { foo, bar as baz }
          ExportSpecifierInfo::Named { local, exported } => {
            let export_str = if let Some(exported) = exported {
              exported.sym.to_string()
            } else {
              local.sym.to_string()
            };

            if should_contains_export_from || export_info.source.is_none() {
              module_export_idents.push((
                export_str,
                ModuleExportIdent::new(
                  module_id.clone(),
                  local.clone(),
                  ModuleExportIdentType::Declaration,
                ),
              ));
            }
          }
          _ => {}
        }
      }
    }
  }

  module_export_idents
}

/// A back edge can read a barrel before its star exports have been expanded.
/// Replay those readers, then propagate actual metadata changes to dependents.
/// Acyclic graphs retain the single DFS pass.
fn complete_cyclic_exports(
  module_graph: &ModuleGraph,
  expand_context: &mut ExpandModuleExportsContext,
  context: Option<&Arc<CompilationContext>>,
) {
  let mut pending = std::mem::take(&mut expand_context.incomplete_readers);
  if pending.is_empty() {
    return;
  }
  let mut dependents: HashMap<ModuleId, Vec<ModuleId>> = HashMap::default();
  for module in module_graph.modules() {
    if module.module_type.is_script() && !module.external {
      for (dependency, _) in module_graph.dependencies(&module.id) {
        dependents
          .entry(dependency)
          .or_default()
          .push(module.id.clone());
      }
    }
  }
  let mut queue: VecDeque<_> = pending.iter().cloned().collect();
  while let Some(module_id) = queue.pop_front() {
    pending.remove(&module_id);
    let previous_exports = expand_context.get_export_ident_map(&module_id);
    let previous_ambiguous = expand_context.get_ambiguous_export_ident_map(&module_id);
    expand_context.visited.remove(&module_id);
    if let Some(context) = context {
      let cm = context.meta.get_module_source_map(&module_id);
      let globals = context.meta.get_globals(&module_id);
      try_with(cm, globals.value(), || {
        expand_module_exports_dfs(&module_id, module_graph, expand_context);
      })
      .unwrap();
    } else {
      expand_module_exports_dfs(&module_id, module_graph, expand_context);
    }
    if previous_exports != expand_context.get_export_ident_map(&module_id)
      || previous_ambiguous != expand_context.get_ambiguous_export_ident_map(&module_id)
    {
      for dependent in dependents.get(&module_id).into_iter().flatten() {
        if pending.insert(dependent.clone()) {
          queue.push_back(dependent.clone());
        }
      }
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
  if module.external || !module.module_type.is_script() {
    return;
  }
  if expand_context.is_visited(module_id) {
    if expand_context.active.contains(module_id) {
      expand_context
        .incomplete_readers
        .extend(expand_context.active.iter().cloned());
    }
    return;
  }

  expand_context.mark_visited(module_id);
  expand_context.active.insert(module_id.clone());

  let module_script_meta = module.meta.as_script();

  // find basic export in current module
  for (export_str, module_export_ident) in
    get_basic_module_export_ident(module_id, module_script_meta, false)
  {
    expand_context.insert_export_ident(module_id, export_str, module_export_ident.clone());
  }

  let mut export_all_stmts = vec![];

  // find export from / import recursively
  for statement in &module_script_meta.statements {
    if let Some(export_info) = &statement.export_info {
      for specifier in &export_info.specifiers {
        match specifier {
          // export * from
          ExportSpecifierInfo::All => {
            export_all_stmts.push(export_info);
          }
          ExportSpecifierInfo::Namespace(swc_id) => {
            // add the exported ident first
            expand_context.insert_export_ident(
              module_id,
              swc_id.sym.to_string(),
              ModuleExportIdent::new(
                module_id.clone(),
                swc_id.clone(),
                ModuleExportIdentType::Declaration,
              ),
            );

            let source = export_info.source.as_ref().unwrap();
            let source_module_id =
              module_graph.get_dep_by_source(module_id, source, Some(ResolveKind::ExportFrom));

            expand_module_exports_dfs(&source_module_id, module_graph, expand_context);

            // add a special export ident for namespace export * as ns in the source module
            expand_context.insert_export_namespace_ident(&source_module_id, module_graph);
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

              expand_context
                .reexport_ident_map
                .entry(module.id.clone())
                .or_default()
                .insert(
                  export_str.clone(),
                  ModuleReExportIdentType::FromExportNamed {
                    local: local.sym.clone(),
                    from_module_id: source_module_id.clone(),
                  },
                );

              let source_module = module_graph.module(&source_module_id).unwrap();

              expand_module_exports_dfs(&source_module_id, module_graph, expand_context);

              if let Some(module_export_ident) =
                expand_context.get_export_ident(&source_module_id, &local.sym)
              {
                expand_context.insert_export_ident(
                  module_id,
                  export_str,
                  module_export_ident.clone(),
                );
              } else if source_module.external {
                expand_context.insert_export_ident(
                  module_id,
                  export_str,
                  ModuleExportIdent::new(
                    source_module_id,
                    local.clone(),
                    ModuleExportIdentType::External,
                  ),
                );
              } else {
                expand_unresolved_import_dfs(
                  &local.sym,
                  local,
                  &source_module_id,
                  false,
                  true,
                  module_graph,
                  expand_context,
                  &mut HashSet::default(),
                );

                if let Some(module_export_ident) =
                  expand_context.get_export_ident(&source_module_id, &local.sym)
                {
                  expand_context.insert_export_ident(
                    module_id,
                    export_str,
                    module_export_ident.clone(),
                  );
                } else {
                  expand_context.insert_export_ident(
                    module_id,
                    export_str,
                    ModuleExportIdent::new(
                      source_module_id,
                      local.clone(),
                      ModuleExportIdentType::Unresolved,
                    ),
                  );
                }
              }
            }
          }
          _ => {}
        }
      }
    }
  }

  // export * should be handled after named export
  for export_info in export_all_stmts {
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

        if let Some(existing_ident) = expand_context.get_export_ident(module_id, &export_str) {
          let existing_ident = existing_ident.as_internal();
          let module_export_ident = module_export_ident.as_internal();

          match (
            &existing_ident.export_type,
            &module_export_ident.export_type,
          ) {
            (ModuleExportIdentType::Declaration, _) => {
              continue;
            }
            (_, ModuleExportIdentType::Declaration) => {
              // override the ambiguous ident with declaration ident
            }
            _ => {
              println!(
                "[Farm warn] export {} is already declared in {}. The declaration in {} will be ignored",
                export_str,
                existing_ident.module_id.to_string(),
                module_export_ident.module_id.to_string()
              );
              continue;
            }
          }
        }

        let reexport_map = expand_context
          .reexport_ident_map
          .entry(module_id.clone())
          .or_default();

        if !reexport_map.contains_key(export_str.as_str()) {
          reexport_map.insert(
            export_str.clone(),
            ModuleReExportIdentType::FromExportAll(source_module_id.clone()),
          );
        }

        expand_context.insert_export_ident(module_id, export_str, module_export_ident.clone());
      }
    }

    // extend ambiguous export ident map
    if let Some(ambiguous_export_ident_map) =
      expand_context.get_ambiguous_export_ident_map(&source_module_id)
    {
      for (export_str, export_idents) in ambiguous_export_ident_map {
        let existing_map = expand_context
          .ambiguous_export_ident_map
          .entry(module_id.clone())
          .or_default();
        let existing_idents = existing_map.entry(export_str).or_default();

        for export_ident in export_idents {
          // do not insert duplicate ident
          if !existing_idents.contains(&export_ident) {
            existing_idents.push(export_ident);
          }
        }
      }
    }

    let source_module = module_graph.module(&source_module_id).unwrap();

    // if the module is not a es module, mark the export * from as ambiguous
    if source_module.external
      || !source_module.module_type.is_script()
      || !source_module.meta.as_script().module_system.is_es_module()
    {
      let export_ident = ModuleExportIdent::new(
        source_module_id.clone(),
        create_ambiguous_export_all_ident(&source_module_id)
          .to_id()
          .into(),
        ModuleExportIdentType::AmbiguousExportAll,
      );
      expand_context.insert_ambiguous_export_ident(
        &source_module_id,
        AMBIGUOUS_EXPORT_ALL.to_string(),
        export_ident.clone(),
      );
      expand_context.insert_ambiguous_export_ident(
        module_id,
        AMBIGUOUS_EXPORT_ALL.to_string(),
        export_ident,
      );
    }
  }

  // import should be handled after all export item are handled
  for statement in &module_script_meta.statements {
    if let Some(import_info) = &statement.import_info {
      let source_module_id =
        module_graph.get_dep_by_source(module_id, &import_info.source, Some(ResolveKind::Import));

      expand_module_exports_dfs(&source_module_id, module_graph, expand_context);

      if import_info
        .specifiers
        .iter()
        .any(|specifier| matches!(specifier, ImportSpecifierInfo::Namespace(_)))
      {
        expand_context.insert_export_namespace_ident(&source_module_id, module_graph);
      }

      // find unresolved ident recursively
      for specifier in &import_info.specifiers {
        match specifier {
          ImportSpecifierInfo::Namespace(_) => { /* ignore namespace as it's handled above */ }
          ImportSpecifierInfo::Named { local, imported } => {
            let ident = if let Some(imported) = imported {
              imported
            } else {
              local
            };

            expand_unresolved_import_dfs(
              &ident.sym,
              ident,
              &source_module_id,
              false,
              false,
              module_graph,
              expand_context,
              &mut HashSet::default(),
            );
          }
          ImportSpecifierInfo::Default(_) => {
            if expand_context
              .get_export_ident(&source_module_id, EXPORT_DEFAULT)
              .is_none()
            {
              expand_context.insert_export_ident(
                &source_module_id,
                EXPORT_DEFAULT.to_string(),
                ModuleExportIdent::new(
                  source_module_id.clone(),
                  create_export_default_ident(&source_module_id)
                    .to_id()
                    .into(),
                  ModuleExportIdentType::Unresolved,
                ),
              );
            }
          }
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
  expand_context.active.remove(module_id);
}

#[allow(clippy::too_many_arguments)]
fn expand_unresolved_import_dfs(
  imported_str: &str,
  ident: &SwcId,
  source_module_id: &ModuleId,
  from_export_all: bool,
  from_export_named: bool,
  module_graph: &ModuleGraph,
  expand_context: &mut ExpandModuleExportsContext,
  visited: &mut HashSet<ModuleId>,
) {
  if visited.contains(source_module_id) {
    return;
  }

  visited.insert(source_module_id.clone());

  if expand_context
    .get_export_ident(source_module_id, imported_str)
    .map(|export_ident| {
      // we have to find the deepest export ident, when tracing export all, we should not find unresolved ident
      !from_export_all
        || !matches!(
          export_ident.as_internal().export_type,
          ModuleExportIdentType::Unresolved | ModuleExportIdentType::External
        )
    })
    .unwrap_or(false)
  {
    return;
  }

  let source_module = module_graph.module(source_module_id).unwrap();

  if source_module.external
    || !source_module.module_type.is_script()
    || source_module
      .meta
      .as_script()
      .module_system
      .contains_commonjs()
  {
    expand_context.insert_export_ident(
      source_module_id,
      imported_str.to_string(),
      ModuleExportIdent::new(
        source_module_id.clone(),
        ident.clone(),
        if source_module.external && from_export_all {
          ModuleExportIdentType::ExternalExportAll
        } else if source_module.external {
          ModuleExportIdentType::External
        } else if from_export_all {
          ModuleExportIdentType::UnresolvedExportAll
        } else {
          ModuleExportIdentType::Unresolved
        },
      ),
    );
    return;
  }

  let source_module_script_meta = source_module.meta.as_script();
  let mut found_ambiguous_ident = false;

  for statement in &source_module_script_meta.statements {
    if let Some(export_info) = &statement.export_info {
      for specifier in &export_info.specifiers {
        if let ExportSpecifierInfo::All = specifier {
          let source = export_info.source.as_ref().unwrap();
          let new_source_module_id =
            module_graph.get_dep_by_source(source_module_id, source, Some(ResolveKind::ExportFrom));

          expand_unresolved_import_dfs(
            imported_str,
            ident,
            &new_source_module_id,
            true,
            false,
            module_graph,
            expand_context,
            visited,
          );

          if let Some(export_ident) =
            expand_context.get_export_ident(&new_source_module_id, imported_str)
          {
            expand_context.insert_ambiguous_export_ident(
              source_module_id,
              imported_str.to_string(),
              export_ident.clone(),
            );
            found_ambiguous_ident = true;

            if !expand_context.contains_export_ident(source_module_id, imported_str) {
              expand_context.insert_export_ident(
                source_module_id,
                imported_str.to_string(),
                export_ident,
              );
            }
          }

          if let Some(ambiguous_idents) =
            expand_context.get_ambiguous_export_idents(&new_source_module_id, imported_str)
          {
            for ambiguous_ident in ambiguous_idents {
              expand_context.insert_ambiguous_export_ident(
                source_module_id,
                imported_str.to_string(),
                ambiguous_ident,
              );
            }
            found_ambiguous_ident = true;
          }
        }
      }
    }
  }

  if !from_export_named
    && !from_export_all
    && !found_ambiguous_ident
    && !module_graph.circle_record.is_in_circle(source_module_id)
  {
    println!(
      "[Farm warn] Can not resolve ambiguous export ident {ident:?}({imported_str:?}) from {source_module_id:?}. Please make sure the export exists. You can file a issue at https://github.com/farm-fe/farm/issues if you need help."
    );
  }
}

struct ExpandModuleExportsContext {
  export_ident_map: HashMap<ModuleId, HashMap<String, ModuleExportIdent>>,
  reexport_ident_map: HashMap<ModuleId, HashMap<String, ModuleReExportIdentType>>,
  ambiguous_export_ident_map: HashMap<ModuleId, HashMap<String, Vec<ModuleExportIdent>>>,
  visited: HashSet<ModuleId>,
  active: HashSet<ModuleId>,
  incomplete_readers: HashSet<ModuleId>,
}

impl ExpandModuleExportsContext {
  pub fn new() -> Self {
    Self {
      export_ident_map: HashMap::default(),
      reexport_ident_map: HashMap::default(),
      ambiguous_export_ident_map: HashMap::default(),
      visited: HashSet::default(),
      active: HashSet::default(),
      incomplete_readers: HashSet::default(),
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
    export_ident: ModuleExportIdent,
  ) {
    self
      .export_ident_map
      .entry(module_id.clone())
      .or_default()
      .insert(export_str, export_ident);
  }

  pub fn contains_export_ident(&self, module_id: &ModuleId, export_str: &str) -> bool {
    self
      .export_ident_map
      .get(module_id)
      .and_then(|export_ident_map| export_ident_map.get(export_str))
      .is_some()
  }

  pub fn insert_ambiguous_export_ident(
    &mut self,
    module_id: &ModuleId,
    export_str: String,
    export_ident: ModuleExportIdent,
  ) {
    let idents = self
      .ambiguous_export_ident_map
      .entry(module_id.clone())
      .or_default()
      .entry(export_str)
      .or_default();
    if !idents.contains(&export_ident) {
      idents.push(export_ident);
    }
  }

  fn insert_export_namespace_ident(
    &mut self,
    source_module_id: &ModuleId,
    module_graph: &ModuleGraph,
  ) {
    let source_module = module_graph.module(source_module_id).unwrap();

    if source_module.external {
      return;
    }

    if !source_module.module_type.is_script() {
      return;
    }

    self.insert_export_ident(
      source_module_id,
      EXPORT_NAMESPACE.to_string(),
      ModuleExportIdent::new(
        source_module_id.clone(),
        create_export_namespace_ident(source_module_id)
          .to_id()
          .into(),
        ModuleExportIdentType::VirtualNamespace,
      ),
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

  pub fn remove_ambiguous_export_ident_map(
    &mut self,
    module_id: &ModuleId,
  ) -> Option<HashMap<String, Vec<ModuleExportIdent>>> {
    self.ambiguous_export_ident_map.remove(module_id)
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

  pub fn get_ambiguous_export_ident_map(
    &self,
    module_id: &ModuleId,
  ) -> Option<HashMap<String, Vec<ModuleExportIdent>>> {
    self.ambiguous_export_ident_map.get(module_id).cloned()
  }

  pub fn get_ambiguous_export_idents(
    &self,
    module_id: &ModuleId,
    export_str: &str,
  ) -> Option<Vec<ModuleExportIdent>> {
    self
      .ambiguous_export_ident_map
      .get(module_id)
      .and_then(|ambiguous_export_ident_map| ambiguous_export_ident_map.get(export_str).cloned())
  }
}

#[cfg(test)]
mod cyclic_star_tests {
  use super::*;
  use farmfe_core::{
    config::{Config, persistent_cache::PersistentCacheConfig},
    module::{
      Module, ModuleMetaData, ModuleSystem, ModuleType,
      meta_data::script::statement::{ExportInfo, ImportInfo, Statement},
      module_graph::ModuleGraphEdgeDataItem,
    },
    swc_common::{Globals, SourceMap},
  };

  #[test]
  fn cyclic_import_during_star_expansion_preserves_all_entry_exports() {
    for order in [
      ["api.js", "leaf.js", "entry.js"],
      ["entry.js", "api.js", "leaf.js"],
      ["leaf.js", "entry.js", "api.js"],
    ] {
      for dynamic in [false, true] {
        for named in [false, true] {
          check_cyclic_exports(order, dynamic, named);
        }
      }
    }
  }

  fn check_cyclic_exports(order: [&str; 3], dynamic: bool, named: bool) {
    let mut config = Config::default();
    *config.persistent_cache = PersistentCacheConfig::Bool(false);
    let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
    let mut graph = ModuleGraph::new();
    // api -> leaf -> entry -> api is one cycle. Starting from the barrel is
    // a valid traversal order; entry must not cache its partially expanded API.
    for (order, name) in order.into_iter().enumerate() {
      let mut module = Module::new(name.into());
      module.module_type = ModuleType::Js;
      module.execution_order = order;
      *module.meta = ModuleMetaData::Script(Box::default());
      module.meta.as_script_mut().module_system = ModuleSystem::EsModule;
      context
        .meta
        .set_module_source_map(&module.id, Arc::new(SourceMap::default()));
      context.meta.set_globals(&module.id, Globals::new());
      graph.add_module(module);
    }
    for (module_name, exported) in [
      ("entry.js", "run"),
      ("leaf.js", "Model"),
      ("leaf.js", "default"),
    ] {
      let ident = SwcId::new(exported.into(), 0);
      graph
        .module_mut(&module_name.into())
        .unwrap()
        .meta
        .as_script_mut()
        .statements
        .push(Statement::new(
          0,
          Some(ExportInfo {
            source: None,
            specifiers: vec![ExportSpecifierInfo::Named {
              local: ident.clone(),
              exported: None,
            }],
            stmt_id: 0,
          }),
          None,
          HashSet::from_iter([ident]),
          false,
        ));
    }
    for (parent, child) in [("entry.js", "api.js"), ("api.js", "leaf.js")] {
      let source = format!("./{child}");
      let meta = graph
        .module_mut(&parent.into())
        .unwrap()
        .meta
        .as_script_mut();
      let id = meta.statements.len();
      meta.statements.push(Statement::new(
        id,
        Some(ExportInfo {
          source: Some(source.clone()),
          specifiers: if named && parent == "entry.js" {
            vec![ExportSpecifierInfo::Named {
              local: SwcId::new("Model".into(), 0),
              exported: Some(SwcId::new("SelectedModel".into(), 0)),
            }]
          } else {
            vec![ExportSpecifierInfo::All]
          },
          stmt_id: id,
        }),
        None,
        HashSet::default(),
        false,
      ));
      graph
        .add_edge_item(
          &parent.into(),
          &child.into(),
          ModuleGraphEdgeDataItem {
            source,
            kind: ResolveKind::ExportFrom,
            order: 0,
          },
        )
        .unwrap();
    }
    graph
      .module_mut(&"leaf.js".into())
      .unwrap()
      .meta
      .as_script_mut()
      .statements
      .push(Statement::new(
        1,
        None,
        Some(ImportInfo {
          source: "./entry.js".into(),
          specifiers: vec![ImportSpecifierInfo::Named {
            local: SwcId::new("run".into(), 0),
            imported: None,
          }],
          stmt_id: 1,
        }),
        HashSet::default(),
        false,
      ));
    graph
      .add_edge_item(
        &"leaf.js".into(),
        &"entry.js".into(),
        ModuleGraphEdgeDataItem {
          source: "./entry.js".into(),
          kind: ResolveKind::Import,
          order: 0,
        },
      )
      .unwrap();

    if dynamic {
      graph.dynamic_entries.insert("api.js".into(), "api".into());
      expand_dynamic_entry_exports(&mut graph);
    } else {
      expand_exports_of_module_graph(&mut graph, &context);
    }
    let entry = graph.module(&"entry.js".into()).unwrap().meta.as_script();
    let name = if named { "SelectedModel" } else { "Model" };
    assert!(
      entry.export_ident_map.contains_key(name),
      "entry must retain Model through completed cyclic barrel"
    );
    assert!(!entry.export_ident_map.contains_key(EXPORT_DEFAULT));
    if named {
      assert!(matches!(entry.reexport_ident_map.get(name),
        Some(ModuleReExportIdentType::FromExportNamed { local, from_module_id })
          if local.as_ref() == "Model" && from_module_id == &ModuleId::from("api.js")));
    } else {
      assert!(matches!(entry.reexport_ident_map.get(name),
        Some(ModuleReExportIdentType::FromExportAll(source)) if source == &ModuleId::from("api.js")));
    }
    let model = entry.export_ident_map.get(name).unwrap().as_internal();
    assert_eq!(model.module_id, ModuleId::from("leaf.js"));
    assert_eq!(model.ident.sym.as_ref(), "Model");
    drop(model);
    let before = entry.export_ident_map.clone();
    if dynamic {
      expand_dynamic_entry_exports(&mut graph);
    } else {
      expand_exports_of_module_graph(&mut graph, &context);
    }
    assert_eq!(
      before,
      graph
        .module(&"entry.js".into())
        .unwrap()
        .meta
        .as_script()
        .export_ident_map
    );
  }
}
