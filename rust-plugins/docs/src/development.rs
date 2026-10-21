//! Rust-owned working plans for native incremental compilation. Publishing a
//! successful HTTP generation remains a separate runtime completion boundary.
use super::*;
use farmfe_core::plugin::UpdateType;

fn full_blog_dependencies<'a>(plan: &'a DocPlan, key: &str) -> Option<&'a [String]> {
  plan
    .routes
    .iter()
    .find(|route| route.page.key == key)
    .and_then(|route| route.page.blog_view.as_ref())
    .filter(|view| view.full_content)
    .map(|view| view.posts.as_slice())
}

impl FarmPluginDocs {
  pub(super) fn update_development(
    &self,
    params: &mut PluginUpdateModulesHookParam,
    context: &Arc<CompilationContext>,
  ) -> Result<Option<()>> {
    if params.paths.is_empty() {
      return Ok(None);
    }
    let options = self
      .options
      .as_ref()
      .map_err(|error| diagnostic("DOCS_OPTIONS", error))?;
    let site = options
      .site
      .as_ref()
      .ok_or_else(|| diagnostic("DOCS_OPTIONS", "development requires site"))?;
    let validated = parse_config(&site.to_string(), Path::new(&context.config.root))
      .map_err(|error| diagnostic("DOCS_CONFIG", error))?;
    // Parsing and every link/index check complete before changing working state.
    let candidate = self.prepare_development(&validated)?;
    let previous = self
      .plan
      .read()
      .map_err(|error| diagnostic("DOCS_STATE", error))?
      .clone()
      .ok_or_else(|| diagnostic("DOCS_STATE", "development plan is not loaded"))?;
    let pending = self
      .pending_development_modules
      .read()
      .map_err(|error| diagnostic("DOCS_STATE", error))?
      .clone();
    if previous.snapshot_id == candidate.snapshot_id && pending.is_empty() {
      return Ok(None);
    }
    let mut paths = params.paths.clone();
    // Content is compiled through its Rust-generated query module, never by
    // accidentally introducing a second bare Markdown module into the graph.
    let graph = context.module_graph.read();
    paths.retain(|(path, _)| {
      let id = ModuleId::from_resolved_path_with_query(path, &context.config.root);
      if previous.modules.iter().any(|module| {
        ModuleId::new(&module.source_path, "", &context.config.root).relative_path()
          == id.relative_path()
      }) {
        return false;
      }
      let asset_id = |asset: &farmfe_docs_plan::PreparedAsset| {
        ModuleId::new(
          &asset.source_path,
          &format!("?farm_docs_asset={}", asset.key),
          &context.config.root,
        )
      };
      // Asset keys are content-addressed. Rebuilding an obsolete query against
      // the new plan would either fail or incorrectly give an old identity new
      // bytes. The changed parent imports the replacement query instead.
      // Also recognize still-loaded old queries after a failed previous batch,
      // whose working plan may already have advanced. Bare imports stay owned
      // by their own plugins and must not be suppressed.
      let owned_asset = previous.assets.iter().any(|asset| asset_id(asset) == id)
        || (id.query_string().starts_with("?farm_docs_asset=") && graph.has_module(&id));
      !owned_asset || candidate.assets.iter().any(|asset| asset_id(asset) == id)
    });
    drop(graph);
    let mut invalidated = vec![ModuleId::new(APP_VIRTUAL_ID, "", &context.config.root)];
    for id in pending {
      if id.resolved_path(&context.config.root) == APP_VIRTUAL_ID {
        continue;
      }
      if candidate.modules.iter().any(|module| {
        ModuleId::new(
          &module.source_path,
          &format!("?farm_docs={}", module.key),
          &context.config.root,
        ) == id
          || ModuleId::new(
            &page_resources::entry_path(&module.key),
            "",
            &context.config.root,
          ) == id
      }) {
        invalidated.push(id);
      }
    }
    {
      let graph = context.module_graph.read();
      for module in &candidate.modules {
        let changed = previous
          .modules
          .iter()
          .find(|old| old.key == module.key)
          .is_none_or(|old| old.code != module.code || old.source_path != module.source_path)
          || full_blog_dependencies(&previous, &module.key)
            != full_blog_dependencies(&candidate, &module.key);
        if changed {
          let content_id = ModuleId::new(
            &module.source_path,
            &format!("?farm_docs={}", module.key),
            &context.config.root,
          );
          let wrapper_id = ModuleId::new(
            &page_resources::entry_path(&module.key),
            "",
            &context.config.root,
          );
          for id in [content_id, wrapper_id] {
            if graph.has_module(&id) && !invalidated.contains(&id) {
              invalidated.push(id);
            }
          }
        }
      }
    }
    for id in &invalidated {
      let path = id.resolved_path_with_query(&context.config.root);
      if !paths.iter().any(|(current, _)| *current == path) {
        paths.push((path, UpdateType::Updated));
      }
    }
    let record =
      serde_json::to_string(&candidate).map_err(|error| diagnostic("DOCS_SERIALIZE", error))?;
    let mut pending = self
      .pending_development_modules
      .write()
      .map_err(|error| diagnostic("DOCS_STATE", error))?;
    *self
      .plan
      .write()
      .map_err(|error| diagnostic("DOCS_STATE", error))? = Some(candidate);
    context.custom.lock().insert(
      farmfe_docs_plan::DEVELOPMENT_PLAN_CONTEXT_KEY.into(),
      Box::new(record),
    );
    *pending = invalidated.clone();
    drop(pending);
    for id in invalidated {
      context.cache_manager.module_cache.invalidate_cache(&id);
    }
    params.paths = paths;
    Ok(Some(()))
  }
}
