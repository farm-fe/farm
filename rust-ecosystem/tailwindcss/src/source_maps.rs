//! Source-map utilities.
//!
//! Simplified Rust port of
//! [`@tailwindcss-node/src/source-maps.ts`](https://github.com/tailwindlabs/tailwindcss/blob/main/packages/%40tailwindcss-node/src/source-maps.ts).
//!
//! Provides a [`SourceMap`] wrapper around a raw JSON source-map string with
//! convenience methods for emitting CSS source-map comments and inline
//! data-URIs.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// A processed source map with convenience helpers.
pub struct SourceMap {
  raw: String,
}

impl SourceMap {
  /// Create a new [`SourceMap`] from a raw JSON string.
  pub fn new(raw: String) -> Self {
    Self { raw }
  }

  /// The raw JSON source-map string.
  pub fn raw(&self) -> &str {
    &self.raw
  }

  /// Generate a CSS comment that points at an external source-map file.
  ///
  /// ```
  /// # use farmfe_ecosystem_tailwindcss::source_maps::SourceMap;
  /// let sm = SourceMap::new(String::new());
  /// assert_eq!(
  ///     sm.comment("app.css.map"),
  ///     "/*# sourceMappingURL=app.css.map */\n"
  /// );
  /// ```
  pub fn comment(&self, url: &str) -> String {
    format!("/*# sourceMappingURL={url} */\n")
  }

  /// Generate a CSS comment with the source map inlined as a `data:` URI.
  pub fn inline(&self) -> String {
    let encoded = BASE64.encode(self.raw.as_bytes());
    self.comment(&format!("data:application/json;base64,{encoded}"))
  }
}
