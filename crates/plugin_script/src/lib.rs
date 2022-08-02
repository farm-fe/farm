use farmfe_core::plugin::Plugin;

/// ScriptPlugin is used to support compiling js/ts/jsx/tsx files to js chunks
pub struct FarmScriptPlugin {}

impl Plugin for FarmScriptPlugin {
  fn name(&self) -> &str {
    "FarmScriptPlugin"
  }
}
