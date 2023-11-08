//! Swc implementation of the sourcemap generator. This implementation needs to be refactor later.
//! Farm needs to provide a more reasonable sourcemap generator.

use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_core::{
  context::CompilationContext,
  module::{module_graph::ModuleGraph, ModuleId},
  swc_common::{
    source_map::{Pos, SourceMapGenConfig},
    BytePos, FileName, LineCol, SourceFile, SourceMap,
  },
  swc_ecma_ast::{Ident, Module as SwcModule},
};
use farmfe_utils::diff_paths;
use sourcemap::SourceMapBuilder;
use swc_atoms::JsWord;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

struct SwcSourceMapConfig<'a> {
  source_file_name: Option<&'a str>,
  /// Output path of the `.map` file.
  output_path: Option<&'a Path>,

  names: &'a HashMap<BytePos, JsWord>,

  inline_sources_content: bool,

  emit_columns: bool,
}

impl SourceMapGenConfig for SwcSourceMapConfig<'_> {
  fn file_name_to_source(&self, f: &FileName) -> String {
    if let Some(file_name) = self.source_file_name {
      return file_name.to_string();
    }

    let base_path = match self.output_path {
      Some(v) => v,
      None => return f.to_string(),
    };
    let target = match f {
      FileName::Real(v) => v,
      _ => return f.to_string(),
    };

    let rel = diff_paths(target, base_path);
    match rel {
      Some(v) => {
        let s = v.to_string_lossy().to_string();
        if cfg!(target_os = "windows") {
          s.replace('\\', "/")
        } else {
          s
        }
      }
      None => f.to_string(),
    }
  }

  fn name_for_bytepos(&self, pos: BytePos) -> Option<&str> {
    self.names.get(&pos).map(|v| &**v)
  }

  fn inline_sources_content(&self, _: &FileName) -> bool {
    self.inline_sources_content
  }

  fn emit_columns(&self, _f: &FileName) -> bool {
    self.emit_columns
  }

  fn skip(&self, f: &FileName) -> bool {
    if let FileName::Custom(s) = f {
      s.starts_with('<')
    } else {
      false
    }
  }
}

pub enum AstModule<'a> {
  Script(&'a SwcModule),
  Css(&'a farmfe_core::swc_css_ast::Stylesheet),
}

pub fn build_source_map(
  mappings: &[(BytePos, LineCol)],
  cm: Arc<SourceMap>,
  ast: AstModule,
  module_graph: &ModuleGraph,
) -> Vec<u8> {
  let mut v = IdentCollector {
    names: Default::default(),
  };

  v.visit_ident(ast);

  let config = SwcSourceMapConfig {
    source_file_name: None,
    output_path: None,
    names: &v.names,
    inline_sources_content: true,
    emit_columns: true,
  };
  let mut src_buf = vec![];
  build_source_map_with_config(cm, mappings, config, module_graph)
    .to_writer(&mut src_buf)
    .unwrap();

  src_buf
}

pub struct IdentCollector {
  names: HashMap<BytePos, JsWord>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}

impl swc_css_visit::Visit for IdentCollector {
  fn visit_ident(&mut self, ident: &farmfe_core::swc_css_ast::Ident) {
    self.names.insert(ident.span.lo, ident.value.clone());
  }
}

impl IdentCollector {
  fn visit_ident(&mut self, module: AstModule) {
    match module {
      AstModule::Script(ast) => ast.visit_with(self),
      AstModule::Css(stylesheet) => {
        use swc_css_visit::VisitWith;
        stylesheet.visit_with(self)
      }
    };
  }
}
