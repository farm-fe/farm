use super::*;
use farmfe_core::{
  config::persistent_cache::PersistentCacheConfig,
  module::ModuleId,
  resource::ResourceOrigin,
  swc_ecma_parser::{Syntax, TsSyntax},
};
use farmfe_toolkit::script::parse_module;

#[test]
fn type_only_dependencies_are_emitted_watched_and_refreshed() {
  let root = std::env::temp_dir().join(format!("farm-dts-types-{}", std::process::id()));
  std::fs::create_dir(&root).unwrap();
  let model = root.join("models.ts");
  std::fs::write(&model, "export type { Product as SelfProduct } from './models.js'; export interface Product { price: number }").unwrap();
  let config = Config {
    root: root.to_string_lossy().to_string(),
    persistent_cache: Box::new(PersistentCacheConfig::Bool(false)),
    ..Default::default()
  };
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  let owner = ModuleId::new(
    &root.join("index.ts").to_string_lossy(),
    "",
    &context.config.root,
  );
  let ast = parse_module(
    &owner,
    Arc::new("export type { Product } from './models.js';".into()),
    Syntax::Typescript(TsSyntax::default()),
    context.config.script.target,
  )
  .unwrap()
  .ast;
  emit_type_dependencies(&ast, &owner, &FarmPluginDtsOptions::default(), &context).unwrap();
  {
    let resources = context.resources_map.lock();
    let resource = resources.get("models.d.ts").unwrap();
    assert!(String::from_utf8_lossy(&resource.bytes).contains("price: number"));
    match &resource.origin {
      ResourceOrigin::Module(id) => assert_eq!(id, &owner),
      _ => panic!("wrong declaration owner"),
    }
  }
  let model_id = ModuleId::new(&model.to_string_lossy(), "", &context.config.root);
  assert!(
    context
      .watch_graph
      .read()
      .relation_roots(&model_id)
      .contains(&&owner)
  );
  std::fs::write(
    &model,
    "export interface Product { price: number; cacheUpdateProbe?: string }",
  )
  .unwrap();
  emit_type_dependencies(&ast, &owner, &FarmPluginDtsOptions::default(), &context).unwrap();
  assert!(
    String::from_utf8_lossy(
      &context
        .resources_map
        .lock()
        .get("models.d.ts")
        .unwrap()
        .bytes
    )
    .contains("cacheUpdateProbe")
  );
  std::fs::remove_file(model).unwrap();
  std::fs::remove_dir(root).unwrap();
}
