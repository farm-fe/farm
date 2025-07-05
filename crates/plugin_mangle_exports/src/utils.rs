use farmfe_core::{module::meta_data::script::ModuleReExportIdentType, HashMap};

pub fn is_reexport_all(
  reexport_ident_map: &HashMap<String, ModuleReExportIdentType>,
  export: &String,
) -> bool {
  reexport_ident_map
    .get(export)
    .map(|reexport_ident_type| {
      matches!(reexport_ident_type, ModuleReExportIdentType::FromExportAll)
    })
    .unwrap_or(false)
}

pub fn get_reexport_named_local(
  reexport_ident_map: &HashMap<String, ModuleReExportIdentType>,
  export: &String,
) -> Option<String> {
  reexport_ident_map
    .get(export)
    .map(|reexport_ident_type| match reexport_ident_type {
      ModuleReExportIdentType::FromExportAll => None,
      ModuleReExportIdentType::FromExportNamed { local } => Some(local.clone()),
    })
    .unwrap_or(None)
}
