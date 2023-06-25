use std::collections::HashMap;
use std::{path::PathBuf, sync::Arc};

use common::generate_runtime;
use farmfe_compiler::Compiler;
use farmfe_core::plugin::UpdateType;
use farmfe_core::{
  config::{preset_env::PresetEnvConfig, Config, CssConfig, Mode, RuntimeConfig, SourcemapConfig},
  plugin::Plugin,
  resource::ResourceType,
};
use farmfe_testing_helpers::fixture;

mod common;

fn create_update_compiler(
  input: HashMap<String, String>,
  cwd: PathBuf,
  crate_path: PathBuf,
  minify: bool,
) -> Compiler {
  let compiler = Compiler::new(
    Config {
      input,
      root: cwd.to_string_lossy().to_string(),
      runtime: generate_runtime(crate_path),
      output: farmfe_core::config::OutputConfig {
        filename: "[resourceName].[ext]".to_string(),
        ..Default::default()
      },
      mode: Mode::Development,
      external: vec!["^react-refresh$".to_string(), "^module$".to_string()],
      sourcemap: SourcemapConfig::Bool(false),
      lazy_compilation: false,
      minify,
      preset_env: Box::new(PresetEnvConfig::Bool(false)),
      ..Default::default()
    },
    vec![],
  )
  .unwrap();

  compiler
}

#[test]
fn update_without_dependencies_change() {
  fixture!("tests/fixtures/update/index.html", |file, crate_path| {
    let cwd = file.parent().unwrap().to_path_buf();
    let compiler = create_update_compiler(
      HashMap::from([("index".to_string(), "./index.html".to_string())]),
      cwd.clone(),
      crate_path,
      false,
    );

    compiler.compile().unwrap();

    let update_file = file
      .parent()
      .unwrap()
      .join("index.ts")
      .to_string_lossy()
      .to_string();
    let result = compiler
      .update(
        vec![(update_file.clone(), UpdateType::Updated)],
        || {},
        true,
      )
      .unwrap();

    assert_eq!(result.added_module_ids.len(), 0);
    assert_eq!(result.updated_module_ids, vec!["index.ts".into()]);
    assert_eq!(result.removed_module_ids.len(), 0);

    assert_eq!(result.resources, "{\n    \"index.ts\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        Object.defineProperty(exports, \"__esModule\", {\n            value: true\n        });\n        farmRequire(\"index.css\");\n        console.log(\"Hello, world!\");\n    }\n};\n");
  });
}

#[test]
fn update_without_dependencies_change_css() {
  fixture!("tests/fixtures/update/index.html", |file, crate_path| {
    let cwd = file.parent().unwrap().to_path_buf();
    let compiler = create_update_compiler(
      HashMap::from([("index".to_string(), "./index.html".to_string())]),
      cwd.clone(),
      crate_path,
      false,
    );

    compiler.compile().unwrap();

    let update_file = file
      .parent()
      .unwrap()
      .join("index.css")
      .to_string_lossy()
      .to_string();
    let result = compiler
      .update(
        vec![(update_file.clone(), UpdateType::Updated)],
        || {},
        true,
      )
      .unwrap();

    assert_eq!(result.added_module_ids.len(), 0);
    assert_eq!(result.updated_module_ids, vec!["index.css".into()]);
    assert_eq!(result.removed_module_ids.len(), 0);

    assert_eq!(result.resources, "{\n    \"index.css\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        const cssCode = `body {\n  color: red;\n}`;\n        const farmId = \"index.css\";\n        const previousStyle = document.querySelector(`style[data-farm-id=\"${farmId}\"]`);\n        const style = document.createElement(\"style\");\n        style.setAttribute(\"data-farm-id\", farmId);\n        style.innerHTML = cssCode;\n        if (previousStyle) {\n            previousStyle.replaceWith(style);\n        } else {\n            document.head.appendChild(style);\n        }\n        module.meta.hot.accept();\n        module.onDispose(()=>{\n            style.remove();\n        });\n    }\n};\n");

    let result = compiler
      .update(
        vec![(update_file.clone(), UpdateType::Updated)],
        || {},
        false,
      )
      .unwrap();

    assert_eq!(result.added_module_ids.len(), 0);
    assert_eq!(result.updated_module_ids, vec!["index.css".into()]);
    assert_eq!(result.removed_module_ids.len(), 0);

    assert_eq!(result.resources, "{\n    \"index.css\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        const cssCode = `body {\n  color: red;\n}`;\n        const farmId = \"index.css\";\n        const previousStyle = document.querySelector(`style[data-farm-id=\"${farmId}\"]`);\n        const style = document.createElement(\"style\");\n        style.setAttribute(\"data-farm-id\", farmId);\n        style.innerHTML = cssCode;\n        if (previousStyle) {\n            previousStyle.replaceWith(style);\n        } else {\n            document.head.appendChild(style);\n        }\n        module.meta.hot.accept();\n        module.onDispose(()=>{\n            style.remove();\n        });\n    }\n};\n");
  });
}
