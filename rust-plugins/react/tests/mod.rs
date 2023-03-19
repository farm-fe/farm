use std::{collections::HashMap, sync::Arc};

use farmfe_compiler::Compiler;
use farmfe_core::config::{Config, OutputConfig, RuntimeConfig};
use farmfe_testing_helpers::fixture;

#[test]
fn test() {
  fixture!("tests/fixtures/button.tsx", |file, cwd| {
    // let config = Config {
    //   input: HashMap::from([("button".to_string(), file.to_string_lossy().to_string())]),
    //   output: OutputConfig {
    //     filename: "button.[ext]".to_string(),
    //     ..Default::default()
    //   },
    //   runtime: RuntimeConfig {
    //     path: file.to_string_lossy().to_string(),
    //     ..Default::default()
    //   },
    //   external: vec![
    //     "^react-refresh$".to_string(),
    //     "^react$".to_string(),
    //     "^@swc/helpers".to_string(),
    //   ],
    //   ..Default::default()
    // };

    // let plugin_react = Arc::new(farmfe_plugin_react::FarmPluginReact::new(
    //   &config,
    //   "".to_string(),
    // ));
    // let compiler = Compiler::new(config, vec![plugin_react as _]).unwrap();

    // compiler.compile().unwrap();

    // let context = compiler.context();
    // let resources_map = context.resources_map.lock();

    // // for (id, resource) in resources_map.iter() {
    // //   let code = std::str::from_utf8(&resource.bytes).unwrap();
    // //   println!("{}: {:?}", id, code);
    // // }

    // let button_js = resources_map.get("button.js").unwrap();

    // let code = std::str::from_utf8(&button_js.bytes).unwrap();
    // println!("{}", code);
  });
}
