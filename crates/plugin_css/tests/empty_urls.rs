use std::sync::Arc;

use farmfe_core::{
  HashMap,
  module::{Module, module_graph::ModuleGraph},
};
use farmfe_plugin_css::source_replace;
use farmfe_toolkit::css::{codegen_css_stylesheet, parse_css_stylesheet};

#[test]
fn empty_urls_survive_source_replacement_without_dependencies() {
  let input = ".a{a:url();b:url('');c:url(\"\");d:url('   ');e:url(  )}";
  let mut parsed = parse_css_stylesheet("empty.css", Arc::new(input.to_string())).unwrap();
  let before = codegen_css_stylesheet(&parsed.ast, false, None, false).0;
  let mut graph = ModuleGraph::new();
  graph.add_module(Module::new("empty.css".into()));
  source_replace(
    &mut parsed.ast,
    &"empty.css".into(),
    &graph,
    &HashMap::default(),
    String::new(),
    vec![],
  );
  assert_eq!(
    codegen_css_stylesheet(&parsed.ast, false, None, false).0,
    before
  );
}

#[test]
#[should_panic(expected = "is not a edge")]
fn missing_nonempty_url_dependency_is_not_silently_ignored() {
  let mut parsed = parse_css_stylesheet(
    "missing.css",
    Arc::new(".a{background:url('./missing.png')}".to_string()),
  )
  .unwrap();
  let mut graph = ModuleGraph::new();
  graph.add_module(Module::new("missing.css".into()));
  source_replace(
    &mut parsed.ast,
    &"missing.css".into(),
    &graph,
    &HashMap::default(),
    String::new(),
    vec![],
  );
}
