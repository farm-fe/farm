use std::env;
use std::fs::read_to_string;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::sync::Arc;

use farmfe_core::config::config_regex::ConfigRegex;
use farmfe_core::config::Config;
use farmfe_core::context::CompilationContext;
use farmfe_core::module::ModuleType;
use farmfe_core::plugin::Plugin;
use farmfe_core::plugin::PluginTransformHookParam;
use farmfe_core::serde_json;
use farmfe_plugin_react_components::find_local_components::ExportType;
use farmfe_plugin_react_components::resolvers::ImportStyle;
use farmfe_plugin_react_components::resolvers::ResolverOption;
use farmfe_plugin_react_components::Dts;
use farmfe_plugin_react_components::ImportMode;
use farmfe_plugin_react_components::{FarmPluginReactComponents, Options};
use rustc_hash::FxHashMap;
#[test]
fn transform() {
  let current_dir = env::current_dir().unwrap();
  let root_path = current_dir.join("playground").to_string_lossy().to_string();
  let test_file = current_dir
    .join("playground/src/test.tsx")
    .to_string_lossy()
    .to_string();
  let id = current_dir
    .join("playground/src/main.tsx")
    .to_string_lossy()
    .to_string();
  let resolvers = [
    ResolverOption {
      module: "antd".to_string(),
      export_type: Some(ExportType::Named),
      import_style: Some(ImportStyle::Bool(false)),
      exclude: None,
      include: None,
      prefix: Some("Ant".to_string()),
    },
    ResolverOption {
      module: "@arco-design/web-react".to_string(),
      export_type: Some(ExportType::Named),
      import_style: Some(ImportStyle::Bool(true)),
      exclude: None,
      include: None,
      prefix: Some("A".to_string()),
    },
  ];
  // let resolvers_components = get_resolvers_result(&root_path.to_string_lossy().to_string(), resolvers.to_vec());
  let option = Options {
    dirs: Some(vec![ConfigRegex::new("src/components")]),
    dts: Some(Dts::Bool(true)),
    import_mode: Some(ImportMode::Relative),
    local: Some(true),
    include: None,
    exclude: None,
    resolvers: Some(resolvers.to_vec()),
  };
  let option = serde_json::to_string(&option).unwrap();
  let config = Config {
    root: root_path,
    ..Default::default()
  };
  let farm_plugin_react_components = FarmPluginReactComponents::new(&config, option);
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());

  let content = read_to_string(id.clone()).unwrap();
  let transform_param = PluginTransformHookParam {
    content,
    module_id: id.clone(),
    module_type: ModuleType::Tsx,
    resolved_path: &id,
    meta: FxHashMap::default(),
    source_map_chain: vec![],
    query: vec![],
  };
  let res = farm_plugin_react_components
    .transform(&transform_param, &context)
    .unwrap()
    .unwrap();
  let file = File::create(test_file).unwrap();
  let mut writer = BufWriter::new(file);
  writeln!(writer, "{}", res.content).unwrap();
  writer.flush().unwrap();
}
