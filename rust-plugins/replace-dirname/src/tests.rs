use super::*;
use farmfe_core::{
  config::persistent_cache::PersistentCacheConfig,
  module::{ModuleId, ModuleType, meta_data::script::ScriptModuleMetaData},
  swc_ecma_ast::EsVersion,
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::{
  script::parse_module,
  swc_ecma_visit::{Visit, VisitWith},
};

fn check_paths(root: &Path, file: &Path) {
  let mut config = Config::default();
  config.root = root.to_str().unwrap().to_owned();
  *config.persistent_cache = PersistentCacheConfig::Bool(false);
  let plugin = FarmPluginReplaceDirname::new(&config, "{}".into());
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  let id = ModuleId::new(
    file.to_str().unwrap(),
    "?config-probe",
    root.to_str().unwrap(),
  );
  let source = Arc::new("const paths = [__dirname, __filename, import.meta.url];".to_owned());
  let parsed = parse_module(
    &id,
    source.clone(),
    Syntax::Es(EsSyntax::default()),
    EsVersion::EsNext,
  )
  .unwrap();
  let mut meta = ModuleMetaData::Script(Box::new(ScriptModuleMetaData {
    ast: parsed.ast,
    ..Default::default()
  }));
  plugin
    .process_module(
      &mut PluginProcessModuleHookParam {
        module_id: &id,
        module_type: &ModuleType::Js,
        meta: &mut meta,
        content: &mut source.clone(),
        source_map_chain: &mut vec![],
      },
      &context,
    )
    .unwrap();
  #[derive(Default)]
  struct Strings(Vec<String>);
  impl Visit for Strings {
    fn visit_str(&mut self, value: &Str) {
      self.0.push(value.value.to_string_lossy().into_owned());
    }
  }
  let mut strings = Strings::default();
  meta.as_script().ast.visit_with(&mut strings);
  assert_eq!(
    strings.0,
    vec![
      file.parent().unwrap().to_str().unwrap().to_owned(),
      file.to_str().unwrap().to_owned(),
      Url::from_file_path(file).unwrap().to_string()
    ]
  );
}

#[test]
fn configuration_paths_follow_compilation_root_instead_of_process_cwd() {
  let cwd = std::env::current_dir().unwrap();
  let root = cwd.join("isolated project");
  check_paths(&root, &root.join("farm.config.ts"));
}

#[test]
fn config_dependencies_outside_root_keep_their_real_file_url_without_query() {
  let cwd = std::env::current_dir().unwrap();
  let root = cwd.join("isolated project");
  check_paths(&root, &cwd.join("配置 directory").join("shared.ts"));
}
