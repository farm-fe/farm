//! Farm Spectrum: restrained purple, cool blue/teal and warm supporting colors.
use two_face::re_exports::syntect::highlighting::{Color, StyleModifier, Theme, ThemeItem};
fn rgb(v: u32) -> Color {
  Color {
    r: (v >> 16) as u8,
    g: (v >> 8) as u8,
    b: v as u8,
    a: 255,
  }
}
pub(crate) fn theme(dark: bool) -> Result<Theme, String> {
  let mut theme = Theme::default();
  theme.name = Some(
    if dark {
      "Farm Spectrum Dark"
    } else {
      "Farm Spectrum Light"
    }
    .into(),
  );
  theme.settings.foreground = Some(rgb(if dark { 0xded8e8 } else { 0x332c40 }));
  theme.settings.background = Some(rgb(if dark { 0x201a2a } else { 0xf8f6fc }));
  for (scope, light, night) in [
    ("comment", 0x74677c, 0xa196ad),
    ("string", 0x3c6d50, 0xa5c8a8),
    ("constant.numeric", 0x9a4835, 0xe5a38e),
    ("keyword, storage, constant.language", 0x8735a2, 0xcba0e3),
    ("keyword.control", 0x8735a2, 0xdbafe2),
    ("variable, entity.name.variable", 0x4e405e, 0xded8e8),
    ("variable.parameter", 0x935123, 0xe2b780),
    (
      "variable.other.property, meta.object-literal.key",
      0x685086,
      0xb8abd6,
    ),
    (
      "variable.other.constant, variable.other.enummember",
      0x865025,
      0xe2b780,
    ),
    (
      "entity.name.type, entity.name.class, entity.name.namespace, support.type, support.class",
      0x325caa,
      0x9ebcf2,
    ),
    ("support.type.primitive", 0x325caa, 0x9ebcf2),
    (
      "entity.name.function, support.function, variable.function",
      0x176a83,
      0x7dc6d5,
    ),
    ("entity.name.tag", 0x8735a2, 0xcaa7df),
    ("entity.other.attribute-name", 0x4e617e, 0xa8c9db),
    ("punctuation", 0x74677c, 0xa196ad),
  ] {
    theme.scopes.push(ThemeItem {
      scope: scope
        .parse()
        .map_err(|e| format!("[DOCS_THEME_SCOPE] {e}"))?,
      style: StyleModifier {
        foreground: Some(rgb(if dark { night } else { light })),
        ..Default::default()
      },
    });
  }
  Ok(theme)
}
