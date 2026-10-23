use super::*;
use farmfe_core::plugin::UpdateType;

#[test]
fn development_manifest_routes_follow_add_delete_and_reject_invalid_replacement() {
  fixture(|plugin, context, root| {
    context
      .module_graph
      .write()
      .entries
      .insert("entry".into(), "entry".into());
    let routes = || {
      let mut resources = farmfe_core::HashMap::default();
      plugin
        .finalize_resources(
          &mut PluginFinalizeResourcesHookParam {
            resources_map: &mut resources,
            config: &context.config,
          },
          context,
        )
        .unwrap();
      let manifest: serde_json::Value =
        serde_json::from_slice(&resources["docs-manifest.json"].bytes).unwrap();
      manifest["routePaths"]
        .as_array()
        .expect("Native manifest must publish route table")
        .clone()
    };
    assert_eq!(routes(), vec![json!("/docs/")]);
    let added = root.join("docs/added.mdx");
    fs::write(&added, "# Added\n").unwrap();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(added.to_string_lossy().into_owned(), UpdateType::Added)],
    };
    plugin.update_modules(&mut update, context).unwrap();
    assert!(routes().contains(&json!("/docs/added/")));
    fs::write(&added, "import {\n").unwrap();
    update.paths[0].1 = UpdateType::Updated;
    assert!(plugin.update_modules(&mut update, context).is_err());
    assert!(
      routes().contains(&json!("/docs/added/")),
      "invalid prepare retains last good routes"
    );
    fs::remove_file(&added).unwrap();
    update.paths[0].1 = UpdateType::Removed;
    plugin.update_modules(&mut update, context).unwrap();
    assert_eq!(routes(), vec![json!("/docs/")]);
  });
}

#[test]
fn full_blog_wrapper_keeps_original_article_edges_for_css_and_lazy_content() {
  fixture(|_, context, root| {
    fs::create_dir(root.join("blog")).unwrap();
    fs::write(root.join("blog/first.md"), "# First\n").unwrap();
    fs::write(root.join("blog/second.md"), "# Second\n").unwrap();
    let plugin = FarmPluginDocs::new(&context.config, json!({"site":{"title":"Full blog","url":"https://example.com","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/","blogListContent":"full"}]}},"development":true}).to_string());
    plugin.build_start(context).unwrap();
    let plan = plugin.plan.read().unwrap().as_ref().unwrap().clone();
    for route in &plan.routes {
      let wrapper = page_resources::entry_path(&route.page.key);
      let loaded = plugin
        .load(
          &PluginLoadHookParam {
            resolved_path: &wrapper,
            module_id: wrapper.clone(),
            query: vec![],
            meta: Default::default(),
          },
          context,
          &PluginHookContext::default(),
        )
        .unwrap()
        .unwrap();
      if let Some(view) = route
        .page
        .blog_view
        .as_ref()
        .filter(|view| view.full_content)
      {
        assert_eq!(view.posts.len(), 2);
        for (index, key) in view.posts.iter().enumerate() {
          assert!(
            loaded
              .content
              .contains(&format!("default as listingPost{index}"))
          );
          assert!(loaded.content.contains(&format!("docs:content:{key}")));
        }
      } else {
        assert!(!loaded.content.contains("listingPost"));
      }
    }
  });
}

#[test]
fn full_blog_wrapper_invalidates_when_article_identity_changes_without_visible_index_changes() {
  fixture(|_, context, root| {
    fs::create_dir(root.join("blog")).unwrap();
    let source = root.join("blog/post.md");
    let body =
      |id| format!("---\nid: {id}\ntitle: Stable\nslug: stable\ndate: 2025-01-02\n---\n# Stable\n");
    fs::write(&source, body("old")).unwrap();
    let plugin = FarmPluginDocs::new(&context.config, json!({"site":{"title":"Full blog","url":"https://example.com","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/","blogListContent":"full"}]}},"development":true}).to_string());
    plugin.build_start(context).unwrap();
    let previous = plugin.plan.read().unwrap().as_ref().unwrap().clone();
    let listing = previous
      .routes
      .iter()
      .find(|route| route.page.kind == "blog-index")
      .unwrap();
    let listing_key = listing.page.key.clone();
    let wrapper_id = ModuleId::new(
      &page_resources::entry_path(&listing_key),
      "",
      &context.config.root,
    );
    context
      .module_graph
      .write()
      .add_module(farmfe_core::module::Module::new(wrapper_id.clone()));
    fs::write(&source, body("new")).unwrap();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(source.to_string_lossy().into_owned(), UpdateType::Updated)],
    };
    plugin.update_modules(&mut update, context).unwrap();
    let next = plugin.plan.read().unwrap().as_ref().unwrap().clone();
    let old_module = previous
      .modules
      .iter()
      .find(|module| module.key == listing_key)
      .unwrap();
    let next_module = next
      .modules
      .iter()
      .find(|module| module.key == listing_key)
      .unwrap();
    assert_eq!(
      old_module.code, next_module.code,
      "visible generated link body does not encode identity changes"
    );
    assert_ne!(
      listing.page.blog_view.as_ref().unwrap().posts,
      next
        .routes
        .iter()
        .find(|route| route.page.key == listing_key)
        .unwrap()
        .page
        .blog_view
        .as_ref()
        .unwrap()
        .posts
    );
    assert!(
      update
        .paths
        .iter()
        .any(
          |(path, _)| ModuleId::from_resolved_path_with_query(path, &context.config.root)
            == wrapper_id
        ),
      "full listing wrapper must rebuild its changed static article edges"
    );
    let wrapper_path = page_resources::entry_path(&listing_key);
    let loaded = plugin
      .load(
        &PluginLoadHookParam {
          resolved_path: &wrapper_path,
          module_id: wrapper_path.clone(),
          query: vec![],
          meta: Default::default(),
        },
        context,
        &PluginHookContext::default(),
      )
      .unwrap()
      .unwrap();
    for key in &listing.page.blog_view.as_ref().unwrap().posts {
      assert!(
        !loaded.content.contains(&format!("docs:content:{key}")),
        "reloaded listing must not retain removed article identities"
      );
    }
    let next_listing = next
      .routes
      .iter()
      .find(|route| route.page.key == listing_key)
      .unwrap();
    for (index, key) in next_listing
      .page
      .blog_view
      .as_ref()
      .unwrap()
      .posts
      .iter()
      .enumerate()
    {
      assert!(loaded.content.contains(&format!(
        "default as listingPost{index} }} from {}",
        serde_json::to_string(&format!("docs:content:{key}")).unwrap()
      )));
    }
    // A later graph failure must not lose this wrapper merely because the
    // working plan already contains the new article identity.
    let unrelated = root.join("component.tsx").to_string_lossy().into_owned();
    update.paths = vec![(unrelated.clone(), UpdateType::Updated)];
    plugin.update_modules(&mut update, context).unwrap();
    assert!(update.paths.iter().any(|(path, _)| {
      ModuleId::from_resolved_path_with_query(path, &context.config.root) == wrapper_id
    }));
    plugin.update_finished(context).unwrap();
    update.paths = vec![(unrelated.clone(), UpdateType::Updated)];
    plugin.update_modules(&mut update, context).unwrap();
    assert_eq!(update.paths.len(), 1);
    assert_eq!(update.paths[0].0, unrelated);
  });
}

fn fixture(test: impl FnOnce(&FarmPluginDocs, &Arc<CompilationContext>, &Path)) {
  let owner = crate::preflight_fixture::FixtureRoot::new().unwrap();
  let root = owner.path();
  fs::create_dir(root.join("docs")).unwrap();
  fs::write(root.join("docs/index.md"), "# Initial development body\n").unwrap();
  let mut config = Config::default();
  config.root = root.to_string_lossy().into_owned();
  let plugin = FarmPluginDocs::new(&config, json!({"site":{"title":"Development","url":"https://example.com","docs":{"collections":[{"id":"main","dir":"docs","routeBase":"/docs/"}]}},"development":true}).to_string());
  let context = Arc::new(CompilationContext::new(config, vec![]).unwrap());
  plugin
    .build_start(&context)
    .expect("Rust development mode prepares its own initial content plan");
  test(&plugin, &context, root);
}

#[test]
fn development_replaces_obsolete_asset_query_without_suppressing_bare_imports() {
  fixture(|plugin, context, root| {
    let source = root.join("docs/index.md");
    let image = root.join("docs/diagram.svg");
    fs::write(
      &image,
      "<svg xmlns=\"http://www.w3.org/2000/svg\"><rect fill=\"red\"/></svg>",
    )
    .unwrap();
    fs::write(&source, "# Image\n\n![Diagram](./diagram.svg)\n").unwrap();
    plugin
      .update_modules(
        &mut PluginUpdateModulesHookParam {
          paths: vec![(source.to_string_lossy().into_owned(), UpdateType::Updated)],
        },
        context,
      )
      .unwrap();
    plugin.update_finished(context).unwrap();
    let old = plugin.plan.read().unwrap().as_ref().unwrap().assets[0].clone();
    let old_query = ModuleId::new(
      &old.source_path,
      &format!("?farm_docs_asset={}", old.key),
      &context.config.root,
    )
    .resolved_path_with_query(&context.config.root);
    fs::write(
      &image,
      "<svg xmlns=\"http://www.w3.org/2000/svg\"><rect fill=\"blue\"/></svg>",
    )
    .unwrap();
    let mut params = PluginUpdateModulesHookParam {
      paths: vec![
        (old_query.clone(), UpdateType::Updated),
        (old.source_path.clone(), UpdateType::Updated),
      ],
    };
    plugin.update_modules(&mut params, context).unwrap();
    let next = plugin.plan.read().unwrap().as_ref().unwrap().assets[0].clone();
    assert_ne!(old.key, next.key);
    assert!(
      !params.paths.iter().any(|(path, _)| path == &old_query),
      "obsolete content-addressed asset query must not load against the new plan"
    );
    assert!(
      params
        .paths
        .iter()
        .any(|(path, _)| path == &old.source_path),
      "another plugin's bare asset import must remain updatable"
    );
    // A failed native batch can leave A in the graph while the working plan
    // already contains B. A repair to C must not try loading A against C.
    context
      .module_graph
      .write()
      .add_module(farmfe_core::module::Module::new(
        ModuleId::from_resolved_path_with_query(&old_query, &context.config.root),
      ));
    fs::write(
      &image,
      "<svg xmlns=\"http://www.w3.org/2000/svg\"><rect fill=\"green\"/></svg>",
    )
    .unwrap();
    let unknown = format!("{}?farm_docs_asset=unknown", old.source_path);
    let mut retry = PluginUpdateModulesHookParam {
      paths: vec![
        (old_query.clone(), UpdateType::Updated),
        (old.source_path.clone(), UpdateType::Updated),
        (unknown.clone(), UpdateType::Updated),
      ],
    };
    plugin.update_modules(&mut retry, context).unwrap();
    assert!(!retry.paths.iter().any(|(path, _)| path == &old_query));
    assert!(retry.paths.iter().any(|(path, _)| path == &old.source_path));
    assert!(
      retry.paths.iter().any(|(path, _)| path == &unknown),
      "unknown queries must still reach strict load validation"
    );
  });
}

#[test]
fn development_html_entry_is_resolved_and_loaded_by_rust() {
  fixture(|plugin, context, _root| {
    let resolved = plugin
      .resolve(
        &PluginResolveHookParam {
          source: "./docs:dev".into(),
          importer: None,
          kind: ResolveKind::Entry("index".into()),
        },
        context,
        &PluginHookContext::default(),
      )
      .unwrap()
      .expect("Rust must resolve the development HTML entry");
    assert_eq!(resolved.resolved_path, "virtual:farm-docs:development");
    let loaded = plugin
      .load(
        &PluginLoadHookParam {
          resolved_path: &resolved.resolved_path,
          query: vec![],
          meta: Default::default(),
          module_id: resolved.resolved_path.clone(),
        },
        context,
        &PluginHookContext::default(),
      )
      .unwrap()
      .expect("Rust must load the development HTML entry");
    assert_eq!(loaded.module_type, ModuleType::Html);
    assert!(loaded.content.contains("id=\"__FARM_DOCS_ROOT__\""));
    assert!(loaded.content.contains("@farmfe/docs/dev-client"));
    assert!(!loaded.content.contains("__FARM_DOCS_DATA__"));
  });
}

#[test]
fn development_prepares_initial_plan_for_browser_without_private_preflight() {
  fixture(|plugin, _context, _root| {
    let guard = plugin.plan.read().unwrap();
    let plan = guard.as_ref().expect("initial Rust plan");
    assert_eq!(plan.modules.len(), 1);
    assert!(plan.modules[0].code.contains("Initial development body"));
    assert!(!plan.snapshot_id.is_empty());
  });
}

#[test]
fn development_refreshes_content_and_registry_from_real_source_in_rust() {
  fixture(|plugin, context, root| {
    let previous = plugin
      .plan
      .read()
      .unwrap()
      .as_ref()
      .unwrap()
      .snapshot_id
      .clone();
    let source = root.join("docs/index.md");
    fs::write(&source, "# Updated development body\n").unwrap();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(source.to_string_lossy().into_owned(), UpdateType::Updated)],
    };
    plugin.update_modules(&mut update, context).unwrap();
    let guard = plugin.plan.read().unwrap();
    let plan = guard.as_ref().unwrap();
    assert_ne!(plan.snapshot_id, previous);
    assert!(plan.modules[0].code.contains("Updated development body"));
    assert!(
      update
        .paths
        .iter()
        .any(|(path, kind)| path == APP_VIRTUAL_ID && matches!(kind, UpdateType::Updated)),
      "the generated registry must join the same native update batch"
    );
  });
}

#[test]
fn development_bad_candidate_retains_working_plan_and_input_batch_then_recovers() {
  fixture(|plugin, context, root| {
    let before = plugin
      .plan
      .read()
      .unwrap()
      .as_ref()
      .unwrap()
      .snapshot_id
      .clone();
    let source = root.join("docs/index.md");
    fs::write(&source, "# Invalid\n\n[Broken](./missing.md)\n").unwrap();
    let path = source.to_string_lossy().into_owned();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(path.clone(), UpdateType::Updated)],
    };
    assert!(plugin.update_modules(&mut update, context).is_err());
    assert_eq!(
      plugin.plan.read().unwrap().as_ref().unwrap().snapshot_id,
      before
    );
    assert_eq!(update.paths.len(), 1);
    assert_eq!(update.paths[0].0, path);
    fs::write(&source, "# Recovered body\n").unwrap();
    plugin.update_modules(&mut update, context).unwrap();
    assert!(
      plugin.plan.read().unwrap().as_ref().unwrap().modules[0]
        .code
        .contains("Recovered body")
    );
  });
}

#[test]
fn development_includes_loaded_content_query_in_same_registry_update() {
  fixture(|plugin, context, root| {
    let module = plugin.plan.read().unwrap().as_ref().unwrap().modules[0].clone();
    let id = ModuleId::new(
      &module.source_path,
      &format!("?farm_docs={}", module.key),
      &context.config.root,
    );
    context
      .module_graph
      .write()
      .add_module(farmfe_core::module::Module::new(id.clone()));
    fs::write(root.join("docs/index.md"), "# Loaded update\n").unwrap();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(module.source_path.clone(), UpdateType::Updated)],
    };
    plugin.update_modules(&mut update, context).unwrap();
    assert!(
      update
        .paths
        .iter()
        .any(|(path, _)| path == &id.resolved_path_with_query(&context.config.root))
    );
    assert!(
      !update
        .paths
        .iter()
        .any(|(path, _)| path == &module.source_path),
      "source={:?}, resolved={:?}, batch={:?}",
      module.source_path,
      ModuleId::from_resolved_path_with_query(&module.source_path, &context.config.root)
        .resolved_path(&context.config.root),
      update.paths
    );
  });
}

#[test]
fn generated_blog_update_paths_resolve_to_the_same_owned_content_identity() {
  fixture(|plugin, context, root| {
    fs::create_dir(root.join("blog")).unwrap();
    fs::write(root.join("blog/post.md"), "# Post\n").unwrap();
    let raw = json!({"title":"Generated updates","url":"https://example.com","blog":{"collections":[{"id":"news","dir":"blog","routeBase":"/blog/"}]}});
    let plan = prepare(&parse_config(&raw.to_string(), root).unwrap()).unwrap();
    let module = plan
      .modules
      .iter()
      .find(|module| module.source_path.starts_with("farm-docs:generated:"))
      .unwrap()
      .clone();
    *plugin.plan.write().unwrap() = Some(plan);
    let id = ModuleId::new(
      &module.source_path,
      &format!("?farm_docs={}", module.key),
      &context.config.root,
    );
    let resolve = |source: String| {
      plugin
        .resolve(
          &PluginResolveHookParam {
            source,
            importer: None,
            kind: ResolveKind::Entry("page".into()),
          },
          context,
          &PluginHookContext::default(),
        )
        .unwrap()
    };
    let resolved = resolve(id.resolved_path_with_query(&context.config.root))
      .expect("native HMR re-resolves the rooted generated content query");
    assert_eq!(resolved.resolved_path, module.source_path);
    assert_eq!(
      resolved.query,
      vec![("farm_docs".to_string(), module.key.clone())]
    );
    assert!(
      resolve(format!(
        "{}?farm_docs=wrong",
        id.resolved_path(&context.config.root)
      ))
      .is_none()
    );
    assert!(
      resolve(format!(
        "{}-unrelated?farm_docs={}",
        id.resolved_path(&context.config.root),
        module.key
      ))
      .is_none()
    );
  });
}

#[test]
fn development_invalidates_loaded_lazy_wrapper_with_its_changed_content() {
  fixture(|plugin, context, root| {
    let module = plugin.plan.read().unwrap().as_ref().unwrap().modules[0].clone();
    let wrapper = page_resources::entry_path(&module.key);
    let wrapper_id = ModuleId::new(&wrapper, "", &context.config.root);
    let content_id = ModuleId::new(
      &module.source_path,
      &format!("?farm_docs={}", module.key),
      &context.config.root,
    );
    {
      let mut graph = context.module_graph.write();
      graph.add_module(farmfe_core::module::Module::new(wrapper_id.clone()));
      graph.add_module(farmfe_core::module::Module::new(content_id.clone()));
    }
    fs::write(
      root.join("docs/index.md"),
      "# Current candidate implementation\n",
    )
    .unwrap();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(module.source_path.clone(), UpdateType::Updated)],
    };
    plugin.update_modules(&mut update, context).unwrap();
    for expected in [wrapper_id.clone(), content_id.clone()] {
      assert!(
        update.paths.iter().any(|(path, kind)| {
          matches!(kind, UpdateType::Updated)
            && ModuleId::from_resolved_path_with_query(path, &context.config.root) == expected
        }),
        "loaded wrapper and content must join one native update: {expected:?}"
      );
    }
    let mut retry = PluginUpdateModulesHookParam {
      paths: vec![(
        root.join("component.tsx").to_string_lossy().into_owned(),
        UpdateType::Updated,
      )],
    };
    plugin.update_modules(&mut retry, context).unwrap();
    assert!(
      retry
        .paths
        .iter()
        .any(
          |(path, _)| ModuleId::from_resolved_path_with_query(path, &context.config.root)
            == wrapper_id
        ),
      "a later graph failure must retain the pending wrapper for retry"
    );
    fs::write(root.join("docs/other.md"), "# Remaining page\n").unwrap();
    fs::remove_file(root.join("docs/index.md")).unwrap();
    let mut removed = PluginUpdateModulesHookParam {
      paths: vec![(module.source_path, UpdateType::Removed)],
    };
    plugin.update_modules(&mut removed, context).unwrap();
    assert!(
      removed.paths.iter().all(|(path, _)| {
        let id = ModuleId::from_resolved_path_with_query(path, &context.config.root);
        id != wrapper_id && id != content_id
      }),
      "deleted content must not reintroduce its pending wrapper"
    );
  });
}

#[test]
fn development_lazy_wrapper_private_identity_resolves_on_update() {
  fixture(|plugin, context, _root| {
    let key = plugin.plan.read().unwrap().as_ref().unwrap().modules[0]
      .key
      .clone();
    let wrapper = page_resources::entry_path(&key);
    let result = plugin
      .resolve(
        &PluginResolveHookParam {
          source: wrapper.clone(),
          importer: None,
          kind: ResolveKind::Entry("page".into()),
        },
        context,
        &PluginHookContext::default(),
      )
      .unwrap()
      .expect("a loaded private wrapper must resolve again during an update");
    assert_eq!(result.resolved_path, wrapper);
  });
}

#[test]
fn development_reindexes_added_and_removed_pages_in_rust() {
  fixture(|plugin, context, root| {
    let added = root.join("docs/new.md");
    fs::write(&added, "# New page\n").unwrap();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(added.to_string_lossy().into_owned(), UpdateType::Added)],
    };
    plugin.update_modules(&mut update, context).unwrap();
    assert_eq!(
      plugin.plan.read().unwrap().as_ref().unwrap().modules.len(),
      2
    );
    fs::remove_file(root.join("docs/index.md")).unwrap();
    update.paths = vec![(
      root.join("docs/index.md").to_string_lossy().into_owned(),
      UpdateType::Removed,
    )];
    plugin.update_modules(&mut update, context).unwrap();
    let guard = plugin.plan.read().unwrap();
    assert_eq!(guard.as_ref().unwrap().modules.len(), 1);
    assert!(guard.as_ref().unwrap().modules[0].code.contains("New page"));
  });
}

#[test]
fn development_loads_content_using_core_restored_path_identity() {
  fixture(|plugin, context, _root| {
    let module = plugin.plan.read().unwrap().as_ref().unwrap().modules[0].clone();
    let id = ModuleId::new(
      &module.source_path,
      &format!("?farm_docs={}", module.key),
      &context.config.root,
    );
    let restored = id.resolved_path(&context.config.root);
    let result = plugin
      .load(
        &PluginLoadHookParam {
          module_id: id.to_string(),
          resolved_path: &restored,
          query: vec![("farm_docs".into(), module.key)],
          meta: Default::default(),
        },
        context,
        &PluginHookContext::default(),
      )
      .expect("load must compare Farm path identities")
      .unwrap();
    assert!(result.content.contains("Initial development body"));
  });
}

#[test]
fn development_retries_pending_content_until_update_completion() {
  fixture(|plugin, context, root| {
    let module = plugin.plan.read().unwrap().as_ref().unwrap().modules[0].clone();
    let id = ModuleId::new(
      &module.source_path,
      &format!("?farm_docs={}", module.key),
      &context.config.root,
    );
    context
      .module_graph
      .write()
      .add_module(farmfe_core::module::Module::new(id.clone()));
    fs::write(root.join("docs/index.md"), "# Pending body\n").unwrap();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(module.source_path.clone(), UpdateType::Updated)],
    };
    plugin.update_modules(&mut update, context).unwrap();
    // A graph parse failure occurs after this hook, before update_finished.
    let component = root.join("component.tsx").to_string_lossy().into_owned();
    update.paths = vec![(component.clone(), UpdateType::Updated)];
    plugin.update_modules(&mut update, context).unwrap();
    assert!(update.paths.iter().any(|(path, _)| path == APP_VIRTUAL_ID));
    assert!(
      update
        .paths
        .iter()
        .any(|(path, _)| path == &id.resolved_path_with_query(&context.config.root))
    );
    plugin.update_finished(context).unwrap();
    update.paths = vec![(component.clone(), UpdateType::Updated)];
    plugin.update_modules(&mut update, context).unwrap();
    assert_eq!(update.paths.len(), 1);
    assert_eq!(update.paths[0].0, component);
  });
}

#[test]
fn development_shares_validated_plan_record_with_other_rust_plugins() {
  fixture(|plugin, context, root| {
    let read_record = || {
      let custom = context.custom.lock();
      let bytes = custom
        .get(farmfe_docs_plan::DEVELOPMENT_PLAN_CONTEXT_KEY)
        .and_then(|value| value.downcast_ref::<String>())
        .expect("Rust-only serialized plan record");
      serde_json::from_str::<DocPlan>(bytes).unwrap()
    };
    let first = read_record();
    assert_eq!(
      first.snapshot_id,
      plugin.plan.read().unwrap().as_ref().unwrap().snapshot_id
    );
    fs::write(root.join("docs/index.md"), "# Shared updated generation\n").unwrap();
    let mut update = PluginUpdateModulesHookParam {
      paths: vec![(
        root.join("docs/index.md").to_string_lossy().into_owned(),
        UpdateType::Updated,
      )],
    };
    plugin.update_modules(&mut update, context).unwrap();
    let second = read_record();
    assert_ne!(second.snapshot_id, first.snapshot_id);
    assert!(second.modules[0].code.contains("Shared updated generation"));
    fs::write(
      root.join("docs/index.md"),
      "# Broken\n\n[Broken](./absent.md)",
    )
    .unwrap();
    assert!(plugin.update_modules(&mut update, context).is_err());
    assert_eq!(read_record().snapshot_id, second.snapshot_id);
  });
}
