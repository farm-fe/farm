use farmfe_core::module::ModuleId;

pub fn get_module_ids_from_compilation_errors(
  errors: &[farmfe_core::error::CompilationError],
) -> Vec<ModuleId> {
  errors
    .iter()
    .filter_map(|e| match e {
      farmfe_core::error::CompilationError::ResolveError { importer, .. } => {
        Some(importer.as_str().into())
      }
      farmfe_core::error::CompilationError::LoadError { resolved_path, .. } => {
        Some(resolved_path.as_str().into())
      }
      farmfe_core::error::CompilationError::TransformError { resolved_path, .. } => {
        Some(resolved_path.as_str().into())
      }
      farmfe_core::error::CompilationError::ParseError { resolved_path, .. } => {
        Some(resolved_path.as_str().into())
      }
      _ => None,
    })
    .collect()
}
