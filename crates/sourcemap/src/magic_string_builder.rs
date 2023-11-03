use sourcemap::{SourceMap, SourceMapBuilder};

/// Build string and source map from strings and their corresponding maps.
///
/// Example:
/// ```ignore
/// use farmfe_sourcemap::magic_string_builder::MagicStringBuilder;
/// let mut builder = MagicStringBuilder::new();
/// builder.append_string("a", "{ "version": 3 }");
/// builder.append_string("b", "{ "version": 3 }");
/// let (string, map) = builder.build();
/// ````
pub struct MagicStringBuilder {
  string_and_maps: Vec<(String, Option<String>)>,
}

impl MagicStringBuilder {
  pub fn new() -> Self {
    Self {
      string_and_maps: vec![],
    }
  }

  pub fn append_string(&mut self, str: &str, map: Option<&str>) {
    self
      .string_and_maps
      .push((str.to_string(), map.map(|m| m.to_string())));
  }

  pub fn prepend_string(&mut self, str: &str, map: Option<&str>) {
    self
      .string_and_maps
      .insert(0, (str.to_string(), map.map(|m| m.to_string())));
  }

  pub fn build(self) -> sourcemap::Result<(String, String)> {
    let mut result_str = String::new();
    let mut result_map = String::new();

    let mut line: usize = 0;
    let mut col: usize = 0;
    let mut sourcemap_builder = SourceMapBuilder::new(None);

    for (str, map) in self.string_and_maps {
      let map = if let Some(map) = map {
        Some(SourceMap::from_slice(map.as_bytes())?)
      } else {
        None
      };
    }

    Ok((result_str, result_map))
  }
}
