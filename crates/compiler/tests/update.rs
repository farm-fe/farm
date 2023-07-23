use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use common::generate_runtime;
use farmfe_compiler::Compiler;
use farmfe_core::config::config_regex::ConfigRegex;
use farmfe_core::config::{preset_env::PresetEnvConfig, Config, Mode, SourcemapConfig};
use farmfe_core::plugin::UpdateType;
use farmfe_core::regex::Regex;
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
      external: vec![
        ConfigRegex(Regex::new("^react-refresh$").unwrap()),
        ConfigRegex(Regex::new("^module$").unwrap()),
      ],
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
  fixture!(
    "tests/fixtures/update/basic/index.html",
    |file, crate_path| {
      let cwd = file.parent().unwrap().to_path_buf();
      let compiler = create_update_compiler(
        HashMap::from([("index".to_string(), "./index.html".to_string())]),
        cwd,
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
          vec![(update_file, UpdateType::Updated)],
          || {},
          true,
        )
        .unwrap();

      assert_eq!(result.added_module_ids.len(), 0);
      assert_eq!(result.updated_module_ids, vec!["index.ts".into()]);
      assert_eq!(result.removed_module_ids.len(), 0);

      assert_eq!(result.resources, "{\n    \"index.ts\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        Object.defineProperty(exports, \"__esModule\", {\n            value: true\n        });\n        farmRequire(\"index.css\");\n        console.log(\"Hello, world!\");\n    }\n};\n");
    }
  );
}

#[test]
fn update_without_dependencies_change_css() {
  fixture!(
    "tests/fixtures/update/basic/index.html",
    |file, crate_path| {
      let cwd = file.parent().unwrap().to_path_buf();
      let compiler = create_update_compiler(
        HashMap::from([("index".to_string(), "./index.html".to_string())]),
        cwd,
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
          vec![(update_file, UpdateType::Updated)],
          || {},
          false,
        )
        .unwrap();

      assert_eq!(result.added_module_ids.len(), 0);
      assert_eq!(result.updated_module_ids, vec!["index.css".into()]);
      assert_eq!(result.removed_module_ids.len(), 0);

      assert_eq!(result.resources, "{\n    \"index.css\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        const cssCode = `body {\n  color: red;\n}`;\n        const farmId = \"index.css\";\n        const previousStyle = document.querySelector(`style[data-farm-id=\"${farmId}\"]`);\n        const style = document.createElement(\"style\");\n        style.setAttribute(\"data-farm-id\", farmId);\n        style.innerHTML = cssCode;\n        if (previousStyle) {\n            previousStyle.replaceWith(style);\n        } else {\n            document.head.appendChild(style);\n        }\n        module.meta.hot.accept();\n        module.onDispose(()=>{\n            style.remove();\n        });\n    }\n};\n");
    }
  );
}

#[test]
fn update_with_dependencies_change_css_modules() {
  fixture!(
    "tests/fixtures/update/css-modules/index.html",
    |file, crate_path| {
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
      // read original index.ts
      let mut original_ts = String::new();
      let mut original_ts_file = File::open(&update_file).unwrap();
      original_ts_file.read_to_string(&mut original_ts).unwrap();
      let mut original_ts_file = File::create(&update_file).unwrap();
      // update index.ts
      original_ts_file
        .write_all(
          original_ts
            .replace(
              "console.log('Hello, world!');",
              "import styles from './index.module.css';\nconsole.log(styles);",
            )
            .as_bytes(),
        )
        .unwrap();

      let result = compiler
        .update(
          vec![(update_file.clone(), UpdateType::Updated)],
          || {},
          true,
        )
        .unwrap();
      assert_eq!(result.added_module_ids.len(), 2,);
      assert!(result.added_module_ids.contains(&"index.module.css".into()));
      assert!(result
        .added_module_ids
        .contains(&"index.module.css.FARM_CSS_MODULES?f1d5b6cc".into()));
      assert_eq!(result.updated_module_ids, vec!["index.ts".into()]);
      assert_eq!(result.removed_module_ids.len(), 0);

      assert_eq!(result.resources, "{\n    \"index.module.css\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        Object.defineProperty(exports, \"__esModule\", {\n            value: true\n        });\n        Object.defineProperty(exports, \"default\", {\n            enumerable: true,\n            get: function() {\n                return _default;\n            }\n        });\n        farmRequire(\"index.module.css.FARM_CSS_MODULES?f1d5b6cc\");\n        var _default = {\n            \"className\": `className-477586ce`\n        };\n    },\n    \"index.module.css.FARM_CSS_MODULES?f1d5b6cc\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        const cssCode = `.className-477586ce {\n  color: red;\n}`;\n        const farmId = \"index.module.css.FARM_CSS_MODULES?f1d5b6cc\";\n        const previousStyle = document.querySelector(`style[data-farm-id=\"${farmId}\"]`);\n        const style = document.createElement(\"style\");\n        style.setAttribute(\"data-farm-id\", farmId);\n        style.innerHTML = cssCode;\n        if (previousStyle) {\n            previousStyle.replaceWith(style);\n        } else {\n            document.head.appendChild(style);\n        }\n        module.meta.hot.accept();\n        module.onDispose(()=>{\n            style.remove();\n        });\n    },\n    \"index.ts\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        Object.defineProperty(exports, \"__esModule\", {\n            value: true\n        });\n        var _interop_require_default = farmRequire(\"@swc/helpers/_/_interop_require_default\");\n        farmRequire(\"index.css\");\n        var _indexmodulecss = _interop_require_default._(farmRequire(\"index.module.css\"));\n        console.log(_indexmodulecss.default);\n    }\n};\n");

      let update_file_css = cwd.join("index.module.css").to_string_lossy().to_string();
      // read original index.module.css
      let mut original_css = String::new();
      let mut original_css_file = File::open(&update_file_css).unwrap();
      original_css_file.read_to_string(&mut original_css).unwrap();
      // update index.module.css
      let mut original_css_file = File::create(&update_file_css).unwrap();
      original_css_file
        .write_all(original_css.replace(".className", ".className2").as_bytes())
        .unwrap();
      let result = compiler
        .update(
          vec![(update_file_css.clone(), UpdateType::Updated)],
          || {},
          true,
        )
        .unwrap();
      assert_eq!(
        result.added_module_ids,
        vec!["index.module.css.FARM_CSS_MODULES?b2914899".into()]
      );
      assert_eq!(result.updated_module_ids, vec!["index.module.css".into()]);
      assert_eq!(
        result.removed_module_ids,
        vec!["index.module.css.FARM_CSS_MODULES?f1d5b6cc".into()]
      );
      // restore index.module.css
      let mut original_css_file = File::create(&update_file_css).unwrap();
      original_css_file
        .write_all(original_css.as_bytes())
        .unwrap();

      // restore index.ts
      let mut original_ts_file = File::create(&update_file).unwrap();
      original_ts_file.write_all(original_ts.as_bytes()).unwrap();
      let result = compiler
        .update(
          vec![(update_file, UpdateType::Updated)],
          || {},
          false,
        )
        .unwrap();

      assert_eq!(result.added_module_ids.len(), 0);
      assert_eq!(result.updated_module_ids, vec!["index.ts".into()]);
      assert_eq!(result.removed_module_ids.len(), 2);
      assert!(result
        .removed_module_ids
        .contains(&"index.module.css".into()));
      assert!(result
        .removed_module_ids
        .contains(&"index.module.css.FARM_CSS_MODULES?b2914899".into()));

      assert_eq!(result.resources, "{\n    \"index.ts\": function(module, exports, farmRequire, dynamicRequire) {\n        \"use strict\";\n        Object.defineProperty(exports, \"__esModule\", {\n            value: true\n        });\n        farmRequire(\"index.css\");\n        console.log(\"Hello, world!\");\n    }\n};\n");
    }
  );
}
