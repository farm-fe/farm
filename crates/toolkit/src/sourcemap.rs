use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

use farmfe_core::{
  config::SourcemapConfig,
  enhanced_magic_string::collapse_sourcemap::read_source_content,
  module::ModuleId,
  rayon::iter::{IntoParallelRefIterator, ParallelIterator},
  relative_path::RelativePath,
  resource::{Resource, ResourceType},
  swc_common::{
    source_map::SourceMapGenConfig, BytePos, FileName, LineCol, SourceMap as SwcSourceMap,
  },
};

use crate::hash::base64_encode;
use farmfe_utils::{hash::base64_decode, relative};

pub use farmfe_core::context::{create_swc_source_map, get_swc_sourcemap_filename};
// reexport sourcemap crate
pub use sourcemap::*;

pub fn get_module_id_from_sourcemap_filename(filename: &str) -> ModuleId {
  filename.into()
}

/// Whether a line is a sourcemap comment line
pub fn is_sourcemap_comment_line(line: &str) -> bool {
  line.starts_with("//# sourceMappingURL=") || line.starts_with("/*# sourceMappingURL=")
}

pub fn append_sourcemap_comment(resource: &mut Resource, map: &Resource, config: &SourcemapConfig) {
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

pub fn build_sourcemap(
  cm: Arc<SwcSourceMap>,
  mappings: &[(BytePos, LineCol)],
) -> sourcemap::SourceMap {
  cm.build_source_map_with_config(mappings, None, FarmSwcSourceMapConfig::default())
}

/// Trace the final bundled sourcemap to original module sourcemap
/// So that the source map of the final bundle can be traced back to the original source file
pub fn trace_module_sourcemap(
  sourcemap: sourcemap::SourceMap,
  module_graph: &farmfe_core::module::module_graph::ModuleGraph,
  root: &str,
) -> sourcemap::SourceMap {
  let mut builder = sourcemap::SourceMapBuilder::new(sourcemap.get_file());
  let mut cached_sourcemap = HashMap::<ModuleId, Vec<sourcemap::SourceMap>>::new();

  let mut add_token = |src_token: &sourcemap::Token,
                       dst_token: &sourcemap::Token,
                       dst_sourcemap: &sourcemap::SourceMap| {
    let new_token = builder.add(
      src_token.get_dst_line(),
      src_token.get_dst_col(),
      dst_token.get_src_line(),
      dst_token.get_src_col(),
      // replace absolute source path with relative path
      dst_token.get_source().map(|source| {
        if Path::new(source).is_absolute() {
          Box::leak(relative(root, source).into_boxed_str())
        } else {
          source
        }
      }),
      dst_token.get_name(),
      dst_token.is_range(),
    );

    if let Some(content) = read_source_content(dst_token.clone(), dst_sourcemap) {
      builder.set_source_contents(new_token.src_id, Some(&content));
    }
  };

  for token in sourcemap.tokens() {
    if let Some(filename) = token.get_source() {
      let module_id = get_module_id_from_sourcemap_filename(filename);
      if !module_graph.has_module(&module_id) {
        add_token(&token, &token, &sourcemap);
        continue;
      }

      let module = module_graph
        .module(&module_id)
        .unwrap_or_else(|| panic!("module {} not found in module graph", module_id));

      if module.source_map_chain.is_empty() {
        add_token(&token, &token, &sourcemap);
        continue;
      }

      let sourcemap_chain = cached_sourcemap.entry(module_id).or_insert_with(|| {
        let mut chain = module
          .source_map_chain
          .par_iter()
          .map(|i| sourcemap::SourceMap::from_slice(i.as_bytes()).unwrap())
          .collect::<Vec<sourcemap::SourceMap>>();
        // reverse the chain to make the last one the original sourcemap
        chain.reverse();
        // filter out the empty sourcemap
        chain.retain(|map| map.get_token_count() > 0);

        chain
      });

      let mut dst_token = token;
      let mut dst_sourcemap = token.sourcemap();

      // trace the token back to original source file
      for orig_map in sourcemap_chain {
        if let Some(orig_token) =
          orig_map.lookup_token(dst_token.get_src_line(), dst_token.get_src_col())
        {
          dst_token = orig_token;
          dst_sourcemap = orig_token.sourcemap();
        }
      }

      add_token(&token, &dst_token, dst_sourcemap);
    } else {
      add_token(&token, &token, &sourcemap);
    }
  }

  builder.into_sourcemap()
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

pub fn load_source_original_sourcemap(
  content: &str,
  resolved_path: &str,
  source_map_comment_prefix: &str,
) -> Option<String> {
  let mut map = None;
  // try load source map when load module content.
  if content.contains(source_map_comment_prefix) {
    let base64_prefix = format!("{source_map_comment_prefix}=data:application/json;base64,");
    // detect that the source map is inline or not
    let source_map = if content.contains(&base64_prefix) {
      // inline source map
      let mut source_map = content.split(&base64_prefix);

      source_map
        .nth(1)
        .map(|source_map| base64_decode(source_map.as_bytes()))
    } else {
      // external source map
      let prefix = format!("{source_map_comment_prefix}=");
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
