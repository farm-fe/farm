use crate::default::{default_camel2_dash, default_lib_dir, default_style_library_path};
use serde::Deserialize;
use std::fmt::Debug;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Options {
  pub library_name: Option<String>, // library name, default ""
  #[serde(default = "default_lib_dir")]
  pub lib_dir: Option<String>, // lib directory, default lib
  #[serde(default = "default_camel2_dash")]
  pub camel2_dash: Option<bool>, // whether parse name to dash mode or not, default true
  #[serde(default = "default_lib_dir")]
  pub style_lib_dir: Option<String>, // style lib directory, default lib
  pub style_library_name: Option<String>, // the style dir. e.g. custon-theme =>  custon-theme/index.css
  #[serde(default = "default_style_library_path")]
  pub style_library_path: Option<String>, // custom style path, default "index.css"
}
