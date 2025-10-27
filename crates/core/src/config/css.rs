use heck::{ToLowerCamelCase, ToSnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};
use swc_css_prefixer::options::Targets;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CssModulesConfig {
  /// The paths regex to match css modules
  pub paths: Vec<String>,
  pub indent_name: String,
  pub locals_conversion: NameConversion,
}

impl Default for CssModulesConfig {
  fn default() -> Self {
    Self {
      paths: vec![String::from("\\.module\\.(css|less|sass|scss)$")],
      indent_name: String::from("[name]-[hash]"),
      locals_conversion: NameConversion::default(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CssPrefixerConfig {
  #[serde(skip_serializing)]
  pub targets: Option<Targets>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CssConfig {
  pub modules: Option<CssModulesConfig>,
  pub prefixer: Option<CssPrefixerConfig>,
  /// transform css module to script module, for example:
  ///
  /// ```css
  /// .foo {
  ///   color: red;
  /// }
  /// ```
  ///
  /// will be transformed to:
  ///
  /// ```js
  /// const cssCode = '.foo { color: red; }';
  /// const farmId = 'foo.module.css';
  /// const previousStyle = document.querySelector('style[data-farm-id="' + farmId + '"]');
  /// const style = document.createElement('style');
  /// style.setAttribute('data-farm-id', farmId);
  /// style.innerHTML = cssCode;
  /// if (previousStyle) {
  ///   previousStyle.remove();
  /// }
  /// document.head.appendChild(style);
  /// ```
  pub transform_to_script: Option<bool>,
}

impl Default for CssConfig {
  fn default() -> Self {
    Self {
      modules: Some(Default::default()),
      prefixer: Some(Default::default()),
      transform_to_script: None,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum NameConversion {
  ///
  /// to keep the original name
  ///
  #[default]
  #[serde(rename = "asIs")]
  AsIs,
  ///
  /// ```md
  /// "It is we who built these palaces and cities."
  /// // =>
  /// "itIsWeWhoBuiltThesePalacesAndCities"
  /// ```
  #[serde(rename = "lowerCamel")]
  LowerCamel,
  /// ```md
  /// "We are not in the least afraid of ruins."
  /// // =>
  /// "WeAreNotInTheLeastAfraidOfRuins"
  /// ```
  #[serde(rename = "upperCamel")]
  UpperCamel,
  /// ```md
  /// "We carry a new world here, in our hearts."
  /// // =>
  /// "we_carry_a_new_world_here_in_our_hearts"
  /// ```
  #[serde(rename = "snake")]
  Snake,
}
impl NameConversion {
  pub fn transform(&self, name: &str) -> String {
    match self {
      NameConversion::LowerCamel => name.to_lower_camel_case(),
      NameConversion::UpperCamel => name.to_upper_camel_case(),
      NameConversion::Snake => name.to_snake_case(),
      NameConversion::AsIs => name.to_string(),
    }
  }
}
