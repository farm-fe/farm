use std::{
  process::Command,
  sync::Arc,
  time::{Duration, Instant},
};

use farmfe_core::{
  config::Config,
  context::CompilationContext,
  module::{Module, ModuleId, ModuleType},
  plugin::{Plugin, PluginHookContext, PluginParseHookParam},
  rayon::{ThreadPoolBuilder, prelude::*},
  resource::resource_pot::{ResourcePot, ResourcePotType},
};
use farmfe_plugin_css::FarmPluginCss;
use farmfe_toolkit::css::codegen_css_stylesheet;

// A subprocess bounds a regression that otherwise deadlocks the test runner.
#[test]
fn parallel_css_pots_finish_without_reentrant_resource_lock() {
  let mut child = Command::new(std::env::current_exe().unwrap())
    .args([
      "--exact",
      "parallel_css_render_child",
      "--ignored",
      "--nocapture",
    ])
    .spawn()
    .unwrap();
  let deadline = Instant::now() + Duration::from_secs(30);
  loop {
    if let Some(status) = child.try_wait().unwrap() {
      assert!(status.success(), "CSS render subprocess failed: {status}");
      break;
    }
    if Instant::now() >= deadline {
      child.kill().unwrap();
      child.wait().unwrap();
      panic!("parallel CSS resource pots deadlocked (30 second timeout)");
    }
    std::thread::sleep(Duration::from_millis(10));
  }
}

#[test]
#[ignore = "invoked by the bounded parent regression"]
fn parallel_css_render_child() {
  let context = Arc::new(CompilationContext::new(Config::default(), vec![]).unwrap());
  let plugin = FarmPluginCss::new(&context.config);
  let mut pots = Vec::new();
  for pot_index in 0..64 {
    let mut pot = ResourcePot::new(&format!("chunk-{pot_index}"), "test", ResourcePotType::Css);
    for module_index in 0..16 {
      let path = format!("/src/chunk-{pot_index}/style-{module_index}.css");
      let id = ModuleId::new(&path, "", "/");
      let content = format!(".chunk-{pot_index}-{module_index} {{ color: red; }}");
      let metadata = plugin
        .parse(
          &PluginParseHookParam {
            module_id: id.clone(),
            resolved_path: path,
            query: vec![],
            module_type: ModuleType::Css,
            content: Arc::new(content),
          },
          &context,
          &PluginHookContext::default(),
        )
        .unwrap()
        .unwrap();
      let mut module = Module::new(id.clone());
      module.module_type = ModuleType::Css;
      module.execution_order = module_index;
      *module.meta = metadata;
      context.module_graph.write().add_module(module);
      pot.add_module(id);
    }
    pots.push(pot);
  }
  let pool = ThreadPoolBuilder::new().num_threads(4).build().unwrap();
  for _ in 0..20 {
    pool.install(|| {
      pots.par_iter().for_each(|pot| {
        let metadata = plugin
          .render_resource_pot(pot, &context, &PluginHookContext::default())
          .unwrap()
          .unwrap();
        let (css, map) = codegen_css_stylesheet(
          &metadata.as_css().ast,
          false,
          Some(context.meta.get_resource_pot_source_map(&pot.id)),
          false,
        );
        assert_eq!(metadata.as_css().ast.rules.len(), 16);
        assert!(css.contains("color: red"));
        let map: farmfe_core::serde_json::Value =
          farmfe_core::serde_json::from_str(&map.unwrap()).unwrap();
        assert_eq!(map["sources"].as_array().unwrap().len(), 16);
      })
    });
  }
}
