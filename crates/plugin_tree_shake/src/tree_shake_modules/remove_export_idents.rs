use farmfe_core::{
  module::{
    meta_data::script::{
      statement::{ExportSpecifierInfo, SwcId},
      ModuleExportIdent, ModuleExportIdentType, ModuleReExportIdentType, ScriptModuleMetaData,
    },
    module_graph::ModuleGraph,
    ModuleId,
  },
  plugin::ResolveKind,
  HashMap, HashSet,
};
use farmfe_toolkit::script::create_export_default_ident;

#[cfg(test)]
mod tests {
  use super::*;
  use farmfe_core::module::{
    meta_data::script::statement::{ExportInfo, Statement},
    module_graph::ModuleGraphEdgeDataItem,
    Module, ModuleMetaData, ModuleType,
  };

  #[test]
  fn removed_intermediate_export_is_pruned_through_all_parent_barrels() {
    let mut graph = ModuleGraph::new();
    let binding = ModuleExportIdent::new(
      "model.js".into(),
      SwcId::new("model_js_default".into(), 0),
      ModuleExportIdentType::Declaration,
    );
    for name in ["root.js", "core.js", "api.js", "model.js"] {
      let mut module = Module::new(name.into());
      module.module_type = ModuleType::Js;
      *module.meta = ModuleMetaData::Script(Default::default());
      let meta = module.meta.as_script_mut();
      meta.export_ident_map.insert(
        if name == "model.js" {
          "default"
        } else {
          "Model"
        }
        .into(),
        binding.clone(),
      );
      if name == "model.js" {
        meta.statements.push(Statement::new(
          0,
          Some(ExportInfo {
            source: None,
            specifiers: vec![ExportSpecifierInfo::Default],
            stmt_id: 0,
          }),
          None,
          HashSet::default(),
          false,
        ));
      }
      graph.add_module(module);
    }
    for (parent, child) in [("root.js", "core.js"), ("core.js", "api.js")] {
      let source = format!("./{child}");
      let meta = graph
        .module_mut(&parent.into())
        .unwrap()
        .meta
        .as_script_mut();
      meta.reexport_ident_map.insert(
        "Model".into(),
        ModuleReExportIdentType::FromExportAll(child.into()),
      );
      meta.statements.push(Statement::new(
        0,
        Some(ExportInfo {
          source: Some(source.clone()),
          specifiers: vec![ExportSpecifierInfo::All],
          stmt_id: 0,
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
    // The declaration survives for another consumer, while api.js's re-export
    // statement has been removed. Parent metadata must still be pruned.
    graph
      .module_mut(&"api.js".into())
      .unwrap()
      .meta
      .as_script_mut()
      .reexport_ident_map
      .insert(
        "Model".into(),
        ModuleReExportIdentType::FromExportNamed {
          from_module_id: "model.js".into(),
          local: "default".into(),
        },
      );
    remove_export_idents(&mut graph);
    for name in ["root.js", "core.js", "api.js"] {
      let meta = graph.module(&name.into()).unwrap().meta.as_script();
      assert!(!meta.export_ident_map.contains_key("Model"), "{name}");
      assert!(!meta.reexport_ident_map.contains_key("Model"), "{name}");
    }
    assert!(graph
      .module(&"model.js".into())
      .unwrap()
      .meta
      .as_script()
      .export_ident_map
      .contains_key("default"));
  }

  #[test]
  fn star_export_does_not_retain_binding_removed_from_source_barrel() {
    let mut graph = ModuleGraph::new();
    let mut source = Module::new("api.js".into());
    source.module_type = ModuleType::Js;
    *source.meta = ModuleMetaData::Script(Default::default());
    graph.add_module(source);
    let mut entry = Module::new("core.js".into());
    entry.module_type = ModuleType::Js;
    *entry.meta = ModuleMetaData::Script(Default::default());
    entry.meta.as_script_mut().statements.push(Statement::new(
      0,
      Some(ExportInfo {
        source: Some("./api.js".into()),
        specifiers: vec![ExportSpecifierInfo::All],
        stmt_id: 0,
      }),
      None,
      HashSet::default(),
      false,
    ));
    graph.add_module(entry);
    graph
      .add_edge_item(
        &"core.js".into(),
        &"api.js".into(),
        ModuleGraphEdgeDataItem {
          source: "./api.js".into(),
          kind: ResolveKind::ExportFrom,
          order: 0,
        },
      )
      .unwrap();
    assert!(!is_module_contains_reexport_ident(
      &"core.js".into(),
      graph.module(&"core.js".into()).unwrap().meta.as_script(),
      "Model",
      &ModuleReExportIdentType::FromExportAll("api.js".into()),
      &graph
    ));
  }
}

fn is_module_contains_export_ident(
  meta: &ScriptModuleMetaData,
  export_ident: &ModuleExportIdent,
) -> bool {
  let export_ident = export_ident.as_internal();

  if !matches!(export_ident.export_type, ModuleExportIdentType::Declaration) {
    return true;
  }

  meta.statements.iter().any(|stmt| {
    if let Some(export_info) = &stmt.export_info {
      for sp in &export_info.specifiers {
        let is_ident_used = match sp {
          ExportSpecifierInfo::Default => {
            if stmt.defined_idents.contains(&export_ident.ident) {
              return true;
            }

            let default_ident: SwcId = create_export_default_ident(&export_ident.module_id)
              .to_id()
              .into();

            default_ident == export_ident.ident
          }
          ExportSpecifierInfo::All => false, // it's defined in other modules
          ExportSpecifierInfo::Named { local, .. } => {
            // is source is not none, means the ident is defined in other modules, we should always return false
            export_info.source.is_none() && local == &export_ident.ident
          }
          ExportSpecifierInfo::Namespace(swc_id) => swc_id == &export_ident.ident,
        };

        if is_ident_used {
          return true;
        }
      }
    }

    false
  })
}

fn is_module_contains_reexport_ident(
  module_id: &ModuleId,
  meta: &ScriptModuleMetaData,
  export_str: &str,
  reexport_ident_type: &ModuleReExportIdentType,
  module_graph: &ModuleGraph,
) -> bool {
  let (source_id, source_export) = match reexport_ident_type {
    ModuleReExportIdentType::FromExportAll(id) => (id, export_str),
    ModuleReExportIdentType::FromExportNamed {
      from_module_id,
      local,
    } => (from_module_id, local.as_str()),
  };
  if let Some(source) = module_graph.module(source_id) {
    if source.module_type.is_script()
      && !source.external
      && !source
        .meta
        .as_script()
        .export_ident_map
        .contains_key(source_export)
    {
      return false;
    }
  }
  meta.statements.iter().any(|stmt| {
    if let Some(export_info) = &stmt.export_info {
      if let Some(source) = export_info.source.as_ref() {
        let source_module_id =
          module_graph.get_dep_by_source(module_id, source, Some(ResolveKind::ExportFrom));

        for sp in &export_info.specifiers {
          let is_reexport_used = match sp {
            ExportSpecifierInfo::Named { local, exported } => {
              let local_exported_str = exported.as_ref().unwrap_or(local).sym.as_str();

              if let ModuleReExportIdentType::FromExportNamed {
                from_module_id,
                local: local_local,
              } = reexport_ident_type
              {
                source_module_id == *from_module_id
                  && local_exported_str == export_str
                  && *local_local == local.sym
              } else {
                false
              }
            }
            ExportSpecifierInfo::All => {
              if let ModuleReExportIdentType::FromExportAll(from_module_id) = reexport_ident_type {
                source_module_id == *from_module_id
              } else {
                false
              }
            }
            ExportSpecifierInfo::Default | ExportSpecifierInfo::Namespace(_) => {
              // default export and namespace export don't reexport idents
              false
            }
          };

          if is_reexport_used {
            return true;
          }
        }
      }
    }

    false
  })
}

pub fn remove_export_idents(module_graph: &mut ModuleGraph) {
  // Removing a binding from one barrel can invalidate its parents. Iterate
  // until no metadata remains that points to an export removed downstream.
  loop {
    let mut module_ident_to_remove = HashMap::<ModuleId, HashSet<String>>::default();

    for module in module_graph.modules() {
      if module.module_type.is_script() {
        let meta = module.meta.as_script();

        for (export_str, export_ident) in &meta.export_ident_map {
          if let Some(defined_module) = module_graph.module(&export_ident.as_internal().module_id) {
            if !defined_module.module_type.is_script() || defined_module.external {
              continue;
            }

            let defined_module_meta = defined_module.meta.as_script();

            if !is_module_contains_export_ident(defined_module_meta, export_ident) {
              // remove export ident cause the module is being removed
              module_ident_to_remove
                .entry(module.id.clone())
                .or_default()
                .insert(export_str.clone());
            }
          } else {
            // remove export ident cause the module is being removed
            module_ident_to_remove
              .entry(module.id.clone())
              .or_default()
              .insert(export_str.clone());
          }
        }

        // for reexport ident, makes sure the reexport ident is not removed
        for (export_str, reexport_ident) in &meta.reexport_ident_map {
          if !is_module_contains_reexport_ident(
            &module.id,
            meta,
            export_str,
            reexport_ident,
            module_graph,
          ) {
            module_ident_to_remove
              .entry(module.id.clone())
              .or_default()
              .insert(export_str.clone());
          }
        }
      }
    }

    if module_ident_to_remove.is_empty() {
      break;
    }
    for (module_id, idents) in module_ident_to_remove {
      let module = module_graph.module_mut(&module_id).unwrap();
      let meta = module.meta.as_script_mut();

      for ident in idents {
        meta.export_ident_map.remove(&ident);
        meta.reexport_ident_map.remove(&ident);
        meta.ambiguous_export_ident_map.remove(&ident);
      }
    }
  }
}
