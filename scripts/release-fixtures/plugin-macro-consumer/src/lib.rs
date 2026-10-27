extern crate self as farmfe_core;

pub const VERSION: &str = "0.5.4";
pub mod config {
    pub struct Config;
}
pub mod plugin {
    pub trait Plugin: Send + Sync {}
}

#[farmfe_macro_plugin::farm_plugin]
pub struct Probe;
impl Probe {
    pub fn new(_: &config::Config, _: String) -> Self {
        Self
    }
}
impl plugin::Plugin for Probe {}

#[test]
fn plugin_macro_contract() {
    assert_eq!(_core_version(), VERSION);
    let _: std::sync::Arc<dyn plugin::Plugin> = _plugin_create(&config::Config, String::new());
}
