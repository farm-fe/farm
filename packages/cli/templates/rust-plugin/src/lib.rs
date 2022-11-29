use farmfe_core::{plugin::{Plugin}}

use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct <FARM-RUST-PLUGIN-NAME-STRUCT> {};

impl <FARM-RUST-PLUGIN-NAME-STRUCT> {
  pub fn name() -> String {
    "<FARM-RUST-PLUGIN-NAME-STRUCT>".to_string()
  }
}