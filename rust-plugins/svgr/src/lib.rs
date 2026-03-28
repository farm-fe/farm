#![deny(clippy::all)]
mod create_filter;
mod options;
mod react_compiler;
mod svg_builder;
use std::collections::HashMap;

use farmfe_core::{
  config::Config,
  module::ModuleType,
  plugin::{Plugin, PluginLoadHookResult},
};
use farmfe_toolkit::fs::read_file_utf8;

pub use create_filter::{create_filter, Filter};
use farmfe_macro_plugin::farm_plugin;
pub use react_compiler::{react_compiler, CompilerParams};
pub use svg_builder::SvgBuilder;
#[farm_plugin]
pub struct FarmPluginSvgr {
  filter: Filter,
  options: Options,
}
use options::Options;

impl FarmPluginSvgr {
  fn new(config: &Config, _options: String) -> Self {
    let options: Options = serde_json::from_str(&_options).unwrap();
    let include: Vec<&str> = options
      .include
      .as_ref()
      .map(|v| v.iter().map(|s| s.as_str()).collect())
      .unwrap_or_else(|| vec!["**/*.svg"]);

    let exclude: Vec<&str> = options
      .exclude
      .as_ref()
      .map(|v| v.iter().map(|s| s.as_str()).collect())
      .unwrap_or_else(Vec::new);

    let filter = create_filter(Some(include), Some(exclude)).expect("Failed to create SVG filter");

    Self { filter, options }
  }
}

impl Plugin for FarmPluginSvgr {
  fn name(&self) -> &str {
    "FarmPluginSvgr"
  }

  fn load(
    &self,
    param: &farmfe_core::plugin::PluginLoadHookParam,
    _context: &std::sync::Arc<farmfe_core::context::CompilationContext>,
    _hook_context: &farmfe_core::plugin::PluginHookContext,
  ) -> farmfe_core::error::Result<Option<farmfe_core::plugin::PluginLoadHookResult>> {
    if param
      .query
      .iter()
      .any(|(key, _)| key == "raw" || key == "url" || key == "inline")
    {
      return Ok(None);
    }

    if !self.filter.should_process(&param.module_id) {
      return Ok(None);
    }
    let is_img = param
      .query
      .iter()
      .any(|(key, value)| key == "img" && value == "true");

    if is_img {
      return Ok(None);
    }

    let svg_content = get_svg_by_local_path(&param.resolved_path);
    let query_map = &param.query.iter().cloned().collect::<HashMap<_, _>>();
    let svg = SvgBuilder::new(&svg_content)
      .fill(query_map.get("fill").cloned())
      .stroke(query_map.get("stroke").cloned())
      .stroke_width(query_map.get("stroke-width").cloned())
      .width(query_map.get("width").cloned())
      .height(query_map.get("height").cloned())
      .class(self.options.default_class.clone())
      .style(self.options.default_style.clone())
      .view_box(None)
      .build();
    let code = react_compiler(CompilerParams {
      svg,
      root_path: None,
      svg_name: None,
    });
    return Ok(Some(PluginLoadHookResult {
      content: code,
      module_type: ModuleType::Jsx,
      source_map: None,
    }));
  }
}

pub fn get_svg_by_local_path(path: &str) -> String {
  let svg_raw = read_file_utf8(path).unwrap();
  svg_raw
}
