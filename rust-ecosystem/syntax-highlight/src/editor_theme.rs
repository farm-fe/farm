//! Familiar editor-style colors mapped to grammar scopes, not semantic guesses.
//! Reference: microsoft/vscode theme-defaults Dark+/Light+ palettes.
use two_face::re_exports::syntect::highlighting::{Color, StyleModifier, Theme, ThemeItem};

fn rgb(value: u32) -> Color {
  Color {
    r: (value >> 16) as u8,
    g: (value >> 8) as u8,
    b: value as u8,
    a: 255,
  }
}
pub(crate) fn theme(dark: bool) -> Result<Theme, String> {
  let foreground = if dark { 0xd4d4d4 } else { 0x000000 };
  let mut theme = Theme::default();
  theme.name = Some(
    if dark {
      "Farm editor dark"
    } else {
      "Farm editor light"
    }
    .into(),
  );
  theme.settings.foreground = Some(rgb(foreground));
  theme.settings.background = Some(rgb(if dark { 0x1e1e1e } else { 0xffffff }));
  // More specific grammar scopes win over broad defaults in Syntect.
  for (scope, light, dark_color) in [
    ("comment", 0x008000, 0x6a9955),
    ("string", 0xa31515, 0xce9178),
    ("constant.numeric", 0x098658, 0xb5cea8),
    ("keyword, storage, constant.language", 0x0000ff, 0x569cd6),
    ("keyword.control", 0xaf00db, 0xc586c0),
    (
      "variable, entity.name.variable, meta.object-literal.key",
      0x001080,
      0x9cdcfe,
    ),
    (
      "variable.other.constant, variable.other.enummember",
      0x0070c1,
      0x4fc1ff,
    ),
    (
      "entity.name.type, entity.name.class, entity.name.namespace, support.type, support.class",
      0x267f99,
      0x4ec9b0,
    ),
    ("support.type.primitive", 0x0000ff, 0x569cd6),
    ("entity.name.function, support.function", 0x795e26, 0xdcdcaa),
    ("entity.name.tag", 0x800000, 0x569cd6),
    ("entity.other.attribute-name", 0xe50000, 0x9cdcfe),
  ] {
    theme.scopes.push(ThemeItem {
      scope: scope
        .parse()
        .map_err(|error| format!("[DOCS_THEME_SCOPE] {error}"))?,
      style: StyleModifier {
        foreground: Some(rgb(if dark { dark_color } else { light })),
        ..StyleModifier::default()
      },
    });
  }
  Ok(theme)
}
