//! Swc implementation of the sourcemap generator.
//! This method has issues with the original sourcemap when generating sourcemap for a resource pot that contains many modules.
//! We will provide a custom FarmSourceMapBuilder to fix this issue later, then this file will be removed.

use std::{collections::HashMap, path::Path, sync::Arc};

use farmfe_core::{
  swc_common::{source_map::SourceMapGenConfig, BytePos, FileName, LineCol, SourceMap},
  swc_ecma_ast::{Ident, Module as SwcModule},
};
use farmfe_utils::diff_paths;
use swc_atoms::JsWord;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

/*
Modified by brightwu, the original source code is from the swc project.

Copyright 2023 강동윤 <kdy1997.dev@gmail.com>

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
 */

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

pub fn build_source_map(
  mappings: &[(BytePos, LineCol)],
  cm: Arc<SourceMap>,
  ast: &SwcModule,
) -> Vec<u8> {
  let mut v = IdentCollector {
    names: Default::default(),
  };

  ast.visit_with(&mut v);

  let config = SwcSourceMapConfig {
    source_file_name: None,
    output_path: None,
    names: &v.names,
    inline_sources_content: true,
    emit_columns: true,
  };
  let mut src_buf = vec![];
  cm.build_source_map_with_config(mappings, None, config)
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
