use swc_core::{
  common::{
    comments::{Comment, Comments, SingleThreadedComments},
    source_map::SourceMapGenConfig,
    sync::Lrc,
    FileName, FilePathMapping, SourceMap,
  },
  ecma::{
    ast::Module,
    codegen::{text_writer::JsWriter, Emitter},
  },
};

struct ContentMapConfig;
impl SourceMapGenConfig for ContentMapConfig {
  fn file_name_to_source(&self, file: &FileName) -> String {
    file.to_string().replace('\\', "/")
  }
  fn inline_sources_content(&self, _file: &FileName) -> bool {
    true
  }
}

/// Preserve the original MDX spans instead of mdxjs::Program.serialize's span erasure.
pub fn emit(
  module: &Module,
  comments: &[Comment],
  source: &str,
  filename: &str,
) -> Result<(String, String), String> {
  let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
  cm.new_source_file(
    Lrc::new(FileName::Custom(filename.into())),
    source.to_string(),
  );
  let stored = SingleThreadedComments::default();
  for comment in comments {
    stored.add_leading(comment.span.lo, comment.clone());
  }
  let mut bytes = vec![];
  let mut mappings = vec![];
  {
    let mut emitter = Emitter {
      cfg: Default::default(),
      cm: cm.clone(),
      comments: Some(&stored),
      wr: JsWriter::new(cm.clone(), "\n", &mut bytes, Some(&mut mappings)),
    };
    emitter
      .emit_module(module)
      .map_err(|error| format!("[DOCS_SOURCEMAP] {error}"))?;
  }
  let code = String::from_utf8(bytes).map_err(|error| format!("[DOCS_SOURCEMAP] {error}"))?;
  let map = cm.build_source_map(&mappings, None, ContentMapConfig);
  let mut serialized = vec![];
  map
    .to_writer(&mut serialized)
    .map_err(|error| format!("[DOCS_SOURCEMAP] {error}"))?;
  Ok((
    code,
    String::from_utf8(serialized).map_err(|error| format!("[DOCS_SOURCEMAP] {error}"))?,
  ))
}
