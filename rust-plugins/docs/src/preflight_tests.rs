use super::*;
use farmfe_core::{HashMap, config::TargetEnv};

fn fixture(preflight: bool, test: impl FnOnce(&FarmPluginDocs, &Arc<CompilationContext>)) {
  let owner = super::preflight_fixture::FixtureRoot::new().unwrap();
  let root = owner.path();
  fs::create_dir(root.join("docs")).unwrap();
  fs::write(root.join("docs/index.md"), "# Preflight\n").unwrap();
  let mut config = Config::default();
  config.root = root.to_string_lossy().into_owned();
  config.output.target_env = TargetEnv::Node;
  let plugin = FarmPluginDocs::new(&config, json!({"site":{"title":"Preflight","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}},"prepare":preflight}).to_string());
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  plugin.build_start(&context).unwrap();
  test(&plugin, &context);
}

fn finalize(
  plugin: &FarmPluginDocs,
  context: &Arc<CompilationContext>,
  resources: &mut HashMap<String, Resource>,
) -> Result<Option<()>> {
  plugin.finalize_resources(
    &mut PluginFinalizeResourcesHookParam {
      resources_map: resources,
      config: &context.config,
    },
    context,
  )
}

#[test]
fn site_and_preflight_early_returns_replace_only_valid_own_outputs() {
  for preflight in [false, true] {
    fixture(preflight, |plugin, context| {
      let mut resources = HashMap::default();
      finalize(plugin, context, &mut resources).unwrap();
      let before: std::collections::BTreeMap<_, _> = resources
        .iter()
        .map(|(name, resource)| (name.clone(), resource.bytes.clone()))
        .collect();
      finalize(plugin, context, &mut resources).unwrap();
      assert_eq!(resources.len(), if preflight { 2 } else { 1 });
      for (name, bytes) in before {
        assert_eq!(resources[&name].bytes, bytes);
      }
      for resource in resources.values() {
        assert!(farmfe_docs_plan::resource_owner::is_owned(
          resource
            .meta
            .get(farmfe_docs_plan::resource_owner::META_KEY)
            .map(String::as_str),
          plugin.name(),
          &resource.name,
          &resource.bytes
        ));
      }
      for name in if preflight {
        vec![
          "docs-public-config.json",
          "DOCS-PUBLIC-config.json",
          "docs-plan.json/foreign.js",
          "DOCS-PLAN.json",
        ]
      } else {
        vec![
          "docs-public-config.json",
          "DOCS-PUBLIC-config.json",
          "docs-public-config.json/foreign.js",
        ]
      } {
        let mut resources = HashMap::default();
        resources.insert(
          name.into(),
          Resource {
            name: name.into(),
            bytes: b"foreign".to_vec(),
            ..Default::default()
          },
        );
        let error = finalize(plugin, context, &mut resources).unwrap_err();
        assert!(error.to_string().contains("COLLISION"), "{error}");
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[name].bytes, b"foreign");
      }
    });
  }
}

#[test]
fn preflight_failure_after_public_config_creation_does_not_publish_partial_map() {
  fixture(true, |plugin, context| {
    let mut resources = HashMap::default();
    finalize(plugin, context, &mut resources).unwrap();
    let before = resources.clone();
    let plan = plugin.plan.write().unwrap().take();
    assert!(finalize(plugin, context, &mut resources).is_err());
    assert_eq!(resources.len(), before.len());
    for (name, resource) in &before {
      assert_eq!(resources[name].bytes, resource.bytes);
      assert_eq!(resources[name].meta, resource.meta);
    }
    *plugin.plan.write().unwrap() = plan;
    finalize(plugin, context, &mut resources).unwrap();
    for (name, resource) in before {
      assert_eq!(resources[&name].bytes, resource.bytes);
    }
  });
}
