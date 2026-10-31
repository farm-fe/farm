//! Shared, validated palette configuration used by native Docs and snippets.
use crate::ThemeName;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use two_face::re_exports::syntect::highlighting::{
  Color, ScopeSelectors, StyleModifier, Theme, ThemeItem,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct HexColor(#[schemars(regex(pattern = "^#[0-9a-fA-F]{6}$"))] String);
impl<'de> Deserialize<'de> for HexColor {
  fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
    let value = String::deserialize(d)?;
    if value.len() != 7
      || !value.starts_with('#')
      || !value[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
      return Err(serde::de::Error::custom(
        "code token colors require #RRGGBB",
      ));
    }
    Ok(Self(value.to_ascii_lowercase()))
  }
}
impl HexColor {
  fn color(&self) -> Color {
    // Every instance is validated by Deserialize; this type has no public constructor.
    let channel =
      |start| u8::from_str_radix(&self.0[start..start + 2], 16).expect("validated hex color");
    Color {
      r: channel(1),
      g: channel(3),
      b: channel(5),
      a: 255,
    }
  }
}

// Reuse one field list for the schema, strict wire parser and scope mapping.
macro_rules! token_colors {
  ($($field:ident => $name:literal : $scope:literal),* $(,)?) => {
    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct TokenColors {
      $(#[serde(rename = $name, skip_serializing_if = "Option::is_none")]
        pub $field: Option<HexColor>,)*
    }
    #[derive(Default, Deserialize)]
    #[serde(default, deny_unknown_fields)]
    struct TokenColorsWire { $(#[serde(rename = $name)] $field: Option<HexColor>,)* }
    impl<'de> Deserialize<'de> for TokenColors {
      fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let map = serde_json::Map::<String, serde_json::Value>::deserialize(d)?;
        let wire: TokenColorsWire = serde_json::from_value(serde_json::Value::Object(map)).map_err(serde::de::Error::custom)?;
        Ok(Self { $($field: wire.$field,)* })
      }
    }
    impl TokenColors {
      pub fn is_empty(&self) -> bool { true $(&& self.$field.is_none())* }
      pub(crate) fn apply(&self, theme: &mut Theme) -> Result<(), String> {
        if let Some(color) = &self.foreground { theme.settings.foreground = Some(color.color()); }
        let mut overrides: Vec<(ScopeSelectors, Color)> = Vec::new();
        $(if !$scope.is_empty() { if let Some(color) = &self.$field {
          overrides.push(($scope.parse().map_err(|e| format!("[DOCS_THEME_SCOPE] {e}"))?, color.color()));
        } })*
        // Presets can contain more specific selectors than a public category.
        // Rewrite each matching selector independently, retaining font/background
        // settings and exclusions. Merely appending a broad rule loses to preset
        // specificity (and equal single-scope rules prefer the earlier entry).
        let mut scopes = Vec::new();
        for item in &theme.scopes {
          for selector in &item.scope.selectors {
            let mut style = item.style;
            if let Some((_, color)) = overrides.iter()
              .filter_map(|(scope, color)| scope.does_match(selector.path.as_slice()).map(|power| (power, *color)))
              .max_by_key(|(power, _)| *power) {
              style.foreground = Some(color);
            }
            scopes.push(ThemeItem { scope: ScopeSelectors { selectors: vec![selector.clone()] }, style });
          }
        }
        let mut custom: Vec<ThemeItem> = overrides.into_iter().map(|(scope, color)| ThemeItem {
          scope,
          style: StyleModifier { foreground: Some(color), ..Default::default() },
        }).collect();
        custom.extend(scopes);
        theme.scopes = custom;
        Ok(())
      }
    }
  }
}
token_colors! {
  foreground => "foreground": "",
  keyword => "keyword": "keyword, storage, constant.language",
  control => "control": "keyword.control",
  function => "function": "entity.name.function, support.function, variable.function",
  type_name => "type": "entity.name.type, entity.name.class, entity.name.namespace, support.type, support.class",
  class_name => "class": "entity.name.class, support.class",
  variable => "variable": "variable, entity.name.variable",
  parameter => "parameter": "variable.parameter",
  property => "property": "variable.other.property, meta.object-literal.key",
  string => "string": "string",
  number => "number": "constant.numeric",
  comment => "comment": "comment",
  tag => "tag": "entity.name.tag",
  attribute => "attribute": "entity.other.attribute-name",
  constant => "constant": "variable.other.constant, variable.other.enummember",
  punctuation => "punctuation": "punctuation",
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TokenOverrides {
  #[serde(default)]
  pub light: TokenColors,
  #[serde(default)]
  pub dark: TokenColors,
}
impl<'de> Deserialize<'de> for TokenOverrides {
  fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
    #[derive(Default, Deserialize)]
    #[serde(default, deny_unknown_fields)]
    struct Wire {
      light: TokenColors,
      dark: TokenColors,
    }
    let map = serde_json::Map::<String, serde_json::Value>::deserialize(d)?;
    let wire: Wire =
      serde_json::from_value(serde_json::Value::Object(map)).map_err(serde::de::Error::custom)?;
    Ok(Self {
      light: wire.light,
      dark: wire.dark,
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CodeThemeConfig {
  #[serde(default = "default_light")]
  pub light: ThemeName,
  #[serde(default = "default_dark")]
  pub dark: ThemeName,
  #[serde(default)]
  pub tokens: TokenOverrides,
}
fn default_light() -> ThemeName {
  ThemeName::VscodeLight
}
fn default_dark() -> ThemeName {
  ThemeName::VscodeDark
}
impl Default for CodeThemeConfig {
  fn default() -> Self {
    Self {
      light: default_light(),
      dark: default_dark(),
      tokens: TokenOverrides::default(),
    }
  }
}
impl<'de> Deserialize<'de> for CodeThemeConfig {
  fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
    #[derive(Deserialize)]
    #[serde(default, deny_unknown_fields)]
    struct Wire {
      light: ThemeName,
      dark: ThemeName,
      tokens: TokenOverrides,
    }
    impl Default for Wire {
      fn default() -> Self {
        let v = CodeThemeConfig::default();
        Self {
          light: v.light,
          dark: v.dark,
          tokens: v.tokens,
        }
      }
    }
    let map = serde_json::Map::<String, serde_json::Value>::deserialize(d)?;
    let wire: Wire =
      serde_json::from_value(serde_json::Value::Object(map)).map_err(serde::de::Error::custom)?;
    Ok(Self {
      light: wire.light,
      dark: wire.dark,
      tokens: wire.tokens,
    })
  }
}
