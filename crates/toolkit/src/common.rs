use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use farmfe_core::{
  config::{comments::CommentsConfig, SourcemapConfig},
  relative_path::RelativePath,
  resource::{Resource, ResourceType},
  swc_common::{
    comments::{Comment, CommentKind, SingleThreadedComments},
    source_map::SourceMapGenConfig,
    BytePos, FileName, LineCol, SourceFile, SourceMap,
  },
};
use farmfe_utils::hash::base64_decode;

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

  // get last path segment
  let mut source_map_url = PathBuf::from(&map.name)
    .components()
    .last()
    .unwrap()
    .as_os_str()
    .to_str()
    .unwrap()
    .to_string();

  if config.is_inline() {
    source_map_url = format!("data:application/json;base64,{}", base64_encode(&map.bytes));
  }

  let source_map_comment = format!(
    "{}{}{}",
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

  fn emit_columns(&self, _f: &FileName) -> bool {
    true
  }
}

/// minify comments, the rule is same as swc, see https://github.com/swc-project/swc/blob/main/crates/swc_compiler_base/src/lib.rs
pub fn minify_comments(comments: &SingleThreadedComments, config: &CommentsConfig) {
  match config {
    // preserve all comments
    CommentsConfig::Bool(true) => {}
    CommentsConfig::Bool(false) => {
      let (mut l, mut t) = comments.borrow_all_mut();
      l.clear();
      t.clear();
    }
    CommentsConfig::License => {
      let preserve_excl = |_: &BytePos, vc: &mut Vec<Comment>| -> bool {
        // Preserve license comments.
        //
        // See https://github.com/terser/terser/blob/798135e04baddd94fea403cfaab4ba8b22b1b524/lib/output.js#L175-L181
        vc.retain(|c: &Comment| {
          c.text.contains("@lic")
            || c.text.contains("@preserve")
            || c.text.contains("@copyright")
            || c.text.contains("@cc_on")
            || (c.kind == CommentKind::Block && c.text.starts_with('!'))
        });
        !vc.is_empty()
      };
      let (mut l, mut t) = comments.borrow_all_mut();

      l.retain(preserve_excl);
      t.retain(preserve_excl);
    }
  }
}

pub fn load_source_original_source_map(
  content: &str,
  resolved_path: &str,
  source_map_comment_prefix: &str,
) -> Option<String> {
  let mut map = None;
  // try load source map when load module content.
  if content.contains(source_map_comment_prefix) {
    let base64_prefix = format!(
      "{}=data:application/json;base64,",
      source_map_comment_prefix
    );
    // detect that the source map is inline or not
    let source_map = if content.contains(&base64_prefix) {
      // inline source map
      let mut source_map = content.split(&base64_prefix);

      source_map
        .nth(1)
        .map(|source_map| base64_decode(source_map.as_bytes()))
    } else {
      // external source map
      let prefix = format!("{}=", source_map_comment_prefix);
      let mut source_map_path = content.split(&prefix);
      let source_map_path = source_map_path.nth(1).unwrap().to_string();
      let resolved_path = Path::new(resolved_path);
      let base_dir = resolved_path.parent().unwrap();
      let source_map_path = RelativePath::new(source_map_path.trim()).to_logical_path(base_dir);

      if source_map_path.exists() {
        let source_map = std::fs::read_to_string(source_map_path).unwrap();
        Some(source_map)
      } else {
        None
      }
    };

    if let Some(src_map) = source_map {
      map = Some(src_map);
    }
  }

  map
}
