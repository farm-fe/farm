use heck::{ToLowerCamelCase, ToSnakeCase, ToUpperCamelCase};
use serde::{Deserialize, Serialize};

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
