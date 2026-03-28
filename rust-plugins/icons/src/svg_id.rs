use farmfe_core::regex::{self, Regex};
use farmfe_toolkit::lazy_static::lazy_static;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;

fn rand_id() -> String {
  let mut rng = rand::thread_rng();
  (0..10)
    .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
    .collect()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SvgResult {
  pub has_id: bool,
  pub svg: String,
  pub inject_scripts: String,
}

lazy_static! {
  static ref RE1: Regex = Regex::new(r#"\b([\w-]+?)="url\(#(.+?)\)""#).unwrap();
  static ref RE2: Regex = Regex::new(r#"\bid="(.+?)""#).unwrap();
}

pub fn handle_svg_id(svg: &str) -> SvgResult {
  let mut svg = svg.to_string();
  let has_id = RE1.is_match(&svg);
  let mut id_map: HashMap<String, String> = HashMap::new();
  let mut inject_scripts = String::new();

  if has_id {
    svg = RE1
      .replace_all(&svg, |caps: &regex::Captures| {
        let attribute = &caps[1];
        let original_id = &caps[2];
        let new_id = format!("uicons-{}", rand_id());
        id_map.insert(original_id.to_string(), new_id.clone());
        format!(r#":{}="'url(#'+idMap['{}']+')'""#, attribute, original_id)
      })
      .to_string();

    svg = RE2
      .replace_all(&svg, |caps: &regex::Captures| {
        let original_id = &caps[1];
        if let Some(_new_id) = id_map.get(original_id) {
          format!(r#":id="idMap['{}']""#, original_id)
        } else {
          caps[0].to_string()
        }
      })
      .to_string();

    inject_scripts = format!(
      "const __randId = () => Math.random().toString(36).substr(2, 10);const idMap = {{{}}};",
      id_map
        .iter()
        .map(|(k, v)| format!(r#""{}": "{}""#, k, v))
        .collect::<Vec<String>>()
        .join(", ")
    );
  }

  SvgResult {
    has_id,
    svg,
    inject_scripts,
  }
}
