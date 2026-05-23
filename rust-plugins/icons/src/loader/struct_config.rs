use super::svg_builder::SvgCustomizations;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct IconifyJSON {
  pub icons: HashMap<String, IconifyIcon>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct IconifyIcon {
  pub left: Option<i32>,
  pub top: Option<i32>,
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub body: String,
  pub h_flip: Option<bool>,
  pub v_flip: Option<bool>,
  pub rotate: Option<i32>,
}

#[derive(Deserialize, Default)]
pub struct IconifyLoaderOptions {
  pub customizations: Option<SvgCustomizations>,
  pub scale: Option<f32>,
}
