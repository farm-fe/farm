use farmfe_core::context::CompilationContext;
use farmfe_plugin_resolve::resolver::DEFAULT_MAIN_FIELDS;

pub fn with_initial_main_fields(mut compilation: CompilationContext) -> CompilationContext {
  compilation.config.resolve.main_fields = Some(DEFAULT_MAIN_FIELDS.to_vec());
  compilation
}
