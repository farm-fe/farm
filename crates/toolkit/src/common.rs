use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

use farmfe_core::{
  config::{
    bool_or_obj::BoolOrObj,
    comments::CommentsConfig,
    config_regex::ConfigRegex,
    minify::{MinifyMode, MinifyOptions},
    SourcemapConfig,
  },
  enhanced_magic_string::collapse_sourcemap::{collapse_sourcemap_chain, read_source_content},
  module::ModuleId,
  rayon::iter::{IntoParallelRefIterator, ParallelIterator},
  relative_path::RelativePath,
  resource::{resource_pot::ResourcePot, Resource, ResourceOrigin, ResourceType},
  serde_json::Value,
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

/// get swc source map filename from module id.
/// you can get module id from sourcemap filename too, by
pub fn get_swc_sourcemap_filename(module_id: &ModuleId) -> FileName {
  FileName::Real(PathBuf::from(module_id.to_string()))
}

pub fn get_module_id_from_sourcemap_filename(filename: &str) -> ModuleId {
  filename.into()
}

/// create a swc source map from a source
pub fn create_swc_source_map(source: Source) -> (Arc<SourceMap>, Arc<SourceFile>) {
  let cm = Arc::new(SourceMap::default());
  let sf = cm.new_source_file_from(Arc::new(FileName::Real(source.path)), source.content);

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

pub fn generate_source_map_resource(resource_pot: &ResourcePot) -> Resource {
  // collapse source map chain
  let source_map_chain = resource_pot
    .meta
    .rendered_map_chain
    .iter()
    .map(|s| sourcemap::SourceMap::from_slice(s.as_bytes()).unwrap())
    .collect::<Vec<_>>();
  let collapsed_sourcemap = collapse_sourcemap_chain(source_map_chain, Default::default());
  let mut src_map = vec![];
  collapsed_sourcemap
    .to_writer(&mut src_map)
    .expect("failed to write sourcemap");
  Resource {
    bytes: src_map,
    name: resource_pot.name.clone(),
    emitted: false,
    resource_type: ResourceType::SourceMap(resource_pot.id.to_string()),
    origin: ResourceOrigin::ResourcePot(resource_pot.id.clone()),
    info: None,
  }
}

pub fn build_source_map(
  cm: Arc<SourceMap>,
  mappings: &[(BytePos, LineCol)],
) -> sourcemap::SourceMap {
  cm.build_source_map_with_config(mappings, None, FarmSwcSourceMapConfig::default())
}

pub fn collapse_sourcemap(
  sourcemap: sourcemap::SourceMap,
  module_graph: &farmfe_core::module::module_graph::ModuleGraph,
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
      dst_token.get_source(),
      dst_token.get_name(),
      dst_token.is_range(),
    );

    if !builder.has_source_contents(new_token.src_id) {
      if let Some(content) = read_source_content(dst_token.clone(), dst_sourcemap) {
        builder.set_source_contents(new_token.src_id, Some(&content));
      }
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
          .map(|i| sourcemap::SourceMap::from_slice(i.as_bytes()).unwrap())
          .collect::<Vec<sourcemap::SourceMap>>();
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

pub struct PathFilter<'a> {
  include: &'a Vec<ConfigRegex>,
  exclude: &'a Vec<ConfigRegex>,
}

impl<'a> PathFilter<'a> {
  pub fn new(include: &'a Vec<ConfigRegex>, exclude: &'a Vec<ConfigRegex>) -> Self {
    Self { include, exclude }
  }

  pub fn execute(&self, path: &str) -> bool {
    (self.include.is_empty() || self.include.iter().any(|regex| regex.is_match(path)))
      && (self.exclude.is_empty() || !self.exclude.iter().any(|regex| regex.is_match(path)))
  }
}

pub struct MinifyBuilder {
  pub minify_options: Option<MinifyOptions>,
  expect_mode: Option<MinifyMode>,
}

impl MinifyBuilder {
  fn is_match(&self, path: &str) -> bool {
    if let Some(ref minify_options) = self.minify_options {
      return PathFilter::new(&minify_options.include, &minify_options.exclude).execute(path);
    }

    false
  }

  fn is_match_mode(&self) -> bool {
    if let Some(expect_mode) = &self.expect_mode {
      return if let Some(ref minify_options) = self.minify_options {
        expect_mode == &minify_options.mode
      } else {
        false
      };
    }
    return true;
  }

  pub fn is_enabled(&self, path: &str) -> bool {
    return self.is_match_mode() && self.is_match(path);
  }

  pub fn create_builder(minify: &BoolOrObj<Value>, mode: Option<MinifyMode>) -> MinifyBuilder {
    let minify_options = Option::<MinifyOptions>::from(minify);

    MinifyBuilder {
      minify_options,
      expect_mode: mode,
    }
  }
}

#[cfg(test)]
mod tests {

  mod minify_builder {
    use super::super::MinifyBuilder;
    use farmfe_core::{
      config::{bool_or_obj::BoolOrObj, minify::MinifyMode},
      serde_json::json,
    };

    #[test]
    fn create_builder() {
      let builder = MinifyBuilder::create_builder(&BoolOrObj::Bool(false), None);
      assert!(builder.minify_options.is_none());
      assert!(!builder.is_enabled("index.html"));

      let builder = MinifyBuilder::create_builder(&BoolOrObj::Bool(true), None);
      assert!(builder.minify_options.is_some());
      assert!(builder.is_enabled("index.html"));
    }

    #[test]
    fn minify_exclude() {
      let builder = MinifyBuilder::create_builder(
        &BoolOrObj::Obj(json!({
          "exclude": ["\\.html$"],
        })),
        None,
      );

      assert!(builder.minify_options.is_some());
      assert!(!builder.is_enabled("index.html"));
    }

    #[test]
    fn minify_include() {
      let builder = MinifyBuilder::create_builder(
        &BoolOrObj::Obj(json!({
          "include": ["\\.html$"],
        })),
        None,
      );

      assert!(builder.is_enabled("index.html"));
      assert!(!builder.is_enabled("index.html1"));
      assert!(!builder.is_enabled("index.js"));
    }

    #[test]
    fn filter_by_mode() {
      let builder = MinifyBuilder::create_builder(
        &BoolOrObj::Obj(json!({
          "mode": "minify-resource-pot",
          "include": [".*"]
        })),
        Some(MinifyMode::Module),
      );

      assert!(!builder.is_enabled("index.html"));
      assert!(!builder.is_enabled("index.html1"));
      assert!(!builder.is_enabled("index.js"));
    }
  }
}
