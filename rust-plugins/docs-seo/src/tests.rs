use super::*;
use farmfe_core::{
  HashMap,
  config::{TargetEnv, persistent_cache::PersistentCacheConfig},
  context::CompilationContext,
  module::{Module, ModuleId},
  plugin::PluginFinalizeResourcesHookParam,
  resource::Resource,
};
use farmfe_docs_config::parse_config;
use serde_json::json;
use std::{fs, sync::Arc};

struct Fixture {
  root: tempfile::TempDir,
  config: Config,
  options: String,
}
impl Fixture {
  fn new() -> Self {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("docs")).unwrap();
    fs::write(root.path().join("docs/index.md"), "# Hook\n").unwrap();
    let raw = json!({"title":"Hooks","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}});
    let plan =
      farmfe_docs_plan::prepare(&parse_config(&raw.to_string(), root.path()).unwrap()).unwrap();
    let plan_path = root.path().join("plan.json");
    fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let mut config = Config::default();
    config.root = root.path().to_str().unwrap().replace('\\', "/");
    config.output.target_env = TargetEnv::Browser;
    config.persistent_cache = Box::new(PersistentCacheConfig::Bool(false));
    let options = json!({"planPath":plan_path.to_str().unwrap()}).to_string();
    Self {
      root,
      config,
      options,
    }
  }
  fn context(&self) -> Arc<CompilationContext> {
    let context = Arc::new(CompilationContext::new(self.config.clone(), vec![]).unwrap());
    let entry = ModuleId::new(
      &format!("{}/entry.js", self.config.root),
      "",
      &self.config.root,
    );
    let mut graph = context.module_graph.write();
    graph.add_module(Module::new(entry.clone()));
    graph.entries.insert(entry, "app".into());
    drop(graph);
    context
  }
}

#[test]
fn browser_hooks_emit_real_resources_before_the_docs_manifest() {
  let fixture = Fixture::new();
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options);
  assert!(
    plugin.priority() > 100,
    "SEO resources must exist before Docs manifest collection"
  );
  plugin.build_start(&context).unwrap();
  let mut resources = HashMap::default();
  plugin
    .finalize_resources(
      &mut PluginFinalizeResourcesHookParam {
        resources_map: &mut resources,
        config: &context.config,
      },
      &context,
    )
    .unwrap();
  assert_eq!(resources.len(), 2);
  assert!(
    String::from_utf8(resources["robots.txt"].bytes.clone())
      .unwrap()
      .contains("https://example.com/guide/sitemap.xml")
  );
  assert!(
    resources
      .values()
      .all(|resource| !resource.emitted && !resource.should_transform_output_filename)
  );
}

#[test]
fn repeated_finalization_replaces_own_resources_without_collisions() {
  let fixture = Fixture::new();
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options);
  plugin.build_start(&context).unwrap();
  let mut resources = HashMap::default();
  plugin
    .finalize_resources(
      &mut PluginFinalizeResourcesHookParam {
        resources_map: &mut resources,
        config: &context.config,
      },
      &context,
    )
    .unwrap();
  let before: std::collections::BTreeMap<_, _> = resources
    .iter()
    .map(|(name, resource)| (name.clone(), resource.bytes.clone()))
    .collect();
  plugin.finalize_resources(
    &mut PluginFinalizeResourcesHookParam { resources_map: &mut resources, config: &context.config },
    &context,
  ).expect("a plugin must replace its own previous generation without treating it as a foreign collision");
  let after: std::collections::BTreeMap<_, _> = resources
    .iter()
    .map(|(name, resource)| (name.clone(), resource.bytes.clone()))
    .collect();
  assert_eq!(before, after);
}

#[test]
fn server_target_does_not_duplicate_browser_seo_resources() {
  let mut fixture = Fixture::new();
  fixture.config.output.target_env = TargetEnv::Node;
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options);
  plugin.build_start(&context).unwrap();
  let mut resources = HashMap::default();
  plugin
    .finalize_resources(
      &mut PluginFinalizeResourcesHookParam {
        resources_map: &mut resources,
        config: &context.config,
      },
      &context,
    )
    .unwrap();
  assert!(resources.is_empty());
}

#[test]
fn malformed_options_bad_plan_and_changed_sources_fail_in_actual_hooks() {
  for options in ["{}", r#"{"planPath":"missing.json","unknown":true}"#] {
    let fixture = Fixture::new();
    let plugin = FarmPluginDocsSeo::new(&fixture.config, options.into());
    assert!(plugin.build_start(&fixture.context()).is_err());
  }
  let fixture = Fixture::new();
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options.clone());
  fs::write(fixture.root.path().join("plan.json"), "{}").unwrap();
  assert!(plugin.build_start(&context).is_err());
  let fixture = Fixture::new();
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options);
  plugin.build_start(&context).unwrap();
  fs::write(fixture.root.path().join("docs/index.md"), "# Changed\n").unwrap();
  let mut resources = HashMap::default();
  assert!(
    plugin
      .finalize_resources(
        &mut PluginFinalizeResourcesHookParam {
          resources_map: &mut resources,
          config: &context.config
        },
        &context
      )
      .is_err()
  );
  assert!(resources.is_empty());
}

#[test]
fn resource_case_and_ancestor_conflicts_fail_before_any_insertion() {
  for name in ["SITEMAP.xml", "sitemap.xml/chunk.js", "robots.txt"] {
    let fixture = Fixture::new();
    let context = fixture.context();
    let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options);
    plugin.build_start(&context).unwrap();
    let mut resources = HashMap::default();
    resources.insert(
      name.into(),
      Resource {
        name: name.into(),
        bytes: b"existing".to_vec(),
        ..Default::default()
      },
    );
    let error = plugin
      .finalize_resources(
        &mut PluginFinalizeResourcesHookParam {
          resources_map: &mut resources,
          config: &context.config,
        },
        &context,
      )
      .unwrap_err();
    assert!(error.to_string().contains("COLLISION"), "{error}");
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[name].bytes, b"existing");
  }
}

fn finalize(
  plugin: &FarmPluginDocsSeo,
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

fn resource_state(
  resources: &HashMap<String, Resource>,
) -> std::collections::BTreeMap<String, (String, Vec<u8>, std::collections::BTreeMap<String, String>)>
{
  resources
    .iter()
    .map(|(name, resource)| {
      (
        name.clone(),
        (
          resource.name.clone(),
          resource.bytes.clone(),
          resource
            .meta
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
        ),
      )
    })
    .collect()
}

#[test]
fn ownership_survives_plugin_recreation_and_removes_retired_output_only() {
  let fixture = Fixture::new();
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options.clone());
  plugin.build_start(&context).unwrap();
  let mut resources = HashMap::default();
  finalize(&plugin, &context, &mut resources).unwrap();
  let name = "retired.xml";
  let bytes = b"old feed".to_vec();
  let mut resource = Resource {
    name: name.into(),
    bytes,
    ..Default::default()
  };
  resource.meta.insert(
    resource_owner::META_KEY.into(),
    resource_owner::stamp(plugin.name(), name, &resource.bytes).unwrap(),
  );
  resources.insert(name.into(), resource);
  resources.insert(
    "foreign.js".into(),
    Resource {
      name: "foreign.js".into(),
      bytes: b"foreign".to_vec(),
      ..Default::default()
    },
  );
  let recreated = FarmPluginDocsSeo::new(&fixture.config, fixture.options);
  recreated.build_start(&context).unwrap();
  finalize(&recreated, &context, &mut resources).unwrap();
  assert!(!resources.contains_key(name));
  assert_eq!(resources["foreign.js"].bytes, b"foreign");
  assert_eq!(resources.len(), 3);
}

#[test]
fn missing_changed_or_malformed_ownership_never_overwrites_a_collision() {
  for mutation in [
    "missing",
    "malformed",
    "bytes",
    "name",
    "owner",
    "version",
    "key",
  ] {
    let fixture = Fixture::new();
    let context = fixture.context();
    let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options);
    plugin.build_start(&context).unwrap();
    let mut resources = HashMap::default();
    finalize(&plugin, &context, &mut resources).unwrap();
    let resource = resources.get_mut("robots.txt").unwrap();
    match mutation {
      "missing" => resource.meta.clear(),
      "malformed" => {
        resource
          .meta
          .insert(resource_owner::META_KEY.into(), "{}".into());
      }
      "bytes" => resource.bytes.push(b'!'),
      "name" => resource.name = "other.txt".into(),
      "owner" => {
        resource.meta.insert(
          resource_owner::META_KEY.into(),
          resource_owner::stamp("OtherPlugin", "robots.txt", &resource.bytes).unwrap(),
        );
      }
      "version" => {
        let marker = resource.meta.get_mut(resource_owner::META_KEY).unwrap();
        *marker = marker.replace("\"version\":1", "\"version\":2");
      }
      "key" => {
        let resource = resources.remove("robots.txt").unwrap();
        resources.insert("ROBOTS.txt".into(), resource);
      }
      _ => unreachable!(),
    }
    let before = resource_state(&resources);
    let error = finalize(&plugin, &context, &mut resources).unwrap_err();
    assert!(
      error.to_string().contains("COLLISION"),
      "{mutation}: {error}"
    );
    assert_eq!(resource_state(&resources), before, "{mutation}");
  }
}

#[test]
fn failed_generation_preserves_previous_resources_then_recovers() {
  let fixture = Fixture::new();
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, fixture.options);
  plugin.build_start(&context).unwrap();
  let mut resources = HashMap::default();
  finalize(&plugin, &context, &mut resources).unwrap();
  let before = resource_state(&resources);
  fs::write(
    fixture.root.path().join("docs/index.md"),
    "# Broken generation\n",
  )
  .unwrap();
  assert!(finalize(&plugin, &context, &mut resources).is_err());
  assert_eq!(resource_state(&resources), before);
  fs::write(fixture.root.path().join("docs/index.md"), "# Hook\n").unwrap();
  finalize(&plugin, &context, &mut resources).unwrap();
  assert_eq!(resource_state(&resources), before);
}

#[test]
fn development_reads_current_rust_context_plan_each_generation() {
  let fixture = Fixture::new();
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, json!({"development":true}).to_string());
  plugin
    .build_start(&context)
    .expect("development SEO does not read a private plan file");
  let mut resources = HashMap::default();
  assert!(finalize(&plugin, &context, &mut resources).is_err());
  assert!(resources.is_empty());
  context.custom.lock().insert(
    farmfe_docs_plan::DEVELOPMENT_PLAN_CONTEXT_KEY.into(),
    Box::new(fs::read_to_string(fixture.root.path().join("plan.json")).unwrap()),
  );
  finalize(&plugin, &context, &mut resources).unwrap();
  let before = resources["sitemap.xml"].bytes.clone();
  fs::write(
    fixture.root.path().join("docs/index.md"),
    "---\nslug: moved\n---\n# Moved\n",
  )
  .unwrap();
  assert!(
    finalize(&plugin, &context, &mut resources).is_err(),
    "old plan cannot describe changed inputs"
  );
  assert_eq!(resources["sitemap.xml"].bytes, before);
  let site = json!({"title":"Hooks","url":"https://example.com","basePath":"/guide/","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}});
  let plan =
    farmfe_docs_plan::prepare(&parse_config(&site.to_string(), fixture.root.path()).unwrap())
      .unwrap();
  context.custom.lock().insert(
    farmfe_docs_plan::DEVELOPMENT_PLAN_CONTEXT_KEY.into(),
    Box::new(serde_json::to_string(&plan).unwrap()),
  );
  finalize(&plugin, &context, &mut resources).unwrap();
  assert_ne!(resources["sitemap.xml"].bytes, before);
  assert!(
    String::from_utf8(resources["sitemap.xml"].bytes.clone())
      .unwrap()
      .contains("/guide/docs/moved/")
  );
  let stable = resource_state(&resources);
  context.custom.lock().insert(
    farmfe_docs_plan::DEVELOPMENT_PLAN_CONTEXT_KEY.into(),
    Box::new("{}".to_string()),
  );
  assert!(finalize(&plugin, &context, &mut resources).is_err());
  assert_eq!(resource_state(&resources), stable);
}

#[test]
fn development_syndication_resources_are_owned_atomic_and_removed_when_disabled() {
  let fixture = Fixture::new();
  fs::create_dir(fixture.root.path().join("pages")).unwrap();
  fs::write(
    fixture.root.path().join("pages/search.tsx"),
    "export const metadata={title:'Search'};export default ()=> <h1>Search</h1>;",
  )
  .unwrap();
  fs::create_dir(fixture.root.path().join("blog")).unwrap();
  fs::write(fixture.root.path().join("blog/post.md"), "# Undated\n").unwrap();
  let mut raw = json!({"title":"Hooks","url":"https://example.com","pages":{"dir":"pages"},"locales":{"default":"en","items":["en","zh"]},
    "blog":{"collections":[{"id":"en-news","dir":"blog","locale":"en","routeBase":"/blog/"},{"id":"zh-news","dir":"blog","locale":"zh","routeBase":"/zh/blog/"}]},
    "seo":{"atom":{"updated":"2026-10-18T00:00:00Z","author":"Contributors"}},
    "search":{"provider":"algolia","options":{"appId":"APP","apiKey":"public","indexName":"docs"},"openSearch":{"pageId":"search","shortName":"Docs","description":"Search docs"}}});
  let context = fixture.context();
  let plugin = FarmPluginDocsSeo::new(&fixture.config, json!({"development":true}).to_string());
  plugin.build_start(&context).unwrap();
  let publish = |raw: &serde_json::Value| {
    let plan =
      farmfe_docs_plan::prepare(&parse_config(&raw.to_string(), fixture.root.path()).unwrap())
        .unwrap();
    context.custom.lock().insert(
      farmfe_docs_plan::DEVELOPMENT_PLAN_CONTEXT_KEY.into(),
      Box::new(serde_json::to_string(&plan).unwrap()),
    );
  };
  publish(&raw);
  let mut resources = HashMap::default();
  finalize(&plugin, &context, &mut resources).unwrap();
  for name in [
    "blog/atom.xml",
    "zh/blog/atom.xml",
    "opensearch.xml",
    "zh/opensearch.xml",
  ] {
    assert!(resources.contains_key(name));
  }
  resources.insert(
    "zh/opensearch.xml".into(),
    Resource {
      name: "zh/opensearch.xml".into(),
      bytes: b"user-owned collision".to_vec(),
      ..Default::default()
    },
  );
  let collided = resource_state(&resources);
  assert!(finalize(&plugin, &context, &mut resources).is_err());
  assert_eq!(resource_state(&resources), collided);
  resources.remove("zh/opensearch.xml");
  finalize(&plugin, &context, &mut resources).unwrap();
  raw.as_object_mut().unwrap().remove("search");
  raw.as_object_mut().unwrap().remove("seo");
  publish(&raw);
  finalize(&plugin, &context, &mut resources).unwrap();
  assert!(
    !resources
      .keys()
      .any(|name| name.ends_with("atom.xml") || name.ends_with("opensearch.xml"))
  );
  assert!(resources.contains_key("blog/rss.xml") && resources.contains_key("zh/blog/rss.xml"));
}
