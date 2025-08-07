use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

use bytes_str::BytesStr;
use farmfe_core::{
  config::SourcemapConfig,
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
pub use swc_sourcemap::*;

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

pub fn build_sourcemap(cm: Arc<SwcSourceMap>, mappings: &[(BytePos, LineCol)]) -> SourceMap {
  // TODO investigate performance comparison of swc_sourcemap and sourcemap and normalize the usage of sourcemap crate
  let swc_sourcemap = cm.build_source_map(mappings, None, FarmSwcSourceMapConfig::default());

  let mut slice = Vec::new();
  swc_sourcemap.to_writer(&mut slice).unwrap();

  SourceMap::from_slice(&slice).unwrap()
}

/// Trace the final bundled sourcemap to original module sourcemap
/// So that the source map of the final bundle can be traced back to the original source file
pub fn trace_module_sourcemap(
  sourcemap: SourceMap,
  module_graph: &farmfe_core::module::module_graph::ModuleGraph,
  root: &str,
) -> SourceMap {
  let mut builder = SourceMapBuilder::new(sourcemap.get_file().cloned());
  let mut cached_sourcemap = HashMap::<ModuleId, Vec<SourceMap>>::new();

  let mut add_token = |src_token: &Token, dst_token: &Token, dst_sourcemap: &SourceMap| {
    let new_token = builder.add(
      src_token.get_dst_line(),
      src_token.get_dst_col(),
      dst_token.get_src_line(),
      dst_token.get_src_col(),
      // replace absolute source path with relative path
      dst_token.get_source().map(|source| {
        if Path::new(source).is_absolute() {
          BytesStr::from_string(relative(root, source))
        } else {
          source.clone()
        }
      }),
      dst_token.get_name().cloned(),
      dst_token.is_range(),
    );

    if let Some(content) = read_source_content(dst_token.clone(), dst_sourcemap) {
      builder.set_source_contents(new_token.src_id, Some(BytesStr::from_string(content)));
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
        .unwrap_or_else(|| panic!("module {} not found in module graph", module_id.to_string()));

      if module.source_map_chain.is_empty() {
        add_token(&token, &token, &sourcemap);
        continue;
      }

      let sourcemap_chain = cached_sourcemap.entry(module_id).or_insert_with(|| {
        let mut chain = module
          .source_map_chain
          .par_iter()
          .map(|i| SourceMap::from_slice(i.as_bytes()).unwrap())
          .collect::<Vec<SourceMap>>();
        // reverse the chain to make the last one the original sourcemap
        chain.reverse();
        // filter out the empty sourcemap
        chain = chain
          .into_iter()
          .filter(|map| map.get_token_count() > 0)
          .collect();
        chain
      });

      let mut dst_token = token.clone();
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

      add_token(&token, &dst_token, &dst_sourcemap);
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
    // TODO: make it configurable
    false
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

use farmfe_utils::file_url_to_path;
use std::cell::{RefCell, RefMut};

pub struct CollapseSourcemapOptions {
  /// if true, inline source content to the source map.
  /// if the source content does not exist and source filename exists, content will be read from source file from disk.
  pub inline_content: bool,

  pub remap_source: Option<Box<dyn Fn(&str) -> String>>,
}

impl Default for CollapseSourcemapOptions {
  fn default() -> Self {
    Self {
      inline_content: true,
      remap_source: None,
    }
  }
}

/// collapse source map chain to a single source.
///
/// transformation: a -> b -> c -> d, source content is a and dest content is d.
/// corresponding input source map: [map_a, map_b, map_c, map_d].
///
/// now we have d and map_d, we want to get a and map_a, we should tracing from map_d to map_a.
///
pub fn collapse_sourcemap_chain(
  mut chain: Vec<SourceMap>,
  opts: CollapseSourcemapOptions,
) -> SourceMap {
  chain.reverse();
  chain = chain
    .into_iter()
    .filter(|map| map.get_token_count() > 0)
    .collect();

  if chain.is_empty() {
    let builder = SourceMapBuilder::new(None);
    return builder.into_sourcemap();
  }

  let dest_map = &chain[0];
  let mut builder = SourceMapBuilder::new(None);
  let mut mapped_src_cache = std::collections::HashMap::new();

  // trace all tokens in cur and update
  for token in dest_map.tokens() {
    let mut last_map_token = token;
    let mut completed_trace = true;

    if chain.len() > 1 {
      for map in &chain[1..] {
        if let Some(map_token) = lookup_token(
          map,
          last_map_token.get_src_line(),
          last_map_token.get_src_col(),
        ) {
          last_map_token = map_token;
        } else {
          completed_trace = false;
          break;
        }
      }
    }

    // if we can't trace back to the first map, ignore this token
    if !completed_trace {
      // builder.add_token(&token, true);
      continue;
    }

    let source = last_map_token.get_source();
    let mut srd_id = None;

    if let Some(src) = source {
      let remapped_src = if let Some(remap_source) = &opts.remap_source {
        mapped_src_cache
          .entry(src)
          .or_insert_with(|| remap_source(src))
          .to_string()
      } else {
        src.to_string()
      };

      srd_id = Some(builder.add_source(BytesStr::from_string(remapped_src)));
    }

    let mut name_id = None;

    if let Some(name) = last_map_token.get_name().or(token.get_name()) {
      name_id = Some(builder.add_name(name.clone()));
    }

    let added_token = builder.add_raw(
      token.get_dst_line(),
      token.get_dst_col(),
      last_map_token.get_src_line(),
      last_map_token.get_src_col(),
      srd_id,
      name_id,
      false,
    );

    if opts.inline_content && srd_id.is_some() && !builder.has_source_contents(srd_id.unwrap()) {
      let src_content = read_source_content(last_map_token, chain.last().unwrap());

      if let Some(src_content) = src_content {
        builder.set_source_contents(added_token.src_id, Some(BytesStr::from_string(src_content)));
      }
    }
  }

  builder.into_sourcemap()
}

/// if map_token is not exact match, we should use the token next to it to make sure the line mapping is correct.
/// this is because lookup_token of [SourceMap] will return the last found token instead of the next if it can't find exact match, which leads to wrong line mapping(mapping to previous line).
pub fn lookup_token<'a>(map: &'a SourceMap, line: u32, col: u32) -> Option<Token<'a>> {
  let token = map.lookup_token(line, col);

  if let Some(token) = token {
    // mapped to the last token of previous line.
    if line > 0 && token.get_dst_line() == line - 1 && token.get_dst_col() > 0 {
      let next_token = map.lookup_token(line + 1, 0);

      if let Some(next_token) = next_token {
        if next_token.get_dst_line() == line {
          return Some(next_token);
        }
      }
    }
  }

  token
}

pub fn read_source_content(token: Token<'_>, map: &SourceMap) -> Option<String> {
  if let Some(view) = token.get_source_view() {
    Some(view.source().to_string())
  } else if let Some(src) = token.get_source() {
    let src = &file_url_to_path(src);
    // try read source content from disk
    let map_file = map.get_file();

    if PathBuf::from(src).is_absolute() || map_file.is_none() {
      std::fs::read_to_string(src).ok()
    } else if let Some(map_file) = map_file {
      let src_file = PathBuf::from(map_file).parent().unwrap().join(src);
      let src_content = std::fs::read_to_string(src_file).ok();

      src_content
    } else {
      None
    }
  } else {
    None
  }
}

pub struct CollapsedSourceMap<'a> {
  pub tokens: RefCell<Vec<Token<'a>>>,
  pub map: SourceMap,
}

impl<'a> CollapsedSourceMap<'a> {
  pub fn new(map: SourceMap) -> Self {
    Self {
      tokens: RefCell::new(vec![]),
      map,
    }
  }

  pub fn tokens(&'a self) -> RefMut<'a, Vec<Token<'a>>> {
    let mut tokens = self.tokens.borrow_mut();

    if tokens.is_empty() {
      *tokens = self.map.tokens().collect::<Vec<_>>();
    }

    tokens
  }
}
