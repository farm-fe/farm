use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  config::SourcemapConfig,
  resource::{Resource, ResourceType},
  swc_common::{source_map::SourceMapGenConfig, BytePos, FileName, LineCol, SourceFile, SourceMap},
};

use crate::hash::base64_encode;

pub struct Source {
  pub path: PathBuf,
  pub content: Arc<String>,
}

/// create a swc source map from a source
pub fn create_swc_source_map(source: Source) -> (Arc<SourceMap>, Arc<SourceFile>) {
  let cm = Arc::new(SourceMap::default());
  let sf = cm.new_source_file_from(FileName::Real(source.path), source.content);

  (cm, sf)
}

pub fn append_source_map_comment(
  resource: &mut Resource,
  map: &Resource,
  config: &SourcemapConfig,
) {
  let source_map_str = match &resource.resource_type {
    ResourceType::Js => "\n//# sourceMappingURL=",
    ResourceType::Css => "\n/*# sourceMappingURL=",
    _ => unreachable!("only js and css need source map"),
  };

  let mut source_map_url = map.name.clone();

  if config.is_inline() {
    source_map_url = format!("data:application/json;base64,{}", base64_encode(&map.bytes));
  }

  let source_map_comment = format!(
    "{}/{}{}",
    source_map_str,
    source_map_url,
    if matches!(resource.resource_type, ResourceType::Css) {
      " */"
    } else {
      ""
    }
  );

  resource.bytes.append(&mut source_map_comment.into_bytes());
}

pub fn build_source_map(
  cm: Arc<SourceMap>,
  mappings: &[(BytePos, LineCol)],
) -> sourcemap::SourceMap {
  cm.build_source_map_with_config(mappings, None, FarmSwcSourceMapConfig::default())
}

pub struct FarmSwcSourceMapConfig {
  inline_sources_content: bool,
}

impl FarmSwcSourceMapConfig {
  pub fn new(inline_sources_content: bool) -> Self {
    Self {
      inline_sources_content,
    }
  }
}

impl Default for FarmSwcSourceMapConfig {
  fn default() -> Self {
    Self {
      inline_sources_content: true,
    }
  }
}

impl SourceMapGenConfig for FarmSwcSourceMapConfig {
  fn inline_sources_content(&self, _: &FileName) -> bool {
    self.inline_sources_content
  }

  fn file_name_to_source(&self, f: &FileName) -> String {
    f.to_string()
  }
}
