use std::sync::Arc;

use farmfe_core::{config::FARM_MODULE_SYSTEM, context::CompilationContext};
use farmfe_toolkit::html::get_farm_global_this;

const PLUGIN_VAR_PREFIX: &str = "__farm_plugin__";

pub fn insert_runtime_plugins(content: String, context: &Arc<CompilationContext>) -> String {
  let plugins = context
    .config
    .runtime
    .plugins
    .iter()
    .enumerate()
    .map(|(i, plugin_path)| {
      let ident = format!("{}{}", PLUGIN_VAR_PREFIX, i);
      let import_stmt = format!(
        "import {} from '{}';",
        ident,
        if cfg!(windows) {
          plugin_path.replace('\\', "\\\\")
        } else {
          plugin_path.to_string()
        }
      );
      (ident, import_stmt)
    })
    .collect::<Vec<_>>();
  let idents = plugins
    .iter()
    .map(|(ident, _)| ident.as_str())
    .collect::<Vec<_>>();
  let imports = plugins
    .iter()
    .map(|(_, import)| import.as_str())
    .collect::<Vec<_>>();

  let farm_global_this = get_farm_global_this(
    &context.config.runtime.namespace,
    &context.config.output.target_env,
  );
  // FARM_GLOBAL_THIS.FARM_MODULE_SYSTEM.setPlugins([PLUGIN_VAR_PREFIX0, PLUGIN_VAR_PREFIX1, ...])
  let plugins_call = format!(
    "{}.{}.setPlugins([{}]);",
    farm_global_this,
    FARM_MODULE_SYSTEM,
    idents.join(", ")
  );

  format!("{}\n{}\n{}", imports.join("\n"), content, plugins_call,)
}
