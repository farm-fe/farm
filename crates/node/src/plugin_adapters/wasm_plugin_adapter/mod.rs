use farmfe_core::plugin::Plugin;

pub struct WasmPluginAdapter {
  name: String,
}

impl WasmPluginAdapter {
  pub fn new(name: String) -> Self {
    Self { name }
  }
}

impl Plugin for WasmPluginAdapter {
  fn name(&self) -> String {
    self.name.clone()
  }
}
