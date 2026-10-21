use std::{path::PathBuf, sync::Arc};

use farmfe_core::{
  HashMap,
  config::{Config, bool_or_obj::BoolOrObj},
  context::{CompilationContext, create_swc_source_map},
  module::{Module, ModuleType},
  plugin::{
    Plugin, PluginAnalyzeDepsHookParam, PluginAnalyzeDepsHookResultEntry, PluginHookContext,
    PluginLoadHookParam, PluginParseHookParam, ResolveKind,
  },
  resource::{
    meta_data::{ResourcePotMetaData, js::JsResourcePotMetaData},
    resource_pot::{ResourcePot, ResourcePotId, ResourcePotType},
  },
};
use farmfe_testing_helpers::fixture;

#[test]
fn worker_processing_preserves_shared_commonjs_module() {
  use farmfe_core::module::{
    ModuleSystem,
    meta_data::script::{FARM_RUNTIME_MODULE_HELPER_ID, FARM_RUNTIME_MODULE_SYSTEM_ID},
  };

  let config = Config::default();
  let plugin = farmfe_plugin_script::FarmPluginScript::new(&config);
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  let hook_context = PluginHookContext {
    caller: None,
    meta: HashMap::default(),
  };
  let shared_id = "shared-parser.js";
  for (id, content) in [
    (shared_id, "module.exports = { value: 7 };"),
    (
      FARM_RUNTIME_MODULE_SYSTEM_ID,
      "export const farmRegister = () => {};",
    ),
    (
      FARM_RUNTIME_MODULE_HELPER_ID,
      "export const helper = () => {};",
    ),
  ] {
    let meta = plugin
      .parse(
        &PluginParseHookParam {
          module_id: id.into(),
          resolved_path: id.to_string(),
          query: vec![],
          module_type: ModuleType::Js,
          content: Arc::new(content.to_string()),
        },
        &context,
        &hook_context,
      )
      .unwrap()
      .unwrap();
    let mut module = Module::new(id.into());
    module.module_type = ModuleType::Js;
    *module.meta = meta;
    module.meta.as_script_mut().module_system = if id == shared_id {
      ModuleSystem::CommonJs
    } else {
      ModuleSystem::EsModule
    };
    context.module_graph.write().add_module(module);
  }
  let original_ast = context
    .module_graph
    .read()
    .module(&shared_id.into())
    .unwrap()
    .meta
    .as_script()
    .ast
    .clone();
  let mut worker = ResourcePot::new(
    &ResourcePotId::from("worker"),
    "worker",
    ResourcePotType::DynamicEntryJs,
  );
  worker.add_module(shared_id.into());
  worker.entry_module = Some(shared_id.into());
  plugin
    .process_resource_pots(&mut vec![&mut worker], &context)
    .unwrap();

  let graph = context.module_graph.read();
  let shared = graph.module(&shared_id.into()).unwrap();
  assert_eq!(
    shared.meta.as_script().module_system,
    ModuleSystem::CommonJs
  );
  assert_eq!(shared.meta.as_script().ast, original_ast);
  assert!(graph.dependencies(&shared.id).is_empty());
}

#[test]
fn load_parse_and_analyze_deps() {
  fixture(
    "tests/fixtures/load_parse_analyze/**/index.*",
    |file: PathBuf, _| {
      let config = Config {
        minify: Box::new(BoolOrObj::Bool(true)),
        ..Default::default()
      };
      let plugin_script = farmfe_plugin_script::FarmPluginScript::new(&config);
      let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
      let id = file.to_string_lossy().to_string();
      let hook_context = PluginHookContext {
        caller: None,
        meta: HashMap::default(),
      };
      let loaded = plugin_script.load(
        &PluginLoadHookParam {
          resolved_path: &id,
          query: vec![],
          meta: HashMap::default(),
          module_id: id.clone(),
        },
        &context,
        &hook_context,
      );

      assert!(loaded.is_ok());
      let loaded = loaded.unwrap();
      assert!(loaded.is_some());
      let loaded = loaded.unwrap();

      let lines: Vec<&str> = loaded.content.lines().collect();
      assert_eq!(
        lines,
        vec![
          "import a from './a';",
          "import b from './b';",
          "",
          "export * from './c';",
          "export { d } from './d';",
          "",
          "console.log(a, b);"
        ]
      );
      assert_eq!(
        loaded.module_type,
        match file.extension().unwrap().to_str().unwrap() {
          "js" => ModuleType::Js,
          "jsx" => ModuleType::Jsx,
          "ts" => ModuleType::Ts,
          "tsx" => ModuleType::Tsx,
          _ => unreachable!("never be here"),
        }
      );

      let module_meta = plugin_script
        .parse(
          &PluginParseHookParam {
            module_id: "any".into(),
            resolved_path: id,
            query: vec![],
            module_type: loaded.module_type.clone(),
            content: Arc::new(loaded.content.clone()),
          },
          &context,
          &hook_context,
        )
        .unwrap()
        .unwrap();

      let ast = module_meta.as_script().ast.clone();
      let mut module = Module::new("any".into());
      *module.meta = module_meta;
      module.module_type = loaded.module_type;

      assert_eq!(module.meta.as_script().ast.body.len(), 5);

      let mut deps = PluginAnalyzeDepsHookParam {
        module: &module,
        deps: vec![],
      };
      plugin_script
        .analyze_deps(&mut deps, &context)
        .unwrap()
        .unwrap();
      assert_eq!(
        deps.deps,
        vec![
          PluginAnalyzeDepsHookResultEntry {
            source: String::from("./a"),
            kind: ResolveKind::Import
          },
          PluginAnalyzeDepsHookResultEntry {
            source: String::from("./b"),
            kind: ResolveKind::Import
          },
          PluginAnalyzeDepsHookResultEntry {
            source: String::from("./c"),
            kind: ResolveKind::ExportFrom
          },
          PluginAnalyzeDepsHookResultEntry {
            source: String::from("./d"),
            kind: ResolveKind::ExportFrom
          }
        ]
      );

      let mut resource_pot =
        ResourcePot::new(&ResourcePotId::from("index"), "any", ResourcePotType::Js);

      resource_pot.resource_pot_type = ResourcePotType::Js;
      resource_pot.meta = ResourcePotMetaData::Js(JsResourcePotMetaData {
        ast,
        ..Default::default()
      });
      context.meta.set_resource_pot_source_map(
        &resource_pot.id,
        create_swc_source_map(&module.id, Arc::new(loaded.content.clone())).0,
      );

      let resources = plugin_script
        .generate_resources(&mut resource_pot, &context, &hook_context)
        .unwrap()
        .unwrap();
      // assert!(resources.source_map.is_some());

      let code = String::from_utf8(resources.resources[0].resource.bytes.clone()).unwrap();

      let lines: Vec<&str> = code.lines().collect();
      assert_eq!(
        lines,
        vec![
          "import a from\"./a\";import b from\"./b\";export*from\"./c\";export{d}from\"./d\";console.log(a,b);"
        ]
      );

      // assert_eq!(
      //   &resources[1].bytes,
      //   "{\"version\":3,\"sources\":[\"any\"],\"sourcesContent\":[\"import a from './a';\\nimport b from './b';\\n\\nexport * from './c';\\nexport { d } from './d';\\n\\nconsole.log(a, b);\\n\"],\"names\":[\"a\",\"b\",\"d\",\"console\",\"log\"],\"mappings\":\"AAAA,OAAOA,OAAO,MAAM;AACpB,OAAOC,OAAO,MAAM;AAEpB,cAAc,MAAM;AACpB,SAASC,CAAC,QAAQ,MAAM;AAExBC,QAAQC,GAAG,CAACJ,GAAGC\"}".as_bytes()
      // )
    },
  );
}
