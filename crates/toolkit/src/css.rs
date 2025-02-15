use std::sync::Arc;

use farmfe_core::{
  context::{get_swc_sourcemap_filename, CompilationContext},
  error::CompilationError,
  module::ModuleId,
  rayon::iter::{IntoParallelRefMutIterator, ParallelIterator},
  regex::Regex,
  swc_common::{comments::SingleThreadedComments, input::SourceFileInput, BytePos, SourceMap},
  swc_css_ast::Stylesheet,
};
use swc_css_codegen::{
  writer::basic::{BasicCssWriter, BasicCssWriterConfig},
  CodeGenerator, CodegenConfig, Emit,
};
use swc_css_parser::{
  lexer::Lexer,
  parser::{Parser, ParserConfig},
};
use swc_css_visit::{VisitMut, VisitMutWith};
use swc_error_reporters::handler::try_with_handler;

use crate::sourcemap::{build_sourcemap, create_swc_source_map};

pub struct ParseCssModuleResult {
  pub ast: Stylesheet,
  pub comments: SingleThreadedComments,
  pub source_map: Arc<SourceMap>,
}

/// parse the input css file content to [Stylesheet]
pub fn parse_css_stylesheet(
  id: &str,
  orig_content: Arc<String>,
) -> farmfe_core::error::Result<ParseCssModuleResult> {
  // swc_css_parser does not support parsing invalid css, so we need to replace known invalid css here
  // 1. replace --: '' to --farm-empty: ''
  let mut content = orig_content.replace("--:", "--farm-empty:");
  // 2. replace filter: xxx.Microsoft.xxx to filter: "xxx.Microsoft.xxx" using regex. fix #1557
  let regex = Regex::new(r#"filter:\s*([^'"]*?)\.Microsoft\.(.*?)(;|\})"#).unwrap();
  content = regex
    .replace_all(&content, "filter:\"$1.Microsoft.$2\"$3")
    .to_string();
  // // 3. replace invalid operator, eg: top: -8px/2 + 1 to top: "-8px/2 + 1" using regex. fix #1748
  // let regex = Regex::new(r#":\s*([^;{}]*?\d\s+\s\d[^;{}]*?)\s*(;|\})"#).unwrap();
  // content = regex.replace_all(&content, ":\"$1\"$2").to_string();

  let (cm, source_file) = create_swc_source_map(&id.into(), Arc::new(content));

  let config = ParserConfig {
    allow_wrong_line_comments: true,
    css_modules: true,
    legacy_nesting: true,
    legacy_ie: true,
  };

  let comments = SingleThreadedComments::default();
  let lexer = Lexer::new(
    SourceFileInput::from(&*source_file),
    Some(&comments),
    config,
  );
  let mut parser = Parser::new(lexer, config);

  let parse_result = parser.parse_all();
  let mut recovered_errors = parser.take_errors();

  if recovered_errors.len() == 0 {
    match parse_result {
      Err(err) => {
        recovered_errors.push(err);
      }
      Ok(m) => {
        return Ok(ParseCssModuleResult {
          ast: m,
          comments,
          source_map: cm,
        });
      }
    }
  }

  try_with_handler(cm, Default::default(), |handler| {
    for err in recovered_errors {
      err.to_diagnostics(handler).emit();
    }

    Err(anyhow::Error::msg("SyntaxError"))
  })
  .map_err(|e| CompilationError::ParseError {
    resolved_path: id.to_string(),
    msg: if let Some(s) = e.downcast_ref::<String>() {
      s.to_string()
    } else if let Some(s) = e.downcast_ref::<&str>() {
      s.to_string()
    } else {
      "failed to handle with unknown panic message".to_string()
    },
  })
}

/// generate css code from [Stylesheet], return css code and source map
pub fn codegen_css_stylesheet(
  stylesheet: &Stylesheet,
  minify: bool,
  cm: Option<Arc<SourceMap>>,
) -> (String, Option<String>) {
  let mut css_code = String::new();
  let mut mappings = Vec::new();
  let css_writer = BasicCssWriter::new(
    &mut css_code,
    if cm.is_some() {
      Some(&mut mappings)
    } else {
      None
    },
    BasicCssWriterConfig::default(),
  );
  let mut gen = CodeGenerator::new(css_writer, CodegenConfig { minify });

  gen.emit(stylesheet).unwrap();

  if let Some(cm) = cm {
    let map = build_sourcemap(cm, &mappings);
    let mut src_map = vec![];
    map.to_writer(&mut src_map).unwrap();

    (css_code, Some(String::from_utf8(src_map).unwrap()))
  } else {
    (css_code, None)
  }
}

pub fn merge_css_sourcemap(
  module_asts: &mut Vec<(ModuleId, Stylesheet)>,
  context: &Arc<CompilationContext>,
) -> Arc<SourceMap> {
  let module_ids: Vec<_> = module_asts.iter().map(|item| &item.0).collect();
  let new_cm = Arc::new(SourceMap::default());

  for module_id in module_ids {
    let cm = context.meta.get_module_source_map(module_id);
    cm.files().iter().for_each(|source_file| {
      new_cm.new_source_file_from(source_file.name.clone(), source_file.src.clone());
    });
  }

  // update Span in parallel
  module_asts.par_iter_mut().for_each(|(module_id, ast)| {
    let filename = get_swc_sourcemap_filename(module_id);
    let source_file = new_cm
      .get_source_file(&filename)
      .unwrap_or_else(|| panic!("no source file found for {:?}", module_id));
    let start_pos = source_file.start_pos;
    ast.visit_mut_with(&mut SpanUpdater { start_pos });
  });

  new_cm
}

struct SpanUpdater {
  pub start_pos: BytePos,
}

impl VisitMut for SpanUpdater {
  fn visit_mut_span(&mut self, node: &mut farmfe_core::swc_common::Span) {
    node.lo = self.start_pos + node.lo;
    node.hi = self.start_pos + node.hi;
  }
}
